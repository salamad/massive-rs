//! Common data models shared across asset classes.
//!
//! These types represent fundamental market data structures that are
//! used consistently across stocks, options, forex, and crypto.

use crate::util::Symbol;
use serde::{Deserialize, Serialize};

/// Aggregate bar (OHLCV) data.
///
/// Represents price and volume data aggregated over a time period,
/// commonly used for charting and technical analysis.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AggregateBar {
    /// Ticker symbol
    #[serde(rename = "T", skip_serializing_if = "Option::is_none")]
    pub ticker: Option<Symbol>,

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

    /// Volume-weighted average price
    #[serde(rename = "vw", skip_serializing_if = "Option::is_none")]
    pub vwap: Option<f64>,

    /// Timestamp (Unix milliseconds)
    #[serde(rename = "t")]
    pub timestamp: i64,

    /// Number of transactions
    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub transactions: Option<u64>,

    /// Whether this is an OTC ticker
    #[serde(default, skip_serializing_if = "is_false")]
    pub otc: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl AggregateBar {
    /// Calculate the bar range (high - low).
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Calculate the bar body size (|close - open|).
    pub fn body(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Check if this is a bullish (green) bar.
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if this is a bearish (red) bar.
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Check if this is a doji (open ≈ close).
    pub fn is_doji(&self, tolerance: f64) -> bool {
        self.body() <= tolerance
    }

    /// Calculate the upper wick size.
    pub fn upper_wick(&self) -> f64 {
        self.high - self.open.max(self.close)
    }

    /// Calculate the lower wick size.
    pub fn lower_wick(&self) -> f64 {
        self.open.min(self.close) - self.low
    }

    /// Get the VWAP or fall back to the midpoint.
    pub fn vwap_or_mid(&self) -> f64 {
        self.vwap.unwrap_or((self.high + self.low) / 2.0)
    }
}

/// Trade data.
///
/// Represents a single trade execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    /// Ticker symbol
    #[serde(rename = "T", skip_serializing_if = "Option::is_none")]
    pub ticker: Option<Symbol>,

    /// Trade price
    #[serde(rename = "p")]
    pub price: f64,

    /// Trade size (shares)
    #[serde(rename = "s")]
    pub size: u64,

    /// Exchange ID
    #[serde(rename = "x")]
    pub exchange: u8,

    /// Trade ID
    #[serde(rename = "i")]
    pub trade_id: String,

    /// SIP timestamp (Unix nanoseconds)
    #[serde(rename = "t")]
    pub sip_timestamp: i64,

    /// Participant timestamp (Unix nanoseconds)
    #[serde(rename = "y", skip_serializing_if = "Option::is_none")]
    pub participant_timestamp: Option<i64>,

    /// TRF timestamp (Unix nanoseconds)
    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub trf_timestamp: Option<i64>,

    /// Sequence number
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,

    /// Trade conditions
    #[serde(rename = "c", default)]
    pub conditions: Vec<i32>,

    /// Tape (1=NYSE, 2=AMEX, 3=NASDAQ)
    #[serde(rename = "z", skip_serializing_if = "Option::is_none")]
    pub tape: Option<u8>,
}

impl Trade {
    /// Calculate the trade value (price * size).
    pub fn value(&self) -> f64 {
        self.price * self.size as f64
    }
}

/// Quote (NBBO) data.
///
/// Represents the National Best Bid and Offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    /// Ticker symbol
    #[serde(rename = "T", skip_serializing_if = "Option::is_none")]
    pub ticker: Option<Symbol>,

    /// Bid price
    #[serde(rename = "bp")]
    pub bid_price: f64,

    /// Bid size
    #[serde(rename = "bs")]
    pub bid_size: u64,

    /// Bid exchange ID
    #[serde(rename = "bx")]
    pub bid_exchange: u8,

    /// Ask price
    #[serde(rename = "ap")]
    pub ask_price: f64,

    /// Ask size
    #[serde(rename = "as")]
    pub ask_size: u64,

    /// Ask exchange ID
    #[serde(rename = "ax")]
    pub ask_exchange: u8,

    /// SIP timestamp (Unix nanoseconds)
    #[serde(rename = "t")]
    pub sip_timestamp: i64,

    /// Participant timestamp (Unix nanoseconds)
    #[serde(rename = "y", skip_serializing_if = "Option::is_none")]
    pub participant_timestamp: Option<i64>,

    /// Sequence number
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,

    /// Quote conditions
    #[serde(rename = "c", default)]
    pub conditions: Vec<i32>,

    /// Tape (1=NYSE, 2=AMEX, 3=NASDAQ)
    #[serde(rename = "z", skip_serializing_if = "Option::is_none")]
    pub tape: Option<u8>,
}

impl Quote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ask_price - self.bid_price
    }

    /// Calculate the spread in basis points.
    pub fn spread_bps(&self) -> f64 {
        if self.mid() == 0.0 {
            0.0
        } else {
            (self.spread() / self.mid()) * 10000.0
        }
    }

    /// Calculate the mid price.
    pub fn mid(&self) -> f64 {
        (self.bid_price + self.ask_price) / 2.0
    }

    /// Calculate the size-weighted mid price.
    pub fn weighted_mid(&self) -> f64 {
        let total_size = self.bid_size + self.ask_size;
        if total_size == 0 {
            return self.mid();
        }
        (self.bid_price * self.ask_size as f64 + self.ask_price * self.bid_size as f64)
            / total_size as f64
    }

    /// Check if the quote is crossed (bid > ask).
    pub fn is_crossed(&self) -> bool {
        self.bid_price > self.ask_price
    }

    /// Check if the quote is locked (bid == ask).
    pub fn is_locked(&self) -> bool {
        (self.bid_price - self.ask_price).abs() < f64::EPSILON
    }
}

