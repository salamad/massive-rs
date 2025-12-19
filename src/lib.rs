//! High-performance Rust client for Massive.com market data APIs.
//!
//! This crate provides both REST and WebSocket clients for accessing
//! market data from the Massive.com (formerly Polygon.io) platform.
//! It is designed for low-latency, high-throughput applications including
//! algorithmic trading and HFT systems.
//!
//! # Features
//!
//! - **REST Client**: Typed API requests with automatic pagination
//! - **WebSocket Client**: Real-time streaming with backpressure handling
//! - **Type Safety**: Strongly-typed models for all API responses
//! - **Performance**: Optimized for sub-10μs message parsing
//! - **Reliability**: Automatic reconnection and retry logic
//!
//! # Feature Flags
//!
//! - `default`: Includes `rustls`, `gzip`, and `ws` features
//! - `rustls`: Use rustls for TLS (recommended)
//! - `native-tls`: Use native TLS instead of rustls
//! - `gzip`: Enable gzip compression for REST requests
//! - `ws`: Enable WebSocket client support
//! - `simd-json`: Use SIMD-accelerated JSON parsing
//! - `decimal`: Use exact decimal arithmetic for prices
//!
//! # Quick Start
//!
//! ## REST API
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
//!     // Fetch aggregate bars
//!     // let bars = client.get_aggs("AAPL", 1, Timespan::Day, "2024-01-01", "2024-01-31").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## WebSocket Streaming
//!
//! ```no_run
//! # #[cfg(feature = "ws")]
//! use massive_rs::ws::{WsClient, Subscription};
//! # #[cfg(feature = "ws")]
//! use massive_rs::config::WsConfig;
//! use futures::StreamExt;
//!
//! # #[cfg(feature = "ws")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = WsClient::new(WsConfig::default())?;
//!     let (handle, mut stream) = client.connect().await?;
//!
//!     // Subscribe to trades
//!     handle.subscribe(&[Subscription::trade("AAPL")]).await?;
//!
//!     // Process events
//!     while let Some(batch) = stream.next().await {
//!         let batch = batch?;
//!         for event in batch.events {
//!             println!("{:?}", event);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! # #[cfg(not(feature = "ws"))]
//! # fn main() {}
//! ```
//!
//! # API Coverage
//!
//! The crate aims to provide complete coverage of the Massive.com API:
//!
//! - **Market Data**: Aggregates, trades, quotes, snapshots
//! - **Reference Data**: Tickers, exchanges, dividends, splits
//! - **Options**: Contracts, chains, Greeks
//! - **Forex/Crypto**: Currency pairs, exchange rates
//!
//! # Error Handling
//!
//! All fallible operations return `Result<T, MassiveError>`. The error type
//! provides detailed information about what went wrong, including API error
//! responses and request IDs for debugging.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::all)]

pub mod auth;
pub mod config;
pub mod error;
pub mod metrics;
pub mod parse;
pub mod util;

pub mod rest;

#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub mod ws;

pub mod models;

// Re-export commonly used types at crate root
pub use auth::{ApiKey, AuthMode};
pub use config::{PaginationMode, RestConfig};
pub use error::{MassiveError, Result};
pub use metrics::{ClientStats, MetricsSink, NoopMetrics, StatsSnapshot, TracingMetrics};
pub use parse::parse_ws_events;

#[cfg(feature = "ws")]
pub use config::{DispatchConfig, Feed, Market, OverflowPolicy, ReconnectConfig, WsConfig};

#[cfg(feature = "ws")]
pub use error::WsError;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User agent string used in requests
pub fn user_agent() -> String {
    format!("massive-rs/{}", VERSION)
}
