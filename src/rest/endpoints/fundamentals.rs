//! Financial fundamentals and short data endpoints.
//!
//! This module provides endpoints for company financials and short interest data:
//!
//! - Balance sheets
//! - Income statements
//! - Cash flow statements
//! - Financial ratios
//! - Short interest
//! - Short volume
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{
//!     GetBalanceSheetsRequest, GetIncomeStatementsRequest, FinancialTimeframe
//! };
//!
//! // Get balance sheets for Apple
//! let balance = GetBalanceSheetsRequest::new("AAPL");
//!
//! // Get quarterly income statements
//! let income = GetIncomeStatementsRequest::new("AAPL")
//!     .timeframe(FinancialTimeframe::Quarterly);
//! ```

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

// ============================================================================
// Financial Timeframe
// ============================================================================

/// Financial statement timeframe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FinancialTimeframe {
    /// Annual financial statements.
    #[default]
    Annual,
    /// Quarterly financial statements.
    Quarterly,
    /// Trailing twelve months.
    TrailingTwelveMonths,
}

impl FinancialTimeframe {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Annual => "annual",
            Self::Quarterly => "quarterly",
            Self::TrailingTwelveMonths => "ttm",
        }
    }
}

// ============================================================================
// Balance Sheet
// ============================================================================

/// Balance sheet financial data.
#[derive(Debug, Clone, Deserialize)]
pub struct BalanceSheet {
    /// CIK number.
    pub cik: Option<String>,
    /// Company name.
    pub company_name: Option<String>,
    /// Stock tickers.
    #[serde(default)]
    pub tickers: Vec<String>,
    /// Start date of the period.
    pub start_date: Option<String>,
    /// End date of the period.
    pub end_date: Option<String>,
    /// Fiscal period (e.g., "Q1", "FY").
    pub fiscal_period: Option<String>,
    /// Fiscal year.
    pub fiscal_year: Option<String>,
    /// Filing date.
    pub filing_date: Option<String>,
    /// Source filing URL.
    pub source_filing_url: Option<String>,

    // Assets
    /// Total assets.
    #[serde(default)]
    pub assets: Option<FinancialValue>,
    /// Current assets.
    #[serde(default)]
    pub current_assets: Option<FinancialValue>,
    /// Non-current assets.
    #[serde(default)]
    pub noncurrent_assets: Option<FinancialValue>,
    /// Cash and equivalents.
    #[serde(default)]
    pub cash_and_cash_equivalents: Option<FinancialValue>,
    /// Accounts receivable.
    #[serde(default)]
    pub accounts_receivable: Option<FinancialValue>,
    /// Inventory.
    #[serde(default)]
    pub inventory: Option<FinancialValue>,
    /// Property, plant, and equipment.
    #[serde(default)]
    pub fixed_assets: Option<FinancialValue>,
    /// Intangible assets.
    #[serde(default)]
    pub intangible_assets: Option<FinancialValue>,

    // Liabilities
    /// Total liabilities.
    #[serde(default)]
    pub liabilities: Option<FinancialValue>,
    /// Current liabilities.
    #[serde(default)]
    pub current_liabilities: Option<FinancialValue>,
    /// Non-current liabilities.
    #[serde(default)]
    pub noncurrent_liabilities: Option<FinancialValue>,
    /// Long-term debt.
    #[serde(default)]
    pub long_term_debt: Option<FinancialValue>,
    /// Accounts payable.
    #[serde(default)]
    pub accounts_payable: Option<FinancialValue>,

    // Equity
    /// Total equity.
    #[serde(default)]
    pub equity: Option<FinancialValue>,
    /// Retained earnings.
    #[serde(default)]
    pub retained_earnings: Option<FinancialValue>,
}

impl BalanceSheet {
    /// Calculate current ratio (current assets / current liabilities).
    pub fn current_ratio(&self) -> Option<f64> {
        let assets = self.current_assets.as_ref()?.value?;
        let liabilities = self.current_liabilities.as_ref()?.value?;
        if liabilities > 0.0 {
            Some(assets / liabilities)
        } else {
            None
        }
    }

    /// Calculate debt-to-equity ratio (total liabilities / total equity).
    pub fn debt_to_equity(&self) -> Option<f64> {
        let liabilities = self.liabilities.as_ref()?.value?;
        let equity = self.equity.as_ref()?.value?;
        if equity > 0.0 {
            Some(liabilities / equity)
        } else {
            None
        }
    }

