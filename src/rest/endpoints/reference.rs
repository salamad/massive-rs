//! Reference data endpoints (tickers, markets, exchanges).
//!
//! This module contains request types for fetching reference data
//! from the Massive API, including:
//!
//! - Ticker listings and details
//! - Exchange information with MIC codes
//! - Trade/quote condition codes
//! - Ticker type definitions
//! - Market holidays calendar

use crate::models::Ticker;
use crate::rest::models::ListEnvelope;
use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Market type filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketType {
    /// US equities
    Stocks,
    /// Options contracts
    Options,
    /// Cryptocurrency
    Crypto,
    /// Foreign exchange
    Forex,
    /// OTC securities
    Otc,
    /// Indices
    Indices,
}

impl std::fmt::Display for MarketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketType::Stocks => write!(f, "stocks"),
            MarketType::Options => write!(f, "options"),
            MarketType::Crypto => write!(f, "crypto"),
            MarketType::Forex => write!(f, "fx"),
            MarketType::Otc => write!(f, "otc"),
            MarketType::Indices => write!(f, "indices"),
        }
    }
}

/// Request for listing tickers.
///
/// Returns a list of tickers matching the filter criteria.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::{GetTickersRequest, MarketType};
///
/// let request = GetTickersRequest::default()
///     .market(MarketType::Stocks)
///     .search("AAPL")
///     .active(true)
///     .limit(100);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetTickersRequest {
    /// Filter by ticker symbol prefix
    pub ticker: Option<String>,
    /// Search by ticker or company name
    pub search: Option<String>,
    /// Filter by market type
    pub market: Option<MarketType>,
    /// Filter by exchange
    pub exchange: Option<String>,
    /// Filter by CUSIP
    pub cusip: Option<String>,
    /// Filter by CIK
    pub cik: Option<String>,
    /// Date for which to check ticker status
    pub date: Option<String>,
    /// Filter by active status
    pub active: Option<bool>,
    /// Maximum results per page
    pub limit: Option<u32>,
    /// Pagination cursor
    pub cursor: Option<String>,
}

impl GetTickersRequest {
    /// Filter by ticker symbol prefix.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Search by ticker or company name.
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// Filter by market type.
    pub fn market(mut self, market: MarketType) -> Self {
        self.market = Some(market);
        self
    }

    /// Filter by exchange.
    pub fn exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = Some(exchange.into());
        self
    }

    /// Filter by active status.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Set maximum results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the pagination cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}

impl RestRequest for GetTickersRequest {
    type Response = ListEnvelope<Ticker>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v3/reference/tickers".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.clone());
        params.push_opt_param("search", self.search.clone());
        params.push_opt_param("market", self.market.map(|m| m.to_string()));
        params.push_opt_param("exchange", self.exchange.clone());
        params.push_opt_param("cusip", self.cusip.clone());
        params.push_opt_param("cik", self.cik.clone());
        params.push_opt_param("date", self.date.clone());
        params.push_opt_param("active", self.active);
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("cursor", self.cursor.clone());
        params
    }
}

impl PaginatableRequest for GetTickersRequest {
    type Item = Ticker;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Request for ticker details.
///
/// Returns detailed information about a specific ticker.
#[derive(Debug, Clone)]
pub struct GetTickerDetailsRequest {
    /// Ticker symbol
    pub ticker: String,
    /// Date for which to get details
    pub date: Option<String>,
}

impl GetTickerDetailsRequest {
    /// Create a new ticker details request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            date: None,
        }
    }

    /// Set the date for historical data.
    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }
}

/// Response from ticker details endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerDetailsResponse {
    /// Status string
    pub status: String,
    /// Request ID
    pub request_id: String,
    /// Ticker details
    pub results: TickerDetails,
}

