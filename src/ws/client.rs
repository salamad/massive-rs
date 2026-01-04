//! WebSocket client implementation.
//!
//! This module contains the core WebSocket client for connecting to
//! Massive real-time data streams with automatic reconnection,
//! backpressure handling, and efficient message dispatch.

use crate::config::{OverflowPolicy, WsConfig};
use crate::error::{MassiveError, WsError};
use crate::ws::models::events::{parse_ws_message, WsEvent};
use crate::ws::protocol::{Subscription, WsAuthMessage, WsSubscribeMessage};
use dashmap::DashSet;
use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, instrument, warn};

/// WebSocket client for Massive.com streaming data.
///
/// This client manages WebSocket connections to the Massive real-time
/// data service, handling authentication, subscriptions, reconnection,
/// and backpressure.
///
/// # Features
///
/// - Automatic reconnection with exponential backoff
/// - Subscription management with resubscribe on reconnect
/// - Backpressure handling with configurable overflow policy
/// - Ping/pong keepalive monitoring
///
/// # Example
///
/// ```no_run
/// use massive_rs::ws::WsClient;
/// use massive_rs::config::WsConfig;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = WsClient::new(WsConfig::default())?;
///     let (handle, stream) = client.connect().await?;
///     // Use handle and stream...
///     Ok(())
/// }
/// ```
pub struct WsClient {
    config: WsConfig,
}

/// Handle for managing an active WebSocket connection.
///
/// The handle allows you to manage subscriptions and close the connection.
/// It is `Clone` and `Send`, so it can be shared across tasks.
#[derive(Clone)]
pub struct WsHandle {
    cmd_tx: mpsc::Sender<WsCommand>,
    state: Arc<WsState>,
    /// Watch channel for connection state changes
    state_rx: watch::Receiver<ConnectionState>,
}

/// Shared state for WebSocket connection.
///
/// This state is shared between the handle and the IO task,
/// providing a view into the connection status.
pub struct WsState {
    /// Whether authentication has succeeded
    pub authenticated: AtomicBool,
    /// Current subscriptions
    pub subscriptions: DashSet<Subscription>,
    /// Timestamp of last message received (Unix millis)
    pub last_message_time: AtomicU64,
    /// Number of messages received
    pub message_count: AtomicU64,
    /// Number of reconnection attempts
    pub reconnect_count: AtomicU32,
    /// Shutdown flag
    shutdown: AtomicBool,
}

/// Connection state for monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connecting to server
    Connecting,
    /// Connected and authenticating
    Authenticating,
    /// Authenticated and ready
    Connected,
    /// Reconnecting after disconnection
    Reconnecting(u32),
    /// Disconnected (terminal state)
    Disconnected,
}

/// Stream of WebSocket events.
pub type WsEventStream =
    std::pin::Pin<Box<dyn futures::Stream<Item = Result<WsMessageBatch, MassiveError>> + Send>>;

/// Batch of WebSocket messages (may contain 1 or more events).
///
/// Messages from the WebSocket are delivered in batches for efficiency.
/// Each batch includes timing information for latency analysis.
#[derive(Debug, Clone)]
pub struct WsMessageBatch {
    /// Events in this batch
    pub events: Vec<WsEvent>,
    /// When this batch was received (monotonic time)
    pub received_at: Instant,
    /// Estimated server-to-client latency if available
    pub latency_hint_ns: Option<u64>,
}

/// Commands sent to the WebSocket IO task.
enum WsCommand {
    Subscribe(Vec<Subscription>, oneshot::Sender<Result<(), MassiveError>>),
    Unsubscribe(Vec<Subscription>, oneshot::Sender<Result<(), MassiveError>>),
    Close(oneshot::Sender<()>),
}

/// Snapshot of connection statistics.
#[derive(Debug, Clone)]
pub struct WsStats {
    /// Total messages received
    pub message_count: u64,
    /// Time since last message
    pub last_message_age: Duration,
    /// Number of reconnection attempts
    pub reconnect_count: u32,
    /// Current subscription count
    pub subscription_count: usize,
}

impl WsClient {
    /// Create a new WebSocket client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is empty.
    pub fn new(config: WsConfig) -> Result<Self, MassiveError> {
        // Validate API key is not empty
        if config.api_key.is_empty() {
            return Err(MassiveError::Auth(
                "API key is empty. Set MASSIVE_API_KEY environment variable or provide a key via WsConfig::new()".into()
            ));
        }
        Ok(Self { config })
    }

