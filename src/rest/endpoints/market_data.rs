//! Market data endpoints (aggregates, trades, quotes).
//!
//! This module contains request types for fetching market data
//! from the Massive API.

use crate::models::AggregateBar;
use crate::rest::models::ListEnvelope;
use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Timespan granularity for aggregate bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Timespan {
    /// Second-level bars
    Second,
    /// Minute-level bars
    #[default]
    Minute,
    /// Hour-level bars
    Hour,
    /// Day-level bars
    Day,
    /// Week-level bars
    Week,
    /// Month-level bars
    Month,
    /// Quarter-level bars
    Quarter,
    /// Year-level bars
    Year,
}

impl std::fmt::Display for Timespan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Timespan::Second => write!(f, "second"),
            Timespan::Minute => write!(f, "minute"),
            Timespan::Hour => write!(f, "hour"),
            Timespan::Day => write!(f, "day"),
            Timespan::Week => write!(f, "week"),
            Timespan::Month => write!(f, "month"),
            Timespan::Quarter => write!(f, "quarter"),
            Timespan::Year => write!(f, "year"),
        }
    }
}

/// Sort direction for results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Ascending order (oldest first)
    #[default]
    Asc,
    /// Descending order (newest first)
    Desc,
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sort::Asc => write!(f, "asc"),
            Sort::Desc => write!(f, "desc"),
        }
    }
}

/// Request for aggregate bars (OHLCV data).
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::GetAggsRequest;
/// use massive_rs::rest::endpoints::Timespan;
///
/// let request = GetAggsRequest::new("AAPL")
///     .multiplier(1)
///     .timespan(Timespan::Day)
///     .from("2024-01-01")
///     .to("2024-01-31");
/// ```
#[derive(Debug, Clone)]
pub struct GetAggsRequest {
    /// Ticker symbol
    pub ticker: String,
    /// Multiplier for the timespan (e.g., 5 for 5-minute bars)
    pub multiplier: u32,
    /// Timespan granularity
    pub timespan: Timespan,
    /// Start date/timestamp
    pub from: String,
    /// End date/timestamp
    pub to: String,
    /// Whether prices are adjusted for splits
    pub adjusted: Option<bool>,
    /// Sort direction
    pub sort: Option<Sort>,
    /// Max results per page
    pub limit: Option<u32>,
}

impl GetAggsRequest {
    /// Create a new aggregates request with defaults.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            multiplier: 1,
            timespan: Timespan::Day,
            from: String::new(),
            to: String::new(),
            adjusted: None,
            sort: None,
            limit: None,
        }
    }

    /// Set the multiplier.
    pub fn multiplier(mut self, multiplier: u32) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Set the timespan.
    pub fn timespan(mut self, timespan: Timespan) -> Self {
        self.timespan = timespan;
        self
    }

    /// Set the start date (YYYY-MM-DD or Unix timestamp).
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = from.into();
        self
    }

    /// Set the end date (YYYY-MM-DD or Unix timestamp).
    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = to.into();
        self
    }

    /// Set whether prices are adjusted for splits.
    pub fn adjusted(mut self, adjusted: bool) -> Self {
        self.adjusted = Some(adjusted);
        self
    }

    /// Set the sort direction.
    pub fn sort(mut self, sort: Sort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Set the maximum results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Response from aggregates endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct AggsResponse {
    /// Status string
    pub status: Option<String>,
    /// Ticker symbol
    pub ticker: Option<String>,
    /// Query count
    #[serde(rename = "queryCount")]
    pub query_count: Option<u64>,
    /// Results count
    #[serde(rename = "resultsCount")]
    pub results_count: Option<u64>,
    /// Whether data is adjusted
    pub adjusted: Option<bool>,
    /// Request ID
    pub request_id: Option<String>,
    /// Results
    #[serde(default)]
    pub results: Vec<AggregateBar>,
    /// Next page URL
    pub next_url: Option<String>,
}

impl RestRequest for GetAggsRequest {
    type Response = AggsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
            self.ticker, self.multiplier, self.timespan, self.from, self.to
        )
        .into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("adjusted", self.adjusted);
        params.push_opt_param("sort", self.sort.map(|s| s.to_string()));
        params.push_opt_param("limit", self.limit);
        params
    }
}

impl PaginatableRequest for GetAggsRequest {
    type Item = AggregateBar;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Request for previous day's data.
///
/// Returns the open, high, low, and close (OHLC) for a ticker symbol
/// on the previous trading day.
#[derive(Debug, Clone)]
pub struct GetPreviousCloseRequest {
    /// Ticker symbol
    pub ticker: String,
    /// Whether prices are adjusted for splits
    pub adjusted: Option<bool>,
}

impl GetPreviousCloseRequest {
    /// Create a new previous close request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            adjusted: None,
        }
    }

    /// Set whether prices are adjusted for splits.
    pub fn adjusted(mut self, adjusted: bool) -> Self {
        self.adjusted = Some(adjusted);
        self
    }
}