/// Detailed ticker information.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerDetails {
    /// Ticker symbol
    pub ticker: String,
    /// Company name
    pub name: String,
    /// Market type
    pub market: String,
    /// Locale (us, global)
    pub locale: String,
    /// Primary exchange
    pub primary_exchange: Option<String>,
    /// Asset type
    #[serde(rename = "type")]
    pub ticker_type: Option<String>,
    /// Whether the ticker is active
    pub active: bool,
    /// Currency code
    pub currency_name: Option<String>,
    /// CIK number
    pub cik: Option<String>,
    /// Composite FIGI
    pub composite_figi: Option<String>,
    /// Share class FIGI
    pub share_class_figi: Option<String>,
    /// Market cap
    pub market_cap: Option<f64>,
    /// Phone number
    pub phone_number: Option<String>,
    /// Company address
    pub address: Option<Address>,
    /// Company description
    pub description: Option<String>,
    /// SIC code
    pub sic_code: Option<String>,
    /// SIC description
    pub sic_description: Option<String>,
    /// Number of employees
    pub total_employees: Option<u64>,
    /// List date
    pub list_date: Option<String>,
    /// Company homepage URL
    pub homepage_url: Option<String>,
    /// Branding info
    pub branding: Option<Branding>,
    /// Share class shares outstanding
    pub share_class_shares_outstanding: Option<u64>,
    /// Weighted shares outstanding
    pub weighted_shares_outstanding: Option<u64>,
    /// Round lot size
    pub round_lot: Option<u32>,
}

/// Company address.
#[derive(Debug, Clone, Deserialize)]
pub struct Address {
    /// Street address
    pub address1: Option<String>,
    /// City
    pub city: Option<String>,
    /// State
    pub state: Option<String>,
    /// Postal code
    pub postal_code: Option<String>,
}

/// Company branding info.
#[derive(Debug, Clone, Deserialize)]
pub struct Branding {
    /// Logo URL
    pub logo_url: Option<String>,
    /// Icon URL
    pub icon_url: Option<String>,
}

impl RestRequest for GetTickerDetailsRequest {
    type Response = TickerDetailsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/reference/tickers/{}", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("date", self.date.clone());
        params
    }
}

// ============================================================================
// Exchange Types and Endpoints
// ============================================================================

/// Type of exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExchangeType {
    /// Primary exchange (NYSE, NASDAQ, etc.)
    Exchange,
    /// Alternative Trading System
    #[serde(rename = "TRF")]
    Trf,
    /// Securities Information Processor
    #[serde(rename = "SIP")]
    Sip,
}

impl std::fmt::Display for ExchangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExchangeType::Exchange => write!(f, "exchange"),
            ExchangeType::Trf => write!(f, "TRF"),
            ExchangeType::Sip => write!(f, "SIP"),
        }
    }
}

/// Exchange information with MIC codes.
///
/// Represents a trading venue where securities can be bought and sold.
#[derive(Debug, Clone, Deserialize)]
pub struct Exchange {
    /// Internal exchange identifier.
    pub id: i32,
    /// Full name of the exchange.
    pub name: String,
    /// Type of exchange (exchange, TRF, SIP).
    #[serde(rename = "type")]
    pub exchange_type: Option<ExchangeType>,
    /// Market Identifier Code (ISO 10383).
    pub mic: Option<String>,
    /// Operating MIC code for the exchange.
    pub operating_mic: Option<String>,
    /// Participant ID for SIP feeds.
    pub participant_id: Option<String>,
    /// Exchange acronym (e.g., "NYSE", "NASDAQ").
    pub acronym: Option<String>,
    /// Asset class traded on this exchange.
    pub asset_class: String,
    /// Locale/region of the exchange (us, global).
    pub locale: String,
    /// Exchange website URL.
    pub url: Option<String>,
}

impl Exchange {
    /// Check if this is a primary exchange.
    pub fn is_primary(&self) -> bool {
        matches!(self.exchange_type, Some(ExchangeType::Exchange))
    }

    /// Check if this is a Trade Reporting Facility.
    pub fn is_trf(&self) -> bool {
        matches!(self.exchange_type, Some(ExchangeType::Trf))
    }

    /// Check if this is a SIP feed.
    pub fn is_sip(&self) -> bool {
        matches!(self.exchange_type, Some(ExchangeType::Sip))
    }

    /// Get the display name, preferring acronym over full name.
    pub fn display_name(&self) -> &str {
        self.acronym.as_deref().unwrap_or(&self.name)
    }
}

/// Response from the exchanges endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangesResponse {
    /// Status string.
    pub status: String,
    /// Request ID.
    pub request_id: String,
    /// Number of results returned.
    pub count: i32,
    /// List of exchanges.
    pub results: Vec<Exchange>,
}

