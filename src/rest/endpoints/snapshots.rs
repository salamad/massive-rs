//! Snapshot endpoints.
//!
//! This module contains request types for fetching real-time snapshot data
//! from the Massive API, including:
//!
//! - Single ticker snapshots
//! - All tickers snapshots
//! - Gainers/losers snapshots
//! - Unified multi-asset snapshots
//! - Grouped daily market summaries

use crate::rest::request::{QueryBuilder, RestRequest};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Ticker snapshot containing current day's data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerSnapshot {
    /// Ticker symbol
    pub ticker: String,
    /// Today's change
    #[serde(rename = "todaysChange")]
    pub todays_change: Option<f64>,
    /// Today's change percentage
    #[serde(rename = "todaysChangePerc")]
    pub todays_change_perc: Option<f64>,
    /// Last updated timestamp
    pub updated: Option<i64>,
    /// Current day aggregate
    pub day: Option<DaySnapshot>,
    /// Last quote
    #[serde(rename = "lastQuote")]
    pub last_quote: Option<SnapshotQuote>,
    /// Last trade
    #[serde(rename = "lastTrade")]
    pub last_trade: Option<SnapshotTrade>,
    /// Previous day aggregate
    #[serde(rename = "prevDay")]
    pub prev_day: Option<DaySnapshot>,
    /// Current minute aggregate
    pub min: Option<MinuteSnapshot>,
}

/// Day aggregate snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaySnapshot {
    /// Open price
    #[serde(rename = "o")]
    pub open: f64,
    /// High price
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price
    #[serde(rename = "l")]
    pub low: f64,
    /// Close price
    #[serde(rename = "c")]
    pub close: f64,
    /// Volume
    #[serde(rename = "v")]
    pub volume: f64,
    /// VWAP
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
}

/// Quote snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotQuote {
    /// Bid price
    #[serde(rename = "p")]
    pub bid_price: f64,
    /// Bid size
    #[serde(rename = "s")]
    pub bid_size: u64,
    /// Ask price (uppercase P in some responses)
    #[serde(rename = "P", default)]
    pub ask_price: Option<f64>,
    /// Ask size (uppercase S in some responses)
    #[serde(rename = "S", default)]
    pub ask_size: Option<u64>,
    /// Timestamp
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
}

/// Trade snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotTrade {
    /// Price
    #[serde(rename = "p")]
    pub price: f64,
    /// Size
    #[serde(rename = "s")]
    pub size: u64,
    /// Exchange ID
    #[serde(rename = "x")]
    pub exchange: Option<u8>,
    /// Trade conditions
    #[serde(rename = "c", default)]
    pub conditions: Vec<i32>,
    /// Trade ID
    #[serde(rename = "i")]
    pub trade_id: Option<String>,
    /// Timestamp
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
}

/// Minute aggregate snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteSnapshot {
    /// Accumulated volume
    #[serde(rename = "av")]
    pub accumulated_volume: Option<f64>,
    /// Open price
    #[serde(rename = "o")]
    pub open: f64,
    /// High price
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price
    #[serde(rename = "l")]
    pub low: f64,
    /// Close price
    #[serde(rename = "c")]
    pub close: f64,
    /// Volume
    #[serde(rename = "v")]
    pub volume: f64,
    /// VWAP
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Number of trades
    #[serde(rename = "n")]
    pub num_trades: Option<u64>,
    /// Timestamp
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
}

/// Request for a single ticker snapshot.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::GetTickerSnapshotRequest;
///
/// let request = GetTickerSnapshotRequest::new("stocks", "AAPL");
/// ```
#[derive(Debug, Clone)]
pub struct GetTickerSnapshotRequest {
    /// Locale (e.g., "us")
    pub locale: String,
    /// Market type (e.g., "stocks")
    pub market_type: String,
    /// Ticker symbol
    pub ticker: String,
}

impl GetTickerSnapshotRequest {
    /// Create a new ticker snapshot request.
    pub fn new(market_type: impl Into<String>, ticker: impl Into<String>) -> Self {
        Self {
            locale: "us".into(),
            market_type: market_type.into(),
            ticker: ticker.into(),
        }
    }