    /// Create a WebSocket client builder.
    pub fn builder() -> WsClientBuilder {
        WsClientBuilder::default()
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &WsConfig {
        &self.config
    }

    /// Connect to the WebSocket server.
    ///
    /// Returns a handle for managing the connection and a stream of events.
    /// The connection includes automatic reconnection on disconnection.
    ///
    /// # Errors
    ///
    /// Returns an error if the initial connection cannot be established or
    /// authentication fails.
    #[instrument(skip(self))]
    pub async fn connect(&self) -> Result<(WsHandle, WsEventStream), MassiveError> {
        let url = self.config.build_url();
        info!(url = %url, "Connecting to WebSocket");

        // Create channels
        let (cmd_tx, cmd_rx) = mpsc::channel::<WsCommand>(32);
        let (state_tx, state_rx) = watch::channel(ConnectionState::Connecting);
        let (event_tx, event_rx) = mpsc::channel(self.config.dispatch.capacity);

        // Create shared state
        let state = Arc::new(WsState {
            authenticated: AtomicBool::new(false),
            subscriptions: DashSet::new(),
            last_message_time: AtomicU64::new(0),
            message_count: AtomicU64::new(0),
            reconnect_count: AtomicU32::new(0),
            shutdown: AtomicBool::new(false),
        });

        // Establish initial connection
        let (ws_stream, _response) = connect_async(&url)
            .await
            .map_err(|e| Box::new(WsError::Connection(e)))?;

        let _ = state_tx.send(ConnectionState::Authenticating);

        // Spawn IO task with reconnection logic
        let io_state = state.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            run_io_loop(ws_stream, cmd_rx, event_tx, io_state, config, state_tx).await;
        });

        // Create handle
        let handle = WsHandle {
            cmd_tx,
            state: state.clone(),
            state_rx,
        };

        // Wait for authentication
        handle.wait_for_auth().await?;

        // Create event stream
        let stream = Box::pin(futures::stream::unfold(event_rx, |mut rx| async move {
            rx.recv().await.map(|batch| (batch, rx))
        }));

        Ok((handle, stream))
    }
}

impl WsHandle {
    /// Subscribe to topics.
    ///
    /// Subscriptions are persisted and will be restored on reconnection.
    ///
    /// # Arguments
    ///
    /// * `topics` - Slice of subscriptions to add
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection is closed.
    pub async fn subscribe(&self, topics: &[Subscription]) -> Result<(), MassiveError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(WsCommand::Subscribe(topics.to_vec(), tx))
            .await
            .map_err(|_| MassiveError::Closed)?;
        rx.await.map_err(|_| MassiveError::Closed)?
    }

    /// Unsubscribe from topics.
    ///
    /// # Arguments
    ///
    /// * `topics` - Slice of subscriptions to remove
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection is closed.
    pub async fn unsubscribe(&self, topics: &[Subscription]) -> Result<(), MassiveError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(WsCommand::Unsubscribe(topics.to_vec(), tx))
            .await
            .map_err(|_| MassiveError::Closed)?;
        rx.await.map_err(|_| MassiveError::Closed)?
    }

    /// Close the connection gracefully.
    pub async fn close(&self) -> Result<(), MassiveError> {
        self.state.shutdown.store(true, Ordering::Release);
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(WsCommand::Close(tx)).await;
        let _ = rx.await;
        Ok(())
    }

    /// Check if authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.state.authenticated.load(Ordering::Acquire)
    }

    /// Get current connection state.
    pub fn connection_state(&self) -> ConnectionState {
        *self.state_rx.borrow()
    }

    /// Get current subscriptions.
    pub fn subscriptions(&self) -> Vec<Subscription> {
        self.state.subscriptions.iter().map(|s| s.clone()).collect()
    }

    /// Get connection statistics.
    pub fn stats(&self) -> WsStats {
        let last_msg = self.state.last_message_time.load(Ordering::Acquire);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        WsStats {
            message_count: self.state.message_count.load(Ordering::Acquire),
            last_message_age: Duration::from_millis(now.saturating_sub(last_msg)),
            reconnect_count: self.state.reconnect_count.load(Ordering::Acquire),
            subscription_count: self.state.subscriptions.len(),
        }
    }

    /// Wait for a state change.
    pub async fn wait_for_state(&mut self, target: ConnectionState) {
        while *self.state_rx.borrow() != target {
            if self.state_rx.changed().await.is_err() {
                break;
            }
        }
    }

    /// Wait for authentication to complete.
    async fn wait_for_auth(&self) -> Result<(), MassiveError> {
        let start = Instant::now();
        let timeout = Duration::from_secs(10);

        while start.elapsed() < timeout {
            if self.is_authenticated() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Err(MassiveError::Ws(Box::new(WsError::AuthFailed(
            "Timeout waiting for auth".into(),
        ))))
    }
}

