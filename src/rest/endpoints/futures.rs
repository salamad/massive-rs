//! Futures contract endpoints.
//!
//! This module provides endpoints for futures data, including:
//!
//! - Contract listings and details
//! - Product catalog and specifications
//! - Trading schedules
//! - Futures snapshots
//!
//! # Feature Flag
//!
//! This module is available when the `futures` feature is enabled.
//!
//! # API Version
//!
//! Note: The Futures API uses experimental `vX` versioning. Handle potential
//! API changes gracefully.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{GetFuturesContractsRequest, GetFuturesProductsRequest};
//!
//! // List all active futures contracts
//! let contracts = GetFuturesContractsRequest::default()
//!     .active(true);
//!
//! // Get product catalog
//! let products = GetFuturesProductsRequest::default();
//! ```

use crate::rest::models::ListEnvelope;
use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// API version for futures endpoints (experimental).
const FUTURES_API_VERSION: &str = "vX";

// ============================================================================
// Futures Contract Types
// ============================================================================

/// Type of futures contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FuturesContractType {
    /// Standard futures contract.
    #[default]
    Standard,
    /// Mini futures contract.
    Mini,
    /// Micro futures contract.
    Micro,
    /// Spread contract.
    Spread,
    /// Calendar spread.
    Calendar,
}

impl std::fmt::Display for FuturesContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuturesContractType::Standard => write!(f, "standard"),
            FuturesContractType::Mini => write!(f, "mini"),
            FuturesContractType::Micro => write!(f, "micro"),
            FuturesContractType::Spread => write!(f, "spread"),
            FuturesContractType::Calendar => write!(f, "calendar"),
        }
    }
}

/// Settlement type for futures contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettlementType {
    /// Cash settlement.
    #[default]
    Cash,
    /// Physical delivery.
    Physical,
}

/// Futures contract with maturity tracking.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FuturesContract {
    /// Futures ticker symbol.
    pub ticker: String,
    /// Contract name.
    pub name: String,
    /// Product code (e.g., "ES" for E-mini S&P 500).
    pub product_code: String,
    /// First trading date.
    pub first_trade_date: Option<String>,
    /// Last trading date.
    pub last_trade_date: Option<String>,
    /// Settlement date.
    pub settlement_date: Option<String>,
    /// Contract maturity date.
    pub maturity: Option<String>,
    /// Days until maturity.
    pub days_to_maturity: Option<i32>,
    /// Whether the contract is currently active.
    #[serde(default)]
    pub active: bool,
    /// Contract type.
    pub contract_type: Option<FuturesContractType>,
    /// Trading venue.
    pub trading_venue: Option<String>,
    /// Minimum price increment (tick size).
    pub trade_tick_size: Option<f64>,
    /// Minimum order quantity.
    pub min_order_quantity: Option<i32>,
    /// Maximum order quantity.
    pub max_order_quantity: Option<i32>,
    /// Contract multiplier.
    pub contract_multiplier: Option<f64>,
    /// Contract unit (e.g., "barrels", "bushels").
    pub contract_unit: Option<String>,
    /// Settlement type.
    pub settlement_type: Option<SettlementType>,
}

impl FuturesContract {
    /// Check if contract is currently tradeable.
    pub fn is_tradeable(&self) -> bool {
        self.active && self.days_to_maturity.map(|d| d > 0).unwrap_or(false)
    }

    /// Check if contract is near expiration (within 5 days).
    pub fn is_near_expiry(&self) -> bool {
        self.days_to_maturity
            .map(|d| d <= 5 && d > 0)
            .unwrap_or(false)
    }

    /// Check if contract has expired.
    pub fn is_expired(&self) -> bool {
        self.days_to_maturity.map(|d| d <= 0).unwrap_or(false)
    }

    /// Get the notional value per contract at a given price.
    pub fn notional_value(&self, price: f64) -> Option<f64> {
        self.contract_multiplier.map(|m| price * m)
    }

    /// Get the value of one tick move.
    pub fn tick_value(&self) -> Option<f64> {
        match (self.trade_tick_size, self.contract_multiplier) {
            (Some(tick), Some(mult)) => Some(tick * mult),
            _ => None,
        }
    }
}

/// Response from the futures contract details endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesContractResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// The contract details.
    pub results: FuturesContract,
}

// ============================================================================
// Futures Contracts List Endpoint
// ============================================================================

