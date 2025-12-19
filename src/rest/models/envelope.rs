//! Response envelope types.
//!
//! The Massive API wraps responses in standard envelope structures
//! that include metadata like status, count, and pagination URLs.

use serde::Deserialize;

/// Standard API response envelope.
///
/// Most Massive API endpoints return responses wrapped in this structure,
/// which includes metadata about the request and pagination information.
///
/// # Type Parameter
///
/// * `T` - The type of the `results` field (can be a single item or Vec)
#[derive(Debug, Clone, Deserialize)]
pub struct ApiEnvelope<T> {
    /// Status string (e.g., "OK", "ERROR")
    pub status: Option<String>,

    /// Number of results in this response
    pub count: Option<u64>,

    /// The actual response data
    pub results: Option<T>,

    /// Request ID for debugging
    pub request_id: Option<String>,

    /// URL for the next page of results (pagination)
    pub next_url: Option<String>,

    /// Error message (if status is not OK)
    pub error: Option<String>,
}

impl<T> ApiEnvelope<T> {
    /// Unwrap results or return default if None.
    ///
    /// This is useful when you expect results but want to handle
    /// empty responses gracefully.
    pub fn into_results(self) -> T
    where
        T: Default,
    {
        self.results.unwrap_or_default()
    }

    /// Check if the response indicates success.
    pub fn is_ok(&self) -> bool {
        self.status.as_deref() == Some("OK")
    }

    /// Check if there are more pages available.
    pub fn has_next_page(&self) -> bool {
        self.next_url.is_some()
    }
}

/// List endpoint envelope with Vec results.
///
/// This is a specialized version of [`ApiEnvelope`] where results
/// are always a Vec, which is common for list endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct ListEnvelope<T> {
    /// Status string (e.g., "OK", "ERROR")
    pub status: Option<String>,

    /// Request ID for debugging
    pub request_id: Option<String>,

    /// Number of results in this response
    pub count: Option<u64>,

    /// URL for the next page of results
    pub next_url: Option<String>,

    /// The list of results
    #[serde(default)]
    pub results: Vec<T>,
}

impl<T> ListEnvelope<T> {
    /// Check if the response indicates success.
    pub fn is_ok(&self) -> bool {
        self.status.as_deref() == Some("OK")
    }

    /// Check if there are more pages available.
    pub fn has_next_page(&self) -> bool {
        self.next_url.is_some()
    }

    /// Check if results are empty.
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the number of results.
    pub fn len(&self) -> usize {
        self.results.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_api_envelope_deserialize() {
        let json = json!({
            "status": "OK",
            "count": 5,
            "results": [1, 2, 3, 4, 5],
            "request_id": "abc123"
        });

        let envelope: ApiEnvelope<Vec<i32>> = serde_json::from_value(json).unwrap();

        assert!(envelope.is_ok());
        assert_eq!(envelope.count, Some(5));
        assert_eq!(envelope.results, Some(vec![1, 2, 3, 4, 5]));
        assert_eq!(envelope.request_id, Some("abc123".to_string()));
        assert!(!envelope.has_next_page());
    }

    #[test]
    fn test_api_envelope_with_pagination() {
        let json = json!({
            "status": "OK",
            "results": [1, 2, 3],
            "next_url": "https://api.massive.com/v2/aggs?cursor=abc"
        });

        let envelope: ApiEnvelope<Vec<i32>> = serde_json::from_value(json).unwrap();

        assert!(envelope.has_next_page());
        assert_eq!(
            envelope.next_url,
            Some("https://api.massive.com/v2/aggs?cursor=abc".to_string())
        );
    }

    #[test]
    fn test_api_envelope_into_results() {
        let envelope: ApiEnvelope<Vec<i32>> = ApiEnvelope {
            status: Some("OK".to_string()),
            count: None,
            results: Some(vec![1, 2, 3]),
            request_id: None,
            next_url: None,
            error: None,
        };

        let results = envelope.into_results();
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[test]
    fn test_api_envelope_into_results_default() {
        let envelope: ApiEnvelope<Vec<i32>> = ApiEnvelope {
            status: Some("OK".to_string()),
            count: None,
            results: None,
            request_id: None,
            next_url: None,
            error: None,
        };

        let results = envelope.into_results();
        assert!(results.is_empty());
    }

    #[test]
    fn test_list_envelope_deserialize() {
        let json = json!({
            "status": "OK",
            "count": 3,
            "results": ["a", "b", "c"]
        });

        let envelope: ListEnvelope<String> = serde_json::from_value(json).unwrap();

        assert!(envelope.is_ok());
        assert_eq!(envelope.len(), 3);
        assert!(!envelope.is_empty());
        assert_eq!(envelope.results, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_list_envelope_empty() {
        let json = json!({
            "status": "OK"
        });

        let envelope: ListEnvelope<String> = serde_json::from_value(json).unwrap();

        assert!(envelope.is_ok());
        assert!(envelope.is_empty());
        assert_eq!(envelope.len(), 0);
    }
}
