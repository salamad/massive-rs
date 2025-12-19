//! Authentication types and API key handling.
//!
//! This module provides secure handling of API keys with protection against
//! accidental logging or exposure in debug output.

use secrecy::{ExposeSecret, SecretString};
use std::env;

/// Wrapper for API key with secure handling.
///
/// This type ensures that the API key is never accidentally logged or exposed
/// in debug output. The key is stored using the `secrecy` crate for protection.
///
/// # Example
///
/// ```
/// use massive_rs::auth::ApiKey;
///
/// // Create from string
/// let key = ApiKey::new("your-api-key");
///
/// // Load from environment variable (recommended)
/// let key = ApiKey::from_env(); // Uses MASSIVE_API_KEY
/// ```
#[derive(Clone)]
pub struct ApiKey(SecretString);

impl ApiKey {
    /// Create from string (takes ownership, prevents logging).
    ///
    /// # Arguments
    ///
    /// * `key` - The API key string
    pub fn new(key: impl Into<String>) -> Self {
        Self(SecretString::from(key.into()))
    }

    /// Load from the `MASSIVE_API_KEY` environment variable.
    ///
    /// Returns `None` if the environment variable is not set.
    pub fn from_env() -> Option<Self> {
        env::var("MASSIVE_API_KEY").ok().map(ApiKey::new)
    }

    /// Load from a custom environment variable.
    ///
    /// # Arguments
    ///
    /// * `var_name` - The name of the environment variable to read
    ///
    /// Returns `None` if the environment variable is not set.
    pub fn from_env_var(var_name: &str) -> Option<Self> {
        env::var(var_name).ok().map(ApiKey::new)
    }

    /// Check if the API key is empty.
    pub fn is_empty(&self) -> bool {
        self.0.expose_secret().is_empty()
    }

    /// Expose the key for use in requests (internal only).
    ///
    /// This method is intentionally `pub(crate)` to prevent external
    /// code from accidentally exposing the key.
    pub(crate) fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl Default for ApiKey {
    fn default() -> Self {
        Self::new("")
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

/// Authentication mode for API requests.
///
/// Massive API supports two authentication methods:
/// - Bearer token in the Authorization header (recommended)
/// - API key as a query parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// Use `Authorization: Bearer <key>` header (recommended).
    ///
    /// This is the more secure option as the key is not visible in URLs
    /// or server logs.
    #[default]
    HeaderBearer,

    /// Use `apiKey` query parameter.
    ///
    /// This method exposes the key in URLs, which may be logged by
    /// proxies and servers. Use only when header auth is not supported.
    QueryParam,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_debug_redacted() {
        let key = ApiKey::new("secret-key-12345");
        let debug_output = format!("{:?}", key);
        assert_eq!(debug_output, "ApiKey(***)");
        assert!(!debug_output.contains("secret"));
    }

    #[test]
    fn test_api_key_expose() {
        let key = ApiKey::new("my-secret-key");
        assert_eq!(key.expose(), "my-secret-key");
    }

    #[test]
    fn test_api_key_is_empty() {
        let empty_key = ApiKey::default();
        assert!(empty_key.is_empty());

        let valid_key = ApiKey::new("some-key");
        assert!(!valid_key.is_empty());
    }

    #[test]
    fn test_api_key_clone() {
        let key1 = ApiKey::new("test-key");
        let key2 = key1.clone();
        assert_eq!(key1.expose(), key2.expose());
    }

    #[test]
    fn test_auth_mode_default() {
        let mode = AuthMode::default();
        assert_eq!(mode, AuthMode::HeaderBearer);
    }
}