/// Request for listing exchanges.
///
/// Returns information about trading venues.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetExchangesRequest;
///
/// // Get all US stock exchanges
/// let request = GetExchangesRequest::default()
///     .asset_class("stocks")
///     .locale("us");
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetExchangesRequest {
    /// Filter by asset class (stocks, options, crypto, fx).
    pub asset_class: Option<String>,
    /// Filter by locale (us, global).
    pub locale: Option<String>,
}

impl GetExchangesRequest {
    /// Filter by asset class.
    pub fn asset_class(mut self, asset_class: impl Into<String>) -> Self {
        self.asset_class = Some(asset_class.into());
        self
    }

    /// Filter by locale.
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }
}

impl RestRequest for GetExchangesRequest {
    type Response = ExchangesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v3/reference/exchanges".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("asset_class", self.asset_class.clone());
        params.push_opt_param("locale", self.locale.clone());
        params
    }
}

// ============================================================================
// Condition Types and Endpoints
// ============================================================================

/// Type of condition (trade, quote, or both).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionType {
    /// Applies to trade data.
    Trade,
    /// Applies to quote (NBBO) data.
    Quote,
}

impl std::fmt::Display for ConditionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionType::Trade => write!(f, "trade"),
            ConditionType::Quote => write!(f, "quote"),
        }
    }
}

/// SIP (Securities Information Processor) mapping for a condition.
#[derive(Debug, Clone, Deserialize)]
pub struct SipMapping {
    /// Consolidated Tape Association (CTA) condition code.
    #[serde(rename = "CTA")]
    pub cta: Option<String>,
    /// UTP (Unlisted Trading Privileges) condition code.
    #[serde(rename = "UTP")]
    pub utp: Option<String>,
    /// Options Price Reporting Authority condition code.
    #[serde(rename = "OPRA")]
    pub opra: Option<String>,
}

/// Update rules for how conditions affect consolidated data.
#[derive(Debug, Clone, Deserialize)]
pub struct ConditionUpdateRules {
    /// Whether this condition updates the high price.
    pub consolidated_updates_high_low: Option<bool>,
    /// Whether this condition updates the last price.
    pub consolidated_updates_last: Option<bool>,
    /// Whether this condition updates volume.
    pub consolidated_updates_volume: Option<bool>,
    /// Whether this condition updates open price.
    pub consolidated_updates_open: Option<bool>,
    /// Market center update rules.
    pub market_center_updates_high_low: Option<bool>,
    /// Market center update rules.
    pub market_center_updates_last: Option<bool>,
    /// Market center update rules.
    pub market_center_updates_volume: Option<bool>,
    /// Market center update rules.
    pub market_center_updates_open: Option<bool>,
}

/// Trade or quote condition code with SIP mapping.
///
/// Conditions provide additional context about trades and quotes,
/// such as whether a trade was a regular sale, odd lot, or special condition.
#[derive(Debug, Clone, Deserialize)]
pub struct Condition {
    /// Condition ID used in trade/quote data.
    pub id: i32,
    /// Human-readable condition name.
    pub name: String,
    /// Type of condition (trade or quote).
    #[serde(rename = "type")]
    pub condition_type: ConditionType,
    /// Asset class this condition applies to.
    pub asset_class: String,
    /// Data types this condition applies to.
    pub data_types: Vec<String>,
    /// SIP feed mapping codes.
    pub sip_mapping: Option<SipMapping>,
    /// Rules for how this condition updates consolidated data.
    pub update_rules: Option<ConditionUpdateRules>,
    /// Legacy condition code.
    pub legacy: Option<bool>,
    /// Description of the condition.
    pub description: Option<String>,
    /// Abbreviated name.
    pub abbreviation: Option<String>,
}

impl Condition {
    /// Check if this is a trade condition.
    pub fn is_trade_condition(&self) -> bool {
        self.condition_type == ConditionType::Trade
    }

    /// Check if this is a quote condition.
    pub fn is_quote_condition(&self) -> bool {
        self.condition_type == ConditionType::Quote
    }

    /// Check if this condition updates consolidated volume.
    pub fn updates_volume(&self) -> bool {
        self.update_rules
            .as_ref()
            .and_then(|r| r.consolidated_updates_volume)
            .unwrap_or(false)
    }