    /// Set the locale.
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }
}

/// Response for single ticker snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerSnapshotResponse {
    /// Status
    pub status: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// The snapshot
    pub ticker: Option<TickerSnapshot>,
}

impl RestRequest for GetTickerSnapshotRequest {
    type Response = TickerSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/snapshot/locale/{}/markets/{}/tickers/{}",
            self.locale, self.market_type, self.ticker
        )
        .into()
    }
}

/// Request for all tickers snapshot.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::GetAllTickersSnapshotRequest;
///
/// let request = GetAllTickersSnapshotRequest::new("stocks")
///     .tickers(&["AAPL", "GOOGL", "MSFT"]);
/// ```
#[derive(Debug, Clone)]
pub struct GetAllTickersSnapshotRequest {
    /// Locale (e.g., "us")
    pub locale: String,
    /// Market type (e.g., "stocks")
    pub market_type: String,
    /// Filter by specific tickers
    pub tickers: Option<Vec<String>>,
    /// Include OTC tickers
    pub include_otc: Option<bool>,
}

impl GetAllTickersSnapshotRequest {
    /// Create a new all tickers snapshot request.
    pub fn new(market_type: impl Into<String>) -> Self {
        Self {
            locale: "us".into(),
            market_type: market_type.into(),
            tickers: None,
            include_otc: None,
        }
    }

    /// Set the locale.
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Filter by specific tickers.
    pub fn tickers(mut self, tickers: &[&str]) -> Self {
        self.tickers = Some(tickers.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Include OTC tickers.
    pub fn include_otc(mut self, include: bool) -> Self {
        self.include_otc = Some(include);
        self
    }
}

/// Response for all tickers snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct AllTickersSnapshotResponse {
    /// Status
    pub status: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// Count
    pub count: Option<u64>,
    /// The snapshots
    #[serde(default)]
    pub tickers: Vec<TickerSnapshot>,
}

impl RestRequest for GetAllTickersSnapshotRequest {
    type Response = AllTickersSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/snapshot/locale/{}/markets/{}/tickers",
            self.locale, self.market_type
        )
        .into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        if let Some(ref tickers) = self.tickers {
            params.push((Cow::Borrowed("tickers"), tickers.join(",")));
        }
        if let Some(include_otc) = self.include_otc {
            params.push((Cow::Borrowed("include_otc"), include_otc.to_string()));
        }
        params
    }
}

/// Request for gainers/losers snapshot.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::GetGainersLosersRequest;
///
/// let request = GetGainersLosersRequest::gainers("stocks");
/// let request = GetGainersLosersRequest::losers("stocks");
/// ```
#[derive(Debug, Clone)]
pub struct GetGainersLosersRequest {
    /// Locale (e.g., "us")
    pub locale: String,
    /// Market type (e.g., "stocks")
    pub market_type: String,
    /// Direction ("gainers" or "losers")
    pub direction: String,
    /// Include OTC tickers
    pub include_otc: Option<bool>,
}

impl GetGainersLosersRequest {
    /// Create a request for top gainers.
    pub fn gainers(market_type: impl Into<String>) -> Self {
        Self {
            locale: "us".into(),
            market_type: market_type.into(),
            direction: "gainers".into(),
            include_otc: None,
        }
    }

    /// Create a request for top losers.
    pub fn losers(market_type: impl Into<String>) -> Self {
        Self {
            locale: "us".into(),
            market_type: market_type.into(),
            direction: "losers".into(),
            include_otc: None,
        }
    }

    /// Set the locale.
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Include OTC tickers.
    pub fn include_otc(mut self, include: bool) -> Self {
        self.include_otc = Some(include);
        self
    }
}

impl RestRequest for GetGainersLosersRequest {
    type Response = AllTickersSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/snapshot/locale/{}/markets/{}/{}",
            self.locale, self.market_type, self.direction
        )
        .into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        if let Some(include_otc) = self.include_otc {
            params.push((Cow::Borrowed("include_otc"), include_otc.to_string()));
        }
        params
    }
}

