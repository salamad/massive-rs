//! Options contract endpoints.
//!
//! This module provides endpoints for options data, including:
//!
//! - Contract listings and details
//! - Options chain snapshots with greeks
//! - Options-specific trades and quotes
//!
//! # Feature Flag
//!
//! This module is available when the `options` feature is enabled.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{GetOptionsContractsRequest, GetOptionsChainRequest, ContractType};
//!
//! // List all options contracts for AAPL
//! let contracts = GetOptionsContractsRequest::new("AAPL")
//!     .contract_type(ContractType::Call)
//!     .expiration_date_gte("2024-06-01");
//!
//! // Get the full options chain
//! let chain = GetOptionsChainRequest::new("AAPL");
//! ```

use crate::rest::models::ListEnvelope;
use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

// ============================================================================
// Option Contract Types
// ============================================================================

/// Type of options contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractType {
    /// Call option - right to buy.
    #[default]
    Call,
    /// Put option - right to sell.
    Put,
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractType::Call => write!(f, "call"),
            ContractType::Put => write!(f, "put"),
        }
    }
}

/// Exercise style for options contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExerciseStyle {
    /// American-style (can be exercised any time before expiration).
    #[default]
    American,
    /// European-style (can only be exercised at expiration).
    European,
    /// Bermuda-style (can be exercised on specific dates).
    Bermuda,
}

impl std::fmt::Display for ExerciseStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExerciseStyle::American => write!(f, "american"),
            ExerciseStyle::European => write!(f, "european"),
            ExerciseStyle::Bermuda => write!(f, "bermuda"),
        }
    }
}

/// Additional underlying asset for complex options.
#[derive(Debug, Clone, Deserialize)]
pub struct AdditionalUnderlying {
    /// Underlying ticker symbol.
    pub underlying: String,
    /// Quantity of the underlying.
    pub amount: Option<f64>,
    /// Underlying asset type.
    #[serde(rename = "type")]
    pub underlying_type: Option<String>,
}

/// Options contract with full details.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OptionsContract {
    /// OCC options ticker symbol (e.g., "O:AAPL251219C00150000").
    pub ticker: String,
    /// Underlying ticker symbol.
    pub underlying_ticker: String,
    /// Contract type (call or put).
    pub contract_type: ContractType,
    /// Exercise style (american, european, bermuda).
    pub exercise_style: ExerciseStyle,
    /// Contract expiration date (YYYY-MM-DD).
    pub expiration_date: String,
    /// Strike price.
    pub strike_price: f64,
    /// Number of shares per contract (usually 100).
    pub shares_per_contract: i32,
    /// Primary exchange for this contract.
    pub primary_exchange: Option<String>,
    /// CFI (Classification of Financial Instruments) code.
    pub cfi: Option<String>,
    /// Additional underlying assets for complex options.
    #[serde(default)]
    pub additional_underlyings: Vec<AdditionalUnderlying>,
}

impl OptionsContract {
    /// Check if this is a call option.
    pub fn is_call(&self) -> bool {
        self.contract_type == ContractType::Call
    }

    /// Check if this is a put option.
    pub fn is_put(&self) -> bool {
        self.contract_type == ContractType::Put
    }

    /// Check if this is an American-style option.
    pub fn is_american(&self) -> bool {
        self.exercise_style == ExerciseStyle::American
    }

    /// Get the notional value per contract.
    pub fn notional_value(&self) -> f64 {
        self.strike_price * self.shares_per_contract as f64
    }

    /// Calculate the moneyness relative to the underlying price.
    ///
    /// Returns a positive value for ITM, negative for OTM.
    pub fn moneyness(&self, underlying_price: f64) -> f64 {
        match self.contract_type {
            ContractType::Call => underlying_price - self.strike_price,
            ContractType::Put => self.strike_price - underlying_price,
        }
    }

    /// Check if the option is in the money.
    pub fn is_itm(&self, underlying_price: f64) -> bool {
        self.moneyness(underlying_price) > 0.0
    }

