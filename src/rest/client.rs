//! REST client implementation.
//!
//! This module contains the core [`RestClient`] type for making HTTP requests
//! to the Massive REST API.

use crate::config::RestConfig;
use crate::error::MassiveError;
use reqwest::Client;
use std::sync::Arc;
use tracing::instrument;

/// REST API client for Massive.com.
///
/// This client handles HTTP communication with the Massive REST API,
/// including authentication, pagination, and error handling.
///
/// # Example
///
/// ```no_run
/// use massive_rs::rest::RestClient;
/// use massive_rs::config::RestConfig;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = RestClient::new(RestConfig::default())?;
///     // Make API calls...
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct RestClient {
    inner: Arc<RestClientInner>,
}

struct RestClientInner {
    #[allow(dead_code)] // Used in Phase 2 for request execution
    http: Client,
    config: RestConfig,
}

impl RestClient {
    /// Create a new REST client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    #[instrument(skip_all)]
    pub fn new(config: RestConfig) -> Result<Self, MassiveError> {
        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90));

        #[cfg(feature = "gzip")]
        {
            builder = builder.gzip(true);
        }

        if let Some(ua) = &config.user_agent {
            builder = builder.user_agent(ua);
        } else {
            builder = builder.user_agent(crate::user_agent());
        }

        let http = builder.build()?;

        Ok(Self {
            inner: Arc::new(RestClientInner { http, config }),
        })
    }

    /// Create a client from just an API key using default settings.
    ///
    /// # Arguments
    ///
    /// * `key` - The API key for authentication
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn from_api_key(key: impl Into<String>) -> Result<Self, MassiveError> {
        let config = RestConfig::new(key);
        Self::new(config)
    }

    /// Get a reference to the underlying configuration.
    pub fn config(&self) -> &RestConfig {
        &self.inner.config
    }

    /// Get a reference to the underlying HTTP client.
    #[allow(dead_code)] // Used in Phase 2 for request execution
    pub(crate) fn http(&self) -> &Client {
        &self.inner.http
    }
}

impl std::fmt::Debug for RestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestClient")
            .field("config", &self.inner.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ApiKey;

    #[test]
    fn test_create_client() {
        let config = RestConfig {
            api_key: ApiKey::new("test-key"),
            ..Default::default()
        };
        let client = RestClient::new(config).unwrap();
        assert!(!client.config().api_key.is_empty());
    }

    #[test]
    fn test_from_api_key() {
        let client = RestClient::from_api_key("test-key").unwrap();
        assert!(!client.config().api_key.is_empty());
    }

    #[test]
    fn test_client_debug() {
        let client = RestClient::from_api_key("secret").unwrap();
        let debug = format!("{:?}", client);
        // API key should be redacted
        assert!(!debug.contains("secret"));
        assert!(debug.contains("RestClient"));
    }

    #[test]
    fn test_client_clone() {
        let client1 = RestClient::from_api_key("key").unwrap();
        let client2 = client1.clone();

        // Both should point to same inner state (Arc)
        assert!(Arc::ptr_eq(&client1.inner, &client2.inner));
    }
}
