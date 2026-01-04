//! Cryptocurrency endpoints.
//!
//! This module provides endpoints for cryptocurrency data, including:
//!
//! - Crypto trades and quotes
//! - Historical OHLCV bars
//! - Real-time snapshots
//! - Open/close data
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{GetCryptoSnapshotRequest, GetCryptoOpenCloseRequest};
//!
//! // Get all crypto snapshots
//! let all = GetCryptoSnapshotRequest::all();
//!
//! // Get specific pairs
//! let specific = GetCryptoSnapshotRequest::tickers(&["X:BTCUSD", "X:ETHUSD"]);
//!
//! // Get daily open/close for BTC-USD
//! let open_close = GetCryptoOpenCloseRequest::new("BTC", "USD", "2024-01-02");
//! ```

use crate::rest::request::{QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

// ============================================================================
// Crypto Snapshot Types
// ============================================================================

/// Crypto ticker snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoSnapshot {
    /// Crypto pair ticker (e.g., "X:BTCUSD").
    pub ticker: String,
    /// Today's change.
    #[serde(rename = "todaysChange")]
    pub todays_change: Option<f64>,
    /// Today's change percentage.
    #[serde(rename = "todaysChangePerc")]
    pub todays_change_perc: Option<f64>,
    /// Last updated timestamp.
    pub updated: Option<i64>,
    /// Day's aggregate data.
    pub day: Option<CryptoDayData>,
    /// Last trade.
    #[serde(rename = "lastTrade")]
    pub last_trade: Option<CryptoSnapshotTrade>,
    /// Previous day's data.
    #[serde(rename = "prevDay")]
    pub prev_day: Option<CryptoDayData>,
    /// Current minute aggregate.
    pub min: Option<CryptoMinuteData>,
    /// Fair market value (for stablecoins).
    pub fmv: Option<f64>,
}

impl CryptoSnapshot {
    /// Get the market cap estimate if volume and price are available.
    pub fn notional_volume(&self) -> Option<f64> {
        self.day
            .as_ref()
            .and_then(|d| d.volume.map(|v| v * d.vwap.unwrap_or(d.close)))
    }
}

/// Day aggregate for crypto.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoDayData {
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
    /// Volume.
    #[serde(rename = "v")]
    pub volume: Option<f64>,
    /// VWAP.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
}

impl CryptoDayData {
    /// Calculate the daily range.
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Calculate the daily change.
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

    /// Calculate the notional volume (volume * vwap).
    pub fn notional_volume(&self) -> Option<f64> {
        self.volume.map(|v| v * self.vwap.unwrap_or(self.close))
    }

    /// Calculate volatility as range / close.
    pub fn volatility(&self) -> f64 {
        if self.close > 0.0 {
            (self.range() / self.close) * 100.0
        } else {
            0.0
        }
    }
}

/// Trade data in crypto snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoSnapshotTrade {
    /// Price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Size.
    #[serde(rename = "s")]
    pub size: Option<f64>,
    /// Exchange ID.
    #[serde(rename = "x")]
    pub exchange: Option<i32>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
    /// Conditions.
    #[serde(rename = "c")]
    pub conditions: Option<Vec<i32>>,
}

impl CryptoSnapshotTrade {
    /// Calculate trade value (price * size).
    pub fn value(&self) -> Option<f64> {
        self.size.map(|s| self.price * s)
    }
}

/// Minute aggregate for crypto.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoMinuteData {
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
    /// Volume.
    #[serde(rename = "v")]
    pub volume: Option<f64>,
    /// VWAP.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
    /// Number of trades.
    #[serde(rename = "n")]
    pub num_trades: Option<i64>,
}

/// Response from crypto snapshot endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoSnapshotResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Snapshot data.
    #[serde(default)]
    pub tickers: Vec<CryptoSnapshot>,
}

/// Request for crypto snapshot.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetCryptoSnapshotRequest;
///
/// // Get all crypto pairs
/// let all = GetCryptoSnapshotRequest::all();
///
/// // Get specific pairs
/// let specific = GetCryptoSnapshotRequest::tickers(&["X:BTCUSD", "X:ETHUSD"]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetCryptoSnapshotRequest {
    /// Filter by specific tickers.
    pub tickers: Option<Vec<String>>,
}

impl GetCryptoSnapshotRequest {
    /// Request all crypto pairs.
    pub fn all() -> Self {
        Self::default()
    }

    /// Request specific tickers.
    pub fn tickers(tickers: &[&str]) -> Self {
        Self {
            tickers: Some(tickers.iter().map(|s| s.to_string()).collect()),
        }
    }
}