/// Daily bar data (from snapshots/previous close).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyBar {
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

    /// Volume-weighted average price
    #[serde(rename = "vw", skip_serializing_if = "Option::is_none")]
    pub vwap: Option<f64>,
}

/// Ticker details.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Ticker {
    /// Ticker symbol
    pub ticker: String,

    /// Company name
    pub name: String,

    /// Market type (stocks, options, etc.)
    pub market: String,

    /// Locale (us, global)
    pub locale: String,

    /// Primary exchange
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_exchange: Option<String>,

    /// Asset type (CS, ETF, etc.)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ticker_type: Option<String>,

    /// Whether the ticker is active
    #[serde(default)]
    pub active: bool,

    /// CIK number (for SEC filings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cik: Option<String>,

    /// CUSIP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite_figi: Option<String>,

    /// Currency code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_name: Option<String>,

    /// Last updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_utc: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_bar_calculations() {
        let bar = AggregateBar {
            ticker: Some("AAPL".into()),
            open: 150.0,
            high: 155.0,
            low: 148.0,
            close: 153.0,
            volume: 1000000.0,
            vwap: Some(151.5),
            timestamp: 1703001234567,
            transactions: Some(5000),
            otc: false,
        };

        assert_eq!(bar.range(), 7.0);
        assert_eq!(bar.body(), 3.0);
        assert!(bar.is_bullish());
        assert!(!bar.is_bearish());
        assert!(!bar.is_doji(0.5));
        assert_eq!(bar.upper_wick(), 2.0); // 155 - 153
        assert_eq!(bar.lower_wick(), 2.0); // 150 - 148
        assert_eq!(bar.vwap_or_mid(), 151.5);
    }

    #[test]
    fn test_aggregate_bar_doji() {
        let bar = AggregateBar {
            ticker: Some("AAPL".into()),
            open: 150.0,
            high: 152.0,
            low: 148.0,
            close: 150.1,
            volume: 1000000.0,
            vwap: None,
            timestamp: 1703001234567,
            transactions: None,
            otc: false,
        };

        assert!(bar.is_doji(0.5));
        assert_eq!(bar.vwap_or_mid(), 150.0); // (152 + 148) / 2
    }

    #[test]
    fn test_trade_value() {
        let trade = Trade {
            ticker: Some("AAPL".into()),
            price: 150.0,
            size: 100,
            exchange: 4,
            trade_id: "123".into(),
            sip_timestamp: 1703001234567890123,
            participant_timestamp: None,
            trf_timestamp: None,
            sequence: Some(1),
            conditions: vec![],
            tape: Some(3),
        };

        assert_eq!(trade.value(), 15000.0);
    }

    #[test]
    fn test_quote_calculations() {
        let quote = Quote {
            ticker: Some("AAPL".into()),
            bid_price: 150.00,
            bid_size: 100,
            bid_exchange: 4,
            ask_price: 150.10,
            ask_size: 200,
            ask_exchange: 4,
            sip_timestamp: 1703001234567890123,
            participant_timestamp: None,
            sequence: None,
            conditions: vec![],
            tape: None,
        };

        assert!((quote.spread() - 0.10).abs() < 0.001);
        assert!((quote.mid() - 150.05).abs() < 0.001);
        assert!(!quote.is_crossed());
        assert!(!quote.is_locked());

        // Weighted mid should be closer to bid (larger ask size pulls mid toward bid)
        let wmid = quote.weighted_mid();
        assert!(wmid < quote.mid());
    }

    #[test]
    fn test_quote_crossed() {
        let quote = Quote {
            ticker: Some("AAPL".into()),
            bid_price: 150.10,
            bid_size: 100,
            bid_exchange: 4,
            ask_price: 150.00,
            ask_size: 100,
            ask_exchange: 4,
            sip_timestamp: 1703001234567890123,
            participant_timestamp: None,
            sequence: None,
            conditions: vec![],
            tape: None,
        };

        assert!(quote.is_crossed());
    }

    #[test]
    fn test_aggregate_bar_serde() {
        let bar = AggregateBar {
            ticker: Some("AAPL".into()),
            open: 150.0,
            high: 155.0,
            low: 148.0,
            close: 153.0,
            volume: 1000000.0,
            vwap: Some(151.5),
            timestamp: 1703001234567,
            transactions: Some(5000),
            otc: false,
        };

        let json = serde_json::to_string(&bar).unwrap();
        let parsed: AggregateBar = serde_json::from_str(&json).unwrap();
        assert_eq!(bar, parsed);
    }

    #[test]
    fn test_aggregate_bar_deserialize_api_format() {
        // This is the format returned by the Massive API
        let json = r#"{
            "T": "AAPL",
            "o": 150.0,
            "h": 155.0,
            "l": 148.0,
            "c": 153.0,
            "v": 1000000.0,
            "vw": 151.5,
            "t": 1703001234567,
            "n": 5000
        }"#;

        let bar: AggregateBar = serde_json::from_str(json).unwrap();
        assert_eq!(bar.ticker, Some("AAPL".into()));
        assert_eq!(bar.open, 150.0);
        assert_eq!(bar.transactions, Some(5000));
    }
}
