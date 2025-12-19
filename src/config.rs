//! Configuration types for REST and WebSocket clients.
//!
//! This module provides configuration structures for both the REST and
//! WebSocket clients, including URL configuration, timeouts, authentication
//! mode, and pagination behavior.

use crate::auth::{ApiKey, AuthMode};
use std::time::Duration;
use url::Url;

/// Default REST API base URL.
pub const DEFAULT_REST_URL: &str = "https://api.massive.com";

/// Default real-time WebSocket URL.
pub const DEFAULT_WS_REALTIME_URL: &str = "wss://socket.massive.com";

/// Default delayed WebSocket URL.
pub const DEFAULT_WS_DELAYED_URL: &str = "wss://delayed.massive.com";

/// REST API client configuration.
///
/// This structure contains all settings needed to configure the REST client,
/// including the API endpoint, authentication, timeouts, and pagination behavior.
///
/// # Example
///
/// ```
/// use massive_rs::config::RestConfig;
/// use massive_rs::auth::ApiKey;
/// use std::time::Duration;
///
/// let config = RestConfig {
///     api_key: ApiKey::new("your-api-key"),
///     request_timeout: Duration::from_secs(60),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RestConfig {
    /// Base URL for REST API (default: <https://api.massive.com>).
    pub base_url: Url,

    /// API key for authentication.
    pub api_key: ApiKey,

    /// Authentication mode (header vs query param).
    pub auth_mode: AuthMode,

    /// Connection timeout.
    pub connect_timeout: Duration,

    /// Request timeout (per request).
    pub request_timeout: Duration,

    /// Pagination behavior.
    pub pagination: PaginationMode,

    /// Enable debug tracing.
    pub trace: bool,

    /// Custom User-Agent string.
    pub user_agent: Option<String>,

    /// Maximum retry attempts for transient errors.
    pub max_retries: u32,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse(DEFAULT_REST_URL).expect("default URL is valid"),
            api_key: ApiKey::from_env().unwrap_or_default(),
            auth_mode: AuthMode::HeaderBearer,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            pagination: PaginationMode::Auto,
            trace: false,
            user_agent: None,
            max_retries: 3,
        }
    }
}

impl RestConfig {
    /// Create a new configuration with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: ApiKey::new(api_key),
            ..Default::default()
        }
    }

    /// Set the base URL.
    pub fn with_base_url(mut self, url: Url) -> Self {
        self.base_url = url;
        self
    }

    /// Set the authentication mode.
    pub fn with_auth_mode(mut self, mode: AuthMode) -> Self {
        self.auth_mode = mode;
        self
    }

    /// Set the connection timeout.
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set the pagination mode.
    pub fn with_pagination(mut self, mode: PaginationMode) -> Self {
        self.pagination = mode;
        self
    }

    /// Enable debug tracing.
    pub fn with_trace(mut self, enabled: bool) -> Self {
        self.trace = enabled;
        self
    }

    /// Set a custom user agent.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }
}

/// WebSocket client configuration.
///
/// This structure contains all settings for the WebSocket client,
/// including feed type, market selection, timeouts, and reconnection behavior.
#[derive(Debug, Clone)]
#[cfg(feature = "ws")]
pub struct WsConfig {
    /// Feed type (real-time vs delayed).
    pub feed: Feed,

    /// Market/asset class.
    pub market: Market,

    /// API key for authentication.
    pub api_key: ApiKey,

    /// Connection timeout.
    pub connect_timeout: Duration,

    /// Idle timeout before ping.
    pub idle_timeout: Duration,

    /// Ping interval for keepalive.
    pub ping_interval: Duration,

    /// Reconnection configuration.
    pub reconnect: ReconnectConfig,

    /// Dispatch/backpressure configuration.
    pub dispatch: DispatchConfig,
}

#[cfg(feature = "ws")]
impl Default for WsConfig {
    fn default() -> Self {
        Self {
            feed: Feed::RealTime,
            market: Market::Stocks,
            api_key: ApiKey::from_env().unwrap_or_default(),
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(30),
            ping_interval: Duration::from_secs(15),
            reconnect: ReconnectConfig::default(),
            dispatch: DispatchConfig::default(),
        }
    }
}

#[cfg(feature = "ws")]
impl WsConfig {
    /// Create a new configuration with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: ApiKey::new(api_key),
            ..Default::default()
        }
    }

    /// Set the feed type.
    pub fn with_feed(mut self, feed: Feed) -> Self {
        self.feed = feed;
        self
    }

    /// Set the market.
    pub fn with_market(mut self, market: Market) -> Self {
        self.market = market;
        self
    }

    /// Build the WebSocket URL for this configuration.
    pub fn build_url(&self) -> String {
        let host = match self.feed {
            Feed::RealTime => "socket.massive.com",
            Feed::Delayed => "delayed.massive.com",
        };
        format!("wss://{}/{}", host, self.market.as_path())
    }
}

/// WebSocket feed type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Feed {
    /// Real-time data: wss://socket.massive.com
    #[default]
    RealTime,
    /// 15-minute delayed: wss://delayed.massive.com
    Delayed,
}

/// Market/asset class for WebSocket connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Market {
    /// US equities
    #[default]
    Stocks,
    /// Options contracts
    Options,
    /// Futures contracts
    Futures,
    /// Market indices
    Indices,
    /// Foreign exchange
    Forex,
    /// Cryptocurrency
    Crypto,
}

impl Market {
    /// Get the URL path segment for this market.
    pub fn as_path(&self) -> &'static str {
        match self {
            Market::Stocks => "stocks",
            Market::Options => "options",
            Market::Futures => "futures",
            Market::Indices => "indices",
            Market::Forex => "forex",
            Market::Crypto => "crypto",
        }
    }
}

