//! Crypto-specific models.
//!
//! This module contains types for cryptocurrency market data.

use serde::{Deserialize, Serialize};

/// Crypto trading pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPair {
    /// Ticker symbol (e.g., "X:BTCUSD")
    pub ticker: String,
    /// Base currency (e.g., "BTC")
    pub base_currency: String,
    /// Quote currency (e.g., "USD")
    pub quote_currency: String,
    /// Currency name
    pub name: Option<String>,
    /// Market (always "crypto")
    pub market: Option<String>,
    /// Primary exchange
    pub primary_exchange: Option<String>,
}

impl CryptoPair {
    /// Create a crypto pair from base and quote currencies.
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        let base = base.into();
        let quote = quote.into();
        Self {
            ticker: format!("X:{}{}", base, quote),
            base_currency: base,
            quote_currency: quote,
            name: None,
            market: Some("crypto".into()),
            primary_exchange: None,
        }
    }
}

/// Crypto trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoTrade {
    /// Trade ID
    pub id: Option<String>,
    /// Price
    pub price: f64,
    /// Size
    pub size: f64,
    /// Exchange
    pub exchange: Option<u8>,
    /// Trade conditions
    #[serde(default)]
    pub conditions: Vec<i32>,
    /// Timestamp (Unix ms)
    pub timestamp: i64,
    /// Received timestamp
    pub received_timestamp: Option<i64>,
}

impl CryptoTrade {
    /// Calculate the trade value (price * size).
    pub fn value(&self) -> f64 {
        self.price * self.size
    }
}

/// Crypto quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoQuote {
    /// Ask price
    pub ask_price: f64,
    /// Ask size
    pub ask_size: f64,
    /// Bid price
    pub bid_price: f64,
    /// Bid size
    pub bid_size: f64,
    /// Ask exchange
    pub ask_exchange: Option<u8>,
    /// Bid exchange
    pub bid_exchange: Option<u8>,
    /// Timestamp
    pub timestamp: i64,
}

impl CryptoQuote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ask_price - self.bid_price
    }

    /// Calculate the spread percentage.
    pub fn spread_percent(&self) -> f64 {
        let mid = (self.ask_price + self.bid_price) / 2.0;
        if mid > 0.0 {
            self.spread() / mid * 100.0
        } else {
            0.0
        }
    }

    /// Calculate the mid price.
    pub fn mid_price(&self) -> f64 {
        (self.ask_price + self.bid_price) / 2.0
    }
}

/// Crypto aggregate bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoBar {
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
    /// Volume (in base currency)
    #[serde(rename = "v")]
    pub volume: f64,
    /// VWAP
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Number of transactions
    #[serde(rename = "n")]
    pub num_transactions: Option<u64>,
    /// Timestamp (Unix ms)
    #[serde(rename = "t")]
    pub timestamp: i64,
}

impl CryptoBar {
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

    /// Calculate approximate notional volume (volume * vwap or close).
    pub fn notional_volume(&self) -> f64 {
        self.volume * self.vwap.unwrap_or(self.close)
    }
}

/// Crypto exchange information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoExchange {
    /// Exchange ID
    pub id: u8,
    /// Exchange type (usually "exchange")
    #[serde(rename = "type")]
    pub exchange_type: Option<String>,
    /// Exchange name
    pub name: String,
    /// Exchange URL
    pub url: Option<String>,
    /// Market type
    pub market: Option<String>,
}

/// Crypto snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoSnapshot {
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
    pub day: Option<CryptoBar>,
    /// Last trade
    #[serde(rename = "lastTrade")]
    pub last_trade: Option<CryptoTrade>,
    /// Previous day aggregate
    #[serde(rename = "prevDay")]
    pub prev_day: Option<CryptoBar>,
    /// Minute aggregate
    pub min: Option<CryptoBar>,
}

/// Daily open/close for crypto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoOpenClose {
    /// Symbol
    pub symbol: String,
    /// Is crypto market
    #[serde(rename = "isUTC")]
    pub is_utc: bool,
    /// Day
    pub day: String,
    /// Open price
    pub open: f64,
    /// Close price
    pub close: f64,
    /// Open trades
    #[serde(rename = "openTrades")]
    pub open_trades: Option<Vec<CryptoTrade>>,
    /// Closing trades
    #[serde(rename = "closingTrades")]
    pub closing_trades: Option<Vec<CryptoTrade>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_pair_new() {
        let pair = CryptoPair::new("BTC", "USD");
        assert_eq!(pair.ticker, "X:BTCUSD");
        assert_eq!(pair.base_currency, "BTC");
        assert_eq!(pair.quote_currency, "USD");
    }

    #[test]
    fn test_crypto_trade_value() {
        let trade = CryptoTrade {
            id: Some("123".into()),
            price: 42000.0,
            size: 0.5,
            exchange: Some(1),
            conditions: vec![],
            timestamp: 1703001234567,
            received_timestamp: None,
        };

        assert_eq!(trade.value(), 21000.0);
    }

    #[test]
    fn test_crypto_quote_spread() {
        let quote = CryptoQuote {
            ask_price: 42050.0,
            ask_size: 1.0,
            bid_price: 42000.0,
            bid_size: 2.0,
            ask_exchange: Some(1),
            bid_exchange: Some(2),
            timestamp: 1703001234567,
        };

        assert_eq!(quote.spread(), 50.0);
        assert!((quote.mid_price() - 42025.0).abs() < 0.01);
    }

    #[test]
    fn test_crypto_bar_calculations() {
        let bar = CryptoBar {
            open: 42000.0,
            high: 43000.0,
            low: 41500.0,
            close: 42500.0,
            volume: 100.0,
            vwap: Some(42250.0),
            num_transactions: Some(5000),
            timestamp: 1703001234567,
        };

        assert_eq!(bar.change(), 500.0);
        assert!(bar.is_bullish());
        assert!(!bar.is_bearish());
        assert_eq!(bar.range(), 1500.0);
        assert_eq!(bar.notional_volume(), 4225000.0);
    }

    #[test]
    fn test_crypto_bar_deserialize() {
        let json = r#"{
            "o": 42000.0,
            "h": 43000.0,
            "l": 41500.0,
            "c": 42500.0,
            "v": 100.0,
            "vw": 42250.0,
            "n": 5000,
            "t": 1703001234567
        }"#;

        let bar: CryptoBar = serde_json::from_str(json).unwrap();
        assert_eq!(bar.open, 42000.0);
        assert_eq!(bar.high, 43000.0);
    }

    #[test]
    fn test_crypto_quote_deserialize() {
        let json = r#"{
            "ask_price": 42050.0,
            "ask_size": 1.0,
            "bid_price": 42000.0,
            "bid_size": 2.0,
            "timestamp": 1703001234567
        }"#;

        let quote: CryptoQuote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.ask_price, 42050.0);
        assert_eq!(quote.bid_price, 42000.0);
    }
}
