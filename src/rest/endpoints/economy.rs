//! Economy data endpoints.
//!
//! This module provides endpoints for macroeconomic data:
//!
//! - Treasury yields
//! - Inflation data (CPI)
//! - Fed funds rate
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{GetTreasuryYieldsRequest, GetInflationRequest};
//!
//! // Get treasury yields
//! let yields = GetTreasuryYieldsRequest::new();
//!
//! // Get inflation data
//! let inflation = GetInflationRequest::new();
//! ```

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

// ============================================================================
// Treasury Yields
// ============================================================================

/// Treasury yield data for various maturities.
#[derive(Debug, Clone, Deserialize)]
pub struct TreasuryYield {
    /// Date.
    pub date: Option<String>,
    /// 1-month yield.
    #[serde(rename = "1month")]
    pub yield_1_month: Option<f64>,
    /// 2-month yield.
    #[serde(rename = "2month")]
    pub yield_2_month: Option<f64>,
    /// 3-month yield.
    #[serde(rename = "3month")]
    pub yield_3_month: Option<f64>,
    /// 6-month yield.
    #[serde(rename = "6month")]
    pub yield_6_month: Option<f64>,
    /// 1-year yield.
    #[serde(rename = "1year")]
    pub yield_1_year: Option<f64>,
    /// 2-year yield.
    #[serde(rename = "2year")]
    pub yield_2_year: Option<f64>,
    /// 3-year yield.
    #[serde(rename = "3year")]
    pub yield_3_year: Option<f64>,
    /// 5-year yield.
    #[serde(rename = "5year")]
    pub yield_5_year: Option<f64>,
    /// 7-year yield.
    #[serde(rename = "7year")]
    pub yield_7_year: Option<f64>,
    /// 10-year yield.
    #[serde(rename = "10year")]
    pub yield_10_year: Option<f64>,
    /// 20-year yield.
    #[serde(rename = "20year")]
    pub yield_20_year: Option<f64>,
    /// 30-year yield.
    #[serde(rename = "30year")]
    pub yield_30_year: Option<f64>,
}

impl TreasuryYield {
    /// Calculate 2s10s spread (10-year minus 2-year).
    /// Negative values indicate yield curve inversion.
    pub fn spread_2s10s(&self) -> Option<f64> {
        match (self.yield_10_year, self.yield_2_year) {
            (Some(y10), Some(y2)) => Some(y10 - y2),
            _ => None,
        }
    }

    /// Calculate 3m10y spread (10-year minus 3-month).
    pub fn spread_3m10y(&self) -> Option<f64> {
        match (self.yield_10_year, self.yield_3_month) {
            (Some(y10), Some(y3m)) => Some(y10 - y3m),
            _ => None,
        }
    }

    /// Is the 2s10s curve inverted?
    pub fn is_2s10s_inverted(&self) -> Option<bool> {
        self.spread_2s10s().map(|s| s < 0.0)
    }

    /// Calculate the term premium (30-year minus 2-year).
    pub fn term_premium(&self) -> Option<f64> {
        match (self.yield_30_year, self.yield_2_year) {
            (Some(y30), Some(y2)) => Some(y30 - y2),
            _ => None,
        }
    }
}

/// Response from treasury yields endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TreasuryYieldsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Treasury yield results.
    #[serde(default)]
    pub results: Vec<TreasuryYield>,
}

/// Request for treasury yields.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetTreasuryYieldsRequest;
///
/// let request = GetTreasuryYieldsRequest::new()
///     .date_from("2024-01-01")
///     .limit(30);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetTreasuryYieldsRequest {
    /// Start date filter.
    pub date_from: Option<String>,
    /// End date filter.
    pub date_to: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetTreasuryYieldsRequest {
    /// Create a new treasury yields request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by start date.
    pub fn date_from(mut self, date: impl Into<String>) -> Self {
        self.date_from = Some(date.into());
        self
    }

    /// Filter by end date.
    pub fn date_to(mut self, date: impl Into<String>) -> Self {
        self.date_to = Some(date.into());
        self
    }

    /// Set result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set sort order.
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }
}

