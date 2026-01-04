//! Asset class abstractions for multi-asset endpoint support.
//!
//! This module provides a unified trait system for handling different asset classes
//! (stocks, options, forex, crypto, indices, futures) that share similar API structures
//! but differ in ticker prefixes and endpoint paths.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::asset_class::{AssetClass, Stocks, Options, Forex};
//!
//! // Format tickers with appropriate prefixes
//! assert_eq!(Stocks::format_ticker("AAPL"), "AAPL");
//! assert_eq!(Options::format_ticker("AAPL230120C00150000"), "O:AAPL230120C00150000");
//! assert_eq!(Forex::format_ticker("EURUSD"), "C:EURUSD");
//!
//! // Already-prefixed tickers are unchanged
//! assert_eq!(Options::format_ticker("O:AAPL230120C00150000"), "O:AAPL230120C00150000");
//! ```

use std::fmt;

/// Marker trait for asset classes supporting ticker prefix formatting.
///
/// Each asset class in the Massive API uses specific ticker prefixes:
/// - Stocks: no prefix
/// - Options: `O:` prefix
/// - Forex: `C:` prefix
/// - Crypto: `X:` prefix
/// - Indices: `I:` prefix
/// - Futures: no prefix (uses separate API endpoints)
pub trait AssetClass: fmt::Debug + Clone + Send + Sync + 'static {
    /// The ticker prefix for this asset class (e.g., "O:" for options).
    fn ticker_prefix() -> &'static str;

    /// The market identifier for reference endpoints.
    fn market() -> Option<&'static str>;

    /// The locale for snapshot endpoints.
    fn locale() -> &'static str;

    /// Format a raw ticker with the appropriate prefix.
    ///
    /// If the ticker already has the correct prefix, it is returned unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::asset_class::{AssetClass, Options};
    ///
    /// assert_eq!(Options::format_ticker("AAPL230120C00150000"), "O:AAPL230120C00150000");
    /// assert_eq!(Options::format_ticker("O:AAPL230120C00150000"), "O:AAPL230120C00150000");
    /// ```
    fn format_ticker(ticker: &str) -> String {
        let prefix = Self::ticker_prefix();
        if prefix.is_empty() || ticker.starts_with(prefix) {
            ticker.to_string()
        } else {
            format!("{}{}", prefix, ticker)
        }
    }

    /// Strip the asset class prefix from a ticker if present.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::asset_class::{AssetClass, Options};
    ///
    /// assert_eq!(Options::strip_prefix("O:AAPL230120C00150000"), "AAPL230120C00150000");
    /// assert_eq!(Options::strip_prefix("AAPL230120C00150000"), "AAPL230120C00150000");
    /// ```
    fn strip_prefix(ticker: &str) -> &str {
        let prefix = Self::ticker_prefix();
        ticker.strip_prefix(prefix).unwrap_or(ticker)
    }

    /// Check if a ticker belongs to this asset class based on its prefix.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::asset_class::{AssetClass, Options, Stocks};
    ///
    /// assert!(Options::is_asset_class("O:AAPL230120C00150000"));
    /// assert!(!Options::is_asset_class("AAPL"));
    /// assert!(Stocks::is_asset_class("AAPL")); // Stocks match anything without prefix
    /// ```
    fn is_asset_class(ticker: &str) -> bool {
        let prefix = Self::ticker_prefix();
        if prefix.is_empty() {
            // For stocks (no prefix), check that no other prefix is present
            !ticker.starts_with("O:")
                && !ticker.starts_with("C:")
                && !ticker.starts_with("X:")
                && !ticker.starts_with("I:")
        } else {
            ticker.starts_with(prefix)
        }
    }
}

/// Stocks asset class (no prefix).
///
/// Used for US equities and other stock-like instruments.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Stocks;

impl AssetClass for Stocks {
    fn ticker_prefix() -> &'static str {
        ""
    }

    fn market() -> Option<&'static str> {
        Some("stocks")
    }

    fn locale() -> &'static str {
        "us"
    }
}

