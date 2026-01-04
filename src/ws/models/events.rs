//! WebSocket event types.
//!
//! This module defines all event types that can be received from
//! the Massive WebSocket API.

use crate::util::Symbol;
use serde::Deserialize;

/// Unified WebSocket event enum.
///
/// All events received from the WebSocket connection are parsed into
/// this enum. The event type is determined by the `ev` field in the
/// JSON message.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "ev")]
pub enum WsEvent {
    /// Connection/authentication status message.
    #[serde(rename = "status")]
    Status(WsStatusEvent),

    /// Trade event.
    #[serde(rename = "T")]
    Trade(WsTradeEvent),

    /// Quote event (NBBO).
    #[serde(rename = "Q")]
    Quote(WsQuoteEvent),

    /// Second aggregate bar.
    #[serde(rename = "A")]
    SecondAggregate(WsAggregateEvent),

    /// Minute aggregate bar.
    #[serde(rename = "AM")]
    MinuteAggregate(WsAggregateEvent),

    /// Limit Up/Limit Down event.
    #[serde(rename = "LULD")]
    LimitUpLimitDown(WsLuldEvent),

    /// Fair Market Value event.
    #[serde(rename = "FMV")]
    FairMarketValue(WsFmvEvent),

    /// New Order Imbalance event (auctions).
    #[serde(rename = "NOI")]
    OrderImbalance(WsOrderImbalanceEvent),

    /// Index value event.
    #[serde(rename = "V")]
    IndexValue(WsIndexValueEvent),

    /// Crypto trade event.
    #[serde(rename = "XT")]
    CryptoTrade(WsCryptoTradeEvent),

    /// Crypto quote event.
    #[serde(rename = "XQ")]
    CryptoQuote(WsCryptoQuoteEvent),

    /// Crypto aggregate event.
    #[serde(rename = "XA")]
    CryptoAggregate(WsCryptoAggregateEvent),

    /// Crypto L2 book event.
    #[serde(rename = "XL2")]
    CryptoL2(WsCryptoL2Event),

    /// Forex quote event.
    #[serde(rename = "C")]
    ForexQuote(WsForexQuoteEvent),

    /// Forex aggregate event.
    #[serde(rename = "CA")]
    ForexAggregate(WsForexAggregateEvent),

    /// Unknown event type (forward compatibility).
    ///
    /// New event types added to the API won't cause parsing failures.
    #[serde(other)]
    Unknown,
}

/// Status/control message.
///
/// These messages are sent for connection status updates and
/// authentication results.
#[derive(Debug, Clone, Deserialize)]
pub struct WsStatusEvent {
    /// Status string (e.g., "connected", "auth_success", "auth_failed")
    pub status: String,

    /// Optional message with details
    pub message: Option<String>,
}

impl WsStatusEvent {
    /// Check if this is an authentication success message.
    pub fn is_auth_success(&self) -> bool {
        self.status == "auth_success"
    }

    /// Check if this is an authentication failure message.
    pub fn is_auth_failed(&self) -> bool {
        self.status == "auth_failed"
    }

    /// Check if this is a connection status message.
    pub fn is_connected(&self) -> bool {
        self.status == "connected"
    }
}

/// Trade event.
///
/// Represents a single trade execution.
#[derive(Debug, Clone, Deserialize)]
pub struct WsTradeEvent {
    /// Ticker symbol.
    pub sym: Symbol,

    /// Exchange ID.
    pub x: u8,

    /// Trade ID.
    pub i: String,

    /// Tape (1=NYSE, 2=AMEX, 3=NASDAQ).
    pub z: u8,

    /// Trade price.
    pub p: f64,

    /// Trade size (shares).
    pub s: u64,

    /// Trade conditions.
    #[serde(default)]
    pub c: Vec<i32>,

    /// SIP timestamp (Unix milliseconds).
    pub t: i64,

    /// Sequence number.
    pub q: u64,

    /// TRF ID (if applicable).
    pub trfi: Option<u8>,

    /// TRF timestamp (if applicable).
    pub trft: Option<i64>,
}

impl WsTradeEvent {
    /// Get the trade value (price * size).
    pub fn value(&self) -> f64 {
        self.p * self.s as f64
    }
}

/// Quote event (NBBO).
///
/// Represents the National Best Bid and Offer.
#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteEvent {
    /// Ticker symbol.
    pub sym: Symbol,

    /// Bid exchange ID.
    pub bx: u8,

    /// Bid price.
    pub bp: f64,

    /// Bid size (lots).
    pub bs: u64,

    /// Ask exchange ID.
    pub ax: u8,

    /// Ask price.
    pub ap: f64,

    /// Ask size (lots).
    /// Note: renamed from "as" which is a Rust keyword.
    #[serde(rename = "as")]
    pub ask_size: u64,

    /// Quote condition.
    pub c: Option<i32>,

    /// SIP timestamp (Unix milliseconds).
    pub t: i64,
}

