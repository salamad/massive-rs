//! REST API client for Massive.com.
//!
//! This module provides typed access to the Massive REST API, including
//! automatic pagination, retry logic, and response parsing.
//!
//! # Quick Start
//!
//! ```no_run
//! use massive_rs::rest::RestClient;
//! use massive_rs::config::RestConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client with API key from environment
//!     let client = RestClient::new(RestConfig::default())?;
//!
//!     // Execute requests...
//!     Ok(())
//! }
//! ```
//!
//! # Pagination
//!
//! The REST client supports three pagination modes:
//!
//! - [`PaginationMode::Auto`](crate::config::PaginationMode::Auto): Automatically fetch all pages
//! - [`PaginationMode::None`](crate::config::PaginationMode::None): Only fetch the first page
//! - [`PaginationMode::MaxItems(n)`](crate::config::PaginationMode::MaxItems): Stop after n items
//!
//! # Asset Classes
//!
//! The [`asset_class`] module provides type-safe abstractions for different asset classes:
//!
//! - [`Stocks`](asset_class::Stocks): US equities (no prefix)
//! - [`Options`](asset_class::Options): Equity options (`O:` prefix)
//! - [`Forex`](asset_class::Forex): Foreign exchange (`C:` prefix)
//! - [`Crypto`](asset_class::Crypto): Cryptocurrency (`X:` prefix)
//! - [`Indices`](asset_class::Indices): Market indices (`I:` prefix)
//! - [`Futures`](asset_class::Futures): Futures contracts (separate API)
//!
//! # Filters
//!
//! The [`filters`] module provides builders for range comparisons:
//!
//! ```
//! use massive_rs::rest::filters::{RangeFilter, SortOrder};
//!
//! let date_filter = RangeFilter::new()
//!     .gte("2024-01-01".to_string())
//!     .lte("2024-12-31".to_string());
//! ```

pub mod asset_class;
mod client;
pub mod endpoints;
pub mod filters;
pub mod models;
mod pagination;
pub mod request;

pub use client::RestClient;
pub use endpoints::*;
pub use pagination::PageStream;
pub use request::{PaginatableRequest, QueryBuilder, RestRequest};

// Re-export commonly used asset class types
pub use asset_class::{AssetClass, Crypto, Forex, Futures, Indices, Options, Stocks};

// Re-export commonly used filter types
pub use filters::{RangeFilter, SortBuilder, SortOrder, SortSpec};