    /// Check if this condition updates the last price.
    pub fn updates_last(&self) -> bool {
        self.update_rules
            .as_ref()
            .and_then(|r| r.consolidated_updates_last)
            .unwrap_or(false)
    }

    /// Get the CTA condition code if available.
    pub fn cta_code(&self) -> Option<&str> {
        self.sip_mapping.as_ref().and_then(|m| m.cta.as_deref())
    }
}

/// Response from the conditions endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ConditionsResponse {
    /// Status string.
    pub status: String,
    /// Request ID.
    pub request_id: String,
    /// Number of results returned.
    pub count: i32,
    /// List of conditions.
    pub results: Vec<Condition>,
}

/// Request for listing trade/quote conditions.
///
/// Returns condition codes used in trade and quote data.
///
/// # Example
///
/// ```
/// use massive_rs::rest::{GetConditionsRequest, ConditionType};
///
/// // Get all stock trade conditions
/// let request = GetConditionsRequest::default()
///     .asset_class("stocks")
///     .condition_type(ConditionType::Trade);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetConditionsRequest {
    /// Filter by asset class (stocks, options, crypto, fx).
    pub asset_class: Option<String>,
    /// Filter by data type (trade, bbo, nbbo).
    pub data_type: Option<String>,
    /// Filter by condition ID.
    pub id: Option<i32>,
    /// Filter by SIP feed.
    pub sip: Option<String>,
}

impl GetConditionsRequest {
    /// Filter by asset class.
    pub fn asset_class(mut self, asset_class: impl Into<String>) -> Self {
        self.asset_class = Some(asset_class.into());
        self
    }

    /// Filter by condition type (trade or quote).
    pub fn condition_type(mut self, condition_type: ConditionType) -> Self {
        self.data_type = Some(condition_type.to_string());
        self
    }

    /// Filter by data type.
    pub fn data_type(mut self, data_type: impl Into<String>) -> Self {
        self.data_type = Some(data_type.into());
        self
    }

    /// Filter by specific condition ID.
    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    /// Filter by SIP feed (CTA, UTP, OPRA).
    pub fn sip(mut self, sip: impl Into<String>) -> Self {
        self.sip = Some(sip.into());
        self
    }
}

impl RestRequest for GetConditionsRequest {
    type Response = ConditionsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v3/reference/conditions".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("asset_class", self.asset_class.clone());
        params.push_opt_param("data_type", self.data_type.clone());
        params.push_opt_param("id", self.id);
        params.push_opt_param("sip", self.sip.clone());
        params
    }
}

// ============================================================================
// Ticker Types Endpoint
// ============================================================================

/// Definition of a ticker type.
///
/// Describes the various types of securities and instruments.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerType {
    /// Short code for the ticker type (e.g., "CS" for common stock).
    pub code: String,
    /// Full description of the ticker type.
    pub description: String,
    /// Asset class this type belongs to.
    pub asset_class: String,
    /// Locale this type applies to.
    pub locale: String,
}

impl TickerType {
    /// Check if this is a common stock type.
    pub fn is_common_stock(&self) -> bool {
        self.code == "CS"
    }

    /// Check if this is a preferred stock type.
    pub fn is_preferred_stock(&self) -> bool {
        self.code == "PFD"
    }

    /// Check if this is an ETF type.
    pub fn is_etf(&self) -> bool {
        self.code == "ETF"
    }

    /// Check if this is a warrant type.
    pub fn is_warrant(&self) -> bool {
        self.code == "WARRANT"
    }

    /// Check if this is an ADR (American Depositary Receipt).
    pub fn is_adr(&self) -> bool {
        self.code == "ADRC"
    }
}

/// Response from the ticker types endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerTypesResponse {
    /// Status string.
    pub status: String,
    /// Request ID.
    pub request_id: String,
    /// Number of results returned.
    pub count: i32,
    /// List of ticker types.
    pub results: Vec<TickerType>,
}

/// Request for listing ticker types.
///
/// Returns definitions of all ticker type codes.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetTickerTypesRequest;
///
/// // Get all US stock ticker types
/// let request = GetTickerTypesRequest::default()
///     .asset_class("stocks")
///     .locale("us");
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetTickerTypesRequest {
    /// Filter by asset class (stocks, options, crypto, fx).
    pub asset_class: Option<String>,
    /// Filter by locale (us, global).
    pub locale: Option<String>,
}