impl RestRequest for GetTreasuryYieldsRequest {
    type Response = TreasuryYieldsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v1/economy/treasury-yields".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("date.gte", self.date_from.as_ref());
        params.push_opt_param("date.lte", self.date_to.as_ref());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetTreasuryYieldsRequest {
    type Item = TreasuryYield;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Inflation Data
// ============================================================================

/// Inflation data point (CPI-based).
#[derive(Debug, Clone, Deserialize)]
pub struct InflationData {
    /// Date.
    pub date: Option<String>,
    /// CPI value.
    pub value: Option<f64>,
    /// Year-over-year change percentage.
    pub yoy_change: Option<f64>,
    /// Month-over-month change percentage.
    pub mom_change: Option<f64>,
    /// Core CPI (excluding food and energy).
    pub core_value: Option<f64>,
    /// Core YoY change.
    pub core_yoy_change: Option<f64>,
}

impl InflationData {
    /// Is inflation above the Fed's 2% target?
    pub fn is_above_target(&self) -> Option<bool> {
        self.yoy_change.map(|yoy| yoy > 2.0)
    }

    /// Calculate real yield (nominal yield minus inflation).
    pub fn real_yield(&self, nominal_yield: f64) -> Option<f64> {
        self.yoy_change.map(|inf| nominal_yield - inf)
    }
}

/// Response from inflation endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct InflationResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Inflation data results.
    #[serde(default)]
    pub results: Vec<InflationData>,
}

/// Request for inflation data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetInflationRequest;
///
/// let request = GetInflationRequest::new()
///     .date_from("2024-01-01")
///     .limit(12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetInflationRequest {
    /// Start date filter.
    pub date_from: Option<String>,
    /// End date filter.
    pub date_to: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetInflationRequest {
    /// Create a new inflation data request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by start date.
    pub fn date_from(mut self, date: impl Into<String>) -> Self {
        self.date_from = Some(date.into());
        self
    }

    /// Filter by end date.
    pub fn date_to(mut self, date: impl Into<String>) -> Self {
        self.date_to = Some(date.into());
        self
    }

    /// Set result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetInflationRequest {
    type Response = InflationResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v1/economy/inflation".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("date.gte", self.date_from.as_ref());
        params.push_opt_param("date.lte", self.date_to.as_ref());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetInflationRequest {
    type Item = InflationData;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Fed Funds Rate
// ============================================================================

/// Federal funds rate data.
#[derive(Debug, Clone, Deserialize)]
pub struct FedFundsRate {
    /// Date.
    pub date: Option<String>,
    /// Effective federal funds rate.
    pub rate: Option<f64>,
    /// Target rate lower bound.
    pub target_lower: Option<f64>,
    /// Target rate upper bound.
    pub target_upper: Option<f64>,
    /// Rate change from previous.
    pub change: Option<f64>,
}

impl FedFundsRate {
    /// Get the target range as a tuple.
    pub fn target_range(&self) -> Option<(f64, f64)> {
        match (self.target_lower, self.target_upper) {
            (Some(lower), Some(upper)) => Some((lower, upper)),
            _ => None,
        }
    }

    /// Get the midpoint of the target range.
    pub fn target_midpoint(&self) -> Option<f64> {
        match (self.target_lower, self.target_upper) {
            (Some(lower), Some(upper)) => Some((lower + upper) / 2.0),
            _ => None,
        }
    }

    /// Is the effective rate within the target range?
    pub fn is_within_target(&self) -> Option<bool> {
        match (self.rate, self.target_lower, self.target_upper) {
            (Some(rate), Some(lower), Some(upper)) => Some(rate >= lower && rate <= upper),
            _ => None,
        }
    }
}

/// Response from fed funds rate endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FedFundsRateResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Fed funds rate results.
    #[serde(default)]
    pub results: Vec<FedFundsRate>,
}

/// Request for federal funds rate data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFedFundsRateRequest;
///
/// let request = GetFedFundsRateRequest::new()
///     .date_from("2024-01-01");
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetFedFundsRateRequest {
    /// Start date filter.
    pub date_from: Option<String>,
    /// End date filter.
    pub date_to: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetFedFundsRateRequest {
    /// Create a new fed funds rate request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by start date.
    pub fn date_from(mut self, date: impl Into<String>) -> Self {
        self.date_from = Some(date.into());
        self
    }

    /// Filter by end date.
    pub fn date_to(mut self, date: impl Into<String>) -> Self {
        self.date_to = Some(date.into());
        self
    }

    /// Set result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetFedFundsRateRequest {
    type Response = FedFundsRateResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v1/economy/fed-funds-rate".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("date.gte", self.date_from.as_ref());
        params.push_opt_param("date.lte", self.date_to.as_ref());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetFedFundsRateRequest {
    type Item = FedFundsRate;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treasury_yield_spreads() {
        let yields = TreasuryYield {
            date: Some("2024-01-15".to_string()),
            yield_1_month: Some(5.35),
            yield_2_month: Some(5.40),
            yield_3_month: Some(5.38),
            yield_6_month: Some(5.25),
            yield_1_year: Some(4.95),
            yield_2_year: Some(4.35),
            yield_3_year: Some(4.10),
            yield_5_year: Some(3.95),
            yield_7_year: Some(3.98),
            yield_10_year: Some(4.05),
            yield_20_year: Some(4.35),
            yield_30_year: Some(4.25),
        };

        // 2s10s spread: 4.05 - 4.35 = -0.30 (inverted)
        assert!((yields.spread_2s10s().unwrap() - (-0.30)).abs() < 0.01);
        assert!(yields.is_2s10s_inverted().unwrap());

        // 3m10y spread: 4.05 - 5.38 = -1.33 (inverted)
        assert!((yields.spread_3m10y().unwrap() - (-1.33)).abs() < 0.01);

        // Term premium: 4.25 - 4.35 = -0.10
        assert!((yields.term_premium().unwrap() - (-0.10)).abs() < 0.01);
    }

    #[test]
    fn test_inflation_helpers() {
        let inflation = InflationData {
            date: Some("2024-01-01".to_string()),
            value: Some(310.5),
            yoy_change: Some(3.4),
            mom_change: Some(0.3),
            core_value: Some(308.2),
            core_yoy_change: Some(3.9),
        };

        assert!(inflation.is_above_target().unwrap());

        // Real yield with 4.0% nominal: 4.0 - 3.4 = 0.6%
        assert!((inflation.real_yield(4.0).unwrap() - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_fed_funds_rate_helpers() {
        let ffr = FedFundsRate {
            date: Some("2024-01-15".to_string()),
            rate: Some(5.33),
            target_lower: Some(5.25),
            target_upper: Some(5.50),
            change: None,
        };

        assert_eq!(ffr.target_range(), Some((5.25, 5.50)));
        assert!((ffr.target_midpoint().unwrap() - 5.375).abs() < 0.01);
        assert!(ffr.is_within_target().unwrap());
    }

    #[test]
    fn test_get_treasury_yields_request() {
        let req = GetTreasuryYieldsRequest::new()
            .date_from("2024-01-01")
            .limit(30);

        assert_eq!(req.path(), "/v1/economy/treasury-yields");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("date.gte").unwrap(), "2024-01-01");
        assert_eq!(query_map.get("limit").unwrap(), "30");
    }

    #[test]
    fn test_get_inflation_request() {
        let req = GetInflationRequest::new().date_from("2024-01-01").limit(12);

        assert_eq!(req.path(), "/v1/economy/inflation");
    }

    #[test]
    fn test_get_fed_funds_rate_request() {
        let req = GetFedFundsRateRequest::new().date_from("2024-01-01");

        assert_eq!(req.path(), "/v1/economy/fed-funds-rate");
    }

    #[test]
    fn test_treasury_yield_deserialize() {
        let json = r#"{
            "date": "2024-01-15",
            "1month": 5.35,
            "2year": 4.35,
            "10year": 4.05,
            "30year": 4.25
        }"#;

        let yields: TreasuryYield = serde_json::from_str(json).unwrap();
        assert_eq!(yields.date, Some("2024-01-15".to_string()));
        assert_eq!(yields.yield_1_month, Some(5.35));
        assert_eq!(yields.yield_2_year, Some(4.35));
        assert_eq!(yields.yield_10_year, Some(4.05));
    }
}
