//! Partner data endpoints.
//!
//! This module provides endpoints for partner data sources including:
//!
//! - Benzinga earnings, ratings, and analyst data
//! - ETF Global profiles and analytics
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{GetEarningsRequest, GetAnalystRatingsRequest};
//!
//! // Get earnings for Apple
//! let earnings = GetEarningsRequest::new().ticker("AAPL");
//!
//! // Get analyst ratings
//! let ratings = GetAnalystRatingsRequest::new().ticker("AAPL");
//! ```

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

// ============================================================================
// Benzinga Earnings
// ============================================================================

/// Earnings date status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateStatus {
    /// Confirmed date.
    #[default]
    Confirmed,
    /// Tentative date.
    Tentative,
    /// Unknown status.
    #[serde(other)]
    Unknown,
}

/// Earnings time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarningsTime {
    /// Before market open.
    #[serde(alias = "bmo")]
    BeforeMarketOpen,
    /// After market close.
    #[serde(alias = "amc")]
    AfterMarketClose,
    /// During market hours.
    #[serde(alias = "dmh")]
    DuringMarketHours,
    /// Unknown time.
    #[default]
    #[serde(other)]
    Unknown,
}

/// Earnings report data.
#[derive(Debug, Clone, Deserialize)]
pub struct Earnings {
    /// Benzinga ID.
    pub id: Option<String>,
    /// Stock ticker.
    pub ticker: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Earnings date.
    pub date: Option<String>,
    /// Time of day for earnings.
    pub time: Option<EarningsTime>,
    /// Date status (confirmed/tentative).
    pub date_confirmed: Option<DateStatus>,
    /// Exchange.
    pub exchange: Option<String>,
    /// Currency.
    pub currency: Option<String>,

    // EPS data
    /// Actual EPS.
    pub eps: Option<f64>,
    /// Estimated EPS.
    pub eps_est: Option<f64>,
    /// Prior year EPS.
    pub eps_prior: Option<f64>,
    /// EPS surprise.
    pub eps_surprise: Option<f64>,
    /// EPS surprise percentage.
    pub eps_surprise_percent: Option<f64>,

    // Revenue data
    /// Actual revenue.
    pub revenue: Option<f64>,
    /// Estimated revenue.
    pub revenue_est: Option<f64>,
    /// Prior year revenue.
    pub revenue_prior: Option<f64>,
    /// Revenue surprise.
    pub revenue_surprise: Option<f64>,
    /// Revenue surprise percentage.
    pub revenue_surprise_percent: Option<f64>,

    /// Importance score (0-5).
    pub importance: Option<i32>,
    /// Last updated timestamp.
    pub updated: Option<i64>,
}

impl Earnings {
    /// Did EPS beat estimates?
    pub fn is_eps_beat(&self) -> Option<bool> {
        match (self.eps, self.eps_est) {
            (Some(actual), Some(est)) => Some(actual > est),
            _ => None,
        }
    }

    /// Did revenue beat estimates?
    pub fn is_revenue_beat(&self) -> Option<bool> {
        match (self.revenue, self.revenue_est) {
            (Some(actual), Some(est)) => Some(actual > est),
            _ => None,
        }
    }

    /// Calculate EPS growth year-over-year.
    pub fn eps_yoy_growth(&self) -> Option<f64> {
        match (self.eps, self.eps_prior) {
            (Some(current), Some(prior)) if prior != 0.0 => {
                Some(((current - prior) / prior.abs()) * 100.0)
            }
            _ => None,
        }
    }

    /// Calculate revenue growth year-over-year.
    pub fn revenue_yoy_growth(&self) -> Option<f64> {
        match (self.revenue, self.revenue_prior) {
            (Some(current), Some(prior)) if prior > 0.0 => {
                Some(((current - prior) / prior) * 100.0)
            }
            _ => None,
        }
    }
}

/// Response from earnings endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EarningsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Earnings results.
    #[serde(default)]
    pub results: Vec<Earnings>,
}