/// Pagination mode for REST API requests.
///
/// Controls how the client handles paginated responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaginationMode {
    /// Automatically fetch all pages.
    ///
    /// The client will follow `next_url` links until all results are retrieved.
    #[default]
    Auto,

    /// Single page only.
    ///
    /// Only the first page of results is returned; `next_url` is ignored.
    None,

    /// Stop after N total items.
    ///
    /// The client will stop fetching after the specified number of items
    /// have been retrieved, even if more pages are available.
    MaxItems(u64),
}

/// Reconnection configuration for WebSocket clients.
#[derive(Debug, Clone)]
#[cfg(feature = "ws")]
pub struct ReconnectConfig {
    /// Enable automatic reconnection.
    pub enabled: bool,

    /// Initial delay before first retry.
    pub initial_delay: Duration,

    /// Maximum delay between retries.
    pub max_delay: Duration,

    /// Maximum retry attempts (None = unlimited).
    pub max_retries: Option<u32>,

    /// Backoff multiplier.
    pub backoff_multiplier: f64,
}

#[cfg(feature = "ws")]
impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_retries: None,
            backoff_multiplier: 2.0,
        }
    }
}

#[cfg(feature = "ws")]
impl ReconnectConfig {
    /// Create a configuration that disables reconnection.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Calculate delay for given attempt number.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self
                .backoff_multiplier
                .powi(attempt.saturating_sub(1) as i32);

        Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f64) as u64)
    }

    /// Check if should retry.
    pub fn should_retry(&self, attempt: u32) -> bool {
        self.enabled && self.max_retries.map_or(true, |max| attempt < max)
    }
}

/// Dispatch configuration for backpressure handling.
#[derive(Debug, Clone)]
#[cfg(feature = "ws")]
pub struct DispatchConfig {
    /// Channel buffer capacity.
    pub capacity: usize,

    /// Overflow policy when buffer is full.
    pub overflow: OverflowPolicy,

    /// Fanout mode.
    pub fanout: FanoutMode,
}

#[cfg(feature = "ws")]
impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            overflow: OverflowPolicy::DropOldest,
            fanout: FanoutMode::SingleConsumer,
        }
    }
}

/// Policy when buffer is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg(feature = "ws")]
pub enum OverflowPolicy {
    /// Drop oldest messages to make room.
    #[default]
    DropOldest,

    /// Drop newest (incoming) messages.
    DropNewest,

    /// Treat overflow as fatal error.
    ErrorAndClose,
}

/// Fanout mode for multiple consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg(feature = "ws")]
pub enum FanoutMode {
    /// Single consumer (mpsc channel).
    #[default]
    SingleConsumer,

    /// Multiple consumers (broadcast channel).
    Broadcast,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_config_default() {
        let config = RestConfig::default();
        assert_eq!(config.base_url.as_str(), "https://api.massive.com/");
        assert_eq!(config.auth_mode, AuthMode::HeaderBearer);
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert_eq!(config.pagination, PaginationMode::Auto);
        assert!(!config.trace);
    }

    #[test]
    fn test_rest_config_builder() {
        let config = RestConfig::new("test-key")
            .with_auth_mode(AuthMode::QueryParam)
            .with_request_timeout(Duration::from_secs(60))
            .with_pagination(PaginationMode::MaxItems(1000))
            .with_trace(true)
            .with_user_agent("my-app/1.0");

        assert_eq!(config.auth_mode, AuthMode::QueryParam);
        assert_eq!(config.request_timeout, Duration::from_secs(60));
        assert_eq!(config.pagination, PaginationMode::MaxItems(1000));
        assert!(config.trace);
        assert_eq!(config.user_agent, Some("my-app/1.0".to_string()));
    }

    #[test]
    fn test_market_as_path() {
        assert_eq!(Market::Stocks.as_path(), "stocks");
        assert_eq!(Market::Options.as_path(), "options");
        assert_eq!(Market::Futures.as_path(), "futures");
        assert_eq!(Market::Indices.as_path(), "indices");
        assert_eq!(Market::Forex.as_path(), "forex");
        assert_eq!(Market::Crypto.as_path(), "crypto");
    }

    #[test]
    fn test_pagination_mode_default() {
        assert_eq!(PaginationMode::default(), PaginationMode::Auto);
    }

    #[cfg(feature = "ws")]
    #[test]
    fn test_ws_config_build_url() {
        let config = WsConfig::default();
        assert_eq!(config.build_url(), "wss://socket.massive.com/stocks");

        let delayed_config = WsConfig::default()
            .with_feed(Feed::Delayed)
            .with_market(Market::Crypto);
        assert_eq!(
            delayed_config.build_url(),
            "wss://delayed.massive.com/crypto"
        );
    }

    #[cfg(feature = "ws")]
    #[test]
    fn test_reconnect_delay_calculation() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            ..Default::default()
        };

        assert_eq!(config.delay_for_attempt(1), Duration::from_secs(1));
        assert_eq!(config.delay_for_attempt(2), Duration::from_secs(2));
        assert_eq!(config.delay_for_attempt(3), Duration::from_secs(4));
        assert_eq!(config.delay_for_attempt(4), Duration::from_secs(8));
        // Should cap at max_delay
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(60));
    }

    #[cfg(feature = "ws")]
    #[test]
    fn test_reconnect_should_retry() {
        let config = ReconnectConfig {
            enabled: true,
            max_retries: Some(3),
            ..Default::default()
        };

        assert!(config.should_retry(0));
        assert!(config.should_retry(1));
        assert!(config.should_retry(2));
        assert!(!config.should_retry(3));

        let unlimited = ReconnectConfig::default();
        assert!(unlimited.should_retry(100));

        let disabled = ReconnectConfig::disabled();
        assert!(!disabled.should_retry(0));
    }
}
