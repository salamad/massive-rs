//! Corporate actions endpoints (dividends and splits).
//!
//! This module provides access to corporate action data including:
//!
//! - **Dividends**: Cash dividend distributions with adjustment factors
//! - **Splits**: Stock split events with ratio calculations
//!
//! # Feature Flag
//!
//! These endpoints are available when the `corporate-actions` feature is enabled,
//! or they can be used without the feature flag as they are core stock data.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{GetDividendsRequest, GetSplitsRequest, DistributionType};
//!
//! // Get quarterly dividends for AAPL
//! let dividends = GetDividendsRequest::new()
//!     .ticker("AAPL")
//!     .distribution_type(DistributionType::Recurring)
//!     .limit(50);
//!
//! // Get forward splits for AAPL
//! let splits = GetSplitsRequest::new()
//!     .ticker("AAPL")
//!     .limit(10);
//! ```

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

// =============================================================================
// Dividend Types
// =============================================================================

/// Distribution type classification for dividends.
///
/// Describes the nature of a dividend's recurrence pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistributionType {
    /// Paid on a regular schedule.
    Recurring,
    /// One-time or commemorative dividend.
    Special,
    /// Extra payment beyond the regular schedule.
    Supplemental,
    /// Unpredictable or non-recurring.
    Irregular,
    /// Cannot be classified from available data.
    Unknown,
}

impl std::fmt::Display for DistributionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionType::Recurring => write!(f, "recurring"),
            DistributionType::Special => write!(f, "special"),
            DistributionType::Supplemental => write!(f, "supplemental"),
            DistributionType::Irregular => write!(f, "irregular"),
            DistributionType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Dividend frequency constants.
pub mod frequency {
    /// Non-recurring or irregular dividend.
    pub const IRREGULAR: i32 = 0;
    /// Annual dividend (once per year).
    pub const ANNUAL: i32 = 1;
    /// Semi-annual dividend (twice per year).
    pub const SEMI_ANNUAL: i32 = 2;
    /// Trimester dividend (three times per year).
    pub const TRIMESTER: i32 = 3;
    /// Quarterly dividend (four times per year).
    pub const QUARTERLY: i32 = 4;
    /// Monthly dividend.
    pub const MONTHLY: i32 = 12;
    /// Bi-monthly dividend (24 times per year).
    pub const BI_MONTHLY: i32 = 24;
    /// Weekly dividend.
    pub const WEEKLY: i32 = 52;
    /// Bi-weekly dividend.
    pub const BI_WEEKLY: i32 = 104;
    /// Daily dividend.
    pub const DAILY: i32 = 365;
}

/// A single dividend record.
///
/// Contains all information about a cash dividend distribution including
/// payment amounts, dates, and adjustment factors for historical analysis.
#[derive(Debug, Clone, Deserialize)]
pub struct Dividend {
    /// Unique identifier for this dividend record.
    pub id: String,
    /// Stock symbol for the company issuing the dividend.
    pub ticker: String,
    /// Original dividend amount per share in the specified currency.
    pub cash_amount: f64,
    /// Currency code for the dividend payment (e.g., USD, CAD).
    pub currency: String,
    /// Date when the company officially announced the dividend.
    pub declaration_date: Option<String>,
    /// Date when the stock begins trading without the dividend value.
    pub ex_dividend_date: String,
    /// Date when shareholders must be on record to be eligible.
    pub record_date: Option<String>,
    /// Date when the dividend payment is distributed.
    pub pay_date: Option<String>,
    /// How many times per year this dividend is expected to occur.
    pub frequency: i32,
    /// Classification of this dividend's recurrence pattern.
    pub distribution_type: DistributionType,
    /// Cumulative adjustment factor for historical price normalization.
    pub historical_adjustment_factor: f64,
    /// Dividend amount adjusted for subsequent stock splits.
    pub split_adjusted_cash_amount: f64,
}

impl Dividend {
    /// Check if this is a regular, recurring dividend.
    pub fn is_recurring(&self) -> bool {
        self.distribution_type == DistributionType::Recurring
    }

    /// Check if this is a special (one-time) dividend.
    pub fn is_special(&self) -> bool {
        self.distribution_type == DistributionType::Special
    }

    /// Check if this is a quarterly dividend.
    pub fn is_quarterly(&self) -> bool {
        self.frequency == frequency::QUARTERLY
    }