/// Request for listing futures contracts.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFuturesContractsRequest;
///
/// let request = GetFuturesContractsRequest::default()
///     .product_code("ES")
///     .active(true)
///     .limit(100);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetFuturesContractsRequest {
    /// Filter by product code.
    pub product_code: Option<String>,
    /// Filter by ticker.
    pub ticker: Option<String>,
    /// Filter by active status.
    pub active: Option<bool>,
    /// Filter by trading venue.
    pub trading_venue: Option<String>,
    /// Filter by expiration date (greater than).
    pub expiration_date_gt: Option<String>,
    /// Filter by expiration date (greater than or equal).
    pub expiration_date_gte: Option<String>,
    /// Filter by expiration date (less than).
    pub expiration_date_lt: Option<String>,
    /// Filter by expiration date (less than or equal).
    pub expiration_date_lte: Option<String>,
    /// Sort order.
    pub order: Option<String>,
    /// Maximum results per page.
    pub limit: Option<u32>,
    /// Sort field.
    pub sort: Option<String>,
    /// Pagination cursor.
    pub cursor: Option<String>,
}

impl GetFuturesContractsRequest {
    /// Filter by product code (e.g., "ES", "NQ", "CL").
    pub fn product_code(mut self, code: impl Into<String>) -> Self {
        self.product_code = Some(code.into());
        self
    }

    /// Filter by exact ticker.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by active status.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Filter by trading venue.
    pub fn trading_venue(mut self, venue: impl Into<String>) -> Self {
        self.trading_venue = Some(venue.into());
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

    /// Set sort order (asc or desc).
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Set maximum results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetFuturesContractsRequest {
    type Response = ListEnvelope<FuturesContract>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/futures/{}/contracts", FUTURES_API_VERSION).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("product_code", self.product_code.clone());
        params.push_opt_param("ticker", self.ticker.clone());
        params.push_opt_param("active", self.active);
        params.push_opt_param("trading_venue", self.trading_venue.clone());
        params.push_opt_param("expiration_date.gt", self.expiration_date_gt.clone());
        params.push_opt_param("expiration_date.gte", self.expiration_date_gte.clone());
        params.push_opt_param("expiration_date.lt", self.expiration_date_lt.clone());
        params.push_opt_param("expiration_date.lte", self.expiration_date_lte.clone());
        params.push_opt_param("order", self.order.clone());
        params.push_opt_param("sort", self.sort.clone());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("cursor", self.cursor.clone());
        params
    }
}

impl PaginatableRequest for GetFuturesContractsRequest {
    type Item = FuturesContract;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Futures Contract Details Endpoint
// ============================================================================

/// Request for a single futures contract details.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFuturesContractRequest;
///
/// let request = GetFuturesContractRequest::new("ESH4");
/// ```
#[derive(Debug, Clone)]
pub struct GetFuturesContractRequest {
    /// Futures ticker symbol.
    pub ticker: String,
}

impl GetFuturesContractRequest {
    /// Create a new futures contract request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
        }
    }
}

impl RestRequest for GetFuturesContractRequest {
    type Response = FuturesContractResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/futures/{}/contracts/{}", FUTURES_API_VERSION, self.ticker).into()
    }
}

// ============================================================================
// Futures Products
// ============================================================================

/// Futures product specification.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FuturesProduct {
    /// Product code (e.g., "ES", "NQ", "CL").
    pub code: String,
    /// Product name.
    pub name: String,
    /// Product description.
    pub description: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Trading venue.
    pub trading_venue: Option<String>,
    /// Contract multiplier.
    pub contract_multiplier: Option<f64>,
    /// Contract unit.
    pub contract_unit: Option<String>,
    /// Minimum tick size.
    pub tick_size: Option<f64>,
    /// Tick value in dollars.
    pub tick_value: Option<f64>,
    /// Settlement type.
    pub settlement_type: Option<SettlementType>,
    /// Currency.
    pub currency: Option<String>,
    /// Underlying asset.
    pub underlying: Option<String>,
}

impl FuturesProduct {
    /// Calculate the value of a price move.
    pub fn price_move_value(&self, price_move: f64) -> Option<f64> {
        match (self.tick_size, self.tick_value) {
            (Some(tick_size), Some(tick_value)) if tick_size > 0.0 => {
                Some((price_move / tick_size) * tick_value)
            }
            _ => None,
        }
    }
}

/// Response from the futures product details endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesProductResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// The product details.
    pub results: FuturesProduct,
}