    /// Calculate working capital (current assets - current liabilities).
    pub fn working_capital(&self) -> Option<f64> {
        let assets = self.current_assets.as_ref()?.value?;
        let liabilities = self.current_liabilities.as_ref()?.value?;
        Some(assets - liabilities)
    }

    /// Calculate asset turnover if revenue is provided.
    pub fn asset_turnover(&self, revenue: f64) -> Option<f64> {
        let assets = self.assets.as_ref()?.value?;
        if assets > 0.0 {
            Some(revenue / assets)
        } else {
            None
        }
    }
}

/// A financial value with its unit and label.
#[derive(Debug, Clone, Deserialize)]
pub struct FinancialValue {
    /// The numeric value.
    pub value: Option<f64>,
    /// The unit (e.g., "USD").
    pub unit: Option<String>,
    /// Human-readable label.
    pub label: Option<String>,
    /// Order for display.
    pub order: Option<i32>,
}

/// Response from balance sheets endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BalanceSheetsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Balance sheet results.
    #[serde(default)]
    pub results: Vec<BalanceSheet>,
}

/// Request for balance sheets.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetBalanceSheetsRequest, FinancialTimeframe};
///
/// let request = GetBalanceSheetsRequest::new("AAPL")
///     .timeframe(FinancialTimeframe::Quarterly)
///     .limit(10);
/// ```
#[derive(Debug, Clone)]
pub struct GetBalanceSheetsRequest {
    /// Stock ticker.
    pub ticker: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Company name filter.
    pub company_name: Option<String>,
    /// SIC code filter.
    pub sic: Option<String>,
    /// Filing date filter.
    pub filing_date: Option<String>,
    /// Timeframe (annual, quarterly, ttm).
    pub timeframe: Option<FinancialTimeframe>,
    /// Include sources.
    pub include_sources: Option<bool>,
    /// Sort order.
    pub order: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort by field.
    pub sort: Option<String>,
}

impl GetBalanceSheetsRequest {
    /// Create a new request for a ticker.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: Some(ticker.into()),
            cik: None,
            company_name: None,
            sic: None,
            filing_date: None,
            timeframe: None,
            include_sources: None,
            order: None,
            limit: None,
            sort: None,
        }
    }

    /// Create a request by CIK number.
    pub fn by_cik(cik: impl Into<String>) -> Self {
        Self {
            ticker: None,
            cik: Some(cik.into()),
            company_name: None,
            sic: None,
            filing_date: None,
            timeframe: None,
            include_sources: None,
            order: None,
            limit: None,
            sort: None,
        }
    }

    /// Set the timeframe.
    pub fn timeframe(mut self, timeframe: FinancialTimeframe) -> Self {
        self.timeframe = Some(timeframe);
        self
    }

    /// Set the result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Include source filing URLs.
    pub fn include_sources(mut self, include: bool) -> Self {
        self.include_sources = Some(include);
        self
    }

    /// Set the sort order.
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }
}

impl RestRequest for GetBalanceSheetsRequest {
    type Response = BalanceSheetsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/vX/reference/financials".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.as_ref());
        params.push_opt_param("cik", self.cik.as_ref());
        params.push_opt_param("company_name", self.company_name.as_ref());
        params.push_opt_param("sic", self.sic.as_ref());
        params.push_opt_param("filing_date", self.filing_date.as_ref());
        if let Some(tf) = &self.timeframe {
            params.push((Cow::Borrowed("timeframe"), tf.as_str().to_string()));
        }
        params.push_opt_param("include_sources", self.include_sources);
        params.push_opt_param("order", self.order.as_ref());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("sort", self.sort.as_ref());
        params
    }
}

impl PaginatableRequest for GetBalanceSheetsRequest {
    type Item = BalanceSheet;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Income Statement
// ============================================================================

/// Income statement financial data.
#[derive(Debug, Clone, Deserialize)]
pub struct IncomeStatement {
    /// CIK number.
    pub cik: Option<String>,
    /// Company name.
    pub company_name: Option<String>,
    /// Stock tickers.
    #[serde(default)]
    pub tickers: Vec<String>,
    /// Start date of the period.
    pub start_date: Option<String>,
    /// End date of the period.
    pub end_date: Option<String>,
    /// Fiscal period.
    pub fiscal_period: Option<String>,
    /// Fiscal year.
    pub fiscal_year: Option<String>,
    /// Filing date.
    pub filing_date: Option<String>,

