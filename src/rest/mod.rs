//! REST API client for Massive.com.
//!
//! This module provides typed access to the Massive REST API, including
//! automatic pagination, retry logic, and response parsing.

mod client;
pub mod models;

pub use client::RestClient;