// ============================================================================
// Unified Multi-Asset Snapshot (v3/snapshot)
// ============================================================================

/// Market status in a unified snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotMarketStatus {
    /// Market is open for trading.
    Open,
    /// Market is closed.
    Closed,
    /// Extended hours trading.
    Extended,
    /// Pre-market trading.
    #[serde(rename = "pre-market")]
    PreMarket,
    /// After-hours trading.
    #[serde(rename = "after-hours")]
    AfterHours,
}

/// Trading session data in a unified snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotSession {
    /// Session close price.
    pub close: Option<f64>,
    /// Session high price.
    pub high: Option<f64>,
    /// Session low price.
    pub low: Option<f64>,
    /// Session open price.
    pub open: Option<f64>,
    /// Session volume.
    pub volume: Option<f64>,
    /// Session change from previous.
    pub change: Option<f64>,
    /// Session change percentage.
    pub change_percent: Option<f64>,
    /// Whether the session is in early trading.
    pub early_trading_change: Option<f64>,
    /// Early trading change percentage.
    pub early_trading_change_percent: Option<f64>,
    /// Late trading change.
    pub late_trading_change: Option<f64>,
    /// Late trading change percentage.
    pub late_trading_change_percent: Option<f64>,
    /// Previous close price.
    pub previous_close: Option<f64>,
}

/// Last quote in a unified snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotLastQuote {
    /// Ask price.
    pub ask: f64,
    /// Ask size (shares or contracts).
    pub ask_size: Option<f64>,
    /// Bid price.
    pub bid: f64,
    /// Bid size (shares or contracts).
    pub bid_size: Option<f64>,
    /// Ask exchange ID.
    pub ask_exchange: Option<u8>,
    /// Bid exchange ID.
    pub bid_exchange: Option<u8>,
    /// Quote timestamp (nanoseconds).
    pub last_updated: Option<i64>,
    /// Midpoint price.
    pub midpoint: Option<f64>,
    /// Timeframe of the data.
    pub timeframe: Option<String>,
}

impl SnapshotLastQuote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate the spread as a percentage of the midpoint.
    pub fn spread_percent(&self) -> Option<f64> {
        let mid = self
            .midpoint
            .or_else(|| Some((self.bid + self.ask) / 2.0))?;
        if mid > 0.0 {
            Some((self.spread() / mid) * 100.0)
        } else {
            None
        }
    }
}

/// Last trade in a unified snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotLastTrade {
    /// Trade price.
    pub price: f64,
    /// Trade size.
    pub size: Option<f64>,
    /// Exchange ID.
    pub exchange: Option<u8>,
    /// Trade ID.
    pub id: Option<String>,
    /// Trade conditions.
    #[serde(default)]
    pub conditions: Vec<i32>,
    /// Trade timestamp (nanoseconds).
    pub last_updated: Option<i64>,
    /// Participant timestamp.
    pub participant_timestamp: Option<i64>,
    /// SIP timestamp.
    pub sip_timestamp: Option<i64>,
    /// Timeframe of the data.
    pub timeframe: Option<String>,
}

/// Greeks for options in a unified snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotGreeks {
    /// Delta - rate of change in option price per $1 change in underlying.
    pub delta: Option<f64>,
    /// Gamma - rate of change in delta per $1 change in underlying.
    pub gamma: Option<f64>,
    /// Theta - rate of time decay per day.
    pub theta: Option<f64>,
    /// Vega - rate of change per 1% change in implied volatility.
    pub vega: Option<f64>,
}

impl SnapshotGreeks {
    /// Check if this option has significant delta (>0.1).
    pub fn has_significant_delta(&self) -> bool {
        self.delta.map(|d| d.abs() > 0.1).unwrap_or(false)
    }

    /// Check if the delta indicates a call-like option (positive delta).
    pub fn is_call_like(&self) -> bool {
        self.delta.map(|d| d > 0.0).unwrap_or(false)
    }