    // Revenue
    /// Total revenues.
    #[serde(default)]
    pub revenues: Option<FinancialValue>,
    /// Cost of revenue.
    #[serde(default)]
    pub cost_of_revenue: Option<FinancialValue>,
    /// Gross profit.
    #[serde(default)]
    pub gross_profit: Option<FinancialValue>,

    // Operating
    /// Operating expenses.
    #[serde(default)]
    pub operating_expenses: Option<FinancialValue>,
    /// Operating income.
    #[serde(default)]
    pub operating_income_loss: Option<FinancialValue>,

    // Net income
    /// Income before taxes.
    #[serde(default)]
    pub income_loss_from_continuing_operations_before_tax: Option<FinancialValue>,
    /// Income tax expense.
    #[serde(default)]
    pub income_tax_expense_benefit: Option<FinancialValue>,
    /// Net income.
    #[serde(default)]
    pub net_income_loss: Option<FinancialValue>,

    // Per share
    /// Basic EPS.
    #[serde(default)]
    pub basic_earnings_per_share: Option<FinancialValue>,
    /// Diluted EPS.
    #[serde(default)]
    pub diluted_earnings_per_share: Option<FinancialValue>,
    /// Basic shares outstanding.
    #[serde(default)]
    pub basic_average_shares: Option<FinancialValue>,
    /// Diluted shares outstanding.
    #[serde(default)]
    pub diluted_average_shares: Option<FinancialValue>,
}

impl IncomeStatement {
    /// Calculate gross margin (gross profit / revenues).
    pub fn gross_margin(&self) -> Option<f64> {
        let gross = self.gross_profit.as_ref()?.value?;
        let revenue = self.revenues.as_ref()?.value?;
        if revenue > 0.0 {
            Some((gross / revenue) * 100.0)
        } else {
            None
        }
    }

    /// Calculate operating margin (operating income / revenues).
    pub fn operating_margin(&self) -> Option<f64> {
        let operating = self.operating_income_loss.as_ref()?.value?;
        let revenue = self.revenues.as_ref()?.value?;
        if revenue > 0.0 {
            Some((operating / revenue) * 100.0)
        } else {
            None
        }
    }

    /// Calculate net margin (net income / revenues).
    pub fn net_margin(&self) -> Option<f64> {
        let net = self.net_income_loss.as_ref()?.value?;
        let revenue = self.revenues.as_ref()?.value?;
        if revenue > 0.0 {
            Some((net / revenue) * 100.0)
        } else {
            None
        }
    }

    /// Calculate effective tax rate.
    pub fn effective_tax_rate(&self) -> Option<f64> {
        let tax = self.income_tax_expense_benefit.as_ref()?.value?;
        let pretax = self
            .income_loss_from_continuing_operations_before_tax
            .as_ref()?
            .value?;
        if pretax.abs() > 0.0 {
            Some((tax / pretax) * 100.0)
        } else {
            None
        }
    }
}

/// Response from income statements endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IncomeStatementsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Income statement results.
    #[serde(default)]
    pub results: Vec<IncomeStatement>,
}

/// Request for income statements.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetIncomeStatementsRequest, FinancialTimeframe};
///
/// let request = GetIncomeStatementsRequest::new("AAPL")
///     .timeframe(FinancialTimeframe::Annual)
///     .limit(5);
/// ```
#[derive(Debug, Clone)]
pub struct GetIncomeStatementsRequest {
    /// Stock ticker.
    pub ticker: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Timeframe (annual, quarterly, ttm).
    pub timeframe: Option<FinancialTimeframe>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetIncomeStatementsRequest {
    /// Create a new request for a ticker.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: Some(ticker.into()),
            cik: None,
            timeframe: None,
            limit: None,
            order: None,
        }
    }

    /// Set the timeframe.
    pub fn timeframe(mut self, timeframe: FinancialTimeframe) -> Self {
        self.timeframe = Some(timeframe);
        self
    }

    /// Set the result limit.
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

