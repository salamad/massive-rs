//! REST client implementation.
//!
//! This module contains the core [`RestClient`] type for making HTTP requests
//! to the Massive REST API.

use crate::auth::AuthMode;
use crate::config::{PaginationMode, RestConfig};
use crate::error::{ApiErrorResponse, MassiveError};
use crate::rest::pagination::PageStream;
use crate::rest::request::{PaginatableRequest, RestRequest};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument, warn};

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
            .pool_idle_timeout(Duration::from_secs(90));

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

    /// Execute a typed request.
    ///
    /// # Arguments
    ///
    /// * `req` - The request to execute
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    #[instrument(skip(self, req), fields(path = %req.path()))]
    pub async fn execute<R>(&self, req: R) -> Result<R::Response, MassiveError>
    where
        R: RestRequest,
    {
        let url = self.build_url(&req)?;
        let method = req.method();

        debug!(method = %method, url = %url, "Executing request");

        let mut request = self.inner.http.request(method.clone(), url);

        // Apply authentication
        request = self.apply_auth(request);

        // Apply body if present
        if let Some(body) = req.body() {
            request = request
                .header("Content-Type", "application/json")
                .body(body);
        }

        // Execute with retry logic
        let response = self.execute_with_retry(request, req.idempotent()).await?;

        // Parse response
        self.parse_response::<R::Response>(response).await
    }

    /// Stream paginated results.
    ///
    /// Returns a stream that automatically follows `next_url` links
    /// to fetch all pages of results.
    ///
    /// # Arguments
    ///
    /// * `req` - The paginated request to execute
    ///
    /// # Example
    ///
    /// ```no_run
    /// use massive_rs::rest::RestClient;
    /// use massive_rs::rest::endpoints::GetAggsRequest;
    /// use massive_rs::rest::endpoints::Timespan;
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RestClient::from_api_key(std::env::var("MASSIVE_API_KEY")?)?;
    /// let request = GetAggsRequest::new("AAPL")
    ///     .multiplier(1)
    ///     .timespan(Timespan::Day)
    ///     .from("2024-01-01")
    ///     .to("2024-01-31");
    ///
    /// let mut stream = client.stream(request);
    /// while let Some(result) = stream.next().await {
    ///     let item = result?;
    ///     println!("{:?}", item);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream<R>(
        &self,
        req: R,
    ) -> impl futures::Stream<Item = Result<R::Item, MassiveError>> + Send + 'static
    where
        R: PaginatableRequest + Unpin + Send + 'static,
        R::Response: DeserializeOwned + Send + 'static,
        R::Item: Unpin,
    {
        PageStream::new(self.clone(), req, self.inner.config.pagination)
    }

    /// Stream paginated results with a custom pagination mode.
    ///
    /// # Arguments
    ///
    /// * `req` - The paginated request to execute
    /// * `mode` - The pagination mode to use
    pub fn stream_with_mode<R>(
        &self,
        req: R,
        mode: PaginationMode,
    ) -> impl futures::Stream<Item = Result<R::Item, MassiveError>> + Send + 'static
    where
        R: PaginatableRequest + Unpin + Send + 'static,
        R::Response: DeserializeOwned + Send + 'static,
        R::Item: Unpin,
    {
        PageStream::new(self.clone(), req, mode)
    }

    /// Build the full URL for a request.
    fn build_url<R: RestRequest>(&self, req: &R) -> Result<url::Url, MassiveError> {
        let mut url = self.inner.config.base_url.clone();
        url.set_path(&req.path());

        let query = req.query();
        if !query.is_empty() {
            url.query_pairs_mut()
                .extend_pairs(query.iter().map(|(k, v)| (k.as_ref(), v.as_str())));
        }

        // Add API key as query param if using that auth mode
        if matches!(self.inner.config.auth_mode, AuthMode::QueryParam) {
            url.query_pairs_mut()
                .append_pair("apiKey", self.inner.config.api_key.expose());
        }

        Ok(url)
    }

    /// Fetch a URL directly (used for pagination next_url).
    ///
    /// The `next_url` from Massive API responses is a complete URL including
    /// the API key as a query parameter, so we fetch it directly.
    pub(crate) async fn fetch_url<T>(&self, url: &str) -> Result<T, MassiveError>
    where
        T: DeserializeOwned,
    {
        debug!(url = %url, "Fetching URL directly");

        let mut request = self.inner.http.get(url);

        // next_url typically includes apiKey, but add auth header for safety
        if matches!(self.inner.config.auth_mode, AuthMode::HeaderBearer) {
            request = request.header(
                "Authorization",
                format!("Bearer {}", self.inner.config.api_key.expose()),
            );
        }

        let response = self.execute_with_retry(request, true).await?;
        self.parse_response(response).await
    }

    /// Apply authentication to a request.
    fn apply_auth(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.inner.config.auth_mode {
            AuthMode::HeaderBearer => {
                request = request.header(
                    "Authorization",
                    format!("Bearer {}", self.inner.config.api_key.expose()),
                );
            }
            AuthMode::QueryParam => {
                // Query param auth is added in build_url
            }
        }
        request
    }

    /// Execute a request with retry logic.
    async fn execute_with_retry(
        &self,
        request: reqwest::RequestBuilder,
        idempotent: bool,
    ) -> Result<Response, MassiveError> {
        let mut attempts = 0;
        let max_attempts = if idempotent {
            self.inner.config.max_retries
        } else {
            1
        };

        loop {
            attempts += 1;

            // Clone request for retry (reqwest doesn't allow reuse)
            let req = request
                .try_clone()
                .ok_or(MassiveError::InvalidArgument("Request body not cloneable"))?;

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();

                    // Handle rate limiting
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = resp
                            .headers()
                            .get("Retry-After")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(Duration::from_secs);

                        return Err(MassiveError::RateLimited {
                            retry_after,
                            request_id: extract_request_id(&resp),
                        });
                    }

                    // Retry on 502/503/504 if idempotent
                    if idempotent && attempts < max_attempts && matches!(status.as_u16(), 502..=504)
                    {
                        warn!(status = %status, attempt = attempts, "Retrying request");
                        tokio::time::sleep(backoff_delay(attempts)).await;
                        continue;
                    }

                    return Ok(resp);
                }
                Err(e) if idempotent && attempts < max_attempts && e.is_connect() => {
                    warn!(error = %e, attempt = attempts, "Connection error, retrying");
                    tokio::time::sleep(backoff_delay(attempts)).await;
                    continue;
                }
                Err(e) if e.is_timeout() => {
                    return Err(MassiveError::Timeout);
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    /// Parse the response body.
    async fn parse_response<T>(&self, response: Response) -> Result<T, MassiveError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let request_id = extract_request_id(&response);

        let bytes = response.bytes().await?;

        if !status.is_success() {
            // Try to parse as API error
            if let Ok(api_error) = serde_json::from_slice::<ApiErrorResponse>(&bytes) {
                return Err(MassiveError::Api(api_error));
            }

            return Err(MassiveError::HttpStatus {
                status: status.as_u16(),
                body: bytes,
                request_id,
            });
        }

        serde_json::from_slice(&bytes).map_err(|e| MassiveError::Deserialize {
            source: e,
            body_snippet: String::from_utf8_lossy(&bytes[..bytes.len().min(500)]).to_string(),
        })
    }
}

