//! Common test utilities and helpers.
//!
//! This module provides shared infrastructure for integration tests,
//! including API client setup and test configuration.

#![allow(dead_code)]

use massive_rs::auth::ApiKey;
use massive_rs::config::RestConfig;
use massive_rs::rest::RestClient;

#[cfg(feature = "ws")]
use massive_rs::config::WsConfig;

/// Environment variable name for the API key (Polygon.io compatible).
pub const API_KEY_ENV_VAR: &str = "POLYGON_API_KEY";

/// Check if the API key is available.
///
/// Returns `true` if the `POLYGON_API_KEY` environment variable is set.
pub fn has_api_key() -> bool {
    std::env::var(API_KEY_ENV_VAR).is_ok()
}

/// Get the API key from environment.
///
/// # Panics
///
/// Panics if `POLYGON_API_KEY` is not set.
pub fn get_api_key() -> String {
    std::env::var(API_KEY_ENV_VAR)
        .expect("POLYGON_API_KEY environment variable must be set for integration tests")
}

/// Create a REST client configured with the API key from environment.
///
/// # Panics
///
/// Panics if `POLYGON_API_KEY` is not set or if the client cannot be created.
pub fn create_rest_client() -> RestClient {
    let api_key = get_api_key();

    // Use the real Polygon.io API URL
    let config = RestConfig {
        base_url: url::Url::parse("https://api.polygon.io").expect("valid URL"),
        api_key: ApiKey::new(&api_key),
        ..Default::default()
    };

    RestClient::new(config).expect("Failed to create REST client")
}

/// Create a REST client with custom base URL.
///
/// # Arguments
///
/// * `base_url` - The base URL for the API
pub fn create_rest_client_with_url(base_url: &str) -> RestClient {
    let api_key = get_api_key();

    let config = RestConfig {
        base_url: url::Url::parse(base_url).expect("valid URL"),
        api_key: ApiKey::new(&api_key),
        ..Default::default()
    };

    RestClient::new(config).expect("Failed to create REST client")
}

/// Create a WebSocket configuration with the API key from environment.
#[cfg(feature = "ws")]
pub fn create_ws_config() -> WsConfig {
    use massive_rs::config::{Feed, Market, ReconnectConfig};

    let api_key = get_api_key();

    WsConfig {
        feed: Feed::Delayed, // Use delayed feed for testing to avoid real-time data limits
        market: Market::Stocks,
        api_key: ApiKey::new(&api_key),
        reconnect: ReconnectConfig {
            enabled: true,
            max_retries: Some(3),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Create a WebSocket configuration for a specific market.
#[cfg(feature = "ws")]
pub fn create_ws_config_for_market(market: massive_rs::config::Market) -> WsConfig {
    use massive_rs::config::{Feed, ReconnectConfig};

    let api_key = get_api_key();

    WsConfig {
        feed: Feed::Delayed,
        market,
        api_key: ApiKey::new(&api_key),
        reconnect: ReconnectConfig {
            enabled: true,
            max_retries: Some(3),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Skip test if API key is not available.
///
/// Returns `()` if the API key is set, otherwise prints a skip message.
#[macro_export]
macro_rules! skip_without_api_key {
    () => {
        if !$crate::common::has_api_key() {
            eprintln!("Skipping test: {} not set", $crate::common::API_KEY_ENV_VAR);
            return;
        }
    };
}

/// A test ticker symbol that's known to have data.
pub const TEST_TICKER: &str = "AAPL";

/// An alternative test ticker.
pub const TEST_TICKER_ALT: &str = "MSFT";

/// A crypto ticker for testing.
pub const TEST_CRYPTO_TICKER: &str = "X:BTCUSD";

/// A forex ticker for testing.
pub const TEST_FOREX_TICKER: &str = "C:EURUSD";

/// A recent historical date for testing (should have market data).
pub fn test_date() -> String {
    // Use a date that's definitely in the past and had market activity
    "2024-01-15".to_string()
}

/// A date range for aggregate testing.
pub fn test_date_range() -> (String, String) {
    ("2024-01-01".to_string(), "2024-01-31".to_string())
}

/// Initialize test logging (tracing subscriber).
pub fn init_test_logging() {
    use tracing_subscriber::EnvFilter;

    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_test_writer()
        .try_init();
}