    /// Check if the option is out of the money.
    pub fn is_otm(&self, underlying_price: f64) -> bool {
        self.moneyness(underlying_price) < 0.0
    }

    /// Check if the option is at the money (within 1%).
    pub fn is_atm(&self, underlying_price: f64) -> bool {
        (self.strike_price - underlying_price).abs() / underlying_price < 0.01
    }
}

/// Response from the options contract details endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OptionsContractResponse {
    /// Status string.
    pub status: String,
    /// Request ID.
    pub request_id: String,
    /// The contract details.
    pub results: OptionsContract,
}

// ============================================================================
// Options Contracts List Endpoint
// ============================================================================

/// Request for listing options contracts.
///
/// Returns a list of options contracts matching the filter criteria.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetOptionsContractsRequest, ContractType};
///
/// let request = GetOptionsContractsRequest::new("AAPL")
///     .contract_type(ContractType::Call)
///     .expiration_date_gte("2024-06-01")
///     .strike_price_gte(150.0)
///     .limit(100);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetOptionsContractsRequest {
    /// Filter by underlying ticker.
    pub underlying_ticker: Option<String>,
    /// Filter by exact option ticker.
    pub ticker: Option<String>,
    /// Filter by contract type.
    pub contract_type: Option<ContractType>,
    /// Filter by expiration date (exact).
    pub expiration_date: Option<String>,
    /// Filter by expiration date (greater than).
    pub expiration_date_gt: Option<String>,
    /// Filter by expiration date (greater than or equal).
    pub expiration_date_gte: Option<String>,
    /// Filter by expiration date (less than).
    pub expiration_date_lt: Option<String>,
    /// Filter by expiration date (less than or equal).
    pub expiration_date_lte: Option<String>,
    /// Filter by strike price (exact).
    pub strike_price: Option<f64>,
    /// Filter by strike price (greater than).
    pub strike_price_gt: Option<f64>,
    /// Filter by strike price (greater than or equal).
    pub strike_price_gte: Option<f64>,
    /// Filter by strike price (less than).
    pub strike_price_lt: Option<f64>,
    /// Filter by strike price (less than or equal).
    pub strike_price_lte: Option<f64>,
    /// Filter by expired status.
    pub expired: Option<bool>,
    /// Sort order.
    pub order: Option<String>,
    /// Maximum results per page.
    pub limit: Option<u32>,
    /// Sort field.
    pub sort: Option<String>,
    /// Pagination cursor.
    pub cursor: Option<String>,
}

impl GetOptionsContractsRequest {
    /// Create a new options contracts request.
    pub fn new(underlying_ticker: impl Into<String>) -> Self {
        Self {
            underlying_ticker: Some(underlying_ticker.into()),
            ..Default::default()
        }
    }

    /// Filter by exact option ticker.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by contract type.
    pub fn contract_type(mut self, contract_type: ContractType) -> Self {
        self.contract_type = Some(contract_type);
        self
    }

    /// Filter by exact expiration date.
    pub fn expiration_date(mut self, date: impl Into<String>) -> Self {
        self.expiration_date = Some(date.into());
        self
    }

    /// Filter by expiration date >= value.
    pub fn expiration_date_gte(mut self, date: impl Into<String>) -> Self {
        self.expiration_date_gte = Some(date.into());
        self
    }

    /// Filter by expiration date <= value.
    pub fn expiration_date_lte(mut self, date: impl Into<String>) -> Self {
        self.expiration_date_lte = Some(date.into());
        self
    }

    /// Filter by exact strike price.
    pub fn strike_price(mut self, price: f64) -> Self {
        self.strike_price = Some(price);
        self
    }

    /// Filter by strike price >= value.
    pub fn strike_price_gte(mut self, price: f64) -> Self {
        self.strike_price_gte = Some(price);
        self
    }

    /// Filter by strike price <= value.
    pub fn strike_price_lte(mut self, price: f64) -> Self {
        self.strike_price_lte = Some(price);
        self
    }

    /// Filter by expired status.
    pub fn expired(mut self, expired: bool) -> Self {
        self.expired = Some(expired);
        self
    }

