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
//! - **Forex**: Currency pairs, conversions, snapshots
//! - **Crypto**: Cryptocurrency pairs, L2 book, snapshots
//! - **Fundamentals**: Financial statements, short interest data
//! - **Partner**: Benzinga earnings/ratings, ETF Global data
//! - **Economy**: Treasury yields, inflation, Fed funds rate
//! - **News**: News articles, related companies, ticker events

mod corporate_actions;
mod crypto;
mod economy;
mod forex;
mod fundamentals;
mod futures;
mod indicators;
mod market_data;
mod market_status;
mod news;
mod options;
mod partner;
mod quotes;
mod reference;
mod snapshots;
mod trades;

pub use corporate_actions::*;
pub use crypto::*;
pub use economy::*;
pub use forex::*;
pub use fundamentals::*;
pub use futures::*;
pub use indicators::*;
pub use market_data::*;
pub use market_status::*;
pub use news::*;
pub use options::*;
pub use partner::*;
pub use quotes::*;
pub use reference::*;
pub use snapshots::*;
pub use trades::*;
