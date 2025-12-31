//! Technical indicator endpoints.
//!
//! This module contains request types for fetching technical indicators
//! like RSI (Relative Strength Index) from the Massive API.

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

/// Timespan for indicator aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorTimespan {
    /// Minute-level aggregation.
    Minute,
    /// Hour-level aggregation.
    Hour,
    /// Day-level aggregation.
    Day,
    /// Week-level aggregation.
    Week,
    /// Month-level aggregation.
    Month,
    /// Quarter-level aggregation.
    Quarter,
    /// Year-level aggregation.
    Year,
}

impl std::fmt::Display for IndicatorTimespan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndicatorTimespan::Minute => write!(f, "minute"),
            IndicatorTimespan::Hour => write!(f, "hour"),
            IndicatorTimespan::Day => write!(f, "day"),
            IndicatorTimespan::Week => write!(f, "week"),
            IndicatorTimespan::Month => write!(f, "month"),
            IndicatorTimespan::Quarter => write!(f, "quarter"),
            IndicatorTimespan::Year => write!(f, "year"),
        }
    }
}

/// Price series type for indicator calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesType {
    /// Opening price.
    Open,
    /// Highest price.
    High,
    /// Lowest price.
    Low,
    /// Closing price.
    Close,
}

impl std::fmt::Display for SeriesType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeriesType::Open => write!(f, "open"),
            SeriesType::High => write!(f, "high"),
            SeriesType::Low => write!(f, "low"),
            SeriesType::Close => write!(f, "close"),
        }
    }
}

/// Sort order for results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Ascending order (oldest first).
    Asc,
    /// Descending order (newest first).
    Desc,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Asc => write!(f, "asc"),
            Order::Desc => write!(f, "desc"),
        }
    }
}

/// Request for RSI (Relative Strength Index) indicator values.
///
/// Fetches RSI values for a given stock ticker over a specified time range.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::{GetRsiRequest, IndicatorTimespan, SeriesType};
///
/// let request = GetRsiRequest::new("AAPL")
///     .timespan(IndicatorTimespan::Day)
///     .window(14)
///     .series_type(SeriesType::Close)
///     .limit(50);
/// ```
#[derive(Debug, Clone)]
pub struct GetRsiRequest {
    /// The ticker symbol (required).
    pub ticker: String,
    /// Starting timestamp (YYYY-MM-DD or milliseconds).
    pub timestamp: Option<String>,
    /// Timestamp greater than filter.
    pub timestamp_gt: Option<String>,
    /// Timestamp greater than or equal filter.
    pub timestamp_gte: Option<String>,
    /// Timestamp less than filter.
    pub timestamp_lt: Option<String>,
    /// Timestamp less than or equal filter.
    pub timestamp_lte: Option<String>,
    /// Aggregate time window size.
    pub timespan: Option<IndicatorTimespan>,
    /// Whether to adjust for splits.
    pub adjusted: Option<bool>,
    /// RSI calculation window size (typically 14).
    pub window: Option<u32>,
    /// Price type for calculation.
    pub series_type: Option<SeriesType>,
    /// Whether to include underlying aggregates in response.
    pub expand_underlying: Option<bool>,
    /// Result ordering by timestamp.
    pub order: Option<Order>,
    /// Maximum number of results (default 10, max 5000).
    pub limit: Option<u32>,
}