/// Options asset class (O: prefix).
///
/// Used for equity options contracts. Options tickers follow the OCC symbology:
/// `O:{underlying}{expiration}{type}{strike}`
///
/// Example: `O:AAPL230120C00150000` represents an AAPL call option expiring
/// January 20, 2023 with a $150 strike price.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Options;

impl AssetClass for Options {
    fn ticker_prefix() -> &'static str {
        "O:"
    }

    fn market() -> Option<&'static str> {
        Some("options")
    }

    fn locale() -> &'static str {
        "us"
    }
}

/// Forex asset class (C: prefix).
///
/// Used for foreign exchange currency pairs.
/// Format: `C:{from}{to}` where from and to are 3-letter currency codes.
///
/// Example: `C:EURUSD` represents EUR/USD.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Forex;

impl AssetClass for Forex {
    fn ticker_prefix() -> &'static str {
        "C:"
    }

    fn market() -> Option<&'static str> {
        Some("fx")
    }

    fn locale() -> &'static str {
        "global"
    }
}

impl Forex {
    /// Format a forex pair from currency codes.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::asset_class::Forex;
    ///
    /// assert_eq!(Forex::pair("EUR", "USD"), "C:EURUSD");
    /// ```
    pub fn pair(from: &str, to: &str) -> String {
        format!("C:{}{}", from, to)
    }
}

/// Crypto asset class (X: prefix).
///
/// Used for cryptocurrency trading pairs.
/// Format: `X:{from}{to}` where from is the crypto and to is the quote currency.
///
/// Example: `X:BTCUSD` represents BTC/USD.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Crypto;

impl AssetClass for Crypto {
    fn ticker_prefix() -> &'static str {
        "X:"
    }

    fn market() -> Option<&'static str> {
        Some("crypto")
    }

    fn locale() -> &'static str {
        "global"
    }
}

impl Crypto {
    /// Format a crypto pair from currency codes.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::asset_class::Crypto;
    ///
    /// assert_eq!(Crypto::pair("BTC", "USD"), "X:BTCUSD");
    /// ```
    pub fn pair(from: &str, to: &str) -> String {
        format!("X:{}{}", from, to)
    }
}

/// Indices asset class (I: prefix).
///
/// Used for market indices like SPX, NDX, VIX.
///
/// Example: `I:SPX` represents the S&P 500 index.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Indices;

impl AssetClass for Indices {
    fn ticker_prefix() -> &'static str {
        "I:"
    }

    fn market() -> Option<&'static str> {
        Some("indices")
    }

    fn locale() -> &'static str {
        "us"
    }
}

/// Futures asset class (uses separate API endpoints).
///
/// Futures contracts use different API endpoints (`/futures/vX/...`) rather than
/// ticker prefixes. The ticker format varies by exchange and product.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Futures;

