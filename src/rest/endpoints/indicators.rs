//! Technical indicator endpoints.
//!
//! This module contains request types for fetching technical indicators
//! from the Massive API, including:
//!
//! - RSI (Relative Strength Index)
//! - SMA (Simple Moving Average)
//! - EMA (Exponential Moving Average)
//! - MACD (Moving Average Convergence/Divergence)
//!
//! # Generic Indicator Framework
//!
//! The module provides a generic `Indicator` trait that allows type-safe
//! indicator requests with shared behavior.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{
//!     GetSmaRequest, GetEmaRequest, GetMacdRequest, IndicatorTimespan, SeriesType
//! };
//!
//! // SMA with default window
//! let sma = GetSmaRequest::new("AAPL")
//!     .timespan(IndicatorTimespan::Day)
//!     .window(20);
//!
//! // EMA with custom series type
//! let ema = GetEmaRequest::new("MSFT")
//!     .timespan(IndicatorTimespan::Hour)
//!     .window(12)
//!     .series_type(SeriesType::Close);
//!
//! // MACD with standard parameters
//! let macd = GetMacdRequest::new("GOOG")
//!     .standard_macd();
//! ```

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;
use std::marker::PhantomData;

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

// =============================================================================
// Generic Indicator Framework
// =============================================================================

/// Marker trait for indicator types.
///
/// Each indicator type defines its URL path segment and human-readable name.
/// This trait enables generic indicator requests that share the same structure.
pub trait Indicator: Send + Sync + 'static {
    /// The URL path segment for this indicator (e.g., "rsi", "sma", "ema", "macd").
    const PATH_SEGMENT: &'static str;
    /// Human-readable name of the indicator.
    const NAME: &'static str;
}

/// RSI indicator marker.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rsi;

impl Indicator for Rsi {
    const PATH_SEGMENT: &'static str = "rsi";
    const NAME: &'static str = "Relative Strength Index";
}

/// Simple Moving Average indicator marker.
#[derive(Debug, Clone, Copy, Default)]
pub struct Sma;

impl Indicator for Sma {
    const PATH_SEGMENT: &'static str = "sma";
    const NAME: &'static str = "Simple Moving Average";
}

/// Exponential Moving Average indicator marker.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ema;

impl Indicator for Ema {
    const PATH_SEGMENT: &'static str = "ema";
    const NAME: &'static str = "Exponential Moving Average";
}

/// MACD (Moving Average Convergence/Divergence) indicator marker.
#[derive(Debug, Clone, Copy, Default)]
pub struct Macd;

impl Indicator for Macd {
    const PATH_SEGMENT: &'static str = "macd";
    const NAME: &'static str = "Moving Average Convergence/Divergence";
}

// =============================================================================
// Generic Indicator Request
// =============================================================================

/// Generic indicator request supporting SMA, EMA, and similar indicators.
///
/// This request type can be used with any indicator that follows the standard
/// indicator API pattern (window-based calculation on price series).
///
/// # Type Parameters
///
/// - `I`: The indicator type marker (e.g., `Sma`, `Ema`)
///
/// # Example
///
/// ```
/// use massive_rs::rest::{
///     GetIndicatorRequest, Sma, Ema, IndicatorTimespan, SeriesType
/// };
///
/// // Create an SMA request
/// let sma: GetIndicatorRequest<Sma> = GetIndicatorRequest::new("AAPL")
///     .timespan(IndicatorTimespan::Day)
///     .window(20);
///
/// // Create an EMA request
/// let ema: GetIndicatorRequest<Ema> = GetIndicatorRequest::new("MSFT")
///     .timespan(IndicatorTimespan::Hour)
///     .window(12)
///     .series_type(SeriesType::Close);
/// ```
#[derive(Debug, Clone)]
pub struct GetIndicatorRequest<I: Indicator> {
    /// The ticker symbol (required).
    pub ticker: String,
    /// Starting timestamp filter.
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
    /// Calculation window size.
    pub window: Option<u32>,
    /// Price type for calculation.
    pub series_type: Option<SeriesType>,
    /// Whether to include underlying aggregates in response.
    pub expand_underlying: Option<bool>,
    /// Result ordering by timestamp.
    pub order: Option<Order>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Marker for the indicator type.
    _marker: PhantomData<I>,
}