impl WsQuoteEvent {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ap - self.bp
    }

    /// Calculate the mid price.
    pub fn mid(&self) -> f64 {
        (self.bp + self.ap) / 2.0
    }
}

/// Aggregate bar event (second or minute).
///
/// Contains OHLCV data for a time window.
#[derive(Debug, Clone, Deserialize)]
pub struct WsAggregateEvent {
    /// Ticker symbol.
    pub sym: Symbol,

    /// Volume in this window.
    pub v: u64,

    /// Accumulated volume today.
    pub av: u64,

    /// Official open price (day).
    pub op: f64,

    /// VWAP for this window.
    pub vw: f64,

    /// Open price (window).
    pub o: f64,

    /// Close price (window).
    pub c: f64,

    /// High price (window).
    pub h: f64,

    /// Low price (window).
    pub l: f64,

    /// VWAP today.
    pub a: f64,

    /// Average trade size.
    pub z: u64,

    /// Window start timestamp (Unix milliseconds).
    pub s: i64,

    /// Window end timestamp (Unix milliseconds).
    pub e: i64,

    /// OTC ticker flag.
    #[serde(default)]
    pub otc: bool,
}

impl WsAggregateEvent {
    /// Calculate the bar range (high - low).
    pub fn range(&self) -> f64 {
        self.h - self.l
    }

    /// Check if this is a green (close > open) bar.
    pub fn is_green(&self) -> bool {
        self.c > self.o
    }

    /// Check if this is a red (close < open) bar.
    pub fn is_red(&self) -> bool {
        self.c < self.o
    }
}

/// Limit Up/Limit Down event.
///
/// Circuit breaker events for price limits.
#[derive(Debug, Clone, Deserialize)]
pub struct WsLuldEvent {
    /// Ticker symbol.
    pub sym: Symbol,

    /// Upper price limit.
    pub high_price: f64,

    /// Lower price limit.
    pub low_price: f64,

    /// LULD indicators.
    pub indicators: Vec<i32>,

    /// Tape (1=NYSE, 2=AMEX, 3=NASDAQ).
    pub tape: u8,

    /// Timestamp (Unix milliseconds).
    pub t: i64,
}

/// Fair Market Value event.
///
/// Proprietary price estimate.
#[derive(Debug, Clone, Deserialize)]
pub struct WsFmvEvent {
    /// Ticker symbol.
    pub sym: Symbol,

    /// Fair market value price.
    pub fmv: f64,

    /// Timestamp (Unix milliseconds).
    pub t: i64,
}

// ============================================================================
// New Order Imbalance Event
// ============================================================================

/// Auction type for order imbalance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuctionType {
    /// Opening auction.
    #[default]
    Opening,
    /// Closing auction.
    Closing,
    /// IPO auction.
    Ipo,
    /// Halt auction.
    Halt,
    /// Volatility auction.
    Volatility,
}

/// Imbalance side indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImbalanceSide {
    /// Buy side imbalance.
    Buy,
    /// Sell side imbalance.
    Sell,
    /// No imbalance.
    #[default]
    None,
}

/// New Order Imbalance event.
///
/// Provides auction imbalance data for opening/closing auctions.
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrderImbalanceEvent {
    /// Ticker symbol.
    pub sym: Symbol,

    /// Timestamp (Unix milliseconds).
    pub t: i64,

    /// Type of auction.
    pub auction_type: Option<AuctionType>,

    /// Paired shares (matched in auction).
    pub paired_shares: Option<i64>,

    /// Imbalance shares (unmatched).
    pub imbalance_shares: Option<i64>,

    /// Side with the imbalance.
    pub imbalance_side: Option<ImbalanceSide>,

    /// Reference price.
    pub reference_price: Option<f64>,

    /// Near indicative price.
    pub near_price: Option<f64>,

    /// Far indicative price.
    pub far_price: Option<f64>,
}

impl WsOrderImbalanceEvent {
    /// Calculate imbalance percentage.
    pub fn imbalance_percent(&self) -> Option<f64> {
        match (self.imbalance_shares, self.paired_shares) {
            (Some(imb), Some(paired)) if paired > 0 => Some((imb as f64 / paired as f64) * 100.0),
            _ => None,
        }
    }
}

// ============================================================================
// Index Value Event
// ============================================================================

/// Index value event.
///
/// Real-time index values.
#[derive(Debug, Clone, Deserialize)]
pub struct WsIndexValueEvent {
    /// Index ticker (e.g., "I:SPX").
    pub sym: Symbol,

    /// Index value.
    pub val: f64,

    /// Timestamp (Unix milliseconds).
    pub t: i64,
}

// ============================================================================
// Crypto Events
// ============================================================================

