//! Typed REST API endpoints.
//!
//! This module contains typed request structures for all supported
//! Massive API endpoints.
//!
//! # Endpoint Categories
//!
//! - **Market Data**: Aggregates, open/close data
//! - **Trades**: Historical trade data and last trade
//! - **Quotes**: Historical quote (NBBO) data and last quote
//! - **Snapshots**: Real-time ticker snapshots, gainers/losers, unified snapshots
//! - **Reference**: Ticker metadata, exchanges, markets
//! - **Market Status**: Current trading status for exchanges
//! - **Indicators**: Technical indicators (RSI, SMA, EMA, MACD)
//! - **Corporate Actions**: Dividends and stock splits
//! - **Options**: Options contracts, chain snapshots, greeks
//! - **Futures**: Futures contracts, products, schedules

mod corporate_actions;
mod futures;
mod indicators;
mod market_data;
mod market_status;
mod options;
mod quotes;
mod reference;
mod snapshots;
mod trades;

pub use corporate_actions::*;
pub use futures::*;
pub use indicators::*;
pub use market_data::*;
pub use market_status::*;
pub use options::*;
pub use quotes::*;
pub use reference::*;
pub use snapshots::*;
pub use trades::*;