    /// Calculate the annualized dividend yield given a stock price.
    ///
    /// Returns the yield as a decimal (e.g., 0.02 for 2%).
    pub fn annualized_yield(&self, stock_price: f64) -> Option<f64> {
        if stock_price <= 0.0 || self.frequency <= 0 {
            return None;
        }
        Some((self.cash_amount * self.frequency as f64) / stock_price)
    }

    /// Get the frequency as a human-readable string.
    pub fn frequency_description(&self) -> &'static str {
        match self.frequency {
            frequency::IRREGULAR => "irregular",
            frequency::ANNUAL => "annual",
            frequency::SEMI_ANNUAL => "semi-annual",
            frequency::TRIMESTER => "trimester",
            frequency::QUARTERLY => "quarterly",
            frequency::MONTHLY => "monthly",
            frequency::BI_MONTHLY => "bi-monthly",
            frequency::WEEKLY => "weekly",
            frequency::BI_WEEKLY => "bi-weekly",
            frequency::DAILY => "daily",
            _ => "unknown",
        }
    }

    /// Adjust a historical price for this dividend.
    ///
    /// Multiply a pre-ex-dividend price by this factor to get the
    /// dividend-adjusted price.
    pub fn adjust_price(&self, price: f64) -> f64 {
        price * self.historical_adjustment_factor
    }
}

/// Request for dividend data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetDividendsRequest, DistributionType};
///
/// // Get all dividends for AAPL
/// let request = GetDividendsRequest::new()
///     .ticker("AAPL")
///     .limit(100);
///
/// // Get only quarterly dividends
/// let quarterly = GetDividendsRequest::new()
///     .ticker("MSFT")
///     .distribution_type(DistributionType::Recurring)
///     .frequency(4);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetDividendsRequest {
    /// Filter by ticker symbol.
    pub ticker: Option<String>,
    /// Filter by multiple tickers.
    pub ticker_any_of: Option<Vec<String>>,
    /// Filter for ex-dividend date.
    pub ex_dividend_date: Option<String>,
    /// Filter for ex-dividend date greater than.
    pub ex_dividend_date_gt: Option<String>,
    /// Filter for ex-dividend date greater than or equal.
    pub ex_dividend_date_gte: Option<String>,
    /// Filter for ex-dividend date less than.
    pub ex_dividend_date_lt: Option<String>,
    /// Filter for ex-dividend date less than or equal.
    pub ex_dividend_date_lte: Option<String>,
    /// Filter by frequency.
    pub frequency: Option<i32>,
    /// Filter by distribution type.
    pub distribution_type: Option<DistributionType>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Sort specification.
    pub sort: Option<String>,
}

impl GetDividendsRequest {
    /// Create a new dividends request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by ticker symbol.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by multiple tickers.
    pub fn tickers(mut self, tickers: Vec<String>) -> Self {
        self.ticker_any_of = Some(tickers);
        self
    }

    /// Filter by ex-dividend date.
    pub fn ex_dividend_date(mut self, date: impl Into<String>) -> Self {
        self.ex_dividend_date = Some(date.into());
        self
    }

    /// Filter for ex-dividend dates after the given date.
    pub fn ex_dividend_date_gt(mut self, date: impl Into<String>) -> Self {
        self.ex_dividend_date_gt = Some(date.into());
        self
    }

    /// Filter for ex-dividend dates on or after the given date.
    pub fn ex_dividend_date_gte(mut self, date: impl Into<String>) -> Self {
        self.ex_dividend_date_gte = Some(date.into());
        self
    }

    /// Filter for ex-dividend dates before the given date.
    pub fn ex_dividend_date_lt(mut self, date: impl Into<String>) -> Self {
        self.ex_dividend_date_lt = Some(date.into());
        self
    }

    /// Filter for ex-dividend dates on or before the given date.
    pub fn ex_dividend_date_lte(mut self, date: impl Into<String>) -> Self {
        self.ex_dividend_date_lte = Some(date.into());
        self
    }

    /// Filter for ex-dividend dates in a range (inclusive).
    pub fn ex_dividend_date_range(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        self.ex_dividend_date_gte = Some(from.into());
        self.ex_dividend_date_lte = Some(to.into());
        self
    }

    /// Filter by frequency (e.g., 4 for quarterly).
    pub fn frequency(mut self, freq: i32) -> Self {
        self.frequency = Some(freq);
        self
    }

    /// Filter for quarterly dividends.
    pub fn quarterly(self) -> Self {
        self.frequency(frequency::QUARTERLY)
    }

    /// Filter for monthly dividends.
    pub fn monthly(self) -> Self {
        self.frequency(frequency::MONTHLY)
    }