impl std::fmt::Debug for WsHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsHandle")
            .field("authenticated", &self.is_authenticated())
            .field("connection_state", &self.connection_state())
            .field("subscription_count", &self.state.subscriptions.len())
            .finish()
    }
}

/// Builder for WebSocket client.
#[derive(Default)]
pub struct WsClientBuilder {
    config: Option<WsConfig>,
}

impl WsClientBuilder {
    /// Set the configuration.
    pub fn config(mut self, config: WsConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<WsClient, MassiveError> {
        WsClient::new(self.config.unwrap_or_default())
    }
}

/// Main IO loop with reconnection support.
async fn run_io_loop<S>(
    initial_stream: S,
    mut cmd_rx: mpsc::Receiver<WsCommand>,
    event_tx: mpsc::Sender<Result<WsMessageBatch, MassiveError>>,
    state: Arc<WsState>,
    config: WsConfig,
    state_tx: watch::Sender<ConnectionState>,
) where
    S: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin
        + Send,
{
    let (write, read) = initial_stream.split();

    // Run first connection
    let result = run_connection(
        write,
        read,
        &mut cmd_rx,
        &event_tx,
        &state,
        &config,
        &state_tx,
    )
    .await;

    if result.is_ok() || state.shutdown.load(Ordering::Acquire) {
        info!("Connection closed cleanly");
        let _ = state_tx.send(ConnectionState::Disconnected);
        return;
    }

    // Handle reconnection
    let mut attempt = 0u32;

    loop {
        // Check for shutdown
        if state.shutdown.load(Ordering::Acquire) {
            info!("Shutdown requested, exiting IO loop");
            break;
        }

        attempt += 1;
        state.reconnect_count.store(attempt, Ordering::Release);
        let _ = state_tx.send(ConnectionState::Reconnecting(attempt));

        if !config.reconnect.should_retry(attempt) {
            error!(attempt, "Max reconnection attempts reached");
            let _ = state_tx.send(ConnectionState::Disconnected);
            break;
        }

        let delay = config.reconnect.delay_for_attempt(attempt);
        info!(attempt, ?delay, "Reconnecting after delay");
        tokio::time::sleep(delay).await;

        let url = config.build_url();
        let (ws_stream, _) = match connect_async(&url).await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, attempt, "Reconnection failed");
                continue;
            }
        };

        info!(attempt, "Reconnected successfully");
        state.authenticated.store(false, Ordering::Release);

        let (write, read) = ws_stream.split();

        match run_connection(
            write,
            read,
            &mut cmd_rx,
            &event_tx,
            &state,
            &config,
            &state_tx,
        )
        .await
        {
            Ok(()) => {
                info!("Connection closed cleanly after reconnect");
                let _ = state_tx.send(ConnectionState::Disconnected);
                break;
            }
            Err(e) => {
                warn!(error = %e, "Connection error, will reconnect");
                continue;
            }
        }
    }
}