/// Request for listing futures products.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFuturesProductsRequest;
///
/// let request = GetFuturesProductsRequest::default()
///     .asset_class("equity")
///     .limit(100);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetFuturesProductsRequest {
    /// Filter by asset class.
    pub asset_class: Option<String>,
    /// Filter by trading venue.
    pub trading_venue: Option<String>,
    /// Maximum results per page.
    pub limit: Option<u32>,
    /// Pagination cursor.
    pub cursor: Option<String>,
}

impl GetFuturesProductsRequest {
    /// Filter by asset class.
    pub fn asset_class(mut self, asset_class: impl Into<String>) -> Self {
        self.asset_class = Some(asset_class.into());
        self
    }

    /// Filter by trading venue.
    pub fn trading_venue(mut self, venue: impl Into<String>) -> Self {
        self.trading_venue = Some(venue.into());
        self
    }

    /// Set maximum results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl RestRequest for GetFuturesProductsRequest {
    type Response = ListEnvelope<FuturesProduct>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/futures/{}/products", FUTURES_API_VERSION).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("asset_class", self.asset_class.clone());
        params.push_opt_param("trading_venue", self.trading_venue.clone());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("cursor", self.cursor.clone());
        params
    }
}

impl PaginatableRequest for GetFuturesProductsRequest {
    type Item = FuturesProduct;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Request for a single futures product details.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFuturesProductRequest;
///
/// let request = GetFuturesProductRequest::new("ES");
/// ```
#[derive(Debug, Clone)]
pub struct GetFuturesProductRequest {
    /// Product code.
    pub code: String,
}

impl GetFuturesProductRequest {
    /// Create a new futures product request.
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

impl RestRequest for GetFuturesProductRequest {
    type Response = FuturesProductResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/futures/{}/products/{}", FUTURES_API_VERSION, self.code).into()
    }
}

// ============================================================================
// Trading Schedules
// ============================================================================

/// Trading session within a schedule.
#[derive(Debug, Clone, Deserialize)]
pub struct TradingSession {
    /// Session type (e.g., "regular", "extended").
    pub session_type: Option<String>,
    /// Session open time.
    pub open: Option<String>,
    /// Session close time.
    pub close: Option<String>,
    /// Days of week this session applies.
    pub days: Option<Vec<String>>,
}

/// Trading schedule for a futures product.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FuturesSchedule {
    /// Product code.
    pub product_code: Option<String>,
    /// Trading venue.
    pub trading_venue: Option<String>,
    /// Timezone for schedule times.
    pub timezone: Option<String>,
    /// Trading sessions.
    #[serde(default)]
    pub sessions: Vec<TradingSession>,
}

/// Request for listing all trading schedules.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFuturesSchedulesRequest;
///
/// let request = GetFuturesSchedulesRequest::default();
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetFuturesSchedulesRequest {
    /// Filter by trading venue.
    pub trading_venue: Option<String>,
}

impl GetFuturesSchedulesRequest {
    /// Filter by trading venue.
    pub fn trading_venue(mut self, venue: impl Into<String>) -> Self {
        self.trading_venue = Some(venue.into());
        self
    }
}

impl RestRequest for GetFuturesSchedulesRequest {
    type Response = ListEnvelope<FuturesSchedule>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/futures/{}/schedules", FUTURES_API_VERSION).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("trading_venue", self.trading_venue.clone());
        params
    }
}

/// Request for a product's trading schedule.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFuturesProductScheduleRequest;
///
/// let request = GetFuturesProductScheduleRequest::new("ES");
/// ```
#[derive(Debug, Clone)]
pub struct GetFuturesProductScheduleRequest {
    /// Product code.
    pub code: String,
}

impl GetFuturesProductScheduleRequest {
    /// Create a new product schedule request.
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

impl RestRequest for GetFuturesProductScheduleRequest {
    type Response = ListEnvelope<FuturesSchedule>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/futures/{}/products/{}/schedules",
            FUTURES_API_VERSION, self.code
        )
        .into()
    }
}

// ============================================================================
// Futures Snapshot
// ============================================================================

/// Day statistics for a futures contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesDayStats {
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
    /// Open interest.
    pub open_interest: Option<u64>,
    /// Settlement price.
    pub settlement: Option<f64>,
    /// Previous close.
    pub previous_close: Option<f64>,
}

/// Quote data for a futures contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesQuote {
    /// Bid price.
    pub bid: Option<f64>,
    /// Bid size.
    pub bid_size: Option<f64>,
    /// Ask price.
    pub ask: Option<f64>,
    /// Ask size.
    pub ask_size: Option<f64>,
    /// Quote timestamp.
    pub timestamp: Option<i64>,
}

