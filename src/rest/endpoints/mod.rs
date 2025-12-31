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
//! - **Snapshots**: Real-time ticker snapshots, gainers/losers
//! - **Reference**: Ticker metadata, exchanges, markets
//! - **Market Status**: Current trading status for exchanges
//! - **Indicators**: Technical indicators (RSI, etc.)

mod indicators;
mod market_data;
mod market_status;
mod quotes;
mod reference;
mod snapshots;
mod trades;

pub use indicators::*;
pub use market_data::*;
pub use market_status::*;
pub use quotes::*;
pub use reference::*;
pub use snapshots::*;
pub use trades::*;
