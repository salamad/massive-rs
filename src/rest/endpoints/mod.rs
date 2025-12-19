//! Typed REST API endpoints.
//!
//! This module contains typed request structures for all supported
//! Massive API endpoints.

mod market_data;
mod reference;

pub use market_data::*;
pub use reference::*;