    /// Set the sort order (asc or desc).
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Set the sort field.
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Set maximum results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetOptionsContractsRequest {
    type Response = ListEnvelope<OptionsContract>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v3/reference/options/contracts".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("underlying_ticker", self.underlying_ticker.clone());
        params.push_opt_param("ticker", self.ticker.clone());
        params.push_opt_param("contract_type", self.contract_type.map(|c| c.to_string()));
        params.push_opt_param("expiration_date", self.expiration_date.clone());
        params.push_opt_param("expiration_date.gt", self.expiration_date_gt.clone());
        params.push_opt_param("expiration_date.gte", self.expiration_date_gte.clone());
        params.push_opt_param("expiration_date.lt", self.expiration_date_lt.clone());
        params.push_opt_param("expiration_date.lte", self.expiration_date_lte.clone());
        params.push_opt_param("strike_price", self.strike_price);
        params.push_opt_param("strike_price.gt", self.strike_price_gt);
        params.push_opt_param("strike_price.gte", self.strike_price_gte);
        params.push_opt_param("strike_price.lt", self.strike_price_lt);
        params.push_opt_param("strike_price.lte", self.strike_price_lte);
        params.push_opt_param("expired", self.expired);
        params.push_opt_param("order", self.order.clone());
        params.push_opt_param("sort", self.sort.clone());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("cursor", self.cursor.clone());
        params
    }
}

impl PaginatableRequest for GetOptionsContractsRequest {
    type Item = OptionsContract;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Options Contract Details Endpoint
// ============================================================================

/// Request for a single options contract details.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetOptionsContractRequest;
///
/// let request = GetOptionsContractRequest::new("O:AAPL251219C00150000");
/// ```
#[derive(Debug, Clone)]
pub struct GetOptionsContractRequest {
    /// Options ticker symbol.
    pub ticker: String,
    /// Historical date for contract details.
    pub as_of: Option<String>,
}

impl GetOptionsContractRequest {
    /// Create a new options contract request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            as_of: None,
        }
    }

    /// Set historical date for contract details.
    pub fn as_of(mut self, date: impl Into<String>) -> Self {
        self.as_of = Some(date.into());
        self
    }
}

impl RestRequest for GetOptionsContractRequest {
    type Response = OptionsContractResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/reference/options/contracts/{}", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("as_of", self.as_of.clone());
        params
    }
}

// ============================================================================
// Options Chain Snapshot Types
// ============================================================================

/// Greeks for an options contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionGreeks {
    /// Delta - rate of change in option price per $1 change in underlying.
    pub delta: Option<f64>,
    /// Gamma - rate of change in delta per $1 change in underlying.
    pub gamma: Option<f64>,
    /// Theta - rate of time decay per day.
    pub theta: Option<f64>,
    /// Vega - rate of change per 1% change in implied volatility.
    pub vega: Option<f64>,
}

impl OptionGreeks {
    /// Check if this is a call-like position (positive delta).
    pub fn is_call_like(&self) -> bool {
        self.delta.map(|d| d > 0.0).unwrap_or(false)
    }

    /// Check if this is a put-like position (negative delta).
    pub fn is_put_like(&self) -> bool {
        self.delta.map(|d| d < 0.0).unwrap_or(false)
    }

    /// Estimate the dollar delta (delta * 100 * underlying_price).
    pub fn dollar_delta(&self, underlying_price: f64) -> Option<f64> {
        self.delta.map(|d| d * 100.0 * underlying_price)
    }
}

/// Day statistics for an options contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionDayStats {
    /// Open price.
    pub open: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// Trading volume.
    pub volume: Option<f64>,
    /// Volume-weighted average price.
    pub vwap: Option<f64>,
    /// Change from previous close.
    pub change: Option<f64>,
    /// Change percentage.
    pub change_percent: Option<f64>,
    /// Previous close.
    pub previous_close: Option<f64>,
    /// Last updated timestamp.
    pub last_updated: Option<i64>,
}

