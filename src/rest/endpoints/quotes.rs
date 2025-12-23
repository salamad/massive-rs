//! Quote endpoints.
//!
//! This module contains request types for fetching quote (NBBO) data
//! from the Massive API.

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A single quote (NBBO) record.
///
/// Note: The API uses different field names in v2 vs v3 endpoints.
/// This struct accepts both formats using serde aliases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Ticker symbol
    #[serde(
        rename = "T",
        alias = "ticker",
        skip_serializing_if = "Option::is_none"
    )]
    pub ticker: Option<String>,
    /// Ask exchange ID
    #[serde(rename = "X", alias = "ask_exchange")]
    pub ask_exchange: Option<u8>,
    /// Ask price
    #[serde(rename = "P", alias = "ask_price")]
    pub ask_price: f64,
    /// Ask size
    #[serde(rename = "S", alias = "ask_size")]
    pub ask_size: u64,
    /// Bid exchange ID
    #[serde(rename = "x", alias = "bid_exchange")]
    pub bid_exchange: Option<u8>,
    /// Bid price
    #[serde(rename = "p", alias = "bid_price")]
    pub bid_price: f64,
    /// Bid size
    #[serde(rename = "s", alias = "bid_size")]
    pub bid_size: u64,
    /// Quote conditions
    #[serde(rename = "c", alias = "conditions", default)]
    pub conditions: Vec<i32>,
    /// SIP timestamp
    #[serde(rename = "t", alias = "sip_timestamp")]
    pub sip_timestamp: Option<i64>,
    /// Participant timestamp
    #[serde(rename = "y", alias = "participant_timestamp")]
    pub participant_timestamp: Option<i64>,
    /// Indicators
    #[serde(rename = "i", alias = "indicators", default)]
    pub indicators: Vec<i32>,
    /// Sequence number
    #[serde(rename = "q", alias = "sequence_number")]
    pub sequence_number: Option<u64>,
    /// Tape
    #[serde(rename = "z", alias = "tape")]
    pub tape: Option<u8>,
}

impl Quote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ask_price - self.bid_price
    }

    /// Calculate the mid price.
    pub fn mid_price(&self) -> f64 {
        (self.ask_price + self.bid_price) / 2.0
    }

    /// Check if the market is crossed (bid > ask).
    pub fn is_crossed(&self) -> bool {
        self.bid_price > self.ask_price
    }

    /// Check if the market is locked (bid == ask).
    pub fn is_locked(&self) -> bool {
        (self.bid_price - self.ask_price).abs() < f64::EPSILON
    }
}

/// Request for list of quotes.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::GetQuotesRequest;
///
/// let request = GetQuotesRequest::new("AAPL")
///     .limit(100)
///     .timestamp_gte("2024-01-01");
/// ```
#[derive(Debug, Clone)]
pub struct GetQuotesRequest {
    /// Ticker symbol
    pub ticker: String,
    /// Timestamp greater than
    pub timestamp_gt: Option<String>,
    /// Timestamp greater than or equal
    pub timestamp_gte: Option<String>,
    /// Timestamp less than
    pub timestamp_lt: Option<String>,
    /// Timestamp less than or equal
    pub timestamp_lte: Option<String>,
    /// Sort direction
    pub order: Option<String>,
    /// Maximum results
    pub limit: Option<u32>,
    /// Sort field
    pub sort: Option<String>,
}

impl GetQuotesRequest {
    /// Create a new quotes request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            timestamp_gt: None,
            timestamp_gte: None,
            timestamp_lt: None,
            timestamp_lte: None,
            order: None,
            limit: None,
            sort: None,
        }
    }

    /// Set timestamp greater than.
    pub fn timestamp_gt(mut self, ts: impl Into<String>) -> Self {
        self.timestamp_gt = Some(ts.into());
        self
    }

    /// Set timestamp greater than or equal.
    pub fn timestamp_gte(mut self, ts: impl Into<String>) -> Self {
        self.timestamp_gte = Some(ts.into());
        self
    }

    /// Set timestamp less than.
    pub fn timestamp_lt(mut self, ts: impl Into<String>) -> Self {
        self.timestamp_lt = Some(ts.into());
        self
    }

    /// Set timestamp less than or equal.
    pub fn timestamp_lte(mut self, ts: impl Into<String>) -> Self {
        self.timestamp_lte = Some(ts.into());
        self
    }

    /// Set sort order ("asc" or "desc").
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Set max results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set sort field.
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }
}