/// Crypto trade event.
#[derive(Debug, Clone, Deserialize)]
pub struct WsCryptoTradeEvent {
    /// Crypto pair (e.g., "BTC-USD").
    pub pair: String,

    /// Trade price.
    pub p: f64,

    /// Trade size.
    pub s: f64,

    /// Exchange ID.
    pub x: i32,

    /// Timestamp (Unix milliseconds).
    pub t: i64,

    /// Conditions.
    #[serde(default)]
    pub c: Vec<i32>,

    /// Trade ID.
    pub i: Option<String>,
}

impl WsCryptoTradeEvent {
    /// Calculate trade value.
    pub fn value(&self) -> f64 {
        self.p * self.s
    }
}

/// Crypto quote event.
#[derive(Debug, Clone, Deserialize)]
pub struct WsCryptoQuoteEvent {
    /// Crypto pair (e.g., "BTC-USD").
    pub pair: String,

    /// Bid price.
    pub bp: f64,

    /// Bid size.
    pub bs: f64,

    /// Ask price.
    pub ap: f64,

    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: f64,

    /// Exchange ID.
    pub x: i32,

    /// Timestamp (Unix milliseconds).
    pub t: i64,
}

impl WsCryptoQuoteEvent {
    /// Calculate bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ap - self.bp
    }

    /// Calculate mid price.
    pub fn mid(&self) -> f64 {
        (self.bp + self.ap) / 2.0
    }

    /// Calculate spread as percentage of mid.
    pub fn spread_percent(&self) -> f64 {
        let mid = self.mid();
        if mid > 0.0 {
            (self.spread() / mid) * 100.0
        } else {
            0.0
        }
    }
}

/// Crypto aggregate event.
#[derive(Debug, Clone, Deserialize)]
pub struct WsCryptoAggregateEvent {
    /// Crypto pair (e.g., "BTC-USD").
    pub pair: String,

    /// Open price.
    pub o: f64,

    /// High price.
    pub h: f64,

    /// Low price.
    pub l: f64,

    /// Close price.
    pub c: f64,

    /// Volume.
    pub v: f64,

    /// VWAP.
    pub vw: Option<f64>,

    /// Window start timestamp.
    pub s: i64,

    /// Window end timestamp.
    pub e: i64,
}

impl WsCryptoAggregateEvent {
    /// Calculate bar range.
    pub fn range(&self) -> f64 {
        self.h - self.l
    }

    /// Check if bullish (green) bar.
    pub fn is_bullish(&self) -> bool {
        self.c > self.o
    }
}

/// Crypto L2 book entry.
#[derive(Debug, Clone, Deserialize)]
pub struct WsCryptoL2Entry {
    /// Price level.
    pub p: f64,
    /// Size at level.
    pub s: f64,
}

/// Crypto L2 book event.
#[derive(Debug, Clone, Deserialize)]
pub struct WsCryptoL2Event {
    /// Crypto pair.
    pub pair: String,

    /// Bid levels.
    #[serde(default)]
    pub b: Vec<WsCryptoL2Entry>,

    /// Ask levels.
    #[serde(default)]
    pub a: Vec<WsCryptoL2Entry>,

    /// Timestamp (Unix milliseconds).
    pub t: i64,

    /// Exchange ID.
    pub x: Option<i32>,
}

impl WsCryptoL2Event {
    /// Get the best bid.
    pub fn best_bid(&self) -> Option<f64> {
        self.b.first().map(|e| e.p)
    }

    /// Get the best ask.
    pub fn best_ask(&self) -> Option<f64> {
        self.a.first().map(|e| e.p)
    }

    /// Calculate spread.
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
}

// ============================================================================
// Forex Events
// ============================================================================

/// Forex quote event.
#[derive(Debug, Clone, Deserialize)]
pub struct WsForexQuoteEvent {
    /// Currency pair (e.g., "EUR/USD").
    pub p: String,

    /// Ask price.
    pub a: f64,

    /// Bid price.
    pub b: f64,

    /// Timestamp (Unix milliseconds).
    pub t: i64,

    /// Exchange ID.
    pub x: Option<i32>,
}

impl WsForexQuoteEvent {
    /// Calculate bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.a - self.b
    }

    /// Calculate mid price.
    pub fn mid(&self) -> f64 {
        (self.a + self.b) / 2.0
    }

    /// Calculate spread in pips (standard pairs).
    pub fn spread_pips(&self) -> f64 {
        self.spread() * 10000.0
    }

    /// Calculate spread in pips (JPY pairs).
    pub fn spread_pips_jpy(&self) -> f64 {
        self.spread() * 100.0
    }
}

/// Forex aggregate event.
#[derive(Debug, Clone, Deserialize)]
pub struct WsForexAggregateEvent {
    /// Currency pair.
    pub pair: String,