/// Handle a single WebSocket connection.
#[allow(clippy::too_many_arguments)]
async fn run_connection<W, R>(
    mut write: W,
    mut read: R,
    cmd_rx: &mut mpsc::Receiver<WsCommand>,
    event_tx: &mpsc::Sender<Result<WsMessageBatch, MassiveError>>,
    state: &Arc<WsState>,
    config: &WsConfig,
    state_tx: &watch::Sender<ConnectionState>,
) -> Result<(), MassiveError>
where
    W: futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
    R: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    // Send authentication
    let auth_msg = WsAuthMessage::new(config.api_key.expose());
    let auth_json = serde_json::to_string(&auth_msg)
        .map_err(|_| MassiveError::InvalidArgument("Failed to serialize auth message"))?;
    write
        .send(Message::Text(auth_json))
        .await
        .map_err(|e| Box::new(WsError::Connection(e)))?;

    debug!("Sent authentication message");

    // Resubscribe to existing subscriptions
    let subs: Vec<_> = state.subscriptions.iter().map(|s| s.clone()).collect();
    if !subs.is_empty() {
        let msg = WsSubscribeMessage::subscribe(&subs);
        let sub_json = serde_json::to_string(&msg)
            .map_err(|_| MassiveError::InvalidArgument("Failed to serialize subscribe message"))?;
        write
            .send(Message::Text(sub_json))
            .await
            .map_err(|e| Box::new(WsError::Connection(e)))?;
        debug!(count = subs.len(), "Resubscribed to existing topics");
    }

    // Set up ping interval
    let mut ping_interval = tokio::time::interval(config.ping_interval);
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Track last activity for idle timeout
    let mut last_activity = Instant::now();

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        last_activity = Instant::now();
                        let received_at = Instant::now();
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or(Duration::ZERO)
                            .as_millis() as u64;
                        state.last_message_time.store(now_ms, Ordering::Release);
                        state.message_count.fetch_add(1, Ordering::AcqRel);

                        match parse_ws_message(&text) {
                            Ok(events) => {
                                // Check for auth success/failure
                                for event in &events {
                                    if let WsEvent::Status(status) = event {
                                        if status.is_auth_success() {
                                            state.authenticated.store(true, Ordering::Release);
                                            let _ = state_tx.send(ConnectionState::Connected);
                                            info!("WebSocket authenticated");
                                        } else if status.is_auth_failed() {
                                            error!("WebSocket authentication failed: {:?}", status.message);
                                            return Err(MassiveError::Ws(Box::new(
                                                WsError::AuthFailed(status.message.clone().unwrap_or_default())
                                            )));
                                        }
                                    }
                                }

                                let batch = WsMessageBatch {
                                    events,
                                    received_at,
                                    latency_hint_ns: None,
                                };

                                if try_send_event(event_tx, Ok(batch), config.dispatch.overflow).await.is_err() {
                                    return Err(MassiveError::Ws(Box::new(WsError::BackpressureOverflow)));
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, text = %text, "Failed to parse WebSocket message");
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        last_activity = Instant::now();
                        debug!("Received ping, sending pong");
                        write.send(Message::Pong(data)).await
                            .map_err(|e| Box::new(WsError::Connection(e)))?;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_activity = Instant::now();
                        debug!("Received pong");
                    }
                    Some(Ok(Message::Close(frame))) => {
                        info!(?frame, "WebSocket closed by server");
                        return Err(MassiveError::Ws(Box::new(WsError::Disconnected)));
                    }
                    Some(Ok(Message::Binary(_))) => {
                        debug!("Received unexpected binary message");
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Raw frame, usually not received
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "WebSocket error");
                        return Err(MassiveError::Ws(Box::new(WsError::Connection(e))));
                    }
                    None => {
                        info!("WebSocket stream ended");
                        return Err(MassiveError::Ws(Box::new(WsError::Disconnected)));
                    }
                }
            }

            // Handle commands
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(WsCommand::Subscribe(topics, reply)) => {
                        debug!(?topics, "Processing subscribe command");
                        let msg = WsSubscribeMessage::subscribe(&topics);

                        let result = match serde_json::to_string(&msg) {
                            Ok(json) => write.send(Message::Text(json)).await
                                .map_err(|e| MassiveError::Ws(Box::new(WsError::Connection(e)))),
                            Err(_) => Err(MassiveError::InvalidArgument("Failed to serialize subscribe message")),
                        };

                        if result.is_ok() {
                            for topic in topics {
                                state.subscriptions.insert(topic);
                            }
                        }

                        let _ = reply.send(result);
                    }
                    Some(WsCommand::Unsubscribe(topics, reply)) => {
                        debug!(?topics, "Processing unsubscribe command");
                        let msg = WsSubscribeMessage::unsubscribe(&topics);

                        let result = match serde_json::to_string(&msg) {
                            Ok(json) => write.send(Message::Text(json)).await
                                .map_err(|e| MassiveError::Ws(Box::new(WsError::Connection(e)))),
                            Err(_) => Err(MassiveError::InvalidArgument("Failed to serialize unsubscribe message")),
                        };

                        if result.is_ok() {
                            for topic in &topics {
                                state.subscriptions.remove(topic);
                            }
                        }

                        let _ = reply.send(result);
                    }
                    Some(WsCommand::Close(reply)) => {
                        debug!("Processing close command");
                        let _ = write.send(Message::Close(None)).await;
                        let _ = reply.send(());
                        return Ok(());
                    }
                    None => {
                        debug!("Command channel closed");
                        return Ok(());
                    }
                }
            }

            // Send ping for keepalive
            _ = ping_interval.tick() => {
                if last_activity.elapsed() > config.idle_timeout {
                    warn!("Connection idle timeout, sending ping");
                }
                if let Err(e) = write.send(Message::Ping(vec![])).await {
                    warn!(error = %e, "Failed to send ping");
                    return Err(MassiveError::Ws(Box::new(WsError::Connection(e))));
                }
            }
        }
    }
}

