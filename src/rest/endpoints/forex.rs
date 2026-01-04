//! Forex (Foreign Exchange) endpoints.
//!
//! This module provides endpoints for forex data, including:
//!
//! - Currency pair quotes and conversions
//! - Historical OHLCV bars
//! - Real-time snapshots
//!
//! # Feature Flag
//!
//! This module is available when the `forex` feature is enabled.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::{GetForexQuoteRequest, GetForexConversionRequest};
//!
//! // Get real-time quote for EUR/USD
//! let quote = GetForexQuoteRequest::new("EUR", "USD");
//!
//! // Convert 1000 EUR to USD
//! let conversion = GetForexConversionRequest::new("EUR", "USD")
//!     .amount(1000.0);
//! ```

use crate::rest::request::{QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

// ============================================================================
// Forex Quote Types
// ============================================================================

/// Last quote for a forex pair.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexLastQuote {
    /// Ask price.
    pub ask: f64,
    /// Bid price.
    pub bid: f64,
    /// Exchange ID.
    pub exchange: Option<i32>,
    /// Quote timestamp (Unix milliseconds).
    pub timestamp: i64,
}

impl ForexLastQuote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate the mid price.
    pub fn mid(&self) -> f64 {
        (self.ask + self.bid) / 2.0
    }

    /// Calculate spread in pips (assumes 4 decimal places for most pairs).
    ///
    /// For JPY pairs, use `spread_pips_jpy()` instead.
    pub fn spread_pips(&self) -> f64 {
        self.spread() * 10000.0
    }

    /// Calculate spread in pips for JPY pairs (2 decimal places).
    pub fn spread_pips_jpy(&self) -> f64 {
        self.spread() * 100.0
    }

    /// Calculate the spread as a percentage of mid price.
    pub fn spread_percent(&self) -> f64 {
        let mid = self.mid();
        if mid > 0.0 {
            (self.spread() / mid) * 100.0
        } else {
            0.0
        }
    }
}

/// Response from the forex last quote endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexLastQuoteResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// From currency.
    pub from: Option<String>,
    /// To currency.
    pub to: Option<String>,
    /// The quote data.
    pub last: ForexLastQuote,
}

/// Request for forex last quote.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetForexQuoteRequest;
///
/// let request = GetForexQuoteRequest::new("EUR", "USD");
/// ```
#[derive(Debug, Clone)]
pub struct GetForexQuoteRequest {
    /// From currency code (e.g., "EUR").
    pub from: String,
    /// To currency code (e.g., "USD").
    pub to: String,
}

impl GetForexQuoteRequest {
    /// Create a new forex quote request.
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl RestRequest for GetForexQuoteRequest {
    type Response = ForexLastQuoteResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v1/last_quote/currencies/{}/{}", self.from, self.to).into()
    }
}

// ============================================================================
// Currency Conversion
// ============================================================================

/// Currency conversion result.
#[derive(Debug, Clone, Deserialize)]
pub struct CurrencyConversion {
    /// From currency code.
    pub from: String,
    /// To currency code.
    pub to: String,
    /// Conversion rate.
    pub converted: f64,
    /// Initial amount.
    #[serde(rename = "initialAmount")]
    pub initial_amount: f64,
    /// Last quote data.
    pub last: Option<ForexLastQuote>,
}

impl CurrencyConversion {
    /// Get the exchange rate (converted / initial_amount).
    pub fn rate(&self) -> f64 {
        if self.initial_amount > 0.0 {
            self.converted / self.initial_amount
        } else {
            0.0
        }
    }

    /// Calculate the inverse rate.
    pub fn inverse_rate(&self) -> f64 {
        let rate = self.rate();
        if rate > 0.0 {
            1.0 / rate
        } else {
            0.0
        }
    }
}

/// Response from the currency conversion endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexConversionResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Conversion result.
    #[serde(flatten)]
    pub conversion: CurrencyConversion,
}

/// Request for currency conversion.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetForexConversionRequest;
///
/// let request = GetForexConversionRequest::new("EUR", "USD")
///     .amount(1000.0)
///     .precision(4);
/// ```
#[derive(Debug, Clone)]
pub struct GetForexConversionRequest {
    /// From currency code.
    pub from: String,
    /// To currency code.
    pub to: String,
    /// Amount to convert.
    pub amount: Option<f64>,
    /// Decimal precision for result.
    pub precision: Option<u32>,
}

impl GetForexConversionRequest {
    /// Create a new conversion request.
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount: None,
            precision: None,
        }
    }

    /// Set the amount to convert.
    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the decimal precision.
    pub fn precision(mut self, precision: u32) -> Self {
        self.precision = Some(precision);
        self
    }
}

impl RestRequest for GetForexConversionRequest {
    type Response = ForexConversionResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v1/conversion/{}/{}", self.from, self.to).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("amount", self.amount);
        params.push_opt_param("precision", self.precision);
        params
    }
}

// ============================================================================
// Forex Snapshot
// ============================================================================

/// Forex ticker snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexSnapshot {
    /// Currency pair ticker (e.g., "C:EURUSD").
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
    pub day: Option<ForexDayData>,
    /// Last quote.
    #[serde(rename = "lastQuote")]
    pub last_quote: Option<ForexSnapshotQuote>,
    /// Previous day's data.
    #[serde(rename = "prevDay")]
    pub prev_day: Option<ForexDayData>,
    /// Current minute aggregate.
    pub min: Option<ForexMinuteData>,
}

/// Day aggregate for forex.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexDayData {
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
    /// Volume (may not be available for forex).
    #[serde(rename = "v")]
    pub volume: Option<f64>,
    /// VWAP.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
}