impl RestRequest for GetCryptoSnapshotRequest {
    type Response = CryptoSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v2/snapshot/locale/global/markets/crypto/tickers".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        if let Some(ref tickers) = self.tickers {
            params.push((Cow::Borrowed("tickers"), tickers.join(",")));
        }
        params
    }
}

/// Request for a single crypto ticker snapshot.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetCryptoTickerSnapshotRequest;
///
/// let request = GetCryptoTickerSnapshotRequest::new("X:BTCUSD");
/// ```
#[derive(Debug, Clone)]
pub struct GetCryptoTickerSnapshotRequest {
    /// Ticker symbol.
    pub ticker: String,
}

impl GetCryptoTickerSnapshotRequest {
    /// Create a new request for a specific crypto ticker.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
        }
    }
}

/// Response for single crypto ticker snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoTickerSnapshotResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// The snapshot data.
    pub ticker: CryptoSnapshot,
}

impl RestRequest for GetCryptoTickerSnapshotRequest {
    type Response = CryptoTickerSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/snapshot/locale/global/markets/crypto/tickers/{}",
            self.ticker
        )
        .into()
    }
}

/// Request for crypto gainers/losers.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetCryptoGainersLosersRequest;
///
/// let gainers = GetCryptoGainersLosersRequest::gainers();
/// let losers = GetCryptoGainersLosersRequest::losers();
/// ```
#[derive(Debug, Clone)]
pub struct GetCryptoGainersLosersRequest {
    /// Direction ("gainers" or "losers").
    pub direction: String,
}

impl GetCryptoGainersLosersRequest {
    /// Request top gainers.
    pub fn gainers() -> Self {
        Self {
            direction: "gainers".to_string(),
        }
    }

    /// Request top losers.
    pub fn losers() -> Self {
        Self {
            direction: "losers".to_string(),
        }
    }
}

impl RestRequest for GetCryptoGainersLosersRequest {
    type Response = CryptoSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/snapshot/locale/global/markets/crypto/{}",
            self.direction
        )
        .into()
    }
}

// ============================================================================
// Crypto Open/Close
// ============================================================================

/// Crypto daily open/close data.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoOpenClose {
    /// Symbol (e.g., "BTC").
    pub symbol: String,
    /// Whether trading was active.
    #[serde(rename = "isUTC")]
    pub is_utc: Option<bool>,
    /// The date.
    pub day: Option<String>,
    /// Open price.
    pub open: f64,
    /// High price.
    pub high: f64,
    /// Low price.
    pub low: f64,
    /// Close price.
    pub close: f64,
    /// Volume.
    pub volume: Option<f64>,
    /// Opening trade timestamp.
    #[serde(rename = "openTrades")]
    pub open_trades: Option<Vec<CryptoOpenCloseTrade>>,
    /// Closing trade timestamp.
    #[serde(rename = "closingTrades")]
    pub closing_trades: Option<Vec<CryptoOpenCloseTrade>>,
}

impl CryptoOpenClose {
    /// Calculate the daily range.
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Calculate the daily change.
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
}

/// Trade in open/close response.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoOpenCloseTrade {
    /// Price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Size.
    #[serde(rename = "s")]
    pub size: Option<f64>,
    /// Exchange.
    #[serde(rename = "x")]
    pub exchange: Option<i32>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
    /// Conditions.
    #[serde(rename = "c")]
    pub conditions: Option<Vec<i32>>,
}

/// Response from crypto open/close endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoOpenCloseResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// The open/close data (flattened).
    #[serde(flatten)]
    pub data: CryptoOpenClose,
}

/// Request for crypto daily open/close.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetCryptoOpenCloseRequest;
///
/// let request = GetCryptoOpenCloseRequest::new("BTC", "USD", "2024-01-02");
/// ```
#[derive(Debug, Clone)]
pub struct GetCryptoOpenCloseRequest {
    /// From symbol (e.g., "BTC").
    pub from: String,
    /// To symbol (e.g., "USD").
    pub to: String,
    /// Date in YYYY-MM-DD format.
    pub date: String,
    /// Whether to adjust for splits.
    pub adjusted: Option<bool>,
}

impl GetCryptoOpenCloseRequest {
    /// Create a new crypto open/close request.
    pub fn new(from: impl Into<String>, to: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            date: date.into(),
            adjusted: None,
        }
    }

    /// Set whether to adjust for splits.
    pub fn adjusted(mut self, adjusted: bool) -> Self {
        self.adjusted = Some(adjusted);
        self
    }
}

impl RestRequest for GetCryptoOpenCloseRequest {
    type Response = CryptoOpenCloseResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v1/open-close/crypto/{}/{}/{}",
            self.from, self.to, self.date
        )
        .into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("adjusted", self.adjusted);
        params
    }
}