impl<I: Indicator> GetIndicatorRequest<I> {
    /// Create a new indicator request for the given ticker.
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
            _marker: PhantomData,
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

    /// Set the calculation window size.
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

    /// Set a timestamp range filter (convenience method).
    pub fn timestamp_range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.timestamp_gte = Some(from.into());
        self.timestamp_lte = Some(to.into());
        self
    }

    /// Set ascending order (oldest first).
    pub fn order_asc(self) -> Self {
        self.order(Order::Asc)
    }

    /// Set descending order (newest first).
    pub fn order_desc(self) -> Self {
        self.order(Order::Desc)
    }
}

impl<I: Indicator> RestRequest for GetIndicatorRequest<I> {
    type Response = IndicatorResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v1/indicators/{}/{}", I::PATH_SEGMENT, self.ticker).into()
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

impl<I: Indicator + Clone> PaginatableRequest for GetIndicatorRequest<I> {
    type Item = IndicatorValue;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results.values
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Response from standard indicator endpoints (SMA, EMA, etc.).
#[derive(Debug, Clone, Deserialize)]
pub struct IndicatorResponse {
    /// URL for the next page of results.
    pub next_url: Option<String>,
    /// Request ID for debugging.
    pub request_id: Option<String>,
    /// Indicator results.
    pub results: IndicatorResults,
    /// Response status.
    pub status: Option<String>,
}

/// Standard indicator calculation results.
#[derive(Debug, Clone, Deserialize)]
pub struct IndicatorResults {
    /// Underlying aggregates data reference.
    #[serde(default)]
    pub underlying: Option<UnderlyingReference>,
    /// Indicator values.
    #[serde(default)]
    pub values: Vec<IndicatorValue>,
}

/// A single indicator value at a point in time.
#[derive(Debug, Clone, Deserialize)]
pub struct IndicatorValue {
    /// Timestamp in Unix milliseconds.
    pub timestamp: i64,
    /// Indicator value.
    pub value: f64,
}

impl IndicatorValue {
    /// Create a new indicator value.
    pub fn new(timestamp: i64, value: f64) -> Self {
        Self { timestamp, value }
    }
}

/// Type alias for SMA requests.
pub type GetSmaRequest = GetIndicatorRequest<Sma>;

/// Type alias for EMA requests.
pub type GetEmaRequest = GetIndicatorRequest<Ema>;

// =============================================================================
// MACD (Moving Average Convergence/Divergence)
// =============================================================================

/// Request for MACD indicator values.
///
/// MACD is calculated using three EMAs with configurable windows:
/// - Short window (default: 12)
/// - Long window (default: 26)
/// - Signal window (default: 9)
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetMacdRequest, IndicatorTimespan};
///
/// // Standard MACD (12, 26, 9)
/// let macd = GetMacdRequest::new("AAPL")
///     .standard_macd()
///     .timespan(IndicatorTimespan::Day);
///
/// // Custom MACD parameters
/// let custom_macd = GetMacdRequest::new("MSFT")
///     .short_window(8)
///     .long_window(21)
///     .signal_window(5);
/// ```
#[derive(Debug, Clone)]
pub struct GetMacdRequest {
    /// The ticker symbol (required).
    pub ticker: String,
    /// Starting timestamp filter.
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
    /// Short EMA window (default: 12).
    pub short_window: Option<u32>,
    /// Long EMA window (default: 26).
    pub long_window: Option<u32>,
    /// Signal line EMA window (default: 9).
    pub signal_window: Option<u32>,
    /// Price type for calculation.
    pub series_type: Option<SeriesType>,
    /// Whether to include underlying aggregates in response.
    pub expand_underlying: Option<bool>,
    /// Result ordering by timestamp.
    pub order: Option<Order>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl GetMacdRequest {
    /// Create a new MACD request for the given ticker.
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
            short_window: None,
            long_window: None,
            signal_window: None,
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

    /// Set whether to adjust for splits.
    pub fn adjusted(mut self, adjusted: bool) -> Self {
        self.adjusted = Some(adjusted);
        self
    }

    /// Set the short EMA window (default: 12).
    pub fn short_window(mut self, window: u32) -> Self {
        self.short_window = Some(window);
        self
    }

    /// Set the long EMA window (default: 26).
    pub fn long_window(mut self, window: u32) -> Self {
        self.long_window = Some(window);
        self
    }

    /// Set the signal line EMA window (default: 9).
    pub fn signal_window(mut self, window: u32) -> Self {
        self.signal_window = Some(window);
        self
    }

    /// Set standard MACD parameters (12, 26, 9).
    pub fn standard_macd(self) -> Self {
        self.short_window(12).long_window(26).signal_window(9)
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

    /// Set a timestamp range filter (convenience method).
    pub fn timestamp_range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.timestamp_gte = Some(from.into());
        self.timestamp_lte = Some(to.into());
        self
    }

    /// Set ascending order (oldest first).
    pub fn order_asc(self) -> Self {
        self.order(Order::Asc)
    }

    /// Set descending order (newest first).
    pub fn order_desc(self) -> Self {
        self.order(Order::Desc)
    }
}

impl RestRequest for GetMacdRequest {
    type Response = MacdResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v1/indicators/macd/{}", self.ticker).into()
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
        params.push_opt_param("short_window", self.short_window);
        params.push_opt_param("long_window", self.long_window);
        params.push_opt_param("signal_window", self.signal_window);
        params.push_opt_param("series_type", self.series_type.map(|s| s.to_string()));
        params.push_opt_param("expand_underlying", self.expand_underlying);
        params.push_opt_param("order", self.order.map(|o| o.to_string()));
        params.push_opt_param("limit", self.limit);
        params
    }
}

impl PaginatableRequest for GetMacdRequest {
    type Item = MacdValue;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results.values
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Response from the MACD indicator endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct MacdResponse {
    /// URL for the next page of results.
    pub next_url: Option<String>,
    /// Request ID for debugging.
    pub request_id: Option<String>,
    /// MACD results.
    pub results: MacdResults,
    /// Response status.
    pub status: Option<String>,
}

/// MACD calculation results.
#[derive(Debug, Clone, Deserialize)]
pub struct MacdResults {
    /// Underlying aggregates data reference.
    #[serde(default)]
    pub underlying: Option<UnderlyingReference>,
    /// MACD values.
    #[serde(default)]
    pub values: Vec<MacdValue>,
}

/// A single MACD value at a point in time.
///
/// MACD consists of three components:
/// - **value**: The MACD line (short EMA - long EMA)
/// - **signal**: The signal line (EMA of the MACD line)
/// - **histogram**: The histogram (MACD line - signal line)
#[derive(Debug, Clone, Deserialize)]
pub struct MacdValue {
    /// Timestamp in Unix milliseconds.
    pub timestamp: i64,
    /// MACD line value (short EMA - long EMA).
    pub value: f64,
    /// Signal line value (EMA of the MACD line).
    pub signal: f64,
    /// Histogram value (MACD line - signal line).
    pub histogram: f64,
}

impl MacdValue {
    /// Create a new MACD value.
    pub fn new(timestamp: i64, value: f64, signal: f64, histogram: f64) -> Self {
        Self {
            timestamp,
            value,
            signal,
            histogram,
        }
    }