impl ForexDayData {
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

/// Quote data in forex snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexSnapshotQuote {
    /// Ask price.
    #[serde(rename = "a")]
    pub ask: f64,
    /// Bid price.
    #[serde(rename = "b")]
    pub bid: f64,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
}

impl ForexSnapshotQuote {
    /// Calculate the spread.
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate the mid price.
    pub fn mid(&self) -> f64 {
        (self.ask + self.bid) / 2.0
    }
}

/// Minute aggregate for forex.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexMinuteData {
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
}

/// Response from forex snapshot endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ForexSnapshotResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Snapshot data.
    #[serde(default)]
    pub tickers: Vec<ForexSnapshot>,
}

/// Request for forex snapshot.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetForexSnapshotRequest;
///
/// // Get all forex pairs
/// let all = GetForexSnapshotRequest::all();
///
/// // Get specific pairs
/// let specific = GetForexSnapshotRequest::tickers(&["C:EURUSD", "C:GBPUSD"]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetForexSnapshotRequest {
    /// Filter by specific tickers.
    pub tickers: Option<Vec<String>>,
}

impl GetForexSnapshotRequest {
    /// Request all forex pairs.
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

impl RestRequest for GetForexSnapshotRequest {
    type Response = ForexSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v2/snapshot/locale/global/markets/forex/tickers".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        if let Some(ref tickers) = self.tickers {
            params.push((Cow::Borrowed("tickers"), tickers.join(",")));
        }
        params
    }
}

/// Request for forex gainers/losers.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetForexGainersLosersRequest;
///
/// let gainers = GetForexGainersLosersRequest::gainers();
/// let losers = GetForexGainersLosersRequest::losers();
/// ```
#[derive(Debug, Clone)]
pub struct GetForexGainersLosersRequest {
    /// Direction ("gainers" or "losers").
    pub direction: String,
}

impl GetForexGainersLosersRequest {
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

impl RestRequest for GetForexGainersLosersRequest {
    type Response = ForexSnapshotResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!(
            "/v2/snapshot/locale/global/markets/forex/{}",
            self.direction
        )
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forex_last_quote_helpers() {
        let quote = ForexLastQuote {
            ask: 1.08550,
            bid: 1.08540,
            exchange: Some(48),
            timestamp: 1704067200000,
        };

        assert!((quote.spread() - 0.0001).abs() < 0.00001);
        assert!((quote.mid() - 1.08545).abs() < 0.00001);
        assert!((quote.spread_pips() - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_forex_jpy_pips() {
        let quote = ForexLastQuote {
            ask: 148.55,
            bid: 148.53,
            exchange: None,
            timestamp: 1704067200000,
        };

        // JPY pairs: 0.02 * 100 = 2 pips
        assert!((quote.spread_pips_jpy() - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_currency_conversion_helpers() {
        let conversion = CurrencyConversion {
            from: "EUR".to_string(),
            to: "USD".to_string(),
            converted: 1085.50,
            initial_amount: 1000.0,
            last: None,
        };

        assert!((conversion.rate() - 1.0855).abs() < 0.0001);
        assert!((conversion.inverse_rate() - 0.9212).abs() < 0.001);
    }

    #[test]
    fn test_get_forex_quote_request() {
        let req = GetForexQuoteRequest::new("EUR", "USD");
        assert_eq!(req.path(), "/v1/last_quote/currencies/EUR/USD");
    }

    #[test]
    fn test_get_forex_conversion_request() {
        let req = GetForexConversionRequest::new("EUR", "USD")
            .amount(1000.0)
            .precision(4);

        assert_eq!(req.path(), "/v1/conversion/EUR/USD");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("amount").unwrap(), "1000");
        assert_eq!(query_map.get("precision").unwrap(), "4");
    }

    #[test]
    fn test_get_forex_snapshot_request() {
        let all = GetForexSnapshotRequest::all();
        assert_eq!(
            all.path(),
            "/v2/snapshot/locale/global/markets/forex/tickers"
        );
        assert!(all.query().is_empty());

        let specific = GetForexSnapshotRequest::tickers(&["C:EURUSD", "C:GBPUSD"]);
        let query = specific.query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].1, "C:EURUSD,C:GBPUSD");
    }

    #[test]
    fn test_get_forex_gainers_losers_request() {
        let gainers = GetForexGainersLosersRequest::gainers();
        assert_eq!(
            gainers.path(),
            "/v2/snapshot/locale/global/markets/forex/gainers"
        );

        let losers = GetForexGainersLosersRequest::losers();
        assert_eq!(
            losers.path(),
            "/v2/snapshot/locale/global/markets/forex/losers"
        );
    }

    #[test]
    fn test_forex_day_data_helpers() {
        let day = ForexDayData {
            open: 1.0850,
            high: 1.0900,
            low: 1.0800,
            close: 1.0880,
            volume: None,
            vwap: None,
        };

        assert!((day.range() - 0.01).abs() < 0.0001);
        assert!((day.change() - 0.003).abs() < 0.0001);
        assert!((day.change_percent() - 0.276).abs() < 0.01);
    }

    #[test]
    fn test_forex_snapshot_deserialize() {
        let json = r#"{
            "ticker": "C:EURUSD",
            "todaysChange": 0.0015,
            "todaysChangePerc": 0.14,
            "updated": 1704067200000,
            "day": {
                "o": 1.0850,
                "h": 1.0900,
                "l": 1.0800,
                "c": 1.0880
            },
            "lastQuote": {
                "a": 1.0885,
                "b": 1.0883
            }
        }"#;

        let snapshot: ForexSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.ticker, "C:EURUSD");
        assert!(snapshot.day.is_some());
        assert!(snapshot.last_quote.is_some());

        let quote = snapshot.last_quote.unwrap();
        assert!((quote.spread() - 0.0002).abs() < 0.0001);
    }
}