/// Details about an options contract in the chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChainDetails {
    /// Contract type (call or put).
    pub contract_type: ContractType,
    /// Exercise style.
    pub exercise_style: ExerciseStyle,
    /// Expiration date.
    pub expiration_date: String,
    /// Number of shares per contract.
    pub shares_per_contract: i32,
    /// Strike price.
    pub strike_price: f64,
    /// Options ticker symbol.
    pub ticker: String,
}

/// Last quote for an options contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionLastQuote {
    /// Ask price.
    pub ask: f64,
    /// Ask size.
    pub ask_size: Option<f64>,
    /// Ask exchange ID.
    pub ask_exchange: Option<u8>,
    /// Bid price.
    pub bid: f64,
    /// Bid size.
    pub bid_size: Option<f64>,
    /// Bid exchange ID.
    pub bid_exchange: Option<u8>,
    /// Midpoint price.
    pub midpoint: Option<f64>,
    /// Quote timestamp.
    pub last_updated: Option<i64>,
}

impl OptionLastQuote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate the spread as percentage of midpoint.
    pub fn spread_percent(&self) -> Option<f64> {
        let mid = self.midpoint.unwrap_or((self.bid + self.ask) / 2.0);
        if mid > 0.0 {
            Some((self.spread() / mid) * 100.0)
        } else {
            None
        }
    }
}

/// Last trade for an options contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionLastTrade {
    /// Trade price.
    pub price: f64,
    /// Trade size.
    pub size: Option<f64>,
    /// Exchange ID.
    pub exchange: Option<u8>,
    /// Trade conditions.
    #[serde(default)]
    pub conditions: Vec<i32>,
    /// Trade timestamp.
    pub last_updated: Option<i64>,
    /// SIP timestamp.
    pub sip_timestamp: Option<i64>,
}

/// Underlying asset information for an options contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionUnderlyingAsset {
    /// Underlying ticker symbol.
    pub ticker: String,
    /// Last price of the underlying.
    pub price: Option<f64>,
    /// Change in underlying price.
    pub change: Option<f64>,
    /// Change percentage of underlying.
    pub change_percent: Option<f64>,
    /// Change to break even.
    pub change_to_breakeven: Option<f64>,
    /// Change to break even percentage.
    pub change_to_breakeven_percent: Option<f64>,
}

/// Single result in an options chain snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChainResult {
    /// Break-even price.
    pub break_even_price: Option<f64>,
    /// Day statistics.
    pub day: Option<OptionDayStats>,
    /// Contract details.
    pub details: OptionChainDetails,
    /// Greeks.
    pub greeks: Option<OptionGreeks>,
    /// Implied volatility.
    pub implied_volatility: Option<f64>,
    /// Open interest.
    pub open_interest: Option<u64>,
    /// Last quote.
    pub last_quote: Option<OptionLastQuote>,
    /// Last trade.
    pub last_trade: Option<OptionLastTrade>,
    /// Underlying asset info.
    pub underlying_asset: Option<OptionUnderlyingAsset>,
}

impl OptionChainResult {
    /// Get the contract ticker.
    pub fn ticker(&self) -> &str {
        &self.details.ticker
    }

    /// Check if this is a call option.
    pub fn is_call(&self) -> bool {
        self.details.contract_type == ContractType::Call
    }

    /// Check if this is a put option.
    pub fn is_put(&self) -> bool {
        self.details.contract_type == ContractType::Put
    }

    /// Get the strike price.
    pub fn strike(&self) -> f64 {
        self.details.strike_price
    }

    /// Get the expiration date.
    pub fn expiration(&self) -> &str {
        &self.details.expiration_date
    }

    /// Get the mid price from the last quote.
    pub fn mid_price(&self) -> Option<f64> {
        self.last_quote.as_ref().map(|q| (q.bid + q.ask) / 2.0)
    }

    /// Get the delta value.
    pub fn delta(&self) -> Option<f64> {
        self.greeks.as_ref().and_then(|g| g.delta)
    }

