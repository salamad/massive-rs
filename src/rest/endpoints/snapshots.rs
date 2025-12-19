//! Snapshot endpoints.
//!
//! This module contains request types for fetching real-time snapshot data
//! from the Massive API.

use crate::rest::request::RestRequest;
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
    pub accumulated_volume: Option<u64>,
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
}
