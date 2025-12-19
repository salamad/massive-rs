//! WebSocket client for Massive.com real-time data.
//!
//! This module provides streaming access to real-time market data
//! via WebSocket connections, with automatic reconnection and
//! backpressure handling.

mod client;
pub mod models;
mod protocol;

pub use client::{ConnectionState, WsClient, WsHandle, WsMessageBatch, WsState, WsStats};
pub use models::events::WsEvent;
pub use protocol::Subscription;