    /// Check if MACD line is above signal line (bullish).
    pub fn is_bullish(&self) -> bool {
        self.value > self.signal
    }

    /// Check if MACD line is below signal line (bearish).
    pub fn is_bearish(&self) -> bool {
        self.value < self.signal
    }

    /// Check if histogram is positive (bullish momentum).
    pub fn has_bullish_momentum(&self) -> bool {
        self.histogram > 0.0
    }

    /// Check if histogram is negative (bearish momentum).
    pub fn has_bearish_momentum(&self) -> bool {
        self.histogram < 0.0
    }

    /// Check if both MACD and signal are above zero (strong bullish).
    pub fn is_strong_bullish(&self) -> bool {
        self.value > 0.0 && self.signal > 0.0 && self.is_bullish()
    }

    /// Check if both MACD and signal are below zero (strong bearish).
    pub fn is_strong_bearish(&self) -> bool {
        self.value < 0.0 && self.signal < 0.0 && self.is_bearish()
    }

    /// Check if there's a bullish crossover (MACD crosses above signal).
    ///
    /// Note: This method checks the current state. To detect actual crossovers,
    /// compare consecutive values.
    pub fn at_bullish_crossover(&self) -> bool {
        self.histogram > 0.0 && self.histogram.abs() < 0.01 * self.value.abs().max(0.01)
    }