impl GetRsiRequest {
    /// Create a new RSI request for the given ticker.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            timestamp: None,
            timestamp_gt: None,
            timestamp_gte: None,
            timestamp_lt: None,
            timestamp_lte: None,
            timespan: None,
            adjusted: None,
            window: None,
            series_type: None,
            expand_underlying: None,
            order: None,
            limit: None,
        }
    }

    /// Set the starting timestamp.
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Filter for timestamps greater than the given value.
    pub fn timestamp_gt(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp_gt = Some(timestamp.into());
        self
    }

    /// Filter for timestamps greater than or equal to the given value.
    pub fn timestamp_gte(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp_gte = Some(timestamp.into());
        self
    }

    /// Filter for timestamps less than the given value.
    pub fn timestamp_lt(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp_lt = Some(timestamp.into());
        self
    }

    /// Filter for timestamps less than or equal to the given value.
    pub fn timestamp_lte(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp_lte = Some(timestamp.into());
        self
    }

    /// Set the aggregation timespan.
    pub fn timespan(mut self, timespan: IndicatorTimespan) -> Self {
        self.timespan = Some(timespan);
        self
    }

    /// Set whether to adjust for splits (default: true).
    pub fn adjusted(mut self, adjusted: bool) -> Self {
        self.adjusted = Some(adjusted);
        self
    }

    /// Set the RSI calculation window size (typically 14).
    pub fn window(mut self, window: u32) -> Self {
        self.window = Some(window);
        self
    }

    /// Set the price series type for calculation.
    pub fn series_type(mut self, series_type: SeriesType) -> Self {
        self.series_type = Some(series_type);
        self
    }

    /// Set whether to include underlying aggregates in response.
    pub fn expand_underlying(mut self, expand: bool) -> Self {
        self.expand_underlying = Some(expand);
        self
    }

    /// Set the result ordering.
    pub fn order(mut self, order: Order) -> Self {
        self.order = Some(order);
        self
    }

    /// Set the maximum number of results.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetRsiRequest {
    type Response = RsiResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v1/indicators/rsi/{}", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("timestamp", self.timestamp.clone());
        params.push_opt_param("timestamp.gt", self.timestamp_gt.clone());
        params.push_opt_param("timestamp.gte", self.timestamp_gte.clone());
        params.push_opt_param("timestamp.lt", self.timestamp_lt.clone());
        params.push_opt_param("timestamp.lte", self.timestamp_lte.clone());
        params.push_opt_param("timespan", self.timespan.map(|t| t.to_string()));
        params.push_opt_param("adjusted", self.adjusted);
        params.push_opt_param("window", self.window);
        params.push_opt_param("series_type", self.series_type.map(|s| s.to_string()));
        params.push_opt_param("expand_underlying", self.expand_underlying);
        params.push_opt_param("order", self.order.map(|o| o.to_string()));
        params.push_opt_param("limit", self.limit);
        params
    }
}

impl PaginatableRequest for GetRsiRequest {
    type Item = RsiValue;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results.values
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Response from the RSI indicator endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct RsiResponse {
    /// URL for the next page of results.
    pub next_url: Option<String>,
    /// Request ID for debugging.
    pub request_id: Option<String>,
    /// RSI results.
    pub results: RsiResults,
    /// Response status.
    pub status: Option<String>,
}

/// RSI calculation results.
#[derive(Debug, Clone, Deserialize)]
pub struct RsiResults {
    /// Underlying aggregates data reference.
    #[serde(default)]
    pub underlying: Option<UnderlyingReference>,
    /// RSI values.
    #[serde(default)]
    pub values: Vec<RsiValue>,
}

/// Reference to underlying aggregates.
#[derive(Debug, Clone, Deserialize)]
pub struct UnderlyingReference {
    /// URL to fetch the underlying aggregates.
    pub url: Option<String>,
}

/// A single RSI value at a point in time.
#[derive(Debug, Clone, Deserialize)]
pub struct RsiValue {
    /// Timestamp in Unix milliseconds.
    pub timestamp: i64,
    /// RSI value (0-100).
    pub value: f64,
}

impl RsiValue {
    /// Returns true if the RSI indicates oversold conditions (below 30).
    pub fn is_oversold(&self) -> bool {
        self.value < 30.0
    }

    /// Returns true if the RSI indicates overbought conditions (above 70).
    pub fn is_overbought(&self) -> bool {
        self.value > 70.0
    }