impl AssetClass for Futures {
    fn ticker_prefix() -> &'static str {
        ""
    }

    fn market() -> Option<&'static str> {
        // Futures use separate endpoints, not the unified market parameter
        None
    }

    fn locale() -> &'static str {
        "us"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stocks_format_ticker() {
        assert_eq!(Stocks::format_ticker("AAPL"), "AAPL");
        assert_eq!(Stocks::format_ticker("MSFT"), "MSFT");
    }

    #[test]
    fn test_options_format_ticker() {
        assert_eq!(
            Options::format_ticker("AAPL230120C00150000"),
            "O:AAPL230120C00150000"
        );
        // Already prefixed
        assert_eq!(
            Options::format_ticker("O:AAPL230120C00150000"),
            "O:AAPL230120C00150000"
        );
    }

    #[test]
    fn test_forex_format_ticker() {
        assert_eq!(Forex::format_ticker("EURUSD"), "C:EURUSD");
        assert_eq!(Forex::format_ticker("C:EURUSD"), "C:EURUSD");
    }

    #[test]
    fn test_crypto_format_ticker() {
        assert_eq!(Crypto::format_ticker("BTCUSD"), "X:BTCUSD");
        assert_eq!(Crypto::format_ticker("X:BTCUSD"), "X:BTCUSD");
    }

    #[test]
    fn test_indices_format_ticker() {
        assert_eq!(Indices::format_ticker("SPX"), "I:SPX");
        assert_eq!(Indices::format_ticker("I:SPX"), "I:SPX");
    }

    #[test]
    fn test_futures_format_ticker() {
        // Futures have no prefix
        assert_eq!(Futures::format_ticker("ESH4"), "ESH4");
    }

    #[test]
    fn test_strip_prefix() {
        assert_eq!(
            Options::strip_prefix("O:AAPL230120C00150000"),
            "AAPL230120C00150000"
        );
        assert_eq!(
            Options::strip_prefix("AAPL230120C00150000"),
            "AAPL230120C00150000"
        );
        assert_eq!(Forex::strip_prefix("C:EURUSD"), "EURUSD");
        assert_eq!(Crypto::strip_prefix("X:BTCUSD"), "BTCUSD");
        assert_eq!(Indices::strip_prefix("I:SPX"), "SPX");
    }

    #[test]
    fn test_is_asset_class() {
        // Options
        assert!(Options::is_asset_class("O:AAPL230120C00150000"));
        assert!(!Options::is_asset_class("AAPL"));
        assert!(!Options::is_asset_class("C:EURUSD"));

        // Forex
        assert!(Forex::is_asset_class("C:EURUSD"));
        assert!(!Forex::is_asset_class("EURUSD"));
        assert!(!Forex::is_asset_class("X:BTCUSD"));

        // Crypto
        assert!(Crypto::is_asset_class("X:BTCUSD"));
        assert!(!Crypto::is_asset_class("BTCUSD"));

        // Indices
        assert!(Indices::is_asset_class("I:SPX"));
        assert!(!Indices::is_asset_class("SPX"));

        // Stocks (no prefix = stocks)
        assert!(Stocks::is_asset_class("AAPL"));
        assert!(Stocks::is_asset_class("MSFT"));
        assert!(!Stocks::is_asset_class("O:AAPL230120C00150000"));
        assert!(!Stocks::is_asset_class("C:EURUSD"));
    }

    #[test]
    fn test_forex_pair() {
        assert_eq!(Forex::pair("EUR", "USD"), "C:EURUSD");
        assert_eq!(Forex::pair("GBP", "JPY"), "C:GBPJPY");
    }

    #[test]
    fn test_crypto_pair() {
        assert_eq!(Crypto::pair("BTC", "USD"), "X:BTCUSD");
        assert_eq!(Crypto::pair("ETH", "BTC"), "X:ETHBTC");
    }

    #[test]
    fn test_market_identifiers() {
        assert_eq!(Stocks::market(), Some("stocks"));
        assert_eq!(Options::market(), Some("options"));
        assert_eq!(Forex::market(), Some("fx"));
        assert_eq!(Crypto::market(), Some("crypto"));
        assert_eq!(Indices::market(), Some("indices"));
        assert_eq!(Futures::market(), None);
    }

    #[test]
    fn test_locales() {
        assert_eq!(Stocks::locale(), "us");
        assert_eq!(Options::locale(), "us");
        assert_eq!(Forex::locale(), "global");
        assert_eq!(Crypto::locale(), "global");
        assert_eq!(Indices::locale(), "us");
        assert_eq!(Futures::locale(), "us");
    }

    #[test]
    fn test_asset_class_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Stocks>();
        assert_send_sync::<Options>();
        assert_send_sync::<Forex>();
        assert_send_sync::<Crypto>();
        assert_send_sync::<Indices>();
        assert_send_sync::<Futures>();
    }
}