impl GetTickerTypesRequest {
    /// Filter by asset class.
    pub fn asset_class(mut self, asset_class: impl Into<String>) -> Self {
        self.asset_class = Some(asset_class.into());
        self
    }

    /// Filter by locale.
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }
}

impl RestRequest for GetTickerTypesRequest {
    type Response = TickerTypesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v3/reference/tickers/types".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("asset_class", self.asset_class.clone());
        params.push_opt_param("locale", self.locale.clone());
        params
    }
}

// ============================================================================
// Market Holidays Endpoint
// ============================================================================

/// Market holiday information.
///
/// Represents a scheduled market closure or early close.
#[derive(Debug, Clone, Deserialize)]
pub struct MarketHoliday {
    /// Date of the holiday (YYYY-MM-DD).
    pub date: String,
    /// Name of the holiday.
    pub name: String,
    /// Exchange affected by this holiday.
    pub exchange: String,
    /// Market status on this day (open, closed, early-close).
    pub status: String,
    /// Opening time if early close (ISO 8601).
    pub open: Option<String>,
    /// Closing time if early close (ISO 8601).
    pub close: Option<String>,
}

impl MarketHoliday {
    /// Check if this is a full market closure.
    pub fn is_closed(&self) -> bool {
        self.status == "closed"
    }

    /// Check if this is an early close day.
    pub fn is_early_close(&self) -> bool {
        self.status == "early-close"
    }

    /// Check if the market is open (possibly early close).
    pub fn is_open(&self) -> bool {
        self.status == "open" || self.status == "early-close"
    }

    /// Get trading hours if available.
    ///
    /// Returns `Some((open, close))` if hours are specified.
    pub fn trading_hours(&self) -> Option<(&str, &str)> {
        match (self.open.as_deref(), self.close.as_deref()) {
            (Some(open), Some(close)) => Some((open, close)),
            _ => None,
        }
    }
}

/// Request for upcoming market holidays.
///
/// Returns a list of upcoming market holidays and early closes.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetMarketHolidaysRequest;
///
/// let request = GetMarketHolidaysRequest;
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GetMarketHolidaysRequest;