/// Request for earnings data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetEarningsRequest;
///
/// let request = GetEarningsRequest::new()
///     .ticker("AAPL")
///     .date_from("2024-01-01")
///     .date_to("2024-12-31")
///     .limit(50);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetEarningsRequest {
    /// Filter by ticker.
    pub ticker: Option<String>,
    /// Start date filter.
    pub date_from: Option<String>,
    /// End date filter.
    pub date_to: Option<String>,
    /// Importance filter (0-5).
    pub importance: Option<i32>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
    /// Sort by field.
    pub sort: Option<String>,
}

impl GetEarningsRequest {
    /// Create a new earnings request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by ticker.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
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

    /// Filter by importance (0-5).
    pub fn importance(mut self, importance: i32) -> Self {
        self.importance = Some(importance);
        self
    }

    /// Set result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetEarningsRequest {
    type Response = EarningsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v2/reference/earnings".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.as_ref());
        params.push_opt_param("date.gte", self.date_from.as_ref());
        params.push_opt_param("date.lte", self.date_to.as_ref());
        params.push_opt_param("importance", self.importance);
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params.push_opt_param("sort", self.sort.as_ref());
        params
    }
}

impl PaginatableRequest for GetEarningsRequest {
    type Item = Earnings;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Analyst Ratings
// ============================================================================

/// Rating action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RatingAction {
    /// Initiated coverage.
    Initiated,
    /// Maintained rating.
    #[default]
    Maintained,
    /// Upgraded rating.
    Upgraded,
    /// Downgraded rating.
    Downgraded,
    /// Reiterated rating.
    Reiterated,
    /// Unknown action.
    #[serde(other)]
    Unknown,
}

/// Analyst rating data.
#[derive(Debug, Clone, Deserialize)]
pub struct AnalystRating {
    /// Benzinga ID.
    pub id: Option<String>,
    /// Stock ticker.
    pub ticker: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Rating date.
    pub date: Option<String>,
    /// Action (upgrade/downgrade/etc).
    pub action: Option<RatingAction>,
    /// Analyst name.
    pub analyst: Option<String>,
    /// Analyst firm.
    pub analyst_name: Option<String>,
    /// Previous rating.
    pub rating_prior: Option<String>,
    /// Current rating.
    pub rating_current: Option<String>,
    /// Previous price target.
    pub pt_prior: Option<f64>,
    /// Current price target.
    pub pt_current: Option<f64>,
    /// Currency.
    pub currency: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// URL to source.
    pub url: Option<String>,
    /// Last updated.
    pub updated: Option<i64>,
}

impl AnalystRating {
    /// Calculate price target change.
    pub fn pt_change(&self) -> Option<f64> {
        match (self.pt_current, self.pt_prior) {
            (Some(current), Some(prior)) => Some(current - prior),
            _ => None,
        }
    }

    /// Calculate price target change percentage.
    pub fn pt_change_percent(&self) -> Option<f64> {
        match (self.pt_current, self.pt_prior) {
            (Some(current), Some(prior)) if prior > 0.0 => {
                Some(((current - prior) / prior) * 100.0)
            }
            _ => None,
        }
    }

    /// Is this an upgrade?
    pub fn is_upgrade(&self) -> bool {
        matches!(self.action, Some(RatingAction::Upgraded))
    }

    /// Is this a downgrade?
    pub fn is_downgrade(&self) -> bool {
        matches!(self.action, Some(RatingAction::Downgraded))
    }
}

/// Response from analyst ratings endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AnalystRatingsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Rating results.
    #[serde(default)]
    pub results: Vec<AnalystRating>,
}

