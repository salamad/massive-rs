//! Trade endpoints.
//!
//! This module contains request types for fetching trade data
//! from the Massive API.

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A single trade record.
///
/// Note: The API uses different field names in v2 vs v3 endpoints.
/// This struct accepts both formats using serde aliases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Ticker symbol
    #[serde(
        rename = "T",
        alias = "ticker",
        skip_serializing_if = "Option::is_none"
    )]
    pub ticker: Option<String>,
    /// Trade ID
    #[serde(rename = "i", alias = "id")]
    pub id: Option<String>,
    /// Trade conditions
    #[serde(rename = "c", alias = "conditions", default)]
    pub conditions: Vec<i32>,
    /// Exchange ID
    #[serde(rename = "x", alias = "exchange")]
    pub exchange: Option<u8>,
    /// Price
    #[serde(rename = "p", alias = "price")]
    pub price: f64,
    /// SIP timestamp
    #[serde(rename = "t", alias = "sip_timestamp")]
    pub sip_timestamp: Option<i64>,
    /// Participant timestamp
    #[serde(rename = "y", alias = "participant_timestamp")]
    pub participant_timestamp: Option<i64>,
    /// TRF timestamp
    #[serde(rename = "f", alias = "trf_timestamp")]
    pub trf_timestamp: Option<i64>,
    /// Size
    #[serde(rename = "s", alias = "size")]
    pub size: u64,
    /// Tape (1=NYSE, 2=AMEX, 3=NASDAQ)
    #[serde(rename = "z", alias = "tape")]
    pub tape: Option<u8>,
    /// Sequence number
    #[serde(rename = "q", alias = "sequence_number")]
    pub sequence_number: Option<u64>,
    /// Reporting facility ID / TRF ID
    #[serde(rename = "r", alias = "trf_id")]
    pub reporting_facility: Option<u8>,
}

impl Trade {
    /// Calculate the trade value (price * size).
    pub fn value(&self) -> f64 {
        self.price * self.size as f64
    }
}

/// Request for list of trades.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::GetTradesRequest;
///
/// let request = GetTradesRequest::new("AAPL")
///     .limit(100)
///     .timestamp_gte("2024-01-01");
/// ```
#[derive(Debug, Clone)]
pub struct GetTradesRequest {
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

impl GetTradesRequest {
    /// Create a new trades request.
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

/// Response for trades list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TradesResponse {
    /// Status
    pub status: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// Results
    #[serde(default)]
    pub results: Vec<Trade>,
    /// Next URL for pagination
    pub next_url: Option<String>,
}

impl RestRequest for GetTradesRequest {
    type Response = TradesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/trades/{}", self.ticker).into()
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

impl PaginatableRequest for GetTradesRequest {
    type Item = Trade;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Request for last trade.
///
/// Returns the most recent trade for a ticker symbol.
#[derive(Debug, Clone)]
pub struct GetLastTradeRequest {
    /// Ticker symbol
    pub ticker: String,
}

impl GetLastTradeRequest {
    /// Create a new last trade request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
        }
    }
}

/// Response for last trade endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct LastTradeResponse {
    /// Status
    pub status: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// The last trade
    pub results: Option<Trade>,
}

impl RestRequest for GetLastTradeRequest {
    type Response = LastTradeResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v2/last/trade/{}", self.ticker).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_trades_request_path() {
        let req = GetTradesRequest::new("AAPL");
        assert_eq!(req.path(), "/v3/trades/AAPL");
    }

    #[test]
    fn test_get_trades_request_query() {
        let req = GetTradesRequest::new("AAPL")
            .limit(100)
            .order("desc")
            .timestamp_gte("2024-01-01");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("limit").unwrap(), "100");
        assert_eq!(query_map.get("order").unwrap(), "desc");
        assert_eq!(query_map.get("timestamp.gte").unwrap(), "2024-01-01");
    }

    #[test]
    fn test_get_last_trade_request() {
        let req = GetLastTradeRequest::new("MSFT");
        assert_eq!(req.path(), "/v2/last/trade/MSFT");
    }

    #[test]
    fn test_trade_value() {
        let trade = Trade {
            ticker: Some("AAPL".into()),
            id: Some("123".into()),
            conditions: vec![],
            exchange: Some(4),
            price: 150.50,
            sip_timestamp: Some(1703001234567),
            participant_timestamp: None,
            trf_timestamp: None,
            size: 100,
            tape: Some(3),
            sequence_number: Some(12345),
            reporting_facility: None,
        };

        assert_eq!(trade.value(), 15050.0);
    }

    #[test]
    fn test_trades_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "results": [
                {
                    "i": "trade1",
                    "c": [0],
                    "x": 4,
                    "p": 150.25,
                    "t": 1703001234567,
                    "s": 100,
                    "z": 3
                }
            ],
            "next_url": "https://api.massive.com/v3/trades/AAPL?cursor=abc"
        }"#;

        let response: TradesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, Some("OK".to_string()));
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].price, 150.25);
    }
}