impl RestRequest for GetIncomeStatementsRequest {
    type Response = IncomeStatementsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/vX/reference/financials".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.as_ref());
        params.push_opt_param("cik", self.cik.as_ref());
        if let Some(tf) = &self.timeframe {
            params.push((Cow::Borrowed("timeframe"), tf.as_str().to_string()));
        }
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetIncomeStatementsRequest {
    type Item = IncomeStatement;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Cash Flow Statement
// ============================================================================

/// Cash flow statement data.
#[derive(Debug, Clone, Deserialize)]
pub struct CashFlowStatement {
    /// CIK number.
    pub cik: Option<String>,
    /// Company name.
    pub company_name: Option<String>,
    /// Stock tickers.
    #[serde(default)]
    pub tickers: Vec<String>,
    /// Start date.
    pub start_date: Option<String>,
    /// End date.
    pub end_date: Option<String>,
    /// Fiscal period.
    pub fiscal_period: Option<String>,
    /// Fiscal year.
    pub fiscal_year: Option<String>,

    // Operating activities
    /// Net cash from operating activities.
    #[serde(default)]
    pub net_cash_flow_from_operating_activities: Option<FinancialValue>,

    // Investing activities
    /// Net cash from investing activities.
    #[serde(default)]
    pub net_cash_flow_from_investing_activities: Option<FinancialValue>,
    /// Capital expenditures.
    #[serde(default)]
    pub net_cash_flow_from_investing_activities_continuing: Option<FinancialValue>,

    // Financing activities
    /// Net cash from financing activities.
    #[serde(default)]
    pub net_cash_flow_from_financing_activities: Option<FinancialValue>,

    // Net change
    /// Net change in cash.
    #[serde(default)]
    pub net_cash_flow: Option<FinancialValue>,
}

impl CashFlowStatement {
    /// Calculate free cash flow (operating cash flow - capex).
    /// Note: This is an approximation using available data.
    pub fn free_cash_flow(&self) -> Option<f64> {
        let operating = self
            .net_cash_flow_from_operating_activities
            .as_ref()?
            .value?;
        let investing = self
            .net_cash_flow_from_investing_activities
            .as_ref()
            .and_then(|v| v.value)
            .unwrap_or(0.0);
        // Free cash flow = operating - capex (investing is negative for capex)
        Some(operating + investing)
    }

    /// Calculate cash flow coverage (operating / financing).
    pub fn cash_flow_coverage(&self) -> Option<f64> {
        let operating = self
            .net_cash_flow_from_operating_activities
            .as_ref()?
            .value?;
        let financing = self
            .net_cash_flow_from_financing_activities
            .as_ref()?
            .value?;
        if financing.abs() > 0.0 {
            Some(operating / financing.abs())
        } else {
            None
        }
    }
}

/// Response from cash flow statements endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CashFlowStatementsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Cash flow statement results.
    #[serde(default)]
    pub results: Vec<CashFlowStatement>,
}

/// Request for cash flow statements.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetCashFlowStatementsRequest, FinancialTimeframe};
///
/// let request = GetCashFlowStatementsRequest::new("AAPL")
///     .timeframe(FinancialTimeframe::Annual);
/// ```
#[derive(Debug, Clone)]
pub struct GetCashFlowStatementsRequest {
    /// Stock ticker.
    pub ticker: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Timeframe.
    pub timeframe: Option<FinancialTimeframe>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetCashFlowStatementsRequest {
    /// Create a new request for a ticker.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: Some(ticker.into()),
            cik: None,
            timeframe: None,
            limit: None,
            order: None,
        }
    }

    /// Set the timeframe.
    pub fn timeframe(mut self, timeframe: FinancialTimeframe) -> Self {
        self.timeframe = Some(timeframe);
        self
    }

    /// Set the result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetCashFlowStatementsRequest {
    type Response = CashFlowStatementsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/vX/reference/financials".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.as_ref());
        params.push_opt_param("cik", self.cik.as_ref());
        if let Some(tf) = &self.timeframe {
            params.push((Cow::Borrowed("timeframe"), tf.as_str().to_string()));
        }
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetCashFlowStatementsRequest {
    type Item = CashFlowStatement;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Short Interest
// ============================================================================

/// Short interest data.
#[derive(Debug, Clone, Deserialize)]
pub struct ShortInterest {
    /// Stock ticker.
    pub ticker: Option<String>,
    /// Settlement date.
    pub settlement_date: Option<String>,
    /// Short interest (number of shares).
    pub short_interest: Option<i64>,
    /// Average daily volume.
    pub avg_daily_volume: Option<f64>,
    /// Days to cover.
    pub days_to_cover: Option<f64>,
    /// Change from previous period.
    pub change: Option<i64>,
    /// Percent change.
    pub change_percent: Option<f64>,
}

impl ShortInterest {
    /// Calculate short interest as percentage of volume.
    pub fn short_percent_of_volume(&self) -> Option<f64> {
        let short = self.short_interest? as f64;
        let volume = self.avg_daily_volume?;
        if volume > 0.0 {
            Some((short / volume) * 100.0)
        } else {
            None
        }
    }
}

/// Response from short interest endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ShortInterestResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Short interest results.
    #[serde(default)]
    pub results: Vec<ShortInterest>,
}

/// Request for short interest data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetShortInterestRequest;
///
/// let request = GetShortInterestRequest::new("AAPL")
///     .limit(10);
/// ```
#[derive(Debug, Clone)]
pub struct GetShortInterestRequest {
    /// Stock ticker.
    pub ticker: String,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
}

impl GetShortInterestRequest {
    /// Create a new short interest request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            limit: None,
            order: None,
        }
    }

    /// Set the result limit.
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