    /// Calculate the intrinsic value given the underlying price.
    pub fn intrinsic_value(&self, underlying_price: f64) -> f64 {
        match self.details.contract_type {
            ContractType::Call => (underlying_price - self.details.strike_price).max(0.0),
            ContractType::Put => (self.details.strike_price - underlying_price).max(0.0),
        }
    }

    /// Calculate the extrinsic (time) value given the mid price and underlying.
    pub fn extrinsic_value(&self, underlying_price: f64) -> Option<f64> {
        let mid = self.mid_price()?;
        Some(mid - self.intrinsic_value(underlying_price))
    }
}

/// Response from the options chain snapshot endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OptionsChainResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Options chain results.
    pub results: Vec<OptionChainResult>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
}

impl OptionsChainResponse {
    /// Get all call options.
    pub fn calls(&self) -> impl Iterator<Item = &OptionChainResult> {
        self.results.iter().filter(|r| r.is_call())
    }

    /// Get all put options.
    pub fn puts(&self) -> impl Iterator<Item = &OptionChainResult> {
        self.results.iter().filter(|r| r.is_put())
    }

    /// Get options by expiration date.
    pub fn by_expiration(&self, expiration: &str) -> Vec<&OptionChainResult> {
        self.results
            .iter()
            .filter(|r| r.details.expiration_date == expiration)
            .collect()
    }

    /// Get options by strike price.
    pub fn by_strike(&self, strike: f64, tolerance: f64) -> Vec<&OptionChainResult> {
        self.results
            .iter()
            .filter(|r| (r.details.strike_price - strike).abs() <= tolerance)
            .collect()
    }

    /// Get all unique expiration dates.
    pub fn expirations(&self) -> Vec<&str> {
        let mut dates: Vec<&str> = self
            .results
            .iter()
            .map(|r| r.details.expiration_date.as_str())
            .collect();
        dates.sort();
        dates.dedup();
        dates
    }

    /// Get all unique strike prices.
    pub fn strikes(&self) -> Vec<f64> {
        let mut strikes: Vec<f64> = self
            .results
            .iter()
            .map(|r| r.details.strike_price)
            .collect();
        strikes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        strikes.dedup_by(|a, b| (*a - *b).abs() < 0.0001);
        strikes
    }
}

// ============================================================================
// Options Chain Snapshot Endpoint
// ============================================================================

/// Request for options chain snapshot.
///
/// Returns real-time data for all options contracts on an underlying.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetOptionsChainRequest, ContractType};
///
/// let request = GetOptionsChainRequest::new("AAPL")
///     .contract_type(ContractType::Call)
///     .expiration_date_gte("2024-06-01")
///     .strike_price_gte(150.0)
///     .strike_price_lte(200.0)
///     .limit(250);
/// ```
#[derive(Debug, Clone)]
pub struct GetOptionsChainRequest {
    /// Underlying ticker symbol.
    pub underlying_ticker: String,
    /// Filter by contract type.
    pub contract_type: Option<ContractType>,
    /// Filter by exact expiration date.
    pub expiration_date: Option<String>,
    /// Filter by expiration date >= value.
    pub expiration_date_gte: Option<String>,
    /// Filter by expiration date <= value.
    pub expiration_date_lte: Option<String>,
    /// Filter by exact strike price.
    pub strike_price: Option<f64>,
    /// Filter by strike price >= value.
    pub strike_price_gte: Option<f64>,
    /// Filter by strike price <= value.
    pub strike_price_lte: Option<f64>,
    /// Sort order.
    pub order: Option<String>,
    /// Maximum results per page.
    pub limit: Option<u32>,
    /// Sort field.
    pub sort: Option<String>,
    /// Pagination cursor.
    pub cursor: Option<String>,
}

impl GetOptionsChainRequest {
    /// Create a new options chain request.
    pub fn new(underlying_ticker: impl Into<String>) -> Self {
        Self {
            underlying_ticker: underlying_ticker.into(),
            contract_type: None,
            expiration_date: None,
            expiration_date_gte: None,
            expiration_date_lte: None,
            strike_price: None,
            strike_price_gte: None,
            strike_price_lte: None,
            order: None,
            limit: None,
            sort: None,
            cursor: None,
        }
    }