// ============================================================================
// Crypto Level 2 Book
// ============================================================================

/// Level 2 order book snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoL2Book {
    /// Ticker.
    pub ticker: String,
    /// Bids (price, size pairs).
    pub bids: Vec<CryptoL2Quote>,
    /// Asks (price, size pairs).
    pub asks: Vec<CryptoL2Quote>,
    /// Bid count.
    #[serde(rename = "bidCount")]
    pub bid_count: Option<f64>,
    /// Ask count.
    #[serde(rename = "askCount")]
    pub ask_count: Option<f64>,
    /// Spread.
    pub spread: Option<f64>,
    /// Timestamp.
    pub updated: Option<i64>,
}

impl CryptoL2Book {
    /// Get the best bid price.
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|q| q.price)
    }

    /// Get the best ask price.
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|q| q.price)
    }

    /// Calculate the mid price.
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Calculate the spread from top of book.
    pub fn calculated_spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Calculate total bid volume.
    pub fn total_bid_volume(&self) -> f64 {
        self.bids.iter().filter_map(|q| q.size).sum()
    }

    /// Calculate total ask volume.
    pub fn total_ask_volume(&self) -> f64 {
        self.asks.iter().filter_map(|q| q.size).sum()
    }

    /// Calculate volume imbalance (positive = more bids).
    pub fn volume_imbalance(&self) -> f64 {
        let bid_vol = self.total_bid_volume();
        let ask_vol = self.total_ask_volume();
        let total = bid_vol + ask_vol;
        if total > 0.0 {
            (bid_vol - ask_vol) / total
        } else {
            0.0
        }
    }
}

/// Level 2 quote entry.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoL2Quote {
    /// Price level.
    #[serde(rename = "p")]
    pub price: f64,
    /// Size at this level.
    #[serde(rename = "s")]
    pub size: Option<f64>,
    /// Exchange information.
    #[serde(rename = "x")]
    pub exchange_info: Option<serde_json::Value>,
}

/// Response from L2 book endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoL2BookResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// The L2 book data.
    pub data: CryptoL2Book,
}

/// Request for crypto L2 book.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetCryptoL2BookRequest;
///
/// let request = GetCryptoL2BookRequest::new("X:BTCUSD");
/// ```
#[derive(Debug, Clone)]
pub struct GetCryptoL2BookRequest {
    /// Ticker symbol.
    pub ticker: String,
}

impl GetCryptoL2BookRequest {
    /// Create a new L2 book request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
        }
    }
}

impl RestRequest for GetCryptoL2BookRequest {
    type Response = CryptoL2BookResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/snapshot/locale/global/markets/crypto/tickers/{}/book",
            self.ticker
        )
        .into()
    }
}

// ============================================================================
// Crypto Exchanges
// ============================================================================

/// Crypto exchange information.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoExchange {
    /// Exchange ID.
    pub id: i32,
    /// Exchange type.
    #[serde(rename = "type")]
    pub exchange_type: Option<String>,
    /// Exchange market.
    pub market: Option<String>,
    /// Exchange name.
    pub name: String,
    /// Exchange URL.
    pub url: Option<String>,
}

/// Response from crypto exchanges endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoExchangesResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Exchange list.
    #[serde(default)]
    pub results: Vec<CryptoExchange>,
}

/// Request for crypto exchanges.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetCryptoExchangesRequest;
///
/// let request = GetCryptoExchangesRequest;
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GetCryptoExchangesRequest;

impl RestRequest for GetCryptoExchangesRequest {
    type Response = CryptoExchangesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v1/meta/crypto-exchanges".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_day_data_helpers() {
        let day = CryptoDayData {
            open: 42000.0,
            high: 43000.0,
            low: 41000.0,
            close: 42500.0,
            volume: Some(1000.0),
            vwap: Some(42200.0),
        };