impl RestRequest for GetShortInterestRequest {
    type Response = ShortInterestResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/reference/short-interest/{}", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params
    }
}

impl PaginatableRequest for GetShortInterestRequest {
    type Item = ShortInterest;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Short Volume
// ============================================================================

/// Daily short volume data.
#[derive(Debug, Clone, Deserialize)]
pub struct ShortVolume {
    /// Date.
    pub date: Option<String>,
    /// Stock ticker.
    pub ticker: Option<String>,
    /// Short volume (shares).
    pub short_volume: Option<i64>,
    /// Total volume.
    pub total_volume: Option<i64>,
    /// Short exempt volume.
    pub short_exempt_volume: Option<i64>,
}

impl ShortVolume {
    /// Calculate short volume percentage.
    pub fn short_percent(&self) -> Option<f64> {
        let short = self.short_volume? as f64;
        let total = self.total_volume? as f64;
        if total > 0.0 {
            Some((short / total) * 100.0)
        } else {
            None
        }
    }

    /// Calculate exempt percentage.
    pub fn exempt_percent(&self) -> Option<f64> {
        let exempt = self.short_exempt_volume? as f64;
        let total = self.total_volume? as f64;
        if total > 0.0 {
            Some((exempt / total) * 100.0)
        } else {
            None
        }
    }
}

/// Response from short volume endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ShortVolumeResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// Short volume results.
    #[serde(default)]
    pub results: Vec<ShortVolume>,
}

/// Request for short volume data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetShortVolumeRequest;
///
/// let request = GetShortVolumeRequest::new("AAPL")
///     .limit(30);
/// ```
#[derive(Debug, Clone)]
pub struct GetShortVolumeRequest {
    /// Stock ticker.
    pub ticker: String,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
    /// Date filter.
    pub date: Option<String>,
}

impl GetShortVolumeRequest {
    /// Create a new short volume request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            limit: None,
            order: None,
            date: None,
        }
    }

    /// Set the result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set a date filter.
    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }
}

impl RestRequest for GetShortVolumeRequest {
    type Response = ShortVolumeResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/reference/short-volume/{}", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params.push_opt_param("date", self.date.as_ref());
        params
    }
}

impl PaginatableRequest for GetShortVolumeRequest {
    type Item = ShortVolume;

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
    fn test_financial_timeframe() {
        assert_eq!(FinancialTimeframe::Annual.as_str(), "annual");
        assert_eq!(FinancialTimeframe::Quarterly.as_str(), "quarterly");
        assert_eq!(FinancialTimeframe::TrailingTwelveMonths.as_str(), "ttm");
    }