    /// Filter for annual dividends.
    pub fn annual(self) -> Self {
        self.frequency(frequency::ANNUAL)
    }

    /// Filter by distribution type.
    pub fn distribution_type(mut self, dtype: DistributionType) -> Self {
        self.distribution_type = Some(dtype);
        self
    }

    /// Set maximum number of results.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set sort specification.
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Sort by ex-dividend date ascending.
    pub fn sort_by_date_asc(self) -> Self {
        self.sort("ex_dividend_date.asc")
    }

    /// Sort by ex-dividend date descending.
    pub fn sort_by_date_desc(self) -> Self {
        self.sort("ex_dividend_date.desc")
    }
}

/// Response from the dividends endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct DividendsResponse {
    /// URL for the next page of results.
    pub next_url: Option<String>,
    /// Request ID for debugging.
    pub request_id: Option<String>,
    /// Dividend results.
    #[serde(default)]
    pub results: Vec<Dividend>,
    /// Response status.
    pub status: Option<String>,
}

impl RestRequest for GetDividendsRequest {
    type Response = DividendsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/stocks/v1/dividends".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.clone());
        if let Some(ref tickers) = self.ticker_any_of {
            params.push((Cow::Borrowed("ticker.any_of"), tickers.join(",")));
        }
        params.push_opt_param("ex_dividend_date", self.ex_dividend_date.clone());
        params.push_opt_param("ex_dividend_date.gt", self.ex_dividend_date_gt.clone());
        params.push_opt_param("ex_dividend_date.gte", self.ex_dividend_date_gte.clone());
        params.push_opt_param("ex_dividend_date.lt", self.ex_dividend_date_lt.clone());
        params.push_opt_param("ex_dividend_date.lte", self.ex_dividend_date_lte.clone());
        params.push_opt_param("frequency", self.frequency);
        params.push_opt_param(
            "distribution_type",
            self.distribution_type.map(|d| d.to_string()),
        );
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("sort", self.sort.clone());
        params
    }
}

impl PaginatableRequest for GetDividendsRequest {
    type Item = Dividend;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// =============================================================================
// Split Types
// =============================================================================

/// Adjustment type classification for stock splits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentType {
    /// Share count increases (e.g., 2-for-1 split).
    ForwardSplit,
    /// Share count decreases (e.g., 1-for-10 reverse split).
    ReverseSplit,
    /// Shares issued as a dividend.
    StockDividend,
}

impl std::fmt::Display for AdjustmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdjustmentType::ForwardSplit => write!(f, "forward_split"),
            AdjustmentType::ReverseSplit => write!(f, "reverse_split"),
            AdjustmentType::StockDividend => write!(f, "stock_dividend"),
        }
    }
}

/// A single stock split record.
///
/// Contains information about a stock split event including the split ratio
/// and adjustment factor for historical price normalization.
#[derive(Debug, Clone, Deserialize)]
pub struct Split {
    /// Unique identifier for this split event.
    pub id: String,
    /// Stock symbol for the company that executed the split.
    pub ticker: String,
    /// Date when the stock split was applied.
    pub execution_date: String,
    /// Denominator of the split ratio (old shares).
    pub split_from: f64,
    /// Numerator of the split ratio (new shares).
    pub split_to: f64,
    /// Classification of the share-change event.
    pub adjustment_type: AdjustmentType,
    /// Cumulative adjustment factor for historical price normalization.
    pub historical_adjustment_factor: f64,
}

impl Split {
    /// Calculate the split ratio (new shares per old share).
    ///
    /// For a 2-for-1 split, this returns 2.0.
    /// For a 1-for-10 reverse split, this returns 0.1.
    pub fn ratio(&self) -> f64 {
        if self.split_from > 0.0 {
            self.split_to / self.split_from
        } else {
            1.0
        }
    }

    /// Check if this is a forward split (share count increases).
    pub fn is_forward(&self) -> bool {
        self.split_to > self.split_from
    }

    /// Check if this is a reverse split (share count decreases).
    pub fn is_reverse(&self) -> bool {
        self.split_to < self.split_from
    }

    /// Check if this is a stock dividend.
    pub fn is_stock_dividend(&self) -> bool {
        self.adjustment_type == AdjustmentType::StockDividend
    }