impl RestRequest for GetMarketHolidaysRequest {
    type Response = Vec<MarketHoliday>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v1/marketstatus/upcoming".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_type_display() {
        assert_eq!(MarketType::Stocks.to_string(), "stocks");
        assert_eq!(MarketType::Options.to_string(), "options");
        assert_eq!(MarketType::Crypto.to_string(), "crypto");
        assert_eq!(MarketType::Forex.to_string(), "fx");
        assert_eq!(MarketType::Otc.to_string(), "otc");
        assert_eq!(MarketType::Indices.to_string(), "indices");
    }

    #[test]
    fn test_get_tickers_request_path() {
        let req = GetTickersRequest::default();
        assert_eq!(req.path(), "/v3/reference/tickers");
    }

    #[test]
    fn test_get_tickers_request_query() {
        let req = GetTickersRequest::default()
            .market(MarketType::Stocks)
            .active(true)
            .search("Apple")
            .limit(50);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("market").unwrap(), "stocks");
        assert_eq!(query_map.get("active").unwrap(), "true");
        assert_eq!(query_map.get("search").unwrap(), "Apple");
        assert_eq!(query_map.get("limit").unwrap(), "50");
    }

    #[test]
    fn test_get_ticker_details_request() {
        let req = GetTickerDetailsRequest::new("AAPL");
        assert_eq!(req.path(), "/v3/reference/tickers/AAPL");

        let req_with_date = GetTickerDetailsRequest::new("AAPL").date("2024-01-15");
        let query = req_with_date.query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].1, "2024-01-15");
    }

    #[test]
    fn test_ticker_details_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "results": {
                "ticker": "AAPL",
                "name": "Apple Inc.",
                "market": "stocks",
                "locale": "us",
                "primary_exchange": "XNAS",
                "type": "CS",
                "active": true,
                "currency_name": "usd",
                "cik": "0000320193"
            }
        }"#;

        let response: TickerDetailsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "OK");
        assert_eq!(response.results.ticker, "AAPL");
        assert_eq!(response.results.name, "Apple Inc.");
        assert!(response.results.active);
    }

    // =========================================================================
    // Exchange Tests
    // =========================================================================

    #[test]
    fn test_exchange_type_display() {
        assert_eq!(ExchangeType::Exchange.to_string(), "exchange");
        assert_eq!(ExchangeType::Trf.to_string(), "TRF");
        assert_eq!(ExchangeType::Sip.to_string(), "SIP");
    }

    #[test]
    fn test_exchange_type_deserialize() {
        let json = r#""exchange""#;
        let et: ExchangeType = serde_json::from_str(json).unwrap();
        assert_eq!(et, ExchangeType::Exchange);

        let json = r#""TRF""#;
        let et: ExchangeType = serde_json::from_str(json).unwrap();
        assert_eq!(et, ExchangeType::Trf);
    }

    #[test]
    fn test_exchange_helpers() {
        let exchange = Exchange {
            id: 1,
            name: "New York Stock Exchange".to_string(),
            exchange_type: Some(ExchangeType::Exchange),
            mic: Some("XNYS".to_string()),
            operating_mic: Some("XNYS".to_string()),
            participant_id: Some("N".to_string()),
            acronym: Some("NYSE".to_string()),
            asset_class: "stocks".to_string(),
            locale: "us".to_string(),
            url: Some("https://www.nyse.com".to_string()),
        };

        assert!(exchange.is_primary());
        assert!(!exchange.is_trf());
        assert!(!exchange.is_sip());
        assert_eq!(exchange.display_name(), "NYSE");

        // Test without acronym
        let exchange_no_acronym = Exchange {
            id: 1,
            name: "Test Exchange".to_string(),
            exchange_type: None,
            mic: None,
            operating_mic: None,
            participant_id: None,
            acronym: None,
            asset_class: "stocks".to_string(),
            locale: "us".to_string(),
            url: None,
        };
        assert_eq!(exchange_no_acronym.display_name(), "Test Exchange");
    }

    #[test]
    fn test_get_exchanges_request() {
        let req = GetExchangesRequest::default()
            .asset_class("stocks")
            .locale("us");

        assert_eq!(req.path(), "/v3/reference/exchanges");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("asset_class").unwrap(), "stocks");
        assert_eq!(query_map.get("locale").unwrap(), "us");
    }

    #[test]
    fn test_exchanges_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "count": 1,
            "results": [{
                "id": 1,
                "name": "NYSE",
                "type": "exchange",
                "mic": "XNYS",
                "asset_class": "stocks",
                "locale": "us"
            }]
        }"#;

        let response: ExchangesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(response.results[0].name, "NYSE");
        assert!(response.results[0].is_primary());
    }

    // =========================================================================
    // Condition Tests
    // =========================================================================

    #[test]
    fn test_condition_type_display() {
        assert_eq!(ConditionType::Trade.to_string(), "trade");
        assert_eq!(ConditionType::Quote.to_string(), "quote");
    }

    #[test]
    fn test_condition_helpers() {
        let condition = Condition {
            id: 0,
            name: "Regular Sale".to_string(),
            condition_type: ConditionType::Trade,
            asset_class: "stocks".to_string(),
            data_types: vec!["trade".to_string()],
            sip_mapping: Some(SipMapping {
                cta: Some("@".to_string()),
                utp: Some("@".to_string()),
                opra: None,
            }),
            update_rules: Some(ConditionUpdateRules {
                consolidated_updates_high_low: Some(true),
                consolidated_updates_last: Some(true),
                consolidated_updates_volume: Some(true),
                consolidated_updates_open: Some(true),
                market_center_updates_high_low: Some(true),
                market_center_updates_last: Some(true),
                market_center_updates_volume: Some(true),
                market_center_updates_open: Some(true),
            }),
            legacy: None,
            description: Some("A regular sale".to_string()),
            abbreviation: Some("REG".to_string()),
        };

        assert!(condition.is_trade_condition());
        assert!(!condition.is_quote_condition());
        assert!(condition.updates_volume());
        assert!(condition.updates_last());
        assert_eq!(condition.cta_code(), Some("@"));
    }

    #[test]
    fn test_get_conditions_request() {
        let req = GetConditionsRequest::default()
            .asset_class("stocks")
            .condition_type(ConditionType::Trade);

        assert_eq!(req.path(), "/v3/reference/conditions");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("asset_class").unwrap(), "stocks");
        assert_eq!(query_map.get("data_type").unwrap(), "trade");
    }

    #[test]
    fn test_conditions_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "count": 1,
            "results": [{
                "id": 0,
                "name": "Regular Sale",
                "type": "trade",
                "asset_class": "stocks",
                "data_types": ["trade"]
            }]
        }"#;

        let response: ConditionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(response.results[0].name, "Regular Sale");
        assert!(response.results[0].is_trade_condition());
    }

    // =========================================================================
    // Ticker Types Tests
    // =========================================================================

    #[test]
    fn test_ticker_type_helpers() {
        let cs_type = TickerType {
            code: "CS".to_string(),
            description: "Common Stock".to_string(),
            asset_class: "stocks".to_string(),
            locale: "us".to_string(),
        };

        assert!(cs_type.is_common_stock());
        assert!(!cs_type.is_preferred_stock());
        assert!(!cs_type.is_etf());
        assert!(!cs_type.is_warrant());
        assert!(!cs_type.is_adr());

        let etf_type = TickerType {
            code: "ETF".to_string(),
            description: "Exchange Traded Fund".to_string(),
            asset_class: "stocks".to_string(),
            locale: "us".to_string(),
        };

        assert!(etf_type.is_etf());
        assert!(!etf_type.is_common_stock());
    }

    #[test]
    fn test_get_ticker_types_request() {
        let req = GetTickerTypesRequest::default()
            .asset_class("stocks")
            .locale("us");

        assert_eq!(req.path(), "/v3/reference/tickers/types");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("asset_class").unwrap(), "stocks");
        assert_eq!(query_map.get("locale").unwrap(), "us");
    }

    #[test]
    fn test_ticker_types_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "count": 2,
            "results": [
                {"code": "CS", "description": "Common Stock", "asset_class": "stocks", "locale": "us"},
                {"code": "ETF", "description": "Exchange Traded Fund", "asset_class": "stocks", "locale": "us"}
            ]
        }"#;

        let response: TickerTypesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 2);
        assert!(response.results[0].is_common_stock());
        assert!(response.results[1].is_etf());
    }

    // =========================================================================
    // Market Holidays Tests
    // =========================================================================

    #[test]
    fn test_market_holiday_helpers() {
        let closed_holiday = MarketHoliday {
            date: "2024-12-25".to_string(),
            name: "Christmas".to_string(),
            exchange: "NYSE".to_string(),
            status: "closed".to_string(),
            open: None,
            close: None,
        };

        assert!(closed_holiday.is_closed());
        assert!(!closed_holiday.is_early_close());
        assert!(!closed_holiday.is_open());
        assert!(closed_holiday.trading_hours().is_none());

        let early_close = MarketHoliday {
            date: "2024-12-24".to_string(),
            name: "Christmas Eve".to_string(),
            exchange: "NYSE".to_string(),
            status: "early-close".to_string(),
            open: Some("2024-12-24T09:30:00-05:00".to_string()),
            close: Some("2024-12-24T13:00:00-05:00".to_string()),
        };

        assert!(!early_close.is_closed());
        assert!(early_close.is_early_close());
        assert!(early_close.is_open());
        let (open, close) = early_close.trading_hours().unwrap();
        assert_eq!(open, "2024-12-24T09:30:00-05:00");
        assert_eq!(close, "2024-12-24T13:00:00-05:00");
    }

    #[test]
    fn test_get_market_holidays_request() {
        let req = GetMarketHolidaysRequest;
        assert_eq!(req.path(), "/v1/marketstatus/upcoming");
        assert!(req.query().is_empty());
    }

    #[test]
    fn test_market_holidays_response_deserialize() {
        let json = r#"[
            {"date": "2024-12-25", "name": "Christmas", "exchange": "NYSE", "status": "closed"},
            {"date": "2024-12-24", "name": "Christmas Eve", "exchange": "NYSE", "status": "early-close", "open": "09:30", "close": "13:00"}
        ]"#;

        let response: Vec<MarketHoliday> = serde_json::from_str(json).unwrap();
        assert_eq!(response.len(), 2);
        assert!(response[0].is_closed());
        assert!(response[1].is_early_close());
    }
}