    /// Filter by contract type (call or put).
    pub fn contract_type(mut self, contract_type: ContractType) -> Self {
        self.contract_type = Some(contract_type);
        self
    }

    /// Filter by exact expiration date.
    pub fn expiration_date(mut self, date: impl Into<String>) -> Self {
        self.expiration_date = Some(date.into());
        self
    }

    /// Filter by expiration date >= value.
    pub fn expiration_date_gte(mut self, date: impl Into<String>) -> Self {
        self.expiration_date_gte = Some(date.into());
        self
    }

    /// Filter by expiration date <= value.
    pub fn expiration_date_lte(mut self, date: impl Into<String>) -> Self {
        self.expiration_date_lte = Some(date.into());
        self
    }

    /// Filter by exact strike price.
    pub fn strike_price(mut self, price: f64) -> Self {
        self.strike_price = Some(price);
        self
    }

    /// Filter by strike price >= value.
    pub fn strike_price_gte(mut self, price: f64) -> Self {
        self.strike_price_gte = Some(price);
        self
    }

    /// Filter by strike price <= value.
    pub fn strike_price_lte(mut self, price: f64) -> Self {
        self.strike_price_lte = Some(price);
        self
    }

    /// Set the sort order (asc or desc).
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Set the sort field.
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Set maximum results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetOptionsChainRequest {
    type Response = OptionsChainResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/snapshot/options/{}", self.underlying_ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("contract_type", self.contract_type.map(|c| c.to_string()));
        params.push_opt_param("expiration_date", self.expiration_date.clone());
        params.push_opt_param("expiration_date.gte", self.expiration_date_gte.clone());
        params.push_opt_param("expiration_date.lte", self.expiration_date_lte.clone());
        params.push_opt_param("strike_price", self.strike_price);
        params.push_opt_param("strike_price.gte", self.strike_price_gte);
        params.push_opt_param("strike_price.lte", self.strike_price_lte);
        params.push_opt_param("order", self.order.clone());
        params.push_opt_param("sort", self.sort.clone());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("cursor", self.cursor.clone());
        params
    }
}

impl PaginatableRequest for GetOptionsChainRequest {
    type Item = OptionChainResult;

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
    fn test_contract_type_display() {
        assert_eq!(ContractType::Call.to_string(), "call");
        assert_eq!(ContractType::Put.to_string(), "put");
    }

    #[test]
    fn test_exercise_style_display() {
        assert_eq!(ExerciseStyle::American.to_string(), "american");
        assert_eq!(ExerciseStyle::European.to_string(), "european");
        assert_eq!(ExerciseStyle::Bermuda.to_string(), "bermuda");
    }

    #[test]
    fn test_options_contract_helpers() {
        let contract = OptionsContract {
            ticker: "O:AAPL251219C00150000".to_string(),
            underlying_ticker: "AAPL".to_string(),
            contract_type: ContractType::Call,
            exercise_style: ExerciseStyle::American,
            expiration_date: "2025-12-19".to_string(),
            strike_price: 150.0,
            shares_per_contract: 100,
            primary_exchange: Some("CBOE".to_string()),
            cfi: None,
            additional_underlyings: vec![],
        };

        assert!(contract.is_call());
        assert!(!contract.is_put());
        assert!(contract.is_american());
        assert_eq!(contract.notional_value(), 15000.0);

        // ITM when underlying > strike for calls
        assert!(contract.is_itm(160.0));
        assert!(contract.is_otm(140.0));
        assert!(contract.is_atm(150.5));

        // Moneyness
        assert_eq!(contract.moneyness(160.0), 10.0);
        assert_eq!(contract.moneyness(140.0), -10.0);
    }

    #[test]
    fn test_put_contract_moneyness() {
        let put = OptionsContract {
            ticker: "O:AAPL251219P00150000".to_string(),
            underlying_ticker: "AAPL".to_string(),
            contract_type: ContractType::Put,
            exercise_style: ExerciseStyle::American,
            expiration_date: "2025-12-19".to_string(),
            strike_price: 150.0,
            shares_per_contract: 100,
            primary_exchange: None,
            cfi: None,
            additional_underlyings: vec![],
        };

        // ITM when strike > underlying for puts
        assert!(put.is_itm(140.0));
        assert!(put.is_otm(160.0));
        assert_eq!(put.moneyness(140.0), 10.0);
        assert_eq!(put.moneyness(160.0), -10.0);
    }