    /// Get a human-readable description of the split.
    ///
    /// Returns something like "2-for-1 split" or "1-for-10 reverse split".
    pub fn description(&self) -> String {
        let ratio_str = if self.split_to == self.split_to.floor()
            && self.split_from == self.split_from.floor()
        {
            format!("{}-for-{}", self.split_to as i64, self.split_from as i64)
        } else {
            format!("{:.2}-for-{:.2}", self.split_to, self.split_from)
        };

        match self.adjustment_type {
            AdjustmentType::ForwardSplit => format!("{} split", ratio_str),
            AdjustmentType::ReverseSplit => format!("{} reverse split", ratio_str),
            AdjustmentType::StockDividend => format!("{} stock dividend", ratio_str),
        }
    }

    /// Adjust a historical price for this split.
    ///
    /// Multiply a pre-split price by this factor to get the split-adjusted price.
    pub fn adjust_price(&self, price: f64) -> f64 {
        price * self.historical_adjustment_factor
    }

    /// Adjust a historical share count for this split.
    ///
    /// Divide a pre-split share count by the ratio to get the adjusted count.
    pub fn adjust_shares(&self, shares: f64) -> f64 {
        shares * self.ratio()
    }
}

/// Request for stock split data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetSplitsRequest, AdjustmentType};
///
/// // Get all splits for AAPL
/// let request = GetSplitsRequest::new()
///     .ticker("AAPL")
///     .limit(50);
///
/// // Get only forward splits
/// let forward_splits = GetSplitsRequest::new()
///     .ticker("TSLA")
///     .adjustment_type(AdjustmentType::ForwardSplit);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetSplitsRequest {
    /// Filter by ticker symbol.
    pub ticker: Option<String>,
    /// Filter by multiple tickers.
    pub ticker_any_of: Option<Vec<String>>,
    /// Filter for execution date.
    pub execution_date: Option<String>,
    /// Filter for execution date greater than.
    pub execution_date_gt: Option<String>,
    /// Filter for execution date greater than or equal.
    pub execution_date_gte: Option<String>,
    /// Filter for execution date less than.
    pub execution_date_lt: Option<String>,
    /// Filter for execution date less than or equal.
    pub execution_date_lte: Option<String>,
    /// Filter by adjustment type.
    pub adjustment_type: Option<AdjustmentType>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Sort specification.
    pub sort: Option<String>,
}

impl GetSplitsRequest {
    /// Create a new splits request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by ticker symbol.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by multiple tickers.
    pub fn tickers(mut self, tickers: Vec<String>) -> Self {
        self.ticker_any_of = Some(tickers);
        self
    }

    /// Filter by execution date.
    pub fn execution_date(mut self, date: impl Into<String>) -> Self {
        self.execution_date = Some(date.into());
        self
    }

    /// Filter for execution dates after the given date.
    pub fn execution_date_gt(mut self, date: impl Into<String>) -> Self {
        self.execution_date_gt = Some(date.into());
        self
    }

    /// Filter for execution dates on or after the given date.
    pub fn execution_date_gte(mut self, date: impl Into<String>) -> Self {
        self.execution_date_gte = Some(date.into());
        self
    }

    /// Filter for execution dates before the given date.
    pub fn execution_date_lt(mut self, date: impl Into<String>) -> Self {
        self.execution_date_lt = Some(date.into());
        self
    }

    /// Filter for execution dates on or before the given date.
    pub fn execution_date_lte(mut self, date: impl Into<String>) -> Self {
        self.execution_date_lte = Some(date.into());
        self
    }

    /// Filter for execution dates in a range (inclusive).
    pub fn execution_date_range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.execution_date_gte = Some(from.into());
        self.execution_date_lte = Some(to.into());
        self
    }

    /// Filter by adjustment type.
    pub fn adjustment_type(mut self, atype: AdjustmentType) -> Self {
        self.adjustment_type = Some(atype);
        self
    }

    /// Filter for forward splits only.
    pub fn forward_splits(self) -> Self {
        self.adjustment_type(AdjustmentType::ForwardSplit)
    }

    /// Filter for reverse splits only.
    pub fn reverse_splits(self) -> Self {
        self.adjustment_type(AdjustmentType::ReverseSplit)
    }

    /// Set maximum number of results.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set sort specification.
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Sort by execution date ascending.
    pub fn sort_by_date_asc(self) -> Self {
        self.sort("execution_date.asc")
    }

    /// Sort by execution date descending.
    pub fn sort_by_date_desc(self) -> Self {
        self.sort("execution_date.desc")
    }
}