    /// Check if the delta indicates a put-like option (negative delta).
    pub fn is_put_like(&self) -> bool {
        self.delta.map(|d| d < 0.0).unwrap_or(false)
    }
}

/// Unified snapshot result for a single ticker.
///
/// This struct handles responses from the unified snapshot endpoint,
/// which can return data for stocks, options, forex, crypto, and indices
/// in a single request. Individual tickers may return error information
/// if their data is unavailable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSnapshotResult {
    /// Ticker symbol.
    pub ticker: String,
    /// Asset type (stocks, options, fx, crypto, indices).
    pub asset_type: Option<String>,
    /// Company or instrument name.
    pub name: Option<String>,
    /// Current market status.
    pub market_status: Option<SnapshotMarketStatus>,
    /// Trading session data.
    pub session: Option<SnapshotSession>,
    /// Last quote data.
    pub last_quote: Option<SnapshotLastQuote>,
    /// Last trade data.
    pub last_trade: Option<SnapshotLastTrade>,
    /// Fair market value (for indices).
    pub fmv: Option<f64>,
    /// Index value (for indices).
    pub value: Option<f64>,
    /// Greeks data (for options).
    pub greeks: Option<SnapshotGreeks>,
    /// Implied volatility (for options).
    pub implied_volatility: Option<f64>,
    /// Open interest (for options).
    pub open_interest: Option<u64>,
    /// Break-even price (for options).
    pub break_even_price: Option<f64>,
    /// Underlying ticker (for options).
    pub underlying_ticker: Option<String>,
    /// Error code if request failed for this ticker.
    pub error: Option<String>,
    /// Error message if request failed for this ticker.
    pub message: Option<String>,
}

impl UnifiedSnapshotResult {
    /// Check if this result contains an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Check if this is a stock snapshot.
    pub fn is_stock(&self) -> bool {
        self.asset_type.as_deref() == Some("stocks")
    }

    /// Check if this is an options snapshot.
    pub fn is_options(&self) -> bool {
        self.asset_type.as_deref() == Some("options")
    }

    /// Check if this is a forex snapshot.
    pub fn is_forex(&self) -> bool {
        self.asset_type.as_deref() == Some("fx")
    }

    /// Check if this is a crypto snapshot.
    pub fn is_crypto(&self) -> bool {
        self.asset_type.as_deref() == Some("crypto")
    }

    /// Check if this is an index snapshot.
    pub fn is_index(&self) -> bool {
        self.asset_type.as_deref() == Some("indices")
    }

    /// Get the current price from the last trade.
    pub fn price(&self) -> Option<f64> {
        self.last_trade.as_ref().map(|t| t.price)
    }

    /// Get the change and change percentage from the session.
    pub fn change(&self) -> Option<(f64, f64)> {
        let session = self.session.as_ref()?;
        Some((session.change?, session.change_percent?))
    }
}

/// Response from the unified snapshot endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedSnapshotResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Snapshot results for each ticker.
    pub results: Vec<UnifiedSnapshotResult>,
}

impl UnifiedSnapshotResponse {
    /// Get all successful (non-error) results.
    pub fn successes(&self) -> impl Iterator<Item = &UnifiedSnapshotResult> {
        self.results.iter().filter(|r| !r.is_error())
    }

    /// Get all failed results.
    pub fn failures(&self) -> impl Iterator<Item = &UnifiedSnapshotResult> {
        self.results.iter().filter(|r| r.is_error())
    }

    /// Check if all requests succeeded.
    pub fn all_succeeded(&self) -> bool {
        !self.results.iter().any(|r| r.is_error())
    }
}

/// Request for unified multi-asset snapshot.
///
/// The unified snapshot endpoint allows fetching real-time data for up to 250
/// tickers across multiple asset classes in a single request.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetUnifiedSnapshotRequest;
///
/// let request = GetUnifiedSnapshotRequest::new(&["AAPL", "O:AAPL251219C00150000", "X:BTCUSD"]);
/// ```
#[derive(Debug, Clone)]
pub struct GetUnifiedSnapshotRequest {
    /// List of tickers to fetch (max 250).
    pub tickers: Vec<String>,
}