impl FuturesQuote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> Option<f64> {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Calculate the midpoint price.
    pub fn midpoint(&self) -> Option<f64> {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }
}

/// Trade data for a futures contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesTrade {
    /// Trade price.
    pub price: Option<f64>,
    /// Trade size.
    pub size: Option<f64>,
    /// Trade timestamp.
    pub timestamp: Option<i64>,
}

/// Snapshot for a futures contract.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesSnapshotResult {
    /// Ticker symbol.
    pub ticker: String,
    /// Product code.
    pub product_code: Option<String>,
    /// Contract name.
    pub name: Option<String>,
    /// Days to maturity.
    pub days_to_maturity: Option<i32>,
    /// Day statistics.
    pub day: Option<FuturesDayStats>,
    /// Last quote.
    pub last_quote: Option<FuturesQuote>,
    /// Last trade.
    pub last_trade: Option<FuturesTrade>,
    /// Session change.
    pub change: Option<f64>,
    /// Session change percentage.
    pub change_percent: Option<f64>,
}

impl FuturesSnapshotResult {
    /// Get the current price from the last trade.
    pub fn price(&self) -> Option<f64> {
        self.last_trade.as_ref().and_then(|t| t.price)
    }

    /// Check if the contract is near expiration.
    pub fn is_near_expiry(&self) -> bool {
        self.days_to_maturity
            .map(|d| d <= 5 && d > 0)
            .unwrap_or(false)
    }
}

/// Response from the futures snapshot endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesSnapshotResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Snapshot results.
    pub results: Vec<FuturesSnapshotResult>,
}

/// Request for futures snapshot.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetFuturesSnapshotRequest;
///
/// let request = GetFuturesSnapshotRequest::new(&["ESH4", "NQH4"]);
/// ```
#[derive(Debug, Clone)]
pub struct GetFuturesSnapshotRequest {
    /// Tickers to include.
    pub tickers: Vec<String>,
}

impl GetFuturesSnapshotRequest {
    /// Create a new futures snapshot request.
    pub fn new(tickers: &[&str]) -> Self {
        Self {
            tickers: tickers.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create from a vector of ticker strings.
    pub fn from_vec(tickers: Vec<String>) -> Self {
        Self { tickers }
    }

    /// Add a ticker to the request.
    pub fn add_ticker(mut self, ticker: impl Into<String>) -> Self {
        self.tickers.push(ticker.into());
        self
    }
}

impl RestRequest for GetFuturesSnapshotRequest {
    type Response = FuturesSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/futures/{}/snapshot", FUTURES_API_VERSION).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        if !self.tickers.is_empty() {
            params.push_opt_param("ticker.any_of", Some(self.tickers.join(",")));
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_futures_contract_type_display() {
        assert_eq!(FuturesContractType::Standard.to_string(), "standard");
        assert_eq!(FuturesContractType::Mini.to_string(), "mini");
        assert_eq!(FuturesContractType::Micro.to_string(), "micro");
    }

    #[test]
    fn test_futures_contract_helpers() {
        let contract = FuturesContract {
            ticker: "ESH4".to_string(),
            name: "E-mini S&P 500 March 2024".to_string(),
            product_code: "ES".to_string(),
            first_trade_date: Some("2023-03-20".to_string()),
            last_trade_date: Some("2024-03-15".to_string()),
            settlement_date: Some("2024-03-15".to_string()),
            maturity: Some("2024-03-15".to_string()),
            days_to_maturity: Some(30),
            active: true,
            contract_type: Some(FuturesContractType::Mini),
            trading_venue: Some("CME".to_string()),
            trade_tick_size: Some(0.25),
            min_order_quantity: Some(1),
            max_order_quantity: Some(10000),
            contract_multiplier: Some(50.0),
            contract_unit: Some("index points".to_string()),
            settlement_type: Some(SettlementType::Cash),
        };

        assert!(contract.is_tradeable());
        assert!(!contract.is_near_expiry());
        assert!(!contract.is_expired());
        assert_eq!(contract.notional_value(5000.0), Some(250000.0));
        assert_eq!(contract.tick_value(), Some(12.5)); // 0.25 * 50
    }

    #[test]
    fn test_futures_contract_expiry_states() {
        // Near expiry
        let near_expiry = FuturesContract {
            ticker: "ESH4".to_string(),
            name: "".to_string(),
            product_code: "ES".to_string(),
            days_to_maturity: Some(3),
            active: true,
            ..Default::default()
        };
        assert!(near_expiry.is_near_expiry());
        assert!(near_expiry.is_tradeable());

        // Expired
        let expired = FuturesContract {
            ticker: "ESH4".to_string(),
            name: "".to_string(),
            product_code: "ES".to_string(),
            days_to_maturity: Some(0),
            active: true,
            ..Default::default()
        };
        assert!(expired.is_expired());
        assert!(!expired.is_tradeable());
    }

    #[test]
    fn test_get_futures_contracts_request() {
        let req = GetFuturesContractsRequest::default()
            .product_code("ES")
            .active(true)
            .limit(100);

        assert_eq!(req.path(), "/futures/vX/contracts");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("product_code").unwrap(), "ES");
        assert_eq!(query_map.get("active").unwrap(), "true");
        assert_eq!(query_map.get("limit").unwrap(), "100");
    }

    #[test]
    fn test_get_futures_contract_request() {
        let req = GetFuturesContractRequest::new("ESH4");
        assert_eq!(req.path(), "/futures/vX/contracts/ESH4");
    }

    #[test]
    fn test_get_futures_products_request() {
        let req = GetFuturesProductsRequest::default()
            .asset_class("equity")
            .limit(50);

        assert_eq!(req.path(), "/futures/vX/products");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("asset_class").unwrap(), "equity");
    }

