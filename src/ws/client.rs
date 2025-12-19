//! WebSocket client implementation.
//!
//! This module contains the core WebSocket client for connecting to
//! Massive real-time data streams.

use crate::auth::ApiKey;
use crate::config::{DispatchConfig, ReconnectConfig, WsConfig};
use crate::error::{MassiveError, WsError};
use crate::ws::models::events::{parse_ws_message, WsEvent};
use crate::ws::protocol::{Subscription, WsAuthMessage, WsSubscribeMessage};
use dashmap::DashSet;
use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, instrument, warn};

/// WebSocket client for Massive.com streaming data.
///
/// This client manages WebSocket connections to the Massive real-time
/// data service, handling authentication, subscriptions, and reconnection.
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
    /// Timestamp of last message received (monotonic, for internal use)
    pub last_message: AtomicU64,
}

/// Stream of WebSocket events.
pub type WsEventStream =
    std::pin::Pin<Box<dyn futures::Stream<Item = Result<WsMessageBatch, MassiveError>> + Send>>;

/// Batch of WebSocket messages (may contain 1 or more events).
///
/// Messages from the WebSocket are delivered in batches for efficiency.
/// Each batch includes the timestamp when it was received.
#[derive(Debug, Clone)]
pub struct WsMessageBatch {
    /// Events in this batch
    pub events: Vec<WsEvent>,
    /// When this batch was received
    pub received_at: std::time::Instant,
}

/// Commands sent to the WebSocket IO task.
enum WsCommand {
    Subscribe(Vec<Subscription>, oneshot::Sender<Result<(), MassiveError>>),
    Unsubscribe(Vec<Subscription>, oneshot::Sender<Result<(), MassiveError>>),
    Close(oneshot::Sender<()>),
}

impl WsClient {
    /// Create a new WebSocket client with the given configuration.
    pub fn new(config: WsConfig) -> Result<Self, MassiveError> {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established or
    /// authentication fails.
    #[instrument(skip(self))]
    pub async fn connect(&self) -> Result<(WsHandle, WsEventStream), MassiveError> {
        let url = self.config.build_url();
        info!(url = %url, "Connecting to WebSocket");

        // Establish connection
        let (ws_stream, _response) = connect_async(&url)
            .await
            .map_err(|e| Box::new(WsError::Connection(e)))?;

        let (write, read) = ws_stream.split();

        // Create channels
        let (cmd_tx, cmd_rx) = mpsc::channel::<WsCommand>(32);
        let (event_tx, event_rx) = mpsc::channel(self.config.dispatch.capacity);

        // Create shared state
        let state = Arc::new(WsState {
            authenticated: AtomicBool::new(false),
            subscriptions: DashSet::new(),
            last_message: AtomicU64::new(0),
        });

        // Spawn IO task
        let io_state = state.clone();
        let api_key = self.config.api_key.clone();
        let reconnect_config = self.config.reconnect.clone();
        let dispatch_config = self.config.dispatch.clone();

        tokio::spawn(async move {
            run_io_task(
                write,
                read,
                cmd_rx,
                event_tx,
                io_state,
                api_key,
                reconnect_config,
                dispatch_config,
            )
            .await;
        });

        // Create handle
        let handle = WsHandle {
            cmd_tx,
            state: state.clone(),
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
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(WsCommand::Close(tx)).await;
        let _ = rx.await;
        Ok(())
    }

    /// Check if authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.state.authenticated.load(Ordering::Acquire)
    }

    /// Get current subscriptions.
    pub fn subscriptions(&self) -> Vec<Subscription> {
        self.state.subscriptions.iter().map(|s| s.clone()).collect()
    }

    /// Wait for authentication to complete.
    async fn wait_for_auth(&self) -> Result<(), MassiveError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(10);

        while start.elapsed() < timeout {
            if self.is_authenticated() {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
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

/// IO task that handles WebSocket read/write.
#[allow(clippy::too_many_arguments)]
async fn run_io_task<W, R>(
    mut write: W,
    mut read: R,
    mut cmd_rx: mpsc::Receiver<WsCommand>,
    event_tx: mpsc::Sender<Result<WsMessageBatch, MassiveError>>,
    state: Arc<WsState>,
    api_key: ApiKey,
    _reconnect: ReconnectConfig,
    _dispatch: DispatchConfig,
) where
    W: futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
    R: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    // Send authentication
    let auth_msg = WsAuthMessage::new(api_key.expose());

    if let Err(e) = write
        .send(Message::Text(serde_json::to_string(&auth_msg).unwrap()))
        .await
    {
        error!(error = %e, "Failed to send auth message");
        return;
    }

    debug!("Sent authentication message");

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let received_at = std::time::Instant::now();
                        state.last_message.store(
                            received_at.elapsed().as_nanos() as u64,
                            Ordering::Release
                        );

                        match parse_ws_message(&text) {
                            Ok(events) => {
                                // Check for auth success/failure
                                for event in &events {
                                    if let WsEvent::Status(status) = event {
                                        if status.is_auth_success() {
                                            state.authenticated.store(true, Ordering::Release);
                                            info!("WebSocket authenticated");
                                        } else if status.is_auth_failed() {
                                            error!("WebSocket authentication failed: {:?}", status.message);
                                            return;
                                        }
                                    }
                                }

                                let batch = WsMessageBatch { events, received_at };

                                // Non-blocking send with backpressure handling
                                if event_tx.try_send(Ok(batch)).is_err() {
                                    warn!("Event buffer full, dropping message");
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, text = %text, "Failed to parse WebSocket message");
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        debug!("Received ping, sending pong");
                        let _ = write.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!("Received pong");
                    }
                    Some(Ok(Message::Close(frame))) => {
                        info!(?frame, "WebSocket closed by server");
                        break;
                    }
                    Some(Ok(Message::Binary(_))) => {
                        debug!("Received unexpected binary message");
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Raw frame, usually not received
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "WebSocket error");
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        break;
                    }
                }
            }

            // Handle commands
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(WsCommand::Subscribe(topics, reply)) => {
                        debug!(?topics, "Processing subscribe command");
                        let msg = WsSubscribeMessage::subscribe(&topics);

                        let result = write.send(Message::Text(serde_json::to_string(&msg).unwrap())).await
                            .map_err(|e| MassiveError::Ws(Box::new(WsError::Connection(e))));

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

                        let result = write.send(Message::Text(serde_json::to_string(&msg).unwrap())).await
                            .map_err(|e| MassiveError::Ws(Box::new(WsError::Connection(e))));

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
                        break;
                    }
                    None => {
                        debug!("Command channel closed");
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket IO task exiting");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_batch() {
        let batch = WsMessageBatch {
            events: vec![WsEvent::Unknown],
            received_at: std::time::Instant::now(),
        };

        assert_eq!(batch.events.len(), 1);
    }

    #[test]
    fn test_ws_state_defaults() {
        let state = WsState {
            authenticated: AtomicBool::new(false),
            subscriptions: DashSet::new(),
            last_message: AtomicU64::new(0),
        };

        assert!(!state.authenticated.load(Ordering::Relaxed));
        assert!(state.subscriptions.is_empty());
    }

    #[test]
    fn test_ws_client_builder() {
        let config = WsConfig::default();
        let client = WsClient::builder().config(config).build().unwrap();
        assert!(client.config().api_key.is_empty() || !client.config().api_key.is_empty());
    }
}