    #[test]
    fn test_get_options_contracts_request() {
        let req = GetOptionsContractsRequest::new("AAPL")
            .contract_type(ContractType::Call)
            .expiration_date_gte("2024-06-01")
            .strike_price_gte(150.0)
            .limit(100);

        assert_eq!(req.path(), "/v3/reference/options/contracts");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("underlying_ticker").unwrap(), "AAPL");
        assert_eq!(query_map.get("contract_type").unwrap(), "call");
        assert_eq!(query_map.get("expiration_date.gte").unwrap(), "2024-06-01");
        assert_eq!(query_map.get("strike_price.gte").unwrap(), "150");
    }

    #[test]
    fn test_get_options_contract_request() {
        let req = GetOptionsContractRequest::new("O:AAPL251219C00150000");
        assert_eq!(
            req.path(),
            "/v3/reference/options/contracts/O:AAPL251219C00150000"
        );
    }

    #[test]
    fn test_get_options_chain_request() {
        let req = GetOptionsChainRequest::new("AAPL")
            .contract_type(ContractType::Call)
            .strike_price_gte(150.0)
            .strike_price_lte(200.0);

        assert_eq!(req.path(), "/v3/snapshot/options/AAPL");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("contract_type").unwrap(), "call");
        assert_eq!(query_map.get("strike_price.gte").unwrap(), "150");
        assert_eq!(query_map.get("strike_price.lte").unwrap(), "200");
    }

    #[test]
    fn test_option_greeks_helpers() {
        let call_greeks = OptionGreeks {
            delta: Some(0.65),
            gamma: Some(0.02),
            theta: Some(-0.05),
            vega: Some(0.15),
        };

        assert!(call_greeks.is_call_like());
        assert!(!call_greeks.is_put_like());
        assert_eq!(call_greeks.dollar_delta(150.0), Some(0.65 * 100.0 * 150.0));
    }

    #[test]
    fn test_option_last_quote_helpers() {
        let quote = OptionLastQuote {
            ask: 5.00,
            ask_size: Some(10.0),
            ask_exchange: None,
            bid: 4.80,
            bid_size: Some(20.0),
            bid_exchange: None,
            midpoint: Some(4.90),
            last_updated: None,
        };

        assert!((quote.spread() - 0.20).abs() < 0.001);
        let spread_pct = quote.spread_percent().unwrap();
        assert!((spread_pct - 4.08).abs() < 0.1);
    }

    #[test]
    fn test_option_chain_result_helpers() {
        let result = OptionChainResult {
            break_even_price: Some(155.0),
            day: None,
            details: OptionChainDetails {
                contract_type: ContractType::Call,
                exercise_style: ExerciseStyle::American,
                expiration_date: "2025-12-19".to_string(),
                shares_per_contract: 100,
                strike_price: 150.0,
                ticker: "O:AAPL251219C00150000".to_string(),
            },
            greeks: Some(OptionGreeks {
                delta: Some(0.55),
                gamma: Some(0.02),
                theta: Some(-0.05),
                vega: Some(0.15),
            }),
            implied_volatility: Some(0.25),
            open_interest: Some(5000),
            last_quote: Some(OptionLastQuote {
                ask: 5.10,
                ask_size: Some(10.0),
                ask_exchange: None,
                bid: 4.90,
                bid_size: Some(20.0),
                bid_exchange: None,
                midpoint: Some(5.0),
                last_updated: None,
            }),
            last_trade: None,
            underlying_asset: None,
        };

        assert!(result.is_call());
        assert!(!result.is_put());
        assert_eq!(result.strike(), 150.0);
        assert_eq!(result.expiration(), "2025-12-19");
        assert_eq!(result.mid_price(), Some(5.0));
        assert_eq!(result.delta(), Some(0.55));

        // Intrinsic value when underlying = 160
        assert_eq!(result.intrinsic_value(160.0), 10.0);
        assert_eq!(result.intrinsic_value(140.0), 0.0);

        // Extrinsic value = mid - intrinsic
        assert_eq!(result.extrinsic_value(160.0), Some(-5.0)); // ITM
        assert_eq!(result.extrinsic_value(140.0), Some(5.0)); // OTM (all extrinsic)
    }

    #[test]
    fn test_options_chain_response_helpers() {
        let response = OptionsChainResponse {
            status: Some("OK".to_string()),
            request_id: Some("abc123".to_string()),
            results: vec![
                OptionChainResult {
                    break_even_price: None,
                    day: None,
                    details: OptionChainDetails {
                        contract_type: ContractType::Call,
                        exercise_style: ExerciseStyle::American,
                        expiration_date: "2025-06-20".to_string(),
                        shares_per_contract: 100,
                        strike_price: 150.0,
                        ticker: "O:AAPL250620C00150000".to_string(),
                    },
                    greeks: None,
                    implied_volatility: None,
                    open_interest: None,
                    last_quote: None,
                    last_trade: None,
                    underlying_asset: None,
                },
                OptionChainResult {
                    break_even_price: None,
                    day: None,
                    details: OptionChainDetails {
                        contract_type: ContractType::Put,
                        exercise_style: ExerciseStyle::American,
                        expiration_date: "2025-06-20".to_string(),
                        shares_per_contract: 100,
                        strike_price: 150.0,
                        ticker: "O:AAPL250620P00150000".to_string(),
                    },
                    greeks: None,
                    implied_volatility: None,
                    open_interest: None,
                    last_quote: None,
                    last_trade: None,
                    underlying_asset: None,
                },
                OptionChainResult {
                    break_even_price: None,
                    day: None,
                    details: OptionChainDetails {
                        contract_type: ContractType::Call,
                        exercise_style: ExerciseStyle::American,
                        expiration_date: "2025-12-19".to_string(),
                        shares_per_contract: 100,
                        strike_price: 160.0,
                        ticker: "O:AAPL251219C00160000".to_string(),
                    },
                    greeks: None,
                    implied_volatility: None,
                    open_interest: None,
                    last_quote: None,
                    last_trade: None,
                    underlying_asset: None,
                },
            ],
            next_url: None,
        };

        assert_eq!(response.calls().count(), 2);
        assert_eq!(response.puts().count(), 1);
        assert_eq!(response.expirations().len(), 2);
        assert_eq!(response.strikes().len(), 2);
        assert_eq!(response.by_expiration("2025-06-20").len(), 2);
        assert_eq!(response.by_strike(150.0, 0.01).len(), 2);
    }

    #[test]
    fn test_options_contract_deserialize() {
        let json = r#"{
            "ticker": "O:AAPL251219C00150000",
            "underlying_ticker": "AAPL",
            "contract_type": "call",
            "exercise_style": "american",
            "expiration_date": "2025-12-19",
            "strike_price": 150.0,
            "shares_per_contract": 100,
            "primary_exchange": "CBOE"
        }"#;

        let contract: OptionsContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.ticker, "O:AAPL251219C00150000");
        assert!(contract.is_call());
        assert!(contract.is_american());
    }

    #[test]
    fn test_options_chain_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "results": [
                {
                    "details": {
                        "contract_type": "call",
                        "exercise_style": "american",
                        "expiration_date": "2025-12-19",
                        "shares_per_contract": 100,
                        "strike_price": 150.0,
                        "ticker": "O:AAPL251219C00150000"
                    },
                    "greeks": {
                        "delta": 0.55,
                        "gamma": 0.02,
                        "theta": -0.05,
                        "vega": 0.15
                    },
                    "implied_volatility": 0.25,
                    "open_interest": 5000
                }
            ]
        }"#;

        let response: OptionsChainResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 1);
        assert!(response.results[0].is_call());
        assert_eq!(response.results[0].delta(), Some(0.55));
    }
}