    #[test]
    fn test_get_futures_product_request() {
        let req = GetFuturesProductRequest::new("ES");
        assert_eq!(req.path(), "/futures/vX/products/ES");
    }

    #[test]
    fn test_get_futures_schedules_request() {
        let req = GetFuturesSchedulesRequest::default();
        assert_eq!(req.path(), "/futures/vX/schedules");
    }

    #[test]
    fn test_get_futures_product_schedule_request() {
        let req = GetFuturesProductScheduleRequest::new("ES");
        assert_eq!(req.path(), "/futures/vX/products/ES/schedules");
    }

    #[test]
    fn test_get_futures_snapshot_request() {
        let req = GetFuturesSnapshotRequest::new(&["ESH4", "NQH4"]);
        assert_eq!(req.path(), "/futures/vX/snapshot");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("ticker.any_of").unwrap(), "ESH4,NQH4");
    }

    #[test]
    fn test_futures_quote_helpers() {
        let quote = FuturesQuote {
            bid: Some(5000.00),
            bid_size: Some(100.0),
            ask: Some(5000.25),
            ask_size: Some(50.0),
            timestamp: Some(1704067200000),
        };

        assert_eq!(quote.spread(), Some(0.25));
        assert_eq!(quote.midpoint(), Some(5000.125));
    }

    #[test]
    fn test_futures_product_price_move_value() {
        let product = FuturesProduct {
            code: "ES".to_string(),
            name: "E-mini S&P 500".to_string(),
            tick_size: Some(0.25),
            tick_value: Some(12.50),
            ..Default::default()
        };

        // 10 point move in ES = 10/0.25 = 40 ticks * $12.50 = $500
        assert_eq!(product.price_move_value(10.0), Some(500.0));
    }

    #[test]
    fn test_futures_snapshot_result_helpers() {
        let result = FuturesSnapshotResult {
            ticker: "ESH4".to_string(),
            product_code: Some("ES".to_string()),
            name: Some("E-mini S&P 500".to_string()),
            days_to_maturity: Some(3),
            day: None,
            last_quote: None,
            last_trade: Some(FuturesTrade {
                price: Some(5000.0),
                size: Some(10.0),
                timestamp: None,
            }),
            change: Some(25.0),
            change_percent: Some(0.5),
        };

        assert_eq!(result.price(), Some(5000.0));
        assert!(result.is_near_expiry());
    }

    #[test]
    fn test_futures_contract_deserialize() {
        let json = r#"{
            "ticker": "ESH4",
            "name": "E-mini S&P 500 March 2024",
            "product_code": "ES",
            "days_to_maturity": 30,
            "active": true,
            "contract_type": "mini",
            "trading_venue": "CME",
            "trade_tick_size": 0.25,
            "contract_multiplier": 50.0
        }"#;

        let contract: FuturesContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.ticker, "ESH4");
        assert!(contract.is_tradeable());
        assert_eq!(contract.contract_type, Some(FuturesContractType::Mini));
    }
}
