//! Error types for Massive API operations.
//!
//! This module provides a unified error type [`MassiveError`] that covers
//! all possible error conditions when interacting with the Massive API,
//! including HTTP errors, WebSocket errors, parsing errors, and API-specific
//! error responses.

use bytes::Bytes;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

/// Unified error type for all Massive operations.
///
/// This enum covers all possible error conditions that can occur when
/// using the Massive API client, including transport errors, HTTP errors,
/// parsing errors, and WebSocket-specific errors.
#[derive(Debug, Error)]
pub enum MassiveError {
    /// HTTP transport error from reqwest.
    #[error("Transport error: {0}")]
    Transport(#[from] reqwest::Error),

    /// Request timed out.
    #[error("Request timed out")]
    Timeout,

    /// HTTP error with status code.
    ///
    /// This variant is used when the server returns an HTTP error status
    /// but the response body could not be parsed as an API error.
    #[error("HTTP {status}: {}", body_preview(.body))]
    HttpStatus {
        /// HTTP status code
        status: u16,
        /// Raw response body
        body: Bytes,
        /// Request ID from headers (if available)
        request_id: Option<String>,
    },

    /// Parsed API error response.
    ///
    /// This variant is used when the server returns an error response
    /// that could be successfully parsed as an [`ApiErrorResponse`].
    #[error("API error: {0}")]
    Api(ApiErrorResponse),

    /// JSON deserialization failed.
    #[error("Deserialization error: {source}")]
    Deserialize {
        /// The underlying serde_json error
        #[source]
        source: serde_json::Error,
        /// A snippet of the body that failed to parse
        body_snippet: String,
    },

    /// Invalid argument provided to a method.
    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),

    /// Rate limit exceeded.
    ///
    /// The API returns HTTP 429 when rate limits are exceeded.
    /// The `retry_after` field contains the suggested wait time.
    #[error("Rate limited{}", format_retry(.retry_after))]
    RateLimited {
        /// Suggested time to wait before retrying
        retry_after: Option<Duration>,
        /// Request ID from headers (if available)
        request_id: Option<String>,
    },

    /// WebSocket-specific error.
    #[cfg(feature = "ws")]
    #[error("WebSocket error: {0}")]
    Ws(#[from] Box<WsError>),

    /// Connection closed unexpectedly.
    #[error("Connection closed")]
    Closed,

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// URL parsing error.
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
}

/// WebSocket-specific errors.
#[cfg(feature = "ws")]
#[derive(Debug, Error)]
pub enum WsError {
    /// WebSocket connection error.
    #[error("WebSocket connection error: {0}")]
    Connection(#[from] tokio_tungstenite::tungstenite::Error),

    /// Authentication handshake failed.
    #[error("Authentication handshake failed: {0}")]
    AuthFailed(String),

    /// Protocol violation detected.
    #[error("Protocol violation: {0}")]
    Protocol(String),

    /// Server disconnected unexpectedly.
    #[error("Server disconnected")]
    Disconnected,

    /// Backpressure overflow (buffer full).
    ///
    /// This error occurs when the message consumer cannot keep up
    /// with the incoming message rate and the buffer is full.
    #[error("Backpressure overflow (buffer full)")]
    BackpressureOverflow,

    /// Subscription request failed.
    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),
}

/// Parsed error response from Massive API.
///
/// The API returns structured error responses with status, error message,
/// and optional request ID for debugging.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    /// Status string (e.g., "ERROR", "NOT_FOUND")
    pub status: String,
    /// Error code or short description
    pub error: Option<String>,
    /// Human-readable error message
    pub message: Option<String>,
    /// Request ID for debugging with Massive support
    pub request_id: Option<String>,
}

impl std::fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.error
                .as_deref()
                .or(self.message.as_deref())
                .unwrap_or(&self.status)
        )
    }
}

impl std::error::Error for ApiErrorResponse {}

/// Helper function to create a preview of the response body.
///
/// Truncates long bodies to 200 characters to prevent huge error messages.
fn body_preview(body: &Bytes) -> String {
    let s = String::from_utf8_lossy(body);
    if s.len() > 200 {
        format!("{}...", &s[..200])
    } else {
        s.to_string()
    }
}

/// Helper function to format the retry-after duration.
fn format_retry(retry_after: &Option<Duration>) -> String {
    match retry_after {
        Some(d) => format!(", retry after {:?}", d),
        None => String::new(),
    }
}

/// Result type alias for Massive operations.
pub type Result<T> = std::result::Result<T, MassiveError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_response_display() {
        let err = ApiErrorResponse {
            status: "ERROR".to_string(),
            error: Some("invalid_ticker".to_string()),
            message: Some("The ticker symbol is not valid".to_string()),
            request_id: Some("abc123".to_string()),
        };
        // Should prefer error field over message
        assert_eq!(format!("{}", err), "invalid_ticker");

        let err_no_error = ApiErrorResponse {
            status: "ERROR".to_string(),
            error: None,
            message: Some("Something went wrong".to_string()),
            request_id: None,
        };
        // Should fall back to message
        assert_eq!(format!("{}", err_no_error), "Something went wrong");

        let err_only_status = ApiErrorResponse {
            status: "NOT_FOUND".to_string(),
            error: None,
            message: None,
            request_id: None,
        };
        // Should fall back to status
        assert_eq!(format!("{}", err_only_status), "NOT_FOUND");
    }

    #[test]
    fn test_body_preview_short() {
        let body = Bytes::from("short body");
        assert_eq!(body_preview(&body), "short body");
    }

    #[test]
    fn test_body_preview_long() {
        let long_body = "x".repeat(500);
        let body = Bytes::from(long_body);
        let preview = body_preview(&body);
        assert!(preview.ends_with("..."));
        assert_eq!(preview.len(), 203); // 200 + "..."
    }

    #[test]
    fn test_format_retry_some() {
        let duration = Some(Duration::from_secs(30));
        let formatted = format_retry(&duration);
        assert!(formatted.contains("30s"));
    }

    #[test]
    fn test_format_retry_none() {
        let duration: Option<Duration> = None;
        let formatted = format_retry(&duration);
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_massive_error_display() {
        let err = MassiveError::Timeout;
        assert_eq!(format!("{}", err), "Request timed out");

        let err = MassiveError::InvalidArgument("ticker is required");
        assert_eq!(format!("{}", err), "Invalid argument: ticker is required");

        let err = MassiveError::Closed;
        assert_eq!(format!("{}", err), "Connection closed");

        let err = MassiveError::Auth("invalid key".to_string());
        assert_eq!(format!("{}", err), "Authentication failed: invalid key");
    }

    #[test]
    fn test_rate_limited_display() {
        let err = MassiveError::RateLimited {
            retry_after: Some(Duration::from_secs(60)),
            request_id: Some("req123".to_string()),
        };
        let display = format!("{}", err);
        assert!(display.contains("Rate limited"));
        assert!(display.contains("60s"));
    }
}