    /// Check if there's a bearish crossover (MACD crosses below signal).
    ///
    /// Note: This method checks the current state. To detect actual crossovers,
    /// compare consecutive values.
    pub fn at_bearish_crossover(&self) -> bool {
        self.histogram < 0.0 && self.histogram.abs() < 0.01 * self.value.abs().max(0.01)
    }
}

#[cfg(test)]
mod generic_indicator_tests {
    use super::*;

    #[test]
    fn test_indicator_trait() {
        assert_eq!(Rsi::PATH_SEGMENT, "rsi");
        assert_eq!(Rsi::NAME, "Relative Strength Index");
        assert_eq!(Sma::PATH_SEGMENT, "sma");
        assert_eq!(Sma::NAME, "Simple Moving Average");
        assert_eq!(Ema::PATH_SEGMENT, "ema");
        assert_eq!(Ema::NAME, "Exponential Moving Average");
        assert_eq!(Macd::PATH_SEGMENT, "macd");
        assert_eq!(Macd::NAME, "Moving Average Convergence/Divergence");
    }

    #[test]
    fn test_sma_request_path() {
        let req = GetSmaRequest::new("AAPL");
        assert_eq!(req.path(), "/v1/indicators/sma/AAPL");
        assert_eq!(req.method(), Method::GET);
    }

    #[test]
    fn test_ema_request_path() {
        let req = GetEmaRequest::new("MSFT");
        assert_eq!(req.path(), "/v1/indicators/ema/MSFT");
    }

    #[test]
    fn test_generic_indicator_request_query() {
        let req = GetSmaRequest::new("AAPL")
            .timespan(IndicatorTimespan::Day)
            .window(20)
            .series_type(SeriesType::Close)
            .adjusted(true)
            .order(Order::Desc)
            .limit(100);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("timespan").unwrap(), "day");
        assert_eq!(query_map.get("window").unwrap(), "20");
        assert_eq!(query_map.get("series_type").unwrap(), "close");
        assert_eq!(query_map.get("adjusted").unwrap(), "true");
        assert_eq!(query_map.get("order").unwrap(), "desc");
        assert_eq!(query_map.get("limit").unwrap(), "100");
    }

    #[test]
    fn test_generic_indicator_timestamp_range() {
        let req = GetEmaRequest::new("GOOG").timestamp_range("2024-01-01", "2024-12-31");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("timestamp.gte").unwrap(), "2024-01-01");
        assert_eq!(query_map.get("timestamp.lte").unwrap(), "2024-12-31");
    }

    #[test]
    fn test_indicator_response_deserialize() {
        let json = r#"{
            "request_id": "abc123",
            "status": "OK",
            "results": {
                "values": [
                    {"timestamp": 1705320000000, "value": 150.25},
                    {"timestamp": 1705406400000, "value": 151.50}
                ]
            }
        }"#;