/// Extract the X-Request-Id header from a response.
fn extract_request_id(response: &Response) -> Option<String> {
    response
        .headers()
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

/// Calculate exponential backoff delay.
fn backoff_delay(attempt: u32) -> Duration {
    Duration::from_millis(100 * 2u64.pow(attempt.saturating_sub(1)))
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

    #[test]
    fn test_backoff_delay() {
        assert_eq!(backoff_delay(1), Duration::from_millis(100));
        assert_eq!(backoff_delay(2), Duration::from_millis(200));
        assert_eq!(backoff_delay(3), Duration::from_millis(400));
        assert_eq!(backoff_delay(4), Duration::from_millis(800));
    }

    #[test]
    fn test_build_url_with_query() {
        use reqwest::Method;
        use std::borrow::Cow;

        struct TestReq;

        impl RestRequest for TestReq {
            type Response = serde_json::Value;

            fn method(&self) -> Method {
                Method::GET
            }

            fn path(&self) -> Cow<'static, str> {
                "/v2/test".into()
            }

            fn query(&self) -> Vec<(Cow<'static, str>, String)> {
                vec![
                    (Cow::Borrowed("limit"), "100".to_string()),
                    (Cow::Borrowed("sort"), "asc".to_string()),
                ]
            }
        }

        let client = RestClient::from_api_key("test-key").unwrap();
        let url = client.build_url(&TestReq).unwrap();

        assert_eq!(url.path(), "/v2/test");
        assert!(url.query().unwrap().contains("limit=100"));
        assert!(url.query().unwrap().contains("sort=asc"));
    }
}