        assert!((day.range() - 2000.0).abs() < 0.1);
        assert!((day.change() - 500.0).abs() < 0.1);
        assert!((day.change_percent() - 1.19).abs() < 0.1);
        assert!((day.notional_volume().unwrap() - 42_200_000.0).abs() < 1.0);
        assert!((day.volatility() - 4.71).abs() < 0.1);
    }

    #[test]
    fn test_crypto_snapshot_trade_value() {
        let trade = CryptoSnapshotTrade {
            price: 42000.0,
            size: Some(0.5),
            exchange: Some(1),
            timestamp: Some(1704067200000),
            conditions: None,
        };

        assert_eq!(trade.value(), Some(21000.0));
    }

    #[test]
    fn test_get_crypto_snapshot_request() {
        let all = GetCryptoSnapshotRequest::all();
        assert_eq!(
            all.path(),
            "/v2/snapshot/locale/global/markets/crypto/tickers"
        );
        assert!(all.query().is_empty());

        let specific = GetCryptoSnapshotRequest::tickers(&["X:BTCUSD", "X:ETHUSD"]);
        let query = specific.query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].1, "X:BTCUSD,X:ETHUSD");
    }

    #[test]
    fn test_get_crypto_ticker_snapshot_request() {
        let req = GetCryptoTickerSnapshotRequest::new("X:BTCUSD");
        assert_eq!(
            req.path(),
            "/v2/snapshot/locale/global/markets/crypto/tickers/X:BTCUSD"
        );
    }

    #[test]
    fn test_get_crypto_gainers_losers_request() {
        let gainers = GetCryptoGainersLosersRequest::gainers();
        assert_eq!(
            gainers.path(),
            "/v2/snapshot/locale/global/markets/crypto/gainers"
        );

        let losers = GetCryptoGainersLosersRequest::losers();
        assert_eq!(
            losers.path(),
            "/v2/snapshot/locale/global/markets/crypto/losers"
        );
    }

    #[test]
    fn test_get_crypto_open_close_request() {
        let req = GetCryptoOpenCloseRequest::new("BTC", "USD", "2024-01-02").adjusted(true);

        assert_eq!(req.path(), "/v1/open-close/crypto/BTC/USD/2024-01-02");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("adjusted").unwrap(), "true");
    }

    #[test]
    fn test_crypto_open_close_helpers() {
        let data = CryptoOpenClose {
            symbol: "BTC".to_string(),
            is_utc: Some(true),
            day: Some("2024-01-02".to_string()),
            open: 42000.0,
            high: 43500.0,
            low: 41500.0,
            close: 43000.0,
            volume: Some(15000.0),
            open_trades: None,
            closing_trades: None,
        };

        assert!((data.range() - 2000.0).abs() < 0.1);
        assert!((data.change() - 1000.0).abs() < 0.1);
        assert!((data.change_percent() - 2.38).abs() < 0.1);
    }

    #[test]
    fn test_get_crypto_l2_book_request() {
        let req = GetCryptoL2BookRequest::new("X:BTCUSD");
        assert_eq!(
            req.path(),
            "/v2/snapshot/locale/global/markets/crypto/tickers/X:BTCUSD/book"
        );
    }

    #[test]
    fn test_crypto_l2_book_helpers() {
        let book = CryptoL2Book {
            ticker: "X:BTCUSD".to_string(),
            bids: vec![
                CryptoL2Quote {
                    price: 42000.0,
                    size: Some(1.0),
                    exchange_info: None,
                },
                CryptoL2Quote {
                    price: 41990.0,
                    size: Some(2.0),
                    exchange_info: None,
                },
            ],
            asks: vec![
                CryptoL2Quote {
                    price: 42010.0,
                    size: Some(0.5),
                    exchange_info: None,
                },
                CryptoL2Quote {
                    price: 42020.0,
                    size: Some(1.5),
                    exchange_info: None,
                },
            ],
            bid_count: Some(2.0),
            ask_count: Some(2.0),
            spread: Some(10.0),
            updated: Some(1704067200000),
        };

        assert_eq!(book.best_bid(), Some(42000.0));
        assert_eq!(book.best_ask(), Some(42010.0));
        assert!((book.mid_price().unwrap() - 42005.0).abs() < 0.1);
        assert!((book.calculated_spread().unwrap() - 10.0).abs() < 0.1);
        assert!((book.total_bid_volume() - 3.0).abs() < 0.01);
        assert!((book.total_ask_volume() - 2.0).abs() < 0.01);
        // Imbalance: (3.0 - 2.0) / 5.0 = 0.2
        assert!((book.volume_imbalance() - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_get_crypto_exchanges_request() {
        let req = GetCryptoExchangesRequest;
        assert_eq!(req.path(), "/v1/meta/crypto-exchanges");
    }

    #[test]
    fn test_crypto_snapshot_deserialize() {
        let json = r#"{
            "ticker": "X:BTCUSD",
            "todaysChange": 500.0,
            "todaysChangePerc": 1.2,
            "updated": 1704067200000,
            "day": {
                "o": 42000.0,
                "h": 43000.0,
                "l": 41500.0,
                "c": 42500.0,
                "v": 1500.0
            },
            "lastTrade": {
                "p": 42500.0,
                "s": 0.1,
                "x": 1,
                "t": 1704067200000
            }
        }"#;

        let snapshot: CryptoSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.ticker, "X:BTCUSD");
        assert!(snapshot.day.is_some());
        assert!(snapshot.last_trade.is_some());

        let trade = snapshot.last_trade.unwrap();
        assert_eq!(trade.value(), Some(4250.0));
    }
}