/// Response from the splits endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct SplitsResponse {
    /// URL for the next page of results.
    pub next_url: Option<String>,
    /// Request ID for debugging.
    pub request_id: Option<String>,
    /// Split results.
    #[serde(default)]
    pub results: Vec<Split>,
    /// Response status.
    pub status: Option<String>,
}

impl RestRequest for GetSplitsRequest {
    type Response = SplitsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/stocks/v1/splits".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.clone());
        if let Some(ref tickers) = self.ticker_any_of {
            params.push((Cow::Borrowed("ticker.any_of"), tickers.join(",")));
        }
        params.push_opt_param("execution_date", self.execution_date.clone());
        params.push_opt_param("execution_date.gt", self.execution_date_gt.clone());
        params.push_opt_param("execution_date.gte", self.execution_date_gte.clone());
        params.push_opt_param("execution_date.lt", self.execution_date_lt.clone());
        params.push_opt_param("execution_date.lte", self.execution_date_lte.clone());
        params.push_opt_param(
            "adjustment_type",
            self.adjustment_type.map(|a| a.to_string()),
        );
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("sort", self.sort.clone());
        params
    }
}

impl PaginatableRequest for GetSplitsRequest {
    type Item = Split;

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

    // Dividend tests
    #[test]
    fn test_distribution_type_display() {
        assert_eq!(DistributionType::Recurring.to_string(), "recurring");
        assert_eq!(DistributionType::Special.to_string(), "special");
        assert_eq!(DistributionType::Supplemental.to_string(), "supplemental");
        assert_eq!(DistributionType::Irregular.to_string(), "irregular");
        assert_eq!(DistributionType::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_dividend_request_path() {
        let req = GetDividendsRequest::new().ticker("AAPL");
        assert_eq!(req.path(), "/stocks/v1/dividends");
        assert_eq!(req.method(), Method::GET);
    }

    #[test]
    fn test_dividend_request_query() {
        let req = GetDividendsRequest::new()
            .ticker("AAPL")
            .quarterly()
            .distribution_type(DistributionType::Recurring)
            .limit(100);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("ticker").unwrap(), "AAPL");
        assert_eq!(query_map.get("frequency").unwrap(), "4");
        assert_eq!(query_map.get("distribution_type").unwrap(), "recurring");
        assert_eq!(query_map.get("limit").unwrap(), "100");
    }

    #[test]
    fn test_dividend_request_date_range() {
        let req = GetDividendsRequest::new().ex_dividend_date_range("2024-01-01", "2024-12-31");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("ex_dividend_date.gte").unwrap(), "2024-01-01");
        assert_eq!(query_map.get("ex_dividend_date.lte").unwrap(), "2024-12-31");
    }

    #[test]
    fn test_dividend_response_deserialize() {
        let json = r#"{
            "request_id": "123",
            "status": "OK",
            "results": [
                {
                    "id": "abc123",
                    "ticker": "AAPL",
                    "cash_amount": 0.26,
                    "currency": "USD",
                    "declaration_date": "2024-07-31",
                    "ex_dividend_date": "2024-08-11",
                    "record_date": "2024-08-11",
                    "pay_date": "2024-08-14",
                    "frequency": 4,
                    "distribution_type": "recurring",
                    "historical_adjustment_factor": 0.997899,
                    "split_adjusted_cash_amount": 0.26
                }
            ]
        }"#;