/// Request for analyst ratings.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetAnalystRatingsRequest;
///
/// let request = GetAnalystRatingsRequest::new()
///     .ticker("AAPL")
///     .limit(20);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetAnalystRatingsRequest {
    /// Filter by ticker.
    pub ticker: Option<String>,
    /// Start date filter.
    pub date_from: Option<String>,
    /// End date filter.
    pub date_to: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetAnalystRatingsRequest {
    /// Create a new ratings request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by ticker.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
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

impl RestRequest for GetAnalystRatingsRequest {
    type Response = AnalystRatingsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v2/reference/analyst-ratings".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.as_ref());
        params.push_opt_param("date.gte", self.date_from.as_ref());
        params.push_opt_param("date.lte", self.date_to.as_ref());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetAnalystRatingsRequest {
    type Item = AnalystRating;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// ETF Profiles
// ============================================================================

/// ETF profile data.
#[derive(Debug, Clone, Deserialize)]
pub struct EtfProfile {
    /// ETF ticker.
    pub ticker: Option<String>,
    /// ETF name.
    pub name: Option<String>,
    /// Fund sponsor.
    pub sponsor: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Geographic focus.
    pub geography: Option<String>,
    /// Inception date.
    pub inception_date: Option<String>,
    /// Expense ratio.
    pub expense_ratio: Option<f64>,
    /// Assets under management.
    pub aum: Option<f64>,
    /// Average daily volume.
    pub avg_volume: Option<f64>,
    /// Index tracked.
    pub index_name: Option<String>,
    /// Number of holdings.
    pub holdings_count: Option<i32>,
    /// Description.
    pub description: Option<String>,
}

impl EtfProfile {
    /// Is this a low-cost ETF (expense ratio < 0.20%)?
    pub fn is_low_cost(&self) -> Option<bool> {
        self.expense_ratio.map(|er| er < 0.20)
    }

    /// Is this a large ETF (AUM > $1B)?
    pub fn is_large(&self) -> Option<bool> {
        self.aum.map(|aum| aum > 1_000_000_000.0)
    }
}

/// Response from ETF profiles endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EtfProfilesResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// ETF profile results.
    #[serde(default)]
    pub results: Vec<EtfProfile>,
}

/// Request for ETF profiles.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetEtfProfilesRequest;
///
/// let request = GetEtfProfilesRequest::new()
///     .ticker("SPY")
///     .limit(10);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetEtfProfilesRequest {
    /// Filter by ticker.
    pub ticker: Option<String>,
    /// Filter by sponsor.
    pub sponsor: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetEtfProfilesRequest {
    /// Create a new ETF profiles request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by ticker.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by sponsor.
    pub fn sponsor(mut self, sponsor: impl Into<String>) -> Self {
        self.sponsor = Some(sponsor.into());
        self
    }

    /// Set result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetEtfProfilesRequest {
    type Response = EtfProfilesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/vX/reference/etfs".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.as_ref());
        params.push_opt_param("sponsor", self.sponsor.as_ref());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetEtfProfilesRequest {
    type Item = EtfProfile;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// ETF Holdings
// ============================================================================

/// ETF holding data.
#[derive(Debug, Clone, Deserialize)]
pub struct EtfHolding {
    /// Holding ticker.
    pub ticker: Option<String>,
    /// Holding name.
    pub name: Option<String>,
    /// Weight in portfolio (percentage).
    pub weight: Option<f64>,
    /// Market value.
    pub market_value: Option<f64>,
    /// Number of shares.
    pub shares: Option<f64>,
    /// Sector.
    pub sector: Option<String>,
    /// Asset type.
    pub asset_type: Option<String>,
}

/// Response from ETF holdings endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EtfHoldingsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Holding results.
    #[serde(default)]
    pub results: Vec<EtfHolding>,
}

/// Request for ETF holdings.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetEtfHoldingsRequest;
///
/// let request = GetEtfHoldingsRequest::new("SPY")
///     .limit(100);
/// ```
#[derive(Debug, Clone)]
pub struct GetEtfHoldingsRequest {
    /// ETF ticker.
    pub ticker: String,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetEtfHoldingsRequest {
    /// Create a new ETF holdings request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            limit: None,
            order: None,
        }
    }

    /// Set result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetEtfHoldingsRequest {
    type Response = EtfHoldingsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/vX/reference/etfs/{}/holdings", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetEtfHoldingsRequest {
    type Item = EtfHolding;

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
    fn test_earnings_beat_detection() {
        let earnings = Earnings {
            id: None,
            ticker: Some("AAPL".to_string()),
            name: None,
            date: None,
            time: None,
            date_confirmed: None,
            exchange: None,
            currency: None,
            eps: Some(1.50),
            eps_est: Some(1.40),
            eps_prior: Some(1.20),
            eps_surprise: Some(0.10),
            eps_surprise_percent: Some(7.14),
            revenue: Some(100_000_000.0),
            revenue_est: Some(95_000_000.0),
            revenue_prior: Some(90_000_000.0),
            revenue_surprise: Some(5_000_000.0),
            revenue_surprise_percent: Some(5.26),
            importance: Some(5),
            updated: None,
        };

        assert!(earnings.is_eps_beat().unwrap());
        assert!(earnings.is_revenue_beat().unwrap());
        assert!((earnings.eps_yoy_growth().unwrap() - 25.0).abs() < 0.1);
        assert!((earnings.revenue_yoy_growth().unwrap() - 11.11).abs() < 0.1);
    }

    #[test]
    fn test_analyst_rating_helpers() {
        let rating = AnalystRating {
            id: None,
            ticker: Some("AAPL".to_string()),
            name: None,
            date: None,
            action: Some(RatingAction::Upgraded),
            analyst: None,
            analyst_name: Some("Morgan Stanley".to_string()),
            rating_prior: Some("Hold".to_string()),
            rating_current: Some("Buy".to_string()),
            pt_prior: Some(150.0),
            pt_current: Some(180.0),
            currency: None,
            exchange: None,
            url: None,
            updated: None,
        };

        assert!(rating.is_upgrade());
        assert!(!rating.is_downgrade());
        assert!((rating.pt_change().unwrap() - 30.0).abs() < 0.1);
        assert!((rating.pt_change_percent().unwrap() - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_etf_profile_helpers() {
        let profile = EtfProfile {
            ticker: Some("SPY".to_string()),
            name: Some("SPDR S&P 500 ETF".to_string()),
            sponsor: Some("State Street".to_string()),
            asset_class: Some("Equity".to_string()),
            geography: Some("US".to_string()),
            inception_date: Some("1993-01-22".to_string()),
            expense_ratio: Some(0.09),
            aum: Some(400_000_000_000.0),
            avg_volume: Some(80_000_000.0),
            index_name: Some("S&P 500".to_string()),
            holdings_count: Some(503),
            description: None,
        };

        assert!(profile.is_low_cost().unwrap());
        assert!(profile.is_large().unwrap());
    }

    #[test]
    fn test_get_earnings_request() {
        let req = GetEarningsRequest::new()
            .ticker("AAPL")
            .date_from("2024-01-01")
            .limit(50);

        assert_eq!(req.path(), "/v2/reference/earnings");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("ticker").unwrap(), "AAPL");
        assert_eq!(query_map.get("date.gte").unwrap(), "2024-01-01");
    }

    #[test]
    fn test_get_analyst_ratings_request() {
        let req = GetAnalystRatingsRequest::new().ticker("TSLA").limit(20);

        assert_eq!(req.path(), "/v2/reference/analyst-ratings");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("ticker").unwrap(), "TSLA");
    }

    #[test]
    fn test_get_etf_profiles_request() {
        let req = GetEtfProfilesRequest::new().ticker("SPY").limit(10);

        assert_eq!(req.path(), "/vX/reference/etfs");
    }

    #[test]
    fn test_get_etf_holdings_request() {
        let req = GetEtfHoldingsRequest::new("QQQ").limit(100);

        assert_eq!(req.path(), "/vX/reference/etfs/QQQ/holdings");
    }

    #[test]
    fn test_earnings_deserialize() {
        let json = r#"{
            "id": "123",
            "ticker": "AAPL",
            "name": "Apple Inc",
            "date": "2024-01-25",
            "time": "amc",
            "eps": 2.18,
            "eps_est": 2.10,
            "revenue": 119580000000,
            "revenue_est": 117000000000,
            "importance": 5
        }"#;

        let earnings: Earnings = serde_json::from_str(json).unwrap();
        assert_eq!(earnings.ticker, Some("AAPL".to_string()));
        assert!(earnings.is_eps_beat().unwrap());
    }

    #[test]
    fn test_rating_action_deserialize() {
        let json = r#"{"action": "upgraded"}"#;
        let data: serde_json::Value = serde_json::from_str(json).unwrap();
        let action: RatingAction = serde_json::from_value(data["action"].clone()).unwrap();
        assert_eq!(action, RatingAction::Upgraded);
    }
}