/// Try to send an event with backpressure handling.
async fn try_send_event(
    tx: &mpsc::Sender<Result<WsMessageBatch, MassiveError>>,
    batch: Result<WsMessageBatch, MassiveError>,
    policy: OverflowPolicy,
) -> Result<(), ()> {
    match tx.try_send(batch) {
        Ok(()) => Ok(()),
        Err(mpsc::error::TrySendError::Full(_)) => match policy {
            OverflowPolicy::DropNewest | OverflowPolicy::DropOldest => {
                warn!("Buffer full, dropping message");
                Ok(())
            }
            OverflowPolicy::ErrorAndClose => Err(()),
        },
        Err(mpsc::error::TrySendError::Closed(_)) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_batch() {
        let batch = WsMessageBatch {
            events: vec![WsEvent::Unknown],
            received_at: Instant::now(),
            latency_hint_ns: Some(1000),
        };

        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.latency_hint_ns, Some(1000));
    }

    #[test]
    fn test_ws_state_defaults() {
        let state = WsState {
            authenticated: AtomicBool::new(false),
            subscriptions: DashSet::new(),
            last_message_time: AtomicU64::new(0),
            message_count: AtomicU64::new(0),
            reconnect_count: AtomicU32::new(0),
            shutdown: AtomicBool::new(false),
        };

        assert!(!state.authenticated.load(Ordering::Relaxed));
        assert!(state.subscriptions.is_empty());
        assert_eq!(state.message_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_ws_client_builder() {
        let config = WsConfig::new("test-api-key");
        let client = WsClient::builder().config(config).build().unwrap();
        assert!(!client.config().api_key.is_empty());
    }

    #[test]
    fn test_ws_client_empty_api_key_fails() {
        let config = WsConfig::default(); // Empty API key
        let result = WsClient::new(config);
        assert!(result.is_err());
        match result {
            Err(MassiveError::Auth(msg)) => {
                assert!(msg.contains("API key is empty"));
            }
            _ => panic!("Expected MassiveError::Auth"),
        }
    }

    #[test]
    fn test_connection_state_debug() {
        assert_eq!(format!("{:?}", ConnectionState::Connecting), "Connecting");
        assert_eq!(
            format!("{:?}", ConnectionState::Reconnecting(3)),
            "Reconnecting(3)"
        );
    }

    #[test]
    fn test_ws_stats() {
        let state = Arc::new(WsState {
            authenticated: AtomicBool::new(true),
            subscriptions: DashSet::new(),
            last_message_time: AtomicU64::new(0),
            message_count: AtomicU64::new(42),
            reconnect_count: AtomicU32::new(2),
            shutdown: AtomicBool::new(false),
        });

        state.subscriptions.insert(Subscription::trade("AAPL"));
        state.subscriptions.insert(Subscription::quote("AAPL"));

        let (_, state_rx) = watch::channel(ConnectionState::Connected);
        let (cmd_tx, _) = mpsc::channel(1);

        let handle = WsHandle {
            cmd_tx,
            state,
            state_rx,
        };

        let stats = handle.stats();
        assert_eq!(stats.message_count, 42);
        assert_eq!(stats.reconnect_count, 2);
        assert_eq!(stats.subscription_count, 2);
    }
}