impl GetUnifiedSnapshotRequest {
    /// Create a new unified snapshot request.
    ///
    /// # Arguments
    ///
    /// * `tickers` - List of tickers (max 250). Can include stocks, options (`O:` prefix),
    ///   forex (`C:` prefix), crypto (`X:` prefix), and indices (`I:` prefix).
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

    /// Add multiple tickers to the request.
    pub fn add_tickers(mut self, tickers: &[&str]) -> Self {
        self.tickers.extend(tickers.iter().map(|s| s.to_string()));
        self
    }
}

impl RestRequest for GetUnifiedSnapshotRequest {
    type Response = UnifiedSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v3/snapshot".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker.any_of", Some(self.tickers.join(",")));
        params
    }
}

// ============================================================================
// Grouped Daily Bars
// ============================================================================

/// Grouped daily bar for a single ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedDailyBar {
    /// Ticker symbol.
    #[serde(rename = "T")]
    pub ticker: String,
    /// Open price.
    #[serde(rename = "o")]
    pub open: f64,
    /// High price.
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l")]
    pub low: f64,
    /// Close price.
    #[serde(rename = "c")]
    pub close: f64,
    /// Trading volume.
    #[serde(rename = "v")]
    pub volume: f64,
    /// Volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
    /// Number of trades.
    #[serde(rename = "n")]
    pub num_trades: Option<u64>,
    /// Whether this is from an OTC ticker.
    #[serde(default)]
    pub otc: Option<bool>,
}

impl GroupedDailyBar {
    /// Calculate the daily range (high - low).
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Calculate the daily change (close - open).
    pub fn change(&self) -> f64 {
        self.close - self.open
    }

    /// Calculate the daily change percentage.
    pub fn change_percent(&self) -> f64 {
        if self.open > 0.0 {
            (self.change() / self.open) * 100.0
        } else {
            0.0
        }
    }

    /// Check if this was an up day (close > open).
    pub fn is_up_day(&self) -> bool {
        self.close > self.open
    }

    /// Check if this was a down day (close < open).
    pub fn is_down_day(&self) -> bool {
        self.close < self.open
    }

    /// Calculate the notional trading value.
    pub fn notional_volume(&self) -> Option<f64> {
        self.vwap.map(|vw| vw * self.volume)
    }
}

/// Response from the grouped daily bars endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupedDailyResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Query count.
    #[serde(rename = "queryCount")]
    pub query_count: Option<u64>,
    /// Results count.
    #[serde(rename = "resultsCount")]
    pub results_count: Option<u64>,
    /// Whether results are adjusted for splits.
    pub adjusted: Option<bool>,
    /// Grouped daily bars.
    #[serde(default)]
    pub results: Vec<GroupedDailyBar>,
}

