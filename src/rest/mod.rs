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

mod client;
pub mod endpoints;
pub mod models;
mod pagination;
pub mod request;

pub use client::RestClient;
pub use endpoints::*;
pub use pagination::PageStream;
pub use request::{PaginatableRequest, QueryBuilder, RestRequest};