        let response: DividendsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status.as_deref(), Some("OK"));
        assert_eq!(response.results.len(), 1);

        let div = &response.results[0];
        assert_eq!(div.ticker, "AAPL");
        assert!((div.cash_amount - 0.26).abs() < f64::EPSILON);
        assert_eq!(div.frequency, 4);
        assert_eq!(div.distribution_type, DistributionType::Recurring);
    }

    #[test]
    fn test_dividend_helper_methods() {
        let div = Dividend {
            id: "123".to_string(),
            ticker: "AAPL".to_string(),
            cash_amount: 0.25,
            currency: "USD".to_string(),
            declaration_date: None,
            ex_dividend_date: "2024-08-11".to_string(),
            record_date: None,
            pay_date: None,
            frequency: 4,
            distribution_type: DistributionType::Recurring,
            historical_adjustment_factor: 0.998,
            split_adjusted_cash_amount: 0.25,
        };

        assert!(div.is_recurring());
        assert!(!div.is_special());
        assert!(div.is_quarterly());
        assert_eq!(div.frequency_description(), "quarterly");

        // Annualized yield: 0.25 * 4 = 1.00, at $150 stock price = 0.67%
        let yield_pct = div.annualized_yield(150.0).unwrap();
        assert!((yield_pct - 0.006666666).abs() < 0.0001);

        // Price adjustment
        let adjusted = div.adjust_price(100.0);
        assert!((adjusted - 99.8).abs() < f64::EPSILON);
    }

    // Split tests
    #[test]
    fn test_adjustment_type_display() {
        assert_eq!(AdjustmentType::ForwardSplit.to_string(), "forward_split");
        assert_eq!(AdjustmentType::ReverseSplit.to_string(), "reverse_split");
        assert_eq!(AdjustmentType::StockDividend.to_string(), "stock_dividend");
    }

    #[test]
    fn test_split_request_path() {
        let req = GetSplitsRequest::new().ticker("AAPL");
        assert_eq!(req.path(), "/stocks/v1/splits");
        assert_eq!(req.method(), Method::GET);
    }

    #[test]
    fn test_split_request_query() {
        let req = GetSplitsRequest::new()
            .ticker("TSLA")
            .forward_splits()
            .limit(50);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("ticker").unwrap(), "TSLA");
        assert_eq!(query_map.get("adjustment_type").unwrap(), "forward_split");
        assert_eq!(query_map.get("limit").unwrap(), "50");
    }

    #[test]
    fn test_split_request_date_range() {
        let req = GetSplitsRequest::new().execution_date_range("2020-01-01", "2024-12-31");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("execution_date.gte").unwrap(), "2020-01-01");
        assert_eq!(query_map.get("execution_date.lte").unwrap(), "2024-12-31");
    }

    #[test]
    fn test_split_response_deserialize() {
        let json = r#"{
            "request_id": "456",
            "status": "OK",
            "results": [
                {
                    "id": "def456",
                    "ticker": "AAPL",
                    "execution_date": "2020-08-31",
                    "split_from": 1.0,
                    "split_to": 4.0,
                    "adjustment_type": "forward_split",
                    "historical_adjustment_factor": 0.25
                }
            ]
        }"#;

        let response: SplitsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status.as_deref(), Some("OK"));
        assert_eq!(response.results.len(), 1);

        let split = &response.results[0];
        assert_eq!(split.ticker, "AAPL");
        assert!((split.split_from - 1.0).abs() < f64::EPSILON);
        assert!((split.split_to - 4.0).abs() < f64::EPSILON);
        assert_eq!(split.adjustment_type, AdjustmentType::ForwardSplit);
    }

    #[test]
    fn test_split_helper_methods() {
        let forward = Split {
            id: "123".to_string(),
            ticker: "AAPL".to_string(),
            execution_date: "2020-08-31".to_string(),
            split_from: 1.0,
            split_to: 4.0,
            adjustment_type: AdjustmentType::ForwardSplit,
            historical_adjustment_factor: 0.25,
        };

        assert!(forward.is_forward());
        assert!(!forward.is_reverse());
        assert!(!forward.is_stock_dividend());
        assert!((forward.ratio() - 4.0).abs() < f64::EPSILON);
        assert_eq!(forward.description(), "4-for-1 split");

        // Price adjustment: $100 pre-split -> $25 post-split
        let adjusted = forward.adjust_price(100.0);
        assert!((adjusted - 25.0).abs() < f64::EPSILON);

        // Share adjustment: 100 shares pre-split -> 400 shares post-split
        let adjusted_shares = forward.adjust_shares(100.0);
        assert!((adjusted_shares - 400.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reverse_split() {
        let reverse = Split {
            id: "456".to_string(),
            ticker: "XYZ".to_string(),
            execution_date: "2023-01-15".to_string(),
            split_from: 10.0,
            split_to: 1.0,
            adjustment_type: AdjustmentType::ReverseSplit,
            historical_adjustment_factor: 10.0,
        };

        assert!(!reverse.is_forward());
        assert!(reverse.is_reverse());
        assert!((reverse.ratio() - 0.1).abs() < f64::EPSILON);
        assert_eq!(reverse.description(), "1-for-10 reverse split");
    }

    #[test]
    fn test_dividend_tickers_filter() {
        let req = GetDividendsRequest::new().tickers(vec![
            "AAPL".to_string(),
            "MSFT".to_string(),
            "GOOG".to_string(),
        ]);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("ticker.any_of").unwrap(), "AAPL,MSFT,GOOG");
    }

    #[test]
    fn test_split_tickers_filter() {
        let req = GetSplitsRequest::new().tickers(vec!["TSLA".to_string(), "NVDA".to_string()]);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("ticker.any_of").unwrap(), "TSLA,NVDA");
    }
}
