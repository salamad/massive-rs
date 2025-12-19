//! Data models for Massive API responses.
//!
//! This module contains the types used to represent market data
//! from the Massive API, including aggregates, trades, quotes, and more.
//!
//! # Model Categories
//!
//! - **Common**: Core market data types (AggregateBar, Trade, Quote, Ticker)
//! - **Options**: Options-specific types (OptionContract, Greeks, OptionQuote)
//! - **Forex**: Foreign exchange types (CurrencyPair, ForexQuote, ForexBar)
//! - **Crypto**: Cryptocurrency types (CryptoPair, CryptoTrade, CryptoBar)

mod common;
pub mod crypto;
pub mod forex;
pub mod options;

pub use common::*;
pub use crypto::{CryptoBar, CryptoPair, CryptoQuote, CryptoSnapshot, CryptoTrade};
pub use forex::{
    CurrencyConversion, CurrencyPair, ForexBar, ForexQuote, ForexSnapshot, ForexTrade,
};
pub use options::{ContractType, Greeks, OptionContract, OptionQuote, OptionSnapshot};