    /// Returns true if the RSI is in neutral territory (30-70).
    pub fn is_neutral(&self) -> bool {
        self.value >= 30.0 && self.value <= 70.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_timespan_display() {
        assert_eq!(IndicatorTimespan::Minute.to_string(), "minute");
        assert_eq!(IndicatorTimespan::Hour.to_string(), "hour");
        assert_eq!(IndicatorTimespan::Day.to_string(), "day");
        assert_eq!(IndicatorTimespan::Week.to_string(), "week");
        assert_eq!(IndicatorTimespan::Month.to_string(), "month");
        assert_eq!(IndicatorTimespan::Quarter.to_string(), "quarter");
        assert_eq!(IndicatorTimespan::Year.to_string(), "year");
    }

    #[test]
    fn test_series_type_display() {
        assert_eq!(SeriesType::Open.to_string(), "open");
        assert_eq!(SeriesType::High.to_string(), "high");
        assert_eq!(SeriesType::Low.to_string(), "low");
        assert_eq!(SeriesType::Close.to_string(), "close");
    }

    #[test]
    fn test_order_display() {
        assert_eq!(Order::Asc.to_string(), "asc");
        assert_eq!(Order::Desc.to_string(), "desc");
    }

    #[test]
    fn test_get_rsi_request_path() {
        let req = GetRsiRequest::new("AAPL");
        assert_eq!(req.path(), "/v1/indicators/rsi/AAPL");
        assert_eq!(req.method(), Method::GET);
    }

    #[test]
    fn test_get_rsi_request_query() {
        let req = GetRsiRequest::new("AAPL")
            .timespan(IndicatorTimespan::Day)
            .window(14)
            .series_type(SeriesType::Close)
            .adjusted(true)
            .order(Order::Desc)
            .limit(50);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("timespan").unwrap(), "day");
        assert_eq!(query_map.get("window").unwrap(), "14");
        assert_eq!(query_map.get("series_type").unwrap(), "close");
        assert_eq!(query_map.get("adjusted").unwrap(), "true");
        assert_eq!(query_map.get("order").unwrap(), "desc");
        assert_eq!(query_map.get("limit").unwrap(), "50");
    }

    #[test]
    fn test_get_rsi_request_timestamp_filters() {
        let req = GetRsiRequest::new("AAPL")
            .timestamp_gte("2024-01-01")
            .timestamp_lt("2024-02-01");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("timestamp.gte").unwrap(), "2024-01-01");
        assert_eq!(query_map.get("timestamp.lt").unwrap(), "2024-02-01");
    }

    #[test]
    fn test_rsi_response_deserialize() {
        let json = r#"{
            "request_id": "abc123",
            "status": "OK",
            "results": {
                "values": [
                    {"timestamp": 1705320000000, "value": 45.5},
                    {"timestamp": 1705406400000, "value": 52.3}
                ]
            }
        }"#;

        let response: RsiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status.as_deref(), Some("OK"));
        assert_eq!(response.request_id.as_deref(), Some("abc123"));
        assert_eq!(response.results.values.len(), 2);
        assert_eq!(response.results.values[0].timestamp, 1705320000000);
        assert!((response.results.values[0].value - 45.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rsi_response_with_next_url() {
        let json = r#"{
            "request_id": "abc123",
            "status": "OK",
            "next_url": "https://api.massive.com/v1/indicators/rsi/AAPL?cursor=xyz",
            "results": {
                "underlying": {
                    "url": "https://api.massive.com/v2/aggs/ticker/AAPL/range/1/day/2024-01-01/2024-01-31"
                },
                "values": [
                    {"timestamp": 1705320000000, "value": 28.5}
                ]
            }
        }"#;

        let response: RsiResponse = serde_json::from_str(json).unwrap();
        assert!(response.next_url.is_some());
        assert!(response.results.underlying.is_some());
        assert!(response.results.values[0].is_oversold());
    }

    #[test]
    fn test_rsi_value_conditions() {
        let oversold = RsiValue {
            timestamp: 0,
            value: 25.0,
        };
        assert!(oversold.is_oversold());
        assert!(!oversold.is_overbought());
        assert!(!oversold.is_neutral());

        let overbought = RsiValue {
            timestamp: 0,
            value: 75.0,
        };
        assert!(!overbought.is_oversold());
        assert!(overbought.is_overbought());
        assert!(!overbought.is_neutral());

        let neutral = RsiValue {
            timestamp: 0,
            value: 50.0,
        };
        assert!(!neutral.is_oversold());
        assert!(!neutral.is_overbought());
        assert!(neutral.is_neutral());
    }
}