    /// Open price.
    pub o: f64,

    /// High price.
    pub h: f64,

    /// Low price.
    pub l: f64,

    /// Close price.
    pub c: f64,

    /// Volume.
    pub v: Option<f64>,

    /// Window start timestamp.
    pub s: i64,

    /// Window end timestamp.
    pub e: i64,
}

impl WsForexAggregateEvent {
    /// Calculate bar range.
    pub fn range(&self) -> f64 {
        self.h - self.l
    }

    /// Check if bullish bar.
    pub fn is_bullish(&self) -> bool {
        self.c > self.o
    }
}

/// Parse a WebSocket message (handles both single events and arrays).
///
/// The Massive WebSocket API can send either a single event object or
/// an array of events in one message. This function handles both cases.
pub fn parse_ws_message(text: &str) -> Result<Vec<WsEvent>, serde_json::Error> {
    let trimmed = text.trim();

    if trimmed.starts_with('[') {
        // Array of events
        serde_json::from_str(trimmed)
    } else {
        // Single event
        let event: WsEvent = serde_json::from_str(trimmed)?;
        Ok(vec![event])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_event() {
        let json = r#"{"ev":"status","status":"auth_success","message":"authenticated"}"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::Status(status) => {
                assert!(status.is_auth_success());
                assert_eq!(status.message, Some("authenticated".to_string()));
            }
            _ => panic!("Expected Status event"),
        }
    }

    #[test]
    fn test_parse_trade_event() {
        let json = r#"{"ev":"T","sym":"AAPL","x":4,"i":"12345","z":3,"p":150.25,"s":100,"c":[0],"t":1703001234567,"q":12345}"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::Trade(trade) => {
                assert_eq!(trade.sym.as_str(), "AAPL");
                assert_eq!(trade.p, 150.25);
                assert_eq!(trade.s, 100);
                assert_eq!(trade.value(), 15025.0);
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[test]
    fn test_parse_quote_event() {
        let json = r#"{"ev":"Q","sym":"AAPL","bx":4,"bp":150.00,"bs":100,"ax":4,"ap":150.10,"as":200,"c":0,"t":1703001234567}"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::Quote(quote) => {
                assert_eq!(quote.sym.as_str(), "AAPL");
                assert_eq!(quote.bp, 150.00);
                assert_eq!(quote.ap, 150.10);
                assert!((quote.spread() - 0.10).abs() < 0.001);
                assert!((quote.mid() - 150.05).abs() < 0.001);
            }
            _ => panic!("Expected Quote event"),
        }
    }

    #[test]
    fn test_parse_aggregate_event() {
        let json = r#"{"ev":"AM","sym":"AAPL","v":1000,"av":5000000,"op":148.00,"vw":150.50,"o":150.00,"c":151.00,"h":152.00,"l":149.00,"a":150.25,"z":100,"s":1703001200000,"e":1703001260000}"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::MinuteAggregate(agg) => {
                assert_eq!(agg.sym.as_str(), "AAPL");
                assert_eq!(agg.o, 150.00);
                assert_eq!(agg.c, 151.00);
                assert!(agg.is_green());
                assert!(!agg.is_red());
                assert!((agg.range() - 3.0).abs() < 0.001);
            }
            _ => panic!("Expected MinuteAggregate event"),
        }
    }

    #[test]
    fn test_parse_array_of_events() {
        let json = r#"[{"ev":"status","status":"connected"},{"ev":"T","sym":"AAPL","x":4,"i":"1","z":3,"p":150.00,"s":100,"t":1703001234567,"q":1}]"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], WsEvent::Status(_)));
        assert!(matches!(&events[1], WsEvent::Trade(_)));
    }

    #[test]
    fn test_parse_unknown_event() {
        let json = r#"{"ev":"UNKNOWN_TYPE","foo":"bar"}"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], WsEvent::Unknown));
    }

    #[test]
    fn test_parse_luld_event() {
        let json = r#"{"ev":"LULD","sym":"AAPL","high_price":155.00,"low_price":145.00,"indicators":[1,2],"tape":3,"t":1703001234567}"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::LimitUpLimitDown(luld) => {
                assert_eq!(luld.sym.as_str(), "AAPL");
                assert_eq!(luld.high_price, 155.00);
                assert_eq!(luld.low_price, 145.00);
            }
            _ => panic!("Expected LULD event"),
        }
    }

    #[test]
    fn test_parse_fmv_event() {
        let json = r#"{"ev":"FMV","sym":"AAPL","fmv":150.50,"t":1703001234567}"#;
        let events = parse_ws_message(json).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::FairMarketValue(fmv) => {
                assert_eq!(fmv.sym.as_str(), "AAPL");
                assert_eq!(fmv.fmv, 150.50);
            }
            _ => panic!("Expected FMV event"),
        }
    }
}
