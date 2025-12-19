//! Forex-specific models.
//!
//! This module contains types for foreign exchange market data.

use serde::{Deserialize, Serialize};

/// Currency pair information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyPair {
    /// Base currency (e.g., "EUR")
    pub base_currency: String,
    /// Quote currency (e.g., "USD")
    pub quote_currency: String,
    /// Ticker symbol (e.g., "C:EURUSD")
    pub ticker: String,
    /// Currency name
    pub name: Option<String>,
    /// Market (always "fx")
    pub market: Option<String>,
}

impl CurrencyPair {
    /// Create a currency pair from base and quote currencies.
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        let base = base.into();
        let quote = quote.into();
        Self {
            ticker: format!("C:{}{}", base, quote),
            base_currency: base,
            quote_currency: quote,
            name: None,
            market: Some("fx".into()),
        }
    }
}

/// Forex quote (bid/ask with timestamp).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexQuote {
    /// Ask price
    pub ask: f64,
    /// Bid price
    pub bid: f64,
    /// Exchange
    pub exchange: Option<u8>,
    /// Timestamp (Unix ms)
    pub timestamp: i64,
}

impl ForexQuote {
    /// Calculate the bid-ask spread in pips.
    /// For most pairs, a pip is 0.0001. For JPY pairs, it's 0.01.
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate the mid rate.
    pub fn mid(&self) -> f64 {
        (self.ask + self.bid) / 2.0
    }
}

/// Forex trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexTrade {
    /// Price
    pub price: f64,
    /// Exchange
    pub exchange: Option<u8>,
    /// Timestamp (Unix ms)
    pub timestamp: i64,
}

/// Forex aggregate bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexBar {
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
    /// Volume (number of quotes)
    #[serde(rename = "v")]
    pub volume: f64,
    /// Number of transactions
    #[serde(rename = "n")]
    pub num_transactions: Option<u64>,
    /// Timestamp (Unix ms)
    #[serde(rename = "t")]
    pub timestamp: i64,
}

impl ForexBar {
    /// Calculate the price change.
    pub fn change(&self) -> f64 {
        self.close - self.open
    }

    /// Calculate the price change in percentage.
    pub fn change_percent(&self) -> f64 {
        if self.open != 0.0 {
            (self.close - self.open) / self.open * 100.0
        } else {
            0.0
        }
    }

    /// Calculate the bar range (high - low).
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Check if this is a bullish bar.
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if this is a bearish bar.
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}

/// Currency conversion result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyConversion {
    /// Converted amount
    pub converted: f64,
    /// Base currency
    pub from: String,
    /// Initial amount
    pub initial_amount: f64,
    /// Last quote used
    pub last: Option<ForexQuote>,
    /// Quote currency
    pub to: String,
}

/// Forex snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexSnapshot {
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
    /// Day aggregate
    pub day: Option<ForexBar>,
    /// Last quote
    #[serde(rename = "lastQuote")]
    pub last_quote: Option<ForexQuote>,
    /// Previous day aggregate
    #[serde(rename = "prevDay")]
    pub prev_day: Option<ForexBar>,
    /// Minute aggregate
    pub min: Option<ForexBar>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_pair_new() {
        let pair = CurrencyPair::new("EUR", "USD");
        assert_eq!(pair.ticker, "C:EURUSD");
        assert_eq!(pair.base_currency, "EUR");
        assert_eq!(pair.quote_currency, "USD");
    }

    #[test]
    fn test_forex_quote_spread() {
        let quote = ForexQuote {
            ask: 1.10025,
            bid: 1.10000,
            exchange: Some(1),
            timestamp: 1703001234567,
        };

        assert!((quote.spread() - 0.00025).abs() < 0.000001);
        assert!((quote.mid() - 1.100125).abs() < 0.000001);
    }

    #[test]
    fn test_forex_bar_calculations() {
        let bar = ForexBar {
            open: 1.1000,
            high: 1.1050,
            low: 1.0980,
            close: 1.1030,
            volume: 10000.0,
            num_transactions: Some(5000),
            timestamp: 1703001234567,
        };

        assert!((bar.change() - 0.003).abs() < 0.0001);
        assert!((bar.range() - 0.007).abs() < 0.0001);
        assert!(bar.is_bullish());
        assert!(!bar.is_bearish());
    }

    #[test]
    fn test_forex_quote_deserialize() {
        let json = r#"{
            "ask": 1.10025,
            "bid": 1.10000,
            "exchange": 1,
            "timestamp": 1703001234567
        }"#;

        let quote: ForexQuote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.ask, 1.10025);
        assert_eq!(quote.bid, 1.10000);
    }

    #[test]
    fn test_forex_bar_deserialize() {
        let json = r#"{
            "o": 1.1000,
            "h": 1.1050,
            "l": 1.0980,
            "c": 1.1030,
            "v": 10000.0,
            "n": 5000,
            "t": 1703001234567
        }"#;

        let bar: ForexBar = serde_json::from_str(json).unwrap();
        assert_eq!(bar.open, 1.1000);
        assert_eq!(bar.high, 1.1050);
    }

    #[test]
    fn test_conversion_deserialize() {
        let json = r#"{
            "converted": 110.25,
            "from": "USD",
            "initial_amount": 100.0,
            "to": "EUR"
        }"#;

        let conversion: CurrencyConversion = serde_json::from_str(json).unwrap();
        assert_eq!(conversion.converted, 110.25);
        assert_eq!(conversion.from, "USD");
        assert_eq!(conversion.to, "EUR");
    }
}
