//! REST API request trait and builders.
//!
//! This module defines the [`RestRequest`] trait that all API requests
//! must implement, along with the [`PaginatableRequest`] trait for
//! endpoints that support pagination.

use bytes::Bytes;
use reqwest::Method;
use serde::de::DeserializeOwned;
use std::borrow::Cow;

/// Trait for typed REST API requests.
///
/// Implementations of this trait define the structure of API requests,
/// including the HTTP method, path, query parameters, and response type.
///
/// # Example
///
/// ```ignore
/// use massive_rs::rest::request::RestRequest;
/// use reqwest::Method;
/// use std::borrow::Cow;
///
/// struct GetTickerDetailsRequest {
///     ticker: String,
/// }
///
/// impl RestRequest for GetTickerDetailsRequest {
///     type Response = TickerDetails;
///
///     fn method(&self) -> Method {
///         Method::GET
///     }
///
///     fn path(&self) -> Cow<'static, str> {
///         format!("/v3/reference/tickers/{}", self.ticker).into()
///     }
/// }
/// ```
pub trait RestRequest: Send + Sync {
    /// Response type for this request.
    type Response: DeserializeOwned + Send + 'static;

    /// HTTP method for this request.
    fn method(&self) -> Method;

    /// Request path (with path parameters interpolated).
    ///
    /// The path should start with `/` and not include the base URL.
    fn path(&self) -> Cow<'static, str>;

    /// Query parameters for the request.
    ///
    /// Returns a vector of key-value pairs that will be appended to the URL.
    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        Vec::new()
    }

    /// Request body (JSON).
    ///
    /// Returns `Some` for POST/PUT/PATCH requests with a body.
    fn body(&self) -> Option<Bytes> {
        None
    }

    /// Whether this request is idempotent (safe to retry).
    ///
    /// By default, GET/HEAD/OPTIONS are considered idempotent.
    fn idempotent(&self) -> bool {
        matches!(self.method(), Method::GET | Method::HEAD | Method::OPTIONS)
    }
}

/// Trait for requests that support pagination.
///
/// Paginated endpoints return results with a `next_url` field that
/// points to the next page of results.
///
/// # Example
///
/// ```ignore
/// impl PaginatableRequest for GetAggsRequest {
///     type Item = AggregateBar;
///
///     fn extract_items(response: Self::Response) -> Vec<Self::Item> {
///         response.results
///     }
///
///     fn extract_next_url(response: &Self::Response) -> Option<&str> {
///         response.next_url.as_deref()
///     }
/// }
/// ```
pub trait PaginatableRequest: RestRequest + Clone {
    /// Item type yielded by pagination.
    type Item: DeserializeOwned + Send + 'static;

    /// Extract items from the response.
    fn extract_items(response: Self::Response) -> Vec<Self::Item>;

    /// Extract the next page URL from the response.
    ///
    /// Returns `None` when there are no more pages.
    fn extract_next_url(response: &Self::Response) -> Option<&str>;
}

/// Helper trait for building query parameters.
pub trait QueryBuilder {
    /// Add a required parameter.
    fn push_param(&mut self, key: &'static str, value: impl ToString);

    /// Add an optional parameter.
    fn push_opt_param<T: ToString>(&mut self, key: &'static str, value: Option<T>);
}

impl QueryBuilder for Vec<(Cow<'static, str>, String)> {
    fn push_param(&mut self, key: &'static str, value: impl ToString) {
        self.push((Cow::Borrowed(key), value.to_string()));
    }

    fn push_opt_param<T: ToString>(&mut self, key: &'static str, value: Option<T>) {
        if let Some(v) = value {
            self.push((Cow::Borrowed(key), v.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct TestResponse {
        value: String,
    }

    struct TestRequest {
        id: u32,
        limit: Option<u32>,
    }

    impl RestRequest for TestRequest {
        type Response = TestResponse;

        fn method(&self) -> Method {
            Method::GET
        }

        fn path(&self) -> Cow<'static, str> {
            format!("/test/{}", self.id).into()
        }

        fn query(&self) -> Vec<(Cow<'static, str>, String)> {
            let mut params = Vec::new();
            params.push_opt_param("limit", self.limit);
            params
        }
    }

    #[test]
    fn test_request_path() {
        let req = TestRequest {
            id: 42,
            limit: None,
        };
        assert_eq!(req.path(), "/test/42");
    }

    #[test]
    fn test_request_query() {
        let req = TestRequest {
            id: 1,
            limit: Some(100),
        };
        let query = req.query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].0, "limit");
        assert_eq!(query[0].1, "100");
    }

    #[test]
    fn test_request_idempotent() {
        let req = TestRequest { id: 1, limit: None };
        assert!(req.idempotent());
    }

    #[test]
    fn test_query_builder() {
        let mut params: Vec<(Cow<'static, str>, String)> = Vec::new();
        params.push_param("key", "value");
        params.push_opt_param("optional", Some(42));
        params.push_opt_param::<String>("missing", None);

        assert_eq!(params.len(), 2);
        assert_eq!(params[0], (Cow::Borrowed("key"), "value".to_string()));
        assert_eq!(params[1], (Cow::Borrowed("optional"), "42".to_string()));
    }
}