impl GroupedDailyResponse {
    /// Get the top gainers by change percentage.
    pub fn top_gainers(&self, n: usize) -> Vec<&GroupedDailyBar> {
        let mut bars: Vec<_> = self.results.iter().collect();
        bars.sort_by(|a, b| {
            b.change_percent()
                .partial_cmp(&a.change_percent())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        bars.into_iter().take(n).collect()
    }

    /// Get the top losers by change percentage.
    pub fn top_losers(&self, n: usize) -> Vec<&GroupedDailyBar> {
        let mut bars: Vec<_> = self.results.iter().collect();
        bars.sort_by(|a, b| {
            a.change_percent()
                .partial_cmp(&b.change_percent())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        bars.into_iter().take(n).collect()
    }

    /// Get tickers sorted by volume (highest first).
    pub fn by_volume(&self, n: usize) -> Vec<&GroupedDailyBar> {
        let mut bars: Vec<_> = self.results.iter().collect();
        bars.sort_by(|a, b| {
            b.volume
                .partial_cmp(&a.volume)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        bars.into_iter().take(n).collect()
    }
}

/// Request for grouped daily bars.
///
/// Returns the open, high, low, and close prices for all tickers
/// in a specific market for a given date.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetGroupedDailyRequest;
///
/// // Get all US stock bars for a specific date
/// let request = GetGroupedDailyRequest::new("us", "stocks", "2024-01-15")
///     .adjusted(true)
///     .include_otc(false);
/// ```
#[derive(Debug, Clone)]
pub struct GetGroupedDailyRequest {
    /// Locale (e.g., "us", "global").
    pub locale: String,
    /// Market type (e.g., "stocks", "crypto", "fx").
    pub market: String,
    /// Date in YYYY-MM-DD format.
    pub date: String,
    /// Whether to adjust for splits.
    pub adjusted: Option<bool>,
    /// Include OTC securities.
    pub include_otc: Option<bool>,
}

impl GetGroupedDailyRequest {
    /// Create a new grouped daily request.
    pub fn new(
        locale: impl Into<String>,
        market: impl Into<String>,
        date: impl Into<String>,
    ) -> Self {
        Self {
            locale: locale.into(),
            market: market.into(),
            date: date.into(),
            adjusted: None,
            include_otc: None,
        }
    }

    /// Create a request for US stocks.
    pub fn us_stocks(date: impl Into<String>) -> Self {
        Self::new("us", "stocks", date)
    }

    /// Create a request for US options.
    pub fn us_options(date: impl Into<String>) -> Self {
        Self::new("us", "options", date)
    }

    /// Create a request for global crypto.
    pub fn crypto(date: impl Into<String>) -> Self {
        Self::new("global", "crypto", date)
    }

    /// Create a request for global forex.
    pub fn forex(date: impl Into<String>) -> Self {
        Self::new("global", "fx", date)
    }

    /// Set whether to adjust for splits.
    pub fn adjusted(mut self, adjusted: bool) -> Self {
        self.adjusted = Some(adjusted);
        self
    }

    /// Set whether to include OTC securities.
    pub fn include_otc(mut self, include: bool) -> Self {
        self.include_otc = Some(include);
        self
    }
}

impl RestRequest for GetGroupedDailyRequest {
    type Response = GroupedDailyResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/aggs/grouped/locale/{}/market/{}/{}",
            self.locale, self.market, self.date
        )
        .into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("adjusted", self.adjusted);
        params.push_opt_param("include_otc", self.include_otc);
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ticker_snapshot_path() {
        let req = GetTickerSnapshotRequest::new("stocks", "AAPL");
        assert_eq!(
            req.path(),
            "/v2/snapshot/locale/us/markets/stocks/tickers/AAPL"
        );
    }

    #[test]
    fn test_get_all_tickers_snapshot_path() {
        let req = GetAllTickersSnapshotRequest::new("stocks");
        assert_eq!(req.path(), "/v2/snapshot/locale/us/markets/stocks/tickers");
    }

    #[test]
    fn test_get_all_tickers_with_filter() {
        let req = GetAllTickersSnapshotRequest::new("stocks")
            .tickers(&["AAPL", "GOOGL"])
            .include_otc(false);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("tickers").unwrap(), "AAPL,GOOGL");
        assert_eq!(query_map.get("include_otc").unwrap(), "false");
    }

    #[test]
    fn test_get_gainers_path() {
        let req = GetGainersLosersRequest::gainers("stocks");
        assert_eq!(req.path(), "/v2/snapshot/locale/us/markets/stocks/gainers");
    }

    #[test]
    fn test_get_losers_path() {
        let req = GetGainersLosersRequest::losers("stocks");
        assert_eq!(req.path(), "/v2/snapshot/locale/us/markets/stocks/losers");
    }

    #[test]
    fn test_ticker_snapshot_deserialize() {
        let json = r#"{
            "ticker": "AAPL",
            "todaysChange": 2.50,
            "todaysChangePerc": 1.5,
            "updated": 1703001234567,
            "day": {
                "o": 150.0,
                "h": 155.0,
                "l": 149.0,
                "c": 153.0,
                "v": 1000000,
                "vw": 152.0
            }
        }"#;

        let snapshot: TickerSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.ticker, "AAPL");
        assert_eq!(snapshot.todays_change, Some(2.50));
        assert!(snapshot.day.is_some());
        let day = snapshot.day.unwrap();
        assert_eq!(day.open, 150.0);
        assert_eq!(day.close, 153.0);
    }

    // =========================================================================
    // Unified Snapshot Tests
    // =========================================================================

    #[test]
    fn test_unified_snapshot_request_path() {
        let req = GetUnifiedSnapshotRequest::new(&["AAPL", "GOOGL"]);
        assert_eq!(req.path(), "/v3/snapshot");
    }

    #[test]
    fn test_unified_snapshot_request_query() {
        let req = GetUnifiedSnapshotRequest::new(&["AAPL", "O:AAPL251219C00150000", "X:BTCUSD"]);
        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(
            query_map.get("ticker.any_of").unwrap(),
            "AAPL,O:AAPL251219C00150000,X:BTCUSD"
        );
    }

    #[test]
    fn test_unified_snapshot_result_asset_types() {
        let stock_result = UnifiedSnapshotResult {
            ticker: "AAPL".to_string(),
            asset_type: Some("stocks".to_string()),
            name: Some("Apple Inc.".to_string()),
            market_status: None,
            session: None,
            last_quote: None,
            last_trade: None,
            fmv: None,
            value: None,
            greeks: None,
            implied_volatility: None,
            open_interest: None,
            break_even_price: None,
            underlying_ticker: None,
            error: None,
            message: None,
        };

        assert!(stock_result.is_stock());
        assert!(!stock_result.is_options());
        assert!(!stock_result.is_forex());
        assert!(!stock_result.is_crypto());
        assert!(!stock_result.is_index());
        assert!(!stock_result.is_error());
    }

    #[test]
    fn test_unified_snapshot_result_error() {
        let error_result = UnifiedSnapshotResult {
            ticker: "INVALID".to_string(),
            asset_type: None,
            name: None,
            market_status: None,
            session: None,
            last_quote: None,
            last_trade: None,
            fmv: None,
            value: None,
            greeks: None,
            implied_volatility: None,
            open_interest: None,
            break_even_price: None,
            underlying_ticker: None,
            error: Some("NOT_FOUND".to_string()),
            message: Some("Ticker not found".to_string()),
        };

        assert!(error_result.is_error());
    }

    #[test]
    fn test_snapshot_last_quote_spread() {
        let quote = SnapshotLastQuote {
            ask: 150.10,
            ask_size: Some(100.0),
            bid: 150.00,
            bid_size: Some(200.0),
            ask_exchange: None,
            bid_exchange: None,
            last_updated: None,
            midpoint: Some(150.05),
            timeframe: None,
        };

        assert!((quote.spread() - 0.10).abs() < 0.001);
        let spread_pct = quote.spread_percent().unwrap();
        assert!((spread_pct - 0.0666).abs() < 0.01);
    }

    #[test]
    fn test_snapshot_greeks_helpers() {
        let call_greeks = SnapshotGreeks {
            delta: Some(0.65),
            gamma: Some(0.02),
            theta: Some(-0.05),
            vega: Some(0.15),
        };

        assert!(call_greeks.has_significant_delta());
        assert!(call_greeks.is_call_like());
        assert!(!call_greeks.is_put_like());

        let put_greeks = SnapshotGreeks {
            delta: Some(-0.35),
            gamma: Some(0.02),
            theta: Some(-0.04),
            vega: Some(0.10),
        };

        assert!(put_greeks.has_significant_delta());
        assert!(!put_greeks.is_call_like());
        assert!(put_greeks.is_put_like());
    }

    #[test]
    fn test_unified_snapshot_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "results": [
                {
                    "ticker": "AAPL",
                    "asset_type": "stocks",
                    "name": "Apple Inc.",
                    "last_trade": {"price": 150.25},
                    "session": {"change": 2.50, "change_percent": 1.5}
                },
                {
                    "ticker": "INVALID",
                    "error": "NOT_FOUND",
                    "message": "Ticker not found"
                }
            ]
        }"#;

        let response: UnifiedSnapshotResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 2);