impl RestRequest for GetPreviousCloseRequest {
    type Response = ListEnvelope<AggregateBar>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v2/aggs/ticker/{}/prev", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("adjusted", self.adjusted);
        params
    }
}

/// Request for daily open/close data.
///
/// Returns the open, close and afterhours prices of a stock symbol
/// on a certain date.
#[derive(Debug, Clone)]
pub struct GetDailyOpenCloseRequest {
    /// Ticker symbol
    pub ticker: String,
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Whether prices are adjusted for splits
    pub adjusted: Option<bool>,
}

impl GetDailyOpenCloseRequest {
    /// Create a new daily open/close request.
    pub fn new(ticker: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            date: date.into(),
            adjusted: None,
        }
    }

    /// Set whether prices are adjusted for splits.
    pub fn adjusted(mut self, adjusted: bool) -> Self {
        self.adjusted = Some(adjusted);
        self
    }
}

/// Response from daily open/close endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyOpenCloseResponse {
    /// Status string
    pub status: String,
    /// Starting timestamp
    pub from: String,
    /// Ticker symbol
    pub symbol: String,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume
    pub volume: f64,
    /// After hours price
    #[serde(rename = "afterHours")]
    pub after_hours: Option<f64>,
    /// Premarket price
    #[serde(rename = "preMarket")]
    pub pre_market: Option<f64>,
}

impl RestRequest for GetDailyOpenCloseRequest {
    type Response = DailyOpenCloseResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v1/open-close/{}/{}", self.ticker, self.date).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("adjusted", self.adjusted);
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timespan_display() {
        assert_eq!(Timespan::Second.to_string(), "second");
        assert_eq!(Timespan::Minute.to_string(), "minute");
        assert_eq!(Timespan::Hour.to_string(), "hour");
        assert_eq!(Timespan::Day.to_string(), "day");
        assert_eq!(Timespan::Week.to_string(), "week");
        assert_eq!(Timespan::Month.to_string(), "month");
        assert_eq!(Timespan::Quarter.to_string(), "quarter");
        assert_eq!(Timespan::Year.to_string(), "year");
    }

    #[test]
    fn test_sort_display() {
        assert_eq!(Sort::Asc.to_string(), "asc");
        assert_eq!(Sort::Desc.to_string(), "desc");
    }

    #[test]
    fn test_get_aggs_request_path() {
        let req = GetAggsRequest::new("AAPL")
            .multiplier(1)
            .timespan(Timespan::Day)
            .from("2024-01-01")
            .to("2024-01-31");

        assert_eq!(
            req.path(),
            "/v2/aggs/ticker/AAPL/range/1/day/2024-01-01/2024-01-31"
        );
    }

    #[test]
    fn test_get_aggs_request_query() {
        let req = GetAggsRequest::new("AAPL")
            .from("2024-01-01")
            .to("2024-01-31")
            .adjusted(true)
            .sort(Sort::Desc)
            .limit(100);

        let query = req.query();
        assert_eq!(query.len(), 3);

        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("adjusted").unwrap(), "true");
        assert_eq!(query_map.get("sort").unwrap(), "desc");
        assert_eq!(query_map.get("limit").unwrap(), "100");
    }

    #[test]
    fn test_get_previous_close_request() {
        let req = GetPreviousCloseRequest::new("MSFT").adjusted(true);
        assert_eq!(req.path(), "/v2/aggs/ticker/MSFT/prev");

        let query = req.query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].1, "true");
    }

    #[test]
    fn test_get_daily_open_close_request() {
        let req = GetDailyOpenCloseRequest::new("GOOG", "2024-01-15");
        assert_eq!(req.path(), "/v1/open-close/GOOG/2024-01-15");
    }

    #[test]
    fn test_aggs_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "ticker": "AAPL",
            "queryCount": 10,
            "resultsCount": 10,
            "adjusted": true,
            "results": [
                {
                    "o": 150.0,
                    "h": 155.0,
                    "l": 148.0,
                    "c": 153.0,
                    "v": 1000000,
                    "vw": 151.5,
                    "t": 1703001234567,
                    "n": 5000
                }
            ]
        }"#;

        let response: AggsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, Some("OK".to_string()));
        assert_eq!(response.ticker, Some("AAPL".to_string()));
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].open, 150.0);
    }
}