/// Response for quotes list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct QuotesResponse {
    /// Status
    pub status: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// Results
    #[serde(default)]
    pub results: Vec<Quote>,
    /// Next URL for pagination
    pub next_url: Option<String>,
}

impl RestRequest for GetQuotesRequest {
    type Response = QuotesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/quotes/{}", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("timestamp.gt", self.timestamp_gt.clone());
        params.push_opt_param("timestamp.gte", self.timestamp_gte.clone());
        params.push_opt_param("timestamp.lt", self.timestamp_lt.clone());
        params.push_opt_param("timestamp.lte", self.timestamp_lte.clone());
        params.push_opt_param("order", self.order.clone());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("sort", self.sort.clone());
        params
    }
}

impl PaginatableRequest for GetQuotesRequest {
    type Item = Quote;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Request for last quote (NBBO).
///
/// Returns the most recent NBBO for a ticker symbol.
#[derive(Debug, Clone)]
pub struct GetLastQuoteRequest {
    /// Ticker symbol
    pub ticker: String,
}

impl GetLastQuoteRequest {
    /// Create a new last quote request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
        }
    }
}

/// Response for last quote endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct LastQuoteResponse {
    /// Status
    pub status: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// The last quote
    pub results: Option<Quote>,
}

impl RestRequest for GetLastQuoteRequest {
    type Response = LastQuoteResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v2/last/nbbo/{}", self.ticker).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_quotes_request_path() {
        let req = GetQuotesRequest::new("AAPL");
        assert_eq!(req.path(), "/v3/quotes/AAPL");
    }

    #[test]
    fn test_get_quotes_request_query() {
        let req = GetQuotesRequest::new("AAPL").limit(100).order("desc");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("limit").unwrap(), "100");
        assert_eq!(query_map.get("order").unwrap(), "desc");
    }

    #[test]
    fn test_get_last_quote_request() {
        let req = GetLastQuoteRequest::new("MSFT");
        assert_eq!(req.path(), "/v2/last/nbbo/MSFT");
    }

    #[test]
    fn test_quote_calculations() {
        let quote = Quote {
            ticker: Some("AAPL".into()),
            ask_exchange: Some(4),
            ask_price: 150.50,
            ask_size: 100,
            bid_exchange: Some(7),
            bid_price: 150.40,
            bid_size: 200,
            conditions: vec![],
            sip_timestamp: Some(1703001234567),
            participant_timestamp: None,
            indicators: vec![],
            sequence_number: Some(12345),
            tape: Some(3),
        };

        assert!((quote.spread() - 0.10).abs() < 0.001);
        assert!((quote.mid_price() - 150.45).abs() < 0.001);
        assert!(!quote.is_crossed());
        assert!(!quote.is_locked());
    }

    #[test]
    fn test_quote_crossed() {
        let quote = Quote {
            ticker: None,
            ask_exchange: Some(4),
            ask_price: 150.30,
            ask_size: 100,
            bid_exchange: Some(7),
            bid_price: 150.50,
            bid_size: 200,
            conditions: vec![],
            sip_timestamp: None,
            participant_timestamp: None,
            indicators: vec![],
            sequence_number: None,
            tape: None,
        };

        assert!(quote.is_crossed());
    }

    #[test]
    fn test_quotes_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "results": [
                {
                    "X": 4,
                    "P": 150.50,
                    "S": 100,
                    "x": 7,
                    "p": 150.40,
                    "s": 200,
                    "t": 1703001234567
                }
            ],
            "next_url": "https://api.massive.com/v3/quotes/AAPL?cursor=abc"
        }"#;

        let response: QuotesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, Some("OK".to_string()));
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].ask_price, 150.50);
    }
}