    #[test]
    fn test_balance_sheet_ratios() {
        let bs = BalanceSheet {
            cik: None,
            company_name: Some("Test Corp".to_string()),
            tickers: vec!["TEST".to_string()],
            start_date: None,
            end_date: None,
            fiscal_period: None,
            fiscal_year: None,
            filing_date: None,
            source_filing_url: None,
            assets: Some(FinancialValue {
                value: Some(1_000_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            current_assets: Some(FinancialValue {
                value: Some(500_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            noncurrent_assets: None,
            cash_and_cash_equivalents: None,
            accounts_receivable: None,
            inventory: None,
            fixed_assets: None,
            intangible_assets: None,
            liabilities: Some(FinancialValue {
                value: Some(400_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            current_liabilities: Some(FinancialValue {
                value: Some(200_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            noncurrent_liabilities: None,
            long_term_debt: None,
            accounts_payable: None,
            equity: Some(FinancialValue {
                value: Some(600_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            retained_earnings: None,
        };

        // Current ratio = 500k / 200k = 2.5
        assert!((bs.current_ratio().unwrap() - 2.5).abs() < 0.01);

        // Debt to equity = 400k / 600k = 0.667
        assert!((bs.debt_to_equity().unwrap() - 0.667).abs() < 0.01);

        // Working capital = 500k - 200k = 300k
        assert!((bs.working_capital().unwrap() - 300_000.0).abs() < 1.0);
    }

    #[test]
    fn test_income_statement_margins() {
        let is = IncomeStatement {
            cik: None,
            company_name: None,
            tickers: vec![],
            start_date: None,
            end_date: None,
            fiscal_period: None,
            fiscal_year: None,
            filing_date: None,
            revenues: Some(FinancialValue {
                value: Some(1_000_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            cost_of_revenue: None,
            gross_profit: Some(FinancialValue {
                value: Some(400_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            operating_expenses: None,
            operating_income_loss: Some(FinancialValue {
                value: Some(200_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            income_loss_from_continuing_operations_before_tax: Some(FinancialValue {
                value: Some(180_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            income_tax_expense_benefit: Some(FinancialValue {
                value: Some(36_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            net_income_loss: Some(FinancialValue {
                value: Some(144_000.0),
                unit: Some("USD".to_string()),
                label: None,
                order: None,
            }),
            basic_earnings_per_share: None,
            diluted_earnings_per_share: None,
            basic_average_shares: None,
            diluted_average_shares: None,
        };

        // Gross margin = 40%
        assert!((is.gross_margin().unwrap() - 40.0).abs() < 0.1);

        // Operating margin = 20%
        assert!((is.operating_margin().unwrap() - 20.0).abs() < 0.1);

        // Net margin = 14.4%
        assert!((is.net_margin().unwrap() - 14.4).abs() < 0.1);

        // Effective tax rate = 20%
        assert!((is.effective_tax_rate().unwrap() - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_short_interest_calculations() {
        let si = ShortInterest {
            ticker: Some("TEST".to_string()),
            settlement_date: Some("2024-01-15".to_string()),
            short_interest: Some(1_000_000),
            avg_daily_volume: Some(5_000_000.0),
            days_to_cover: Some(0.2),
            change: Some(50_000),
            change_percent: Some(5.0),
        };

        // Short percent of volume = 20%
        assert!((si.short_percent_of_volume().unwrap() - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_short_volume_calculations() {
        let sv = ShortVolume {
            date: Some("2024-01-15".to_string()),
            ticker: Some("TEST".to_string()),
            short_volume: Some(500_000),
            total_volume: Some(2_000_000),
            short_exempt_volume: Some(10_000),
        };

        // Short percent = 25%
        assert!((sv.short_percent().unwrap() - 25.0).abs() < 0.1);

        // Exempt percent = 0.5%
        assert!((sv.exempt_percent().unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_get_balance_sheets_request() {
        let req = GetBalanceSheetsRequest::new("AAPL")
            .timeframe(FinancialTimeframe::Quarterly)
            .limit(10);

        assert_eq!(req.path(), "/vX/reference/financials");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("ticker").unwrap(), "AAPL");
        assert_eq!(query_map.get("timeframe").unwrap(), "quarterly");
        assert_eq!(query_map.get("limit").unwrap(), "10");
    }

    #[test]
    fn test_get_income_statements_request() {
        let req = GetIncomeStatementsRequest::new("MSFT")
            .timeframe(FinancialTimeframe::Annual)
            .limit(5);

        assert_eq!(req.path(), "/vX/reference/financials");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("ticker").unwrap(), "MSFT");
        assert_eq!(query_map.get("timeframe").unwrap(), "annual");
    }

    #[test]
    fn test_get_short_interest_request() {
        let req = GetShortInterestRequest::new("GME").limit(10);

        assert_eq!(req.path(), "/v3/reference/short-interest/GME");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("limit").unwrap(), "10");
    }

    #[test]
    fn test_get_short_volume_request() {
        let req = GetShortVolumeRequest::new("AMC")
            .limit(30)
            .date("2024-01-15");

        assert_eq!(req.path(), "/v3/reference/short-volume/AMC");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("limit").unwrap(), "30");
        assert_eq!(query_map.get("date").unwrap(), "2024-01-15");
    }
}