        let response: IndicatorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status.as_deref(), Some("OK"));
        assert_eq!(response.results.values.len(), 2);
        assert_eq!(response.results.values[0].timestamp, 1705320000000);
        assert!((response.results.values[0].value - 150.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_indicator_value_new() {
        let value = IndicatorValue::new(1705320000000, 42.5);
        assert_eq!(value.timestamp, 1705320000000);
        assert!((value.value - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_macd_request_path() {
        let req = GetMacdRequest::new("AAPL");
        assert_eq!(req.path(), "/v1/indicators/macd/AAPL");
        assert_eq!(req.method(), Method::GET);
    }

    #[test]
    fn test_macd_request_standard() {
        let req = GetMacdRequest::new("AAPL").standard_macd();

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("short_window").unwrap(), "12");
        assert_eq!(query_map.get("long_window").unwrap(), "26");
        assert_eq!(query_map.get("signal_window").unwrap(), "9");
    }

    #[test]
    fn test_macd_request_custom_windows() {
        let req = GetMacdRequest::new("MSFT")
            .short_window(8)
            .long_window(21)
            .signal_window(5)
            .timespan(IndicatorTimespan::Hour);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("short_window").unwrap(), "8");
        assert_eq!(query_map.get("long_window").unwrap(), "21");
        assert_eq!(query_map.get("signal_window").unwrap(), "5");
        assert_eq!(query_map.get("timespan").unwrap(), "hour");
    }

    #[test]
    fn test_macd_response_deserialize() {
        let json = r#"{
            "request_id": "macd123",
            "status": "OK",
            "results": {
                "values": [
                    {"timestamp": 1705320000000, "value": 1.5, "signal": 1.2, "histogram": 0.3},
                    {"timestamp": 1705406400000, "value": 1.8, "signal": 1.4, "histogram": 0.4}
                ]
            }
        }"#;

        let response: MacdResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status.as_deref(), Some("OK"));
        assert_eq!(response.results.values.len(), 2);

        let first = &response.results.values[0];
        assert_eq!(first.timestamp, 1705320000000);
        assert!((first.value - 1.5).abs() < f64::EPSILON);
        assert!((first.signal - 1.2).abs() < f64::EPSILON);
        assert!((first.histogram - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_macd_value_bullish() {
        let bullish = MacdValue::new(0, 1.5, 1.2, 0.3);
        assert!(bullish.is_bullish());
        assert!(!bullish.is_bearish());
        assert!(bullish.has_bullish_momentum());
        assert!(!bullish.has_bearish_momentum());
    }

    #[test]
    fn test_macd_value_bearish() {
        let bearish = MacdValue::new(0, 1.0, 1.3, -0.3);
        assert!(!bearish.is_bullish());
        assert!(bearish.is_bearish());
        assert!(!bearish.has_bullish_momentum());
        assert!(bearish.has_bearish_momentum());
    }

    #[test]
    fn test_macd_value_strong_bullish() {
        let strong_bullish = MacdValue::new(0, 2.0, 1.5, 0.5);
        assert!(strong_bullish.is_strong_bullish());
        assert!(!strong_bullish.is_strong_bearish());
    }

    #[test]
    fn test_macd_value_strong_bearish() {
        let strong_bearish = MacdValue::new(0, -2.0, -1.5, -0.5);
        assert!(!strong_bearish.is_strong_bullish());
        assert!(strong_bearish.is_strong_bearish());
    }

    #[test]
    fn test_macd_value_new() {
        let value = MacdValue::new(1705320000000, 1.5, 1.2, 0.3);
        assert_eq!(value.timestamp, 1705320000000);
        assert!((value.value - 1.5).abs() < f64::EPSILON);
        assert!((value.signal - 1.2).abs() < f64::EPSILON);
        assert!((value.histogram - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generic_order_helpers() {
        let asc_req = GetSmaRequest::new("AAPL").order_asc();
        let desc_req = GetEmaRequest::new("MSFT").order_desc();

        assert_eq!(asc_req.order, Some(Order::Asc));
        assert_eq!(desc_req.order, Some(Order::Desc));
    }

    #[test]
    fn test_macd_order_helpers() {
        let asc_req = GetMacdRequest::new("AAPL").order_asc();
        let desc_req = GetMacdRequest::new("MSFT").order_desc();

        assert_eq!(asc_req.order, Some(Order::Asc));
        assert_eq!(desc_req.order, Some(Order::Desc));
    }
}