        let successes: Vec<_> = response.successes().collect();
        assert_eq!(successes.len(), 1);
        assert_eq!(successes[0].ticker, "AAPL");
        assert_eq!(successes[0].price(), Some(150.25));

        let failures: Vec<_> = response.failures().collect();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].ticker, "INVALID");

        assert!(!response.all_succeeded());
    }

    // =========================================================================
    // Grouped Daily Tests
    // =========================================================================

    #[test]
    fn test_grouped_daily_request_path() {
        let req = GetGroupedDailyRequest::new("us", "stocks", "2024-01-15");
        assert_eq!(
            req.path(),
            "/v2/aggs/grouped/locale/us/market/stocks/2024-01-15"
        );
    }

    #[test]
    fn test_grouped_daily_request_us_stocks() {
        let req = GetGroupedDailyRequest::us_stocks("2024-01-15");
        assert_eq!(
            req.path(),
            "/v2/aggs/grouped/locale/us/market/stocks/2024-01-15"
        );
    }

    #[test]
    fn test_grouped_daily_request_crypto() {
        let req = GetGroupedDailyRequest::crypto("2024-01-15");
        assert_eq!(
            req.path(),
            "/v2/aggs/grouped/locale/global/market/crypto/2024-01-15"
        );
    }

    #[test]
    fn test_grouped_daily_request_query() {
        let req = GetGroupedDailyRequest::us_stocks("2024-01-15")
            .adjusted(true)
            .include_otc(false);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("adjusted").unwrap(), "true");
        assert_eq!(query_map.get("include_otc").unwrap(), "false");
    }

    #[test]
    fn test_grouped_daily_bar_helpers() {
        let bar = GroupedDailyBar {
            ticker: "AAPL".to_string(),
            open: 150.0,
            high: 155.0,
            low: 148.0,
            close: 153.0,
            volume: 1_000_000.0,
            vwap: Some(152.0),
            timestamp: None,
            num_trades: Some(50_000),
            otc: Some(false),
        };

        assert!((bar.range() - 7.0).abs() < 0.001);
        assert!((bar.change() - 3.0).abs() < 0.001);
        assert!((bar.change_percent() - 2.0).abs() < 0.001);
        assert!(bar.is_up_day());
        assert!(!bar.is_down_day());
        assert_eq!(bar.notional_volume(), Some(152_000_000.0));

        let down_bar = GroupedDailyBar {
            ticker: "MSFT".to_string(),
            open: 300.0,
            high: 305.0,
            low: 295.0,
            close: 297.0,
            volume: 500_000.0,
            vwap: None,
            timestamp: None,
            num_trades: None,
            otc: None,
        };

        assert!(down_bar.is_down_day());
        assert!(!down_bar.is_up_day());
        assert!(down_bar.notional_volume().is_none());
    }

    #[test]
    fn test_grouped_daily_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "queryCount": 2,
            "resultsCount": 2,
            "adjusted": true,
            "results": [
                {"T": "AAPL", "o": 150.0, "h": 155.0, "l": 148.0, "c": 153.0, "v": 1000000, "vw": 152.0},
                {"T": "GOOGL", "o": 100.0, "h": 105.0, "l": 95.0, "c": 97.0, "v": 500000, "vw": 99.0}
            ]
        }"#;

        let response: GroupedDailyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].ticker, "AAPL");
        assert_eq!(response.results[1].ticker, "GOOGL");

        // AAPL: (153-150)/150 * 100 = 2%
        // GOOGL: (97-100)/100 * 100 = -3%
        let gainers = response.top_gainers(1);
        assert_eq!(gainers[0].ticker, "AAPL");

        let losers = response.top_losers(1);
        assert_eq!(losers[0].ticker, "GOOGL");

        let by_volume = response.by_volume(1);
        assert_eq!(by_volume[0].ticker, "AAPL"); // Higher volume
    }
}
