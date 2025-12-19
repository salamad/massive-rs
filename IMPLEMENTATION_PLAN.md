# Massive.com Rust Crate: Multi-Phased Design and Implementation Plan

**Document Version:** 1.0
**Date:** December 2025
**Crate Name:** `massive-rs` (fallback: `massive_rs`)
**Target Rust Edition:** 2021
**MSRV:** 1.75.0 (for async trait stabilization)

---

## Executive Summary

This document provides a comprehensive, actionable implementation plan for developing a production-grade, HFT-ready Rust crate that fully integrates with the Massive.com (formerly Polygon.io) REST and WebSocket APIs. The plan is structured into six phases with clear milestones, deliverables, and acceptance criteria.

### Key Design Principles

1. **Performance First**: Sub-10μs message deserialization, 100k+ msgs/sec throughput
2. **Type Safety**: Strongly-typed APIs with compile-time guarantees
3. **Zero-Copy Where Feasible**: Minimize allocations in hot paths
4. **Spec-Driven**: OpenAPI code generation for complete endpoint coverage
5. **Idiomatic Rust**: Follow RFC 1105 API guidelines, `Send + Sync` where meaningful
6. **HFT-Grade Reliability**: Backpressure handling, automatic reconnection, graceful degradation

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Technology Stack](#2-technology-stack)
3. [Phase 1: Foundation & Core Infrastructure](#3-phase-1-foundation--core-infrastructure)
4. [Phase 2: REST Client Implementation](#4-phase-2-rest-client-implementation)
5. [Phase 3: WebSocket Client Implementation](#5-phase-3-websocket-client-implementation)
6. [Phase 4: Code Generation & Full API Coverage](#6-phase-4-code-generation--full-api-coverage)
7. [Phase 5: Performance Optimization & HFT Hardening](#7-phase-5-performance-optimization--hft-hardening)
8. [Phase 6: Documentation, Testing & Release](#8-phase-6-documentation-testing--release)
9. [Risk Assessment & Mitigation](#9-risk-assessment--mitigation)
10. [Appendices](#10-appendices)

---

## 1. Architecture Overview

### 1.1 High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           massive-rs Crate                               │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   config    │  │    auth     │  │    error    │  │    util     │    │
│  │  ─────────  │  │  ─────────  │  │  ─────────  │  │  ─────────  │    │
│  │ • RestConfig│  │ • ApiKey    │  │ • Massive-  │  │ • UnixMs    │    │
│  │ • WsConfig  │  │ • AuthMode  │  │   Error     │  │ • SmolStr   │    │
│  │ • Timeouts  │  │ • EnvLoader │  │ • WsError   │  │ • URL build │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐│
│  │           rest                  │  │             ws                 ││
│  │  ────────────────────────────  │  │  ────────────────────────────  ││
│  │  ┌──────────┐ ┌──────────────┐ │  │  ┌──────────┐ ┌──────────────┐ ││
│  │  │  client  │ │   request    │ │  │  │  client  │ │   protocol   │ ││
│  │  │ ──────── │ │ ──────────── │ │  │  │ ──────── │ │ ──────────── │ ││
│  │  │RestClient│ │ RequestTrait │ │  │  │ WsClient │ │ Auth/Sub msgs│ ││
│  │  │ execute()│ │ Builders     │ │  │  │ WsHandle │ │ Control msgs │ ││
│  │  └──────────┘ └──────────────┘ │  │  └──────────┘ └──────────────┘ ││
│  │  ┌──────────┐ ┌──────────────┐ │  │  ┌──────────┐ ┌──────────────┐ ││
│  │  │pagination│ │    models    │ │  │  │  models  │ │   dispatch   │ ││
│  │  │ ──────── │ │ ──────────── │ │  │  │ ──────── │ │ ──────────── │ ││
│  │  │ Auto     │ │ Generated +  │ │  │  │ WsEvent  │ │ Backpressure │ ││
│  │  │ MaxItems │ │ Hand-written │ │  │  │ WsTrade  │ │ Fanout modes │ ││
│  │  │ None     │ │ Envelopes    │ │  │  │ WsQuote  │ │ Ring buffers │ ││
│  │  └──────────┘ └──────────────┘ │  │  └──────────┘ └──────────────┘ ││
│  └────────────────────────────────┘  └────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────────────────────────────────────────┐│
│  │                           models                                    ││
│  │  ────────────────────────────────────────────────────────────────  ││
│  │  common.rs │ stocks.rs │ options.rs │ forex.rs │ crypto.rs │ ...   ││
│  │  AggBar    │ Trade     │ OptionQuote│ ForexQuote│ CryptoTrade     ││
│  │  Quote     │ Snapshot  │ Greeks     │ Currency │ Exchange        ││
│  └────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
        ┌───────────────────────────────────────────────────┐
        │               External Services                    │
        ├───────────────────────────────────────────────────┤
        │  REST: https://api.massive.com                     │
        │  WS Real-time: wss://socket.massive.com/{market}   │
        │  WS Delayed: wss://delayed.massive.com/{market}    │
        └───────────────────────────────────────────────────┘
```

### 1.2 Module Structure

```
massive-rs/
├── Cargo.toml
├── build.rs                      # OpenAPI codegen orchestration
├── openapi/
│   ├── spec.json                 # Pinned OpenAPI spec (checksummed)
│   └── checksum.sha256
├── src/
│   ├── lib.rs                    # Public API re-exports, feature gates
│   ├── config.rs                 # RestConfig, WsConfig, base URLs
│   ├── auth.rs                   # ApiKey, AuthMode, env loading
│   ├── error.rs                  # MassiveError, WsError, ApiErrorResponse
│   ├── util.rs                   # UnixMs, URL builders, query serialization
│   │
│   ├── rest/
│   │   ├── mod.rs
│   │   ├── client.rs             # RestClient, connection pooling
│   │   ├── request.rs            # RestRequest trait, RequestBuilder
│   │   ├── pagination.rs         # PaginationMode, PageStream
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── envelope.rs       # ApiEnvelope<T>, ListEnvelope<T>
│   │       └── generated/        # Spec-generated models (via build.rs)
│   │
│   ├── ws/
│   │   ├── mod.rs
│   │   ├── client.rs             # WsClient, WsHandle, connection lifecycle
│   │   ├── protocol.rs           # WsAuthMsg, WsSubMsg, control messages
│   │   ├── reconnect.rs          # ReconnectConfig, exponential backoff
│   │   ├── dispatch.rs           # DispatchConfig, OverflowPolicy, fanout
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── events.rs         # WsEvent enum, event types
│   │       └── messages.rs       # WsTrade, WsQuote, WsAggregate, etc.
│   │
│   └── models/
│       ├── mod.rs
│       ├── common.rs             # AggregateBar, Trade, Quote
│       ├── stocks.rs             # Stock-specific models
│       ├── options.rs            # Options-specific models
│       ├── forex.rs              # Forex models
│       ├── crypto.rs             # Crypto models
│       ├── futures.rs            # Futures models
│       ├── indices.rs            # Index models
│       ├── reference.rs          # Ticker, Exchange, Dividend, Split
│       └── snapshots.rs          # Snapshot response models
│
├── examples/
│   ├── rest_aggregates.rs
│   ├── rest_trades.rs
│   ├── ws_realtime_trades.rs
│   ├── ws_minute_bars.rs
│   └── hft_market_data.rs
│
├── tests/
│   ├── integration/
│   │   ├── rest_client_tests.rs
│   │   └── ws_client_tests.rs
│   ├── fixtures/                 # Golden JSON test files
│   └── mocks/                    # wiremock server setup
│
└── benches/
    ├── json_parsing.rs
    ├── ws_throughput.rs
    └── backpressure.rs
```

### 1.3 Data Flow Diagrams

#### REST Client Data Flow

```
User Code                    RestClient                   Massive API
    │                            │                             │
    │  client.get_aggs(...)      │                             │
    ├───────────────────────────►│                             │
    │                            │  HTTP GET /v2/aggs/...      │
    │                            ├────────────────────────────►│
    │                            │                             │
    │                            │  200 OK + JSON body         │
    │                            │◄────────────────────────────┤
    │                            │                             │
    │                            │  [Parse ApiEnvelope<T>]     │
    │                            │  [Check next_url]           │
    │                            │                             │
    │                            │  HTTP GET {next_url}        │
    │                            ├────────────────────────────►│
    │                            │        ...repeats...        │
    │                            │◄────────────────────────────┤
    │                            │                             │
    │  Stream<Item=AggBar>       │                             │
    │◄───────────────────────────┤                             │
```

#### WebSocket Client Data Flow

```
User Code          WsClient           IO Task          Massive WS
    │                  │                  │                  │
    │ client.connect() │                  │                  │
    ├─────────────────►│                  │                  │
    │                  │  spawn IO task   │                  │
    │                  ├─────────────────►│                  │
    │                  │                  │  WS Connect      │
    │                  │                  ├─────────────────►│
    │                  │                  │  {"ev":"status"} │
    │                  │                  │◄─────────────────┤
    │                  │                  │  auth msg        │
    │                  │                  ├─────────────────►│
    │                  │                  │  auth_success    │
    │                  │                  │◄─────────────────┤
    │ (WsHandle,       │                  │                  │
    │  WsEventStream)  │                  │                  │
    │◄─────────────────┤                  │                  │
    │                  │                  │                  │
    │ handle.subscribe │                  │                  │
    ├─────────────────►├─────────────────►│  subscribe msg   │
    │                  │                  ├─────────────────►│
    │                  │                  │                  │
    │                  │                  │  [T.AAPL, ...]   │
    │                  │                  │◄─────────────────┤
    │                  │  ring buffer/    │                  │
    │                  │  channel push    │                  │
    │◄─────────────────┼──────────────────┤                  │
    │  WsEvent items   │                  │                  │
```

---

## 2. Technology Stack

### 2.1 Core Dependencies

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `tokio` | 1.x | Async runtime | Industry standard, mature, multi-threaded |
| `reqwest` | 0.12.x | HTTP client | Connection pooling, gzip, rustls support |
| `tokio-tungstenite` | 0.24.x | WebSocket client | Native tokio integration, mature |
| `serde` | 1.x | Serialization | De-facto standard |
| `serde_json` | 1.x | JSON parsing | Standard JSON handling |
| `thiserror` | 2.x | Error derivation | Ergonomic error types |
| `bytes` | 1.x | Byte buffers | Zero-copy buffer management |
| `smol_str` | 0.3.x | Small strings | Reduce alloc for symbols |
| `dashmap` | 6.x | Concurrent maps | Lock-free subscription tracking |
| `futures` | 0.3.x | Stream/Future combinators | Pagination streams |
| `tracing` | 0.1.x | Observability | Structured logging, spans |

### 2.2 Optional Dependencies (Feature-Gated)

| Crate | Feature | Purpose |
|-------|---------|---------|
| `simd-json` | `simd-json` | High-performance JSON parsing |
| `native-tls` | `native-tls` | OS-native TLS (alternative to rustls) |
| `rust_decimal` | `decimal` | Exact decimal arithmetic for prices |
| `tokio-rustls` | `rustls` (default) | Modern TLS implementation |

### 2.3 Development Dependencies

| Crate | Purpose |
|-------|---------|
| `wiremock` | HTTP mocking for integration tests |
| `criterion` | Benchmarking framework |
| `proptest` | Property-based testing |
| `tokio-test` | Async test utilities |
| `tracing-subscriber` | Test logging |

### 2.4 Feature Flags

```toml
[features]
default = ["rustls", "gzip", "ws"]

# TLS backends (mutually exclusive in practice)
rustls = ["reqwest/rustls-tls", "tokio-tungstenite/rustls-tls-native-roots"]
native-tls = ["reqwest/native-tls", "tokio-tungstenite/native-tls"]

# Compression
gzip = ["reqwest/gzip"]

# WebSocket support
ws = ["tokio-tungstenite", "dashmap"]

# Performance optimizations
simd-json = ["dep:simd-json"]

# Precision numerics
decimal = ["rust_decimal"]

# Code generation (for maintainers)
codegen = ["dep:openapitor", "dep:heck"]

# Blocking API (for non-async contexts)
blocking = ["tokio/rt-multi-thread"]
```

---

## 3. Phase 1: Foundation & Core Infrastructure

### 3.1 Objectives

- Establish project structure and build configuration
- Implement core types: configuration, authentication, errors
- Create utility types and helpers
- Set up CI/CD pipeline and testing infrastructure

### 3.2 Deliverables

#### 3.2.1 Project Initialization

```toml
# Cargo.toml
[package]
name = "massive-rs"
version = "0.1.0"
edition = "2021"
rust-version = "1.75.0"
license = "MIT OR Apache-2.0"
description = "High-performance Rust client for Massive.com market data APIs"
repository = "https://github.com/your-org/massive-rs"
keywords = ["trading", "market-data", "hft", "websocket", "financial"]
categories = ["api-bindings", "asynchronous", "finance"]

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

#### 3.2.2 Configuration Module (`src/config.rs`)

```rust
use std::time::Duration;
use url::Url;

/// REST API client configuration
#[derive(Debug, Clone)]
pub struct RestConfig {
    /// Base URL for REST API (default: https://api.massive.com)
    pub base_url: Url,

    /// API key for authentication
    pub api_key: ApiKey,

    /// Authentication mode (header vs query param)
    pub auth_mode: AuthMode,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Request timeout (per request)
    pub request_timeout: Duration,

    /// Pagination behavior
    pub pagination: PaginationMode,

    /// Enable debug tracing
    pub trace: bool,

    /// Custom User-Agent string
    pub user_agent: Option<String>,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("https://api.massive.com").unwrap(),
            api_key: ApiKey::from_env().unwrap_or_default(),
            auth_mode: AuthMode::HeaderBearer,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            pagination: PaginationMode::Auto,
            trace: false,
            user_agent: None,
        }
    }
}

/// WebSocket client configuration
#[derive(Debug, Clone)]
pub struct WsConfig {
    /// Feed type (real-time vs delayed)
    pub feed: Feed,

    /// Market/asset class
    pub market: Market,

    /// API key for authentication
    pub api_key: ApiKey,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Idle timeout before ping
    pub idle_timeout: Duration,

    /// Ping interval for keepalive
    pub ping_interval: Duration,

    /// Reconnection configuration
    pub reconnect: ReconnectConfig,

    /// Dispatch/backpressure configuration
    pub dispatch: DispatchConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feed {
    /// Real-time data: wss://socket.massive.com
    RealTime,
    /// 15-minute delayed: wss://delayed.massive.com
    Delayed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    Stocks,
    Options,
    Futures,
    Indices,
    Forex,
    Crypto,
}

impl Market {
    pub fn as_path(&self) -> &'static str {
        match self {
            Market::Stocks => "stocks",
            Market::Options => "options",
            Market::Futures => "futures",
            Market::Indices => "indices",
            Market::Forex => "forex",
            Market::Crypto => "crypto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationMode {
    /// Automatically fetch all pages
    Auto,
    /// Single page only
    None,
    /// Stop after N total items
    MaxItems(u64),
}
```

#### 3.2.3 Authentication Module (`src/auth.rs`)

```rust
use secrecy::{ExposeSecret, Secret};
use std::env;

/// Wrapper for API key with secure handling
#[derive(Clone)]
pub struct ApiKey(Secret<String>);

impl ApiKey {
    /// Create from string (takes ownership, prevents logging)
    pub fn new(key: impl Into<String>) -> Self {
        Self(Secret::new(key.into()))
    }

    /// Load from environment variable
    pub fn from_env() -> Option<Self> {
        env::var("MASSIVE_API_KEY")
            .ok()
            .map(ApiKey::new)
    }

    /// Load from env with custom variable name
    pub fn from_env_var(var_name: &str) -> Option<Self> {
        env::var(var_name).ok().map(ApiKey::new)
    }

    /// Expose the key for use in requests (internal only)
    pub(crate) fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl Default for ApiKey {
    fn default() -> Self {
        Self::new("")
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

/// Authentication mode for API requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// Use Authorization: Bearer header (recommended)
    #[default]
    HeaderBearer,
    /// Use apiKey query parameter
    QueryParam,
}
```

#### 3.2.4 Error Module (`src/error.rs`)

```rust
use bytes::Bytes;
use std::time::Duration;
use thiserror::Error;

/// Unified error type for all Massive operations
#[derive(Debug, Error)]
pub enum MassiveError {
    /// HTTP transport error
    #[error("Transport error: {0}")]
    Transport(#[from] reqwest::Error),

    /// Request timed out
    #[error("Request timed out")]
    Timeout,

    /// HTTP error with status code
    #[error("HTTP {status}: {}", body_preview(.body))]
    HttpStatus {
        status: u16,
        body: Bytes,
        request_id: Option<String>,
    },

    /// Parsed API error response
    #[error("API error: {0}")]
    Api(ApiErrorResponse),

    /// JSON deserialization failed
    #[error("Deserialization error: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        body_snippet: String,
    },

    /// Invalid argument provided
    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),

    /// Rate limit exceeded
    #[error("Rate limited{}", format_retry(.retry_after))]
    RateLimited {
        retry_after: Option<Duration>,
        request_id: Option<String>,
    },

    /// WebSocket-specific error
    #[error("WebSocket error: {0}")]
    Ws(#[from] WsError),

    /// Connection closed
    #[error("Connection closed")]
    Closed,

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Auth(String),
}

/// WebSocket-specific errors
#[derive(Debug, Error)]
pub enum WsError {
    #[error("WebSocket connection error: {0}")]
    Connection(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Authentication handshake failed: {0}")]
    AuthFailed(String),

    #[error("Protocol violation: {0}")]
    Protocol(String),

    #[error("Server disconnected")]
    Disconnected,

    #[error("Backpressure overflow (buffer full)")]
    BackpressureOverflow,

    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),
}

/// Parsed error response from Massive API
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiErrorResponse {
    pub status: String,
    pub error: Option<String>,
    pub message: Option<String>,
    pub request_id: Option<String>,
}

impl std::fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error.as_deref()
            .or(self.message.as_deref())
            .unwrap_or(&self.status))
    }
}

impl std::error::Error for ApiErrorResponse {}

// Helper functions for error formatting
fn body_preview(body: &Bytes) -> String {
    let s = String::from_utf8_lossy(body);
    if s.len() > 200 {
        format!("{}...", &s[..200])
    } else {
        s.to_string()
    }
}

fn format_retry(retry_after: &Option<Duration>) -> String {
    match retry_after {
        Some(d) => format!(", retry after {:?}", d),
        None => String::new(),
    }
}
```

#### 3.2.5 Utility Module (`src/util.rs`)

```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;

/// Unix timestamp in milliseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UnixMs(pub i64);

impl UnixMs {
    pub fn now() -> Self {
        Self(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64)
    }

    pub fn as_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp_millis(self.0)
            .unwrap_or_default()
    }
}

impl Serialize for UnixMs {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for UnixMs {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        i64::deserialize(d).map(UnixMs)
    }
}

/// Unix timestamp in nanoseconds (for SIP timestamps)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UnixNs(pub i64);

/// Type alias for ticker symbols using small-string optimization
pub type Symbol = SmolStr;

/// Create a Symbol from a string slice
pub fn symbol(s: &str) -> Symbol {
    SmolStr::new(s)
}

/// Query parameter builder for REST requests
#[derive(Debug, Default)]
pub struct QueryParams(Vec<(String, String)>);

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, key: impl Into<String>, value: impl ToString) {
        self.0.push((key.into(), value.to_string()));
    }

    pub fn push_opt<T: ToString>(&mut self, key: impl Into<String>, value: Option<T>) {
        if let Some(v) = value {
            self.push(key, v);
        }
    }

    pub fn into_pairs(self) -> Vec<(String, String)> {
        self.0
    }
}
```

### 3.3 Acceptance Criteria - Phase 1

- [ ] Project compiles with `cargo build --all-features`
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt --check` passes
- [ ] All configuration types have `Debug`, `Clone` implementations
- [ ] `ApiKey` never appears in debug output (redacted)
- [ ] `MassiveError` implements `std::error::Error` properly
- [ ] Unit tests pass: `cargo test --lib`
- [ ] Documentation builds: `cargo doc --no-deps`

### 3.4 Milestone Checklist

- [ ] M1.1: Repository initialized with Cargo.toml, CI workflow
- [ ] M1.2: `config` module complete with all configuration types
- [ ] M1.3: `auth` module complete with secure API key handling
- [ ] M1.4: `error` module complete with all error variants
- [ ] M1.5: `util` module complete with timestamp types
- [ ] M1.6: GitHub Actions CI pipeline operational

---

## 4. Phase 2: REST Client Implementation

### 4.1 Objectives

- Implement `RestClient` with connection pooling
- Create `RestRequest` trait for typed endpoints
- Implement pagination (Auto, None, MaxItems modes)
- Handle response envelope parsing
- Implement retry logic with exponential backoff
- Add gzip decompression support

### 4.2 Deliverables

#### 4.2.1 REST Client (`src/rest/client.rs`)

```rust
use crate::{auth::*, config::*, error::*, util::*};
use bytes::Bytes;
use reqwest::{Client, Method, Response};
use std::sync::Arc;
use tracing::{debug, instrument, warn};

/// REST API client for Massive.com
#[derive(Clone)]
pub struct RestClient {
    inner: Arc<RestClientInner>,
}

struct RestClientInner {
    http: Client,
    config: RestConfig,
}

impl RestClient {
    /// Create a new REST client with the given configuration
    pub fn new(config: RestConfig) -> Result<Self, MassiveError> {
        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90));

        #[cfg(feature = "gzip")]
        {
            builder = builder.gzip(true);
        }

        if let Some(ua) = &config.user_agent {
            builder = builder.user_agent(ua);
        } else {
            builder = builder.user_agent(format!(
                "massive-rs/{}",
                env!("CARGO_PKG_VERSION")
            ));
        }

        let http = builder.build()?;

        Ok(Self {
            inner: Arc::new(RestClientInner { http, config }),
        })
    }

    /// Create client from API key only (uses defaults)
    pub fn from_api_key(key: impl Into<String>) -> Result<Self, MassiveError> {
        let config = RestConfig {
            api_key: ApiKey::new(key),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Execute a typed request
    #[instrument(skip(self, req), fields(path = %req.path()))]
    pub async fn execute<R>(&self, req: R) -> Result<R::Response, MassiveError>
    where
        R: RestRequest,
    {
        let url = self.build_url(&req)?;
        let method = req.method();

        let mut request = self.inner.http.request(method.clone(), url);

        // Apply authentication
        request = self.apply_auth(request);

        // Apply body if present
        if let Some(body) = req.body() {
            request = request
                .header("Content-Type", "application/json")
                .body(body);
        }

        // Execute with retry logic
        let response = self.execute_with_retry(request, req.idempotent()).await?;

        // Parse response
        self.parse_response::<R::Response>(response).await
    }

    /// Stream paginated results
    pub fn stream<R>(&self, req: R) -> impl futures::Stream<Item = Result<R::Item, MassiveError>>
    where
        R: PaginatableRequest,
    {
        PageStream::new(self.clone(), req, self.inner.config.pagination)
    }

    fn build_url<R: RestRequest>(&self, req: &R) -> Result<url::Url, MassiveError> {
        let mut url = self.inner.config.base_url.clone();
        url.set_path(&req.path());

        let query = req.query();
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }

        Ok(url)
    }

    /// Fetch a URL directly (used for pagination next_url)
    ///
    /// The `next_url` from Massive API responses is a complete URL including
    /// the API key as a query parameter, so we fetch it directly.
    pub(crate) async fn fetch_url<T>(&self, url: &str) -> Result<T, MassiveError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut request = self.inner.http.get(url);

        // next_url typically includes apiKey, but add auth header for safety
        if matches!(self.inner.config.auth_mode, AuthMode::HeaderBearer) {
            request = request.header(
                "Authorization",
                format!("Bearer {}", self.inner.config.api_key.expose())
            );
        }

        let response = self.execute_with_retry(request, true).await?;
        self.parse_response(response).await
    }

    fn apply_auth(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.inner.config.auth_mode {
            AuthMode::HeaderBearer => {
                request = request.header(
                    "Authorization",
                    format!("Bearer {}", self.inner.config.api_key.expose())
                );
            }
            AuthMode::QueryParam => {
                // Query param auth is added in build_url via query()
            }
        }
        request
    }

    async fn execute_with_retry(
        &self,
        request: reqwest::RequestBuilder,
        idempotent: bool,
    ) -> Result<Response, MassiveError> {
        let mut attempts = 0;
        let max_attempts = if idempotent { 3 } else { 1 };

        loop {
            attempts += 1;

            // Clone request for retry (reqwest doesn't allow reuse)
            let req = request.try_clone()
                .ok_or(MassiveError::InvalidArgument("Request body not cloneable"))?;

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();

                    // Handle rate limiting
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = resp.headers()
                            .get("Retry-After")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(std::time::Duration::from_secs);

                        return Err(MassiveError::RateLimited {
                            retry_after,
                            request_id: extract_request_id(&resp),
                        });
                    }

                    // Retry on 502/503/504 if idempotent
                    if idempotent && attempts < max_attempts
                        && matches!(status.as_u16(), 502 | 503 | 504)
                    {
                        warn!(status = %status, attempt = attempts, "Retrying request");
                        tokio::time::sleep(backoff_delay(attempts)).await;
                        continue;
                    }

                    return Ok(resp);
                }
                Err(e) if idempotent && attempts < max_attempts && e.is_connect() => {
                    warn!(error = %e, attempt = attempts, "Connection error, retrying");
                    tokio::time::sleep(backoff_delay(attempts)).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    async fn parse_response<T>(&self, response: Response) -> Result<T, MassiveError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let request_id = extract_request_id(&response);

        let bytes = response.bytes().await?;

        if !status.is_success() {
            // Try to parse as API error
            if let Ok(api_error) = serde_json::from_slice::<ApiErrorResponse>(&bytes) {
                return Err(MassiveError::Api(api_error));
            }

            return Err(MassiveError::HttpStatus {
                status: status.as_u16(),
                body: bytes,
                request_id,
            });
        }

        serde_json::from_slice(&bytes).map_err(|e| MassiveError::Deserialize {
            source: e,
            body_snippet: String::from_utf8_lossy(&bytes[..bytes.len().min(500)]).to_string(),
        })
    }
}

fn extract_request_id(response: &Response) -> Option<String> {
    response.headers()
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

fn backoff_delay(attempt: u32) -> std::time::Duration {
    std::time::Duration::from_millis(100 * 2u64.pow(attempt - 1))
}
```

#### 4.2.2 Request Trait (`src/rest/request.rs`)

```rust
use bytes::Bytes;
use reqwest::Method;
use serde::de::DeserializeOwned;
use std::borrow::Cow;

/// Trait for typed REST API requests
pub trait RestRequest: Send + Sync {
    /// Response type for this request
    type Response: DeserializeOwned + Send + 'static;

    /// HTTP method
    fn method(&self) -> Method;

    /// Request path (with path parameters interpolated)
    fn path(&self) -> Cow<'static, str>;

    /// Query parameters
    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        Vec::new()
    }

    /// Request body (JSON)
    fn body(&self) -> Option<Bytes> {
        None
    }

    /// Whether this request is idempotent (safe to retry)
    fn idempotent(&self) -> bool {
        matches!(self.method(), Method::GET | Method::HEAD | Method::OPTIONS)
    }
}

/// Trait for requests that support pagination
pub trait PaginatableRequest: RestRequest {
    /// Item type yielded by pagination
    type Item: DeserializeOwned + Send + 'static;

    /// Extract items from response
    fn extract_items(response: Self::Response) -> Vec<Self::Item>;

    /// Extract next URL from response (if any)
    fn extract_next_url(response: &Self::Response) -> Option<&str>;
}
```

#### 4.2.3 Pagination Module (`src/rest/pagination.rs`)

The pagination system handles the `next_url` response attribute that Massive API returns for paginated endpoints. The flow is:

1. Make initial request to endpoint
2. Parse response, extract `results` array and `next_url`
3. If `next_url` is present and pagination mode allows, fetch next page
4. Repeat until `next_url` is absent or pagination limit reached

```rust
use crate::{config::PaginationMode, error::MassiveError, rest::*};
use futures::Stream;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Stream adapter for paginated requests that handles `next_url` chaining
///
/// # How next_url Works
///
/// Massive API returns paginated responses like:
/// ```json
/// {
///   "status": "OK",
///   "results": [...],
///   "next_url": "https://api.massive.com/v2/aggs/...?cursor=abc123&apiKey=***"
/// }
/// ```
///
/// The `next_url` is a complete URL that includes:
/// - The same endpoint path
/// - A cursor/pagination token
/// - The API key (as query param)
///
/// This stream fetches each page via `next_url` until exhausted.
pub struct PageStream<R: PaginatableRequest> {
    client: RestClient,
    /// Initial request (only used for first page)
    initial_request: Option<R>,
    /// URL for next page (from previous response's next_url field)
    next_url: Option<String>,
    /// Pagination mode controlling when to stop
    mode: PaginationMode,
    /// Count of items yielded so far
    items_yielded: u64,
    /// Buffer of items from current page
    buffer: VecDeque<R::Item>,
    /// Current async operation in progress
    in_flight: Option<Pin<Box<dyn std::future::Future<Output = Result<R::Response, MassiveError>> + Send>>>,
    /// Whether we've completed pagination
    done: bool,
}

impl<R: PaginatableRequest + Clone + Send + 'static> PageStream<R> {
    pub fn new(client: RestClient, request: R, mode: PaginationMode) -> Self {
        Self {
            client,
            initial_request: Some(request),
            next_url: None,
            mode,
            items_yielded: 0,
            buffer: VecDeque::new(),
            in_flight: None,
            done: false,
        }
    }

    /// Check if we should continue fetching based on pagination mode
    fn should_continue(&self) -> bool {
        match self.mode {
            PaginationMode::Auto => true,
            PaginationMode::None => self.items_yielded == 0, // Only first page
            PaginationMode::MaxItems(max) => self.items_yielded < max,
        }
    }

    /// Check if we've hit the item limit
    fn at_item_limit(&self) -> bool {
        matches!(self.mode, PaginationMode::MaxItems(max) if self.items_yielded >= max)
    }
}

impl<R> Stream for PageStream<R>
where
    R: PaginatableRequest + Clone + Unpin + Send + 'static,
    R::Response: Send,
{
    type Item = Result<R::Item, MassiveError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // 1. Yield buffered items first (fast path)
            if let Some(item) = this.buffer.pop_front() {
                this.items_yielded += 1;

                // Stop yielding if we hit MaxItems limit
                if this.at_item_limit() {
                    this.done = true;
                }

                return Poll::Ready(Some(Ok(item)));
            }

            // 2. Check if we're done
            if this.done {
                return Poll::Ready(None);
            }

            // 3. Check if there's an in-flight request
            if let Some(ref mut fut) = this.in_flight {
                match fut.as_mut().poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(e)) => {
                        this.done = true;
                        this.in_flight = None;
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Ready(Ok(response)) => {
                        // Extract next_url BEFORE consuming response
                        this.next_url = R::extract_next_url(&response).map(String::from);

                        // Extract items into buffer
                        let items = R::extract_items(response);
                        this.buffer.extend(items);

                        this.in_flight = None;

                        // If no items and no next_url, we're done
                        if this.buffer.is_empty() && this.next_url.is_none() {
                            this.done = true;
                        }

                        // Continue loop to yield from buffer
                        continue;
                    }
                }
            }

            // 4. No in-flight request, decide what to fetch next
            if !this.should_continue() {
                this.done = true;
                continue;
            }

            // 5. Fetch next page via next_url, or initial request
            if let Some(url) = this.next_url.take() {
                // Subsequent pages: fetch via next_url directly
                // The next_url is a complete URL from Massive API
                let client = this.client.clone();
                this.in_flight = Some(Box::pin(async move {
                    client.fetch_url::<R::Response>(&url).await
                }));
            } else if let Some(req) = this.initial_request.take() {
                // First page: execute the initial request
                let client = this.client.clone();
                this.in_flight = Some(Box::pin(async move {
                    client.execute(req).await
                }));
            } else {
                // No next_url and no initial request = done
                this.done = true;
                continue;
            }
        }
    }
}

/// Convenience extension for collecting all pages
impl<R> PageStream<R>
where
    R: PaginatableRequest + Clone + Unpin + Send + 'static,
    R::Response: Send,
{
    /// Collect all items from all pages into a Vec
    ///
    /// # Warning
    /// This loads all data into memory. For large result sets,
    /// prefer using the stream directly.
    pub async fn collect_all(self) -> Result<Vec<R::Item>, MassiveError> {
        use futures::TryStreamExt;
        self.try_collect().await
    }
}
```

**Key Implementation Notes:**

1. **`next_url` is a complete URL**: Massive API returns the full URL including query params and API key, so we fetch it directly via `fetch_url()`

2. **Three pagination modes**:
   - `Auto`: Follow `next_url` until it's absent (fetches everything)
   - `None`: Only fetch first page, ignore `next_url`
   - `MaxItems(n)`: Stop after yielding `n` items, even if more pages exist

3. **Non-blocking buffer**: Items are buffered per-page and yielded one at a time, allowing the stream consumer to process incrementally

4. **Error propagation**: If any page fetch fails, the error is yielded and the stream terminates

#### 4.2.4 Response Envelope (`src/rest/models/envelope.rs`)

```rust
use serde::Deserialize;

/// Standard API response envelope
#[derive(Debug, Clone, Deserialize)]
pub struct ApiEnvelope<T> {
    pub status: Option<String>,
    pub count: Option<u64>,
    pub results: Option<T>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiEnvelope<T> {
    /// Unwrap results or return default
    pub fn into_results(self) -> T
    where
        T: Default,
    {
        self.results.unwrap_or_default()
    }
}

/// List endpoint envelope with Vec results
#[derive(Debug, Clone, Deserialize)]
pub struct ListEnvelope<T> {
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub count: Option<u64>,
    pub next_url: Option<String>,
    #[serde(default)]
    pub results: Vec<T>,
}
```

### 4.3 Example Endpoint Implementations

```rust
// src/rest/endpoints/aggregates.rs

use crate::rest::*;
use crate::models::common::AggregateBar;
use reqwest::Method;
use serde::Deserialize;

/// Request for aggregate bars (OHLCV)
#[derive(Debug, Clone)]
pub struct GetAggsRequest {
    pub ticker: String,
    pub multiplier: u32,
    pub timespan: Timespan,
    pub from: String,
    pub to: String,
    pub adjusted: Option<bool>,
    pub sort: Option<SortOrder>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum Timespan {
    Second, Minute, Hour, Day, Week, Month, Quarter, Year,
}

impl Timespan {
    fn as_str(&self) -> &'static str {
        match self {
            Timespan::Second => "second",
            Timespan::Minute => "minute",
            Timespan::Hour => "hour",
            Timespan::Day => "day",
            Timespan::Week => "week",
            Timespan::Month => "month",
            Timespan::Quarter => "quarter",
            Timespan::Year => "year",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AggsResponse {
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub ticker: Option<String>,
    pub query_count: Option<u64>,
    pub results_count: Option<u64>,
    pub adjusted: Option<bool>,
    #[serde(default)]
    pub results: Vec<AggregateBar>,
    pub next_url: Option<String>,
}

impl RestRequest for GetAggsRequest {
    type Response = AggsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> std::borrow::Cow<'static, str> {
        format!(
            "/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
            self.ticker,
            self.multiplier,
            self.timespan.as_str(),
            self.from,
            self.to
        ).into()
    }

    fn query(&self) -> Vec<(std::borrow::Cow<'static, str>, String)> {
        let mut params = Vec::new();

        if let Some(adj) = self.adjusted {
            params.push(("adjusted".into(), adj.to_string()));
        }
        if let Some(sort) = &self.sort {
            params.push(("sort".into(), sort.as_str().to_string()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".into(), limit.to_string()));
        }

        params
    }
}

impl PaginatableRequest for GetAggsRequest {
    type Item = AggregateBar;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// Ergonomic builder on RestClient
impl RestClient {
    /// Get aggregate bars for a ticker
    pub async fn get_aggs(
        &self,
        ticker: &str,
        multiplier: u32,
        timespan: Timespan,
        from: &str,
        to: &str,
    ) -> Result<Vec<AggregateBar>, MassiveError> {
        let req = GetAggsRequest {
            ticker: ticker.to_string(),
            multiplier,
            timespan,
            from: from.to_string(),
            to: to.to_string(),
            adjusted: Some(true),
            sort: None,
            limit: None,
        };

        let response = self.execute(req).await?;
        Ok(response.results)
    }

    /// Stream aggregate bars with automatic pagination
    pub fn list_aggs(
        &self,
        ticker: &str,
        multiplier: u32,
        timespan: Timespan,
        from: &str,
        to: &str,
    ) -> impl futures::Stream<Item = Result<AggregateBar, MassiveError>> {
        let req = GetAggsRequest {
            ticker: ticker.to_string(),
            multiplier,
            timespan,
            from: from.to_string(),
            to: to.to_string(),
            adjusted: Some(true),
            sort: None,
            limit: Some(50000), // Max page size
        };

        self.stream(req)
    }
}
```

### 4.4 Acceptance Criteria - Phase 2

- [ ] `RestClient::new()` creates client with connection pooling
- [ ] `RestClient::execute()` handles all HTTP methods
- [ ] Authentication works via both header and query param modes
- [ ] Pagination modes work correctly:
  - [ ] `Auto` fetches all pages
  - [ ] `None` returns single page
  - [ ] `MaxItems(n)` stops after n items
- [ ] Retry logic triggers on 502/503/504 for idempotent requests
- [ ] Rate limiting returns `MassiveError::RateLimited` with `Retry-After`
- [ ] Gzip decompression works when feature enabled
- [ ] `X-Request-Id` captured in error responses
- [ ] All endpoint responses deserialize correctly

### 4.5 Milestone Checklist

- [ ] M2.1: `RestClient` with connection pooling
- [ ] M2.2: `RestRequest` trait and builder pattern
- [ ] M2.3: Pagination implementation (all 3 modes)
- [ ] M2.4: Response envelope parsing
- [ ] M2.5: Retry logic with exponential backoff
- [ ] M2.6: Error handling with request ID extraction
- [ ] M2.7: Integration tests with wiremock

---

## 5. Phase 3: WebSocket Client Implementation

### 5.1 Objectives

- Implement `WsClient` with connection lifecycle management
- Handle authentication handshake
- Implement subscription management
- Parse batched JSON messages (single and array)
- Implement backpressure handling with configurable policies
- Add automatic reconnection with exponential backoff

### 5.2 Deliverables

#### 5.2.1 WebSocket Client (`src/ws/client.rs`)

```rust
use crate::{auth::ApiKey, config::*, error::*, ws::*};
use dashmap::DashSet;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, instrument, warn};

/// WebSocket client for Massive.com streaming data
pub struct WsClient {
    config: WsConfig,
}

/// Handle for managing an active WebSocket connection
pub struct WsHandle {
    cmd_tx: mpsc::Sender<WsCommand>,
    state: Arc<WsState>,
}

/// Shared state for WebSocket connection
pub struct WsState {
    pub authenticated: std::sync::atomic::AtomicBool,
    pub subscriptions: DashSet<Subscription>,
    pub last_message: std::sync::atomic::AtomicU64,
}

/// Stream of WebSocket events
pub type WsEventStream = std::pin::Pin<
    Box<dyn futures::Stream<Item = Result<WsMessageBatch, MassiveError>> + Send>
>;

/// Batch of WebSocket messages (may contain 1 or more events)
#[derive(Debug, Clone)]
pub struct WsMessageBatch {
    pub events: Vec<WsEvent>,
    pub received_at: std::time::Instant,
}

/// Commands sent to the WebSocket IO task
enum WsCommand {
    Subscribe(Vec<Subscription>, oneshot::Sender<Result<(), MassiveError>>),
    Unsubscribe(Vec<Subscription>, oneshot::Sender<Result<(), MassiveError>>),
    Close(oneshot::Sender<()>),
}

impl WsClient {
    /// Create a new WebSocket client
    pub fn new(config: WsConfig) -> Result<Self, MassiveError> {
        Ok(Self { config })
    }

    /// Builder for WebSocket client
    pub fn builder() -> WsClientBuilder {
        WsClientBuilder::default()
    }

    /// Connect to the WebSocket server
    #[instrument(skip(self))]
    pub async fn connect(&self) -> Result<(WsHandle, WsEventStream), MassiveError> {
        let url = self.build_url();
        info!(url = %url, "Connecting to WebSocket");

        // Establish connection
        let (ws_stream, _response) = connect_async(&url).await
            .map_err(WsError::Connection)?;

        let (write, read) = ws_stream.split();

        // Create channels
        let (cmd_tx, cmd_rx) = mpsc::channel::<WsCommand>(32);
        let (event_tx, event_rx) = self.create_event_channel();

        // Create shared state
        let state = Arc::new(WsState {
            authenticated: std::sync::atomic::AtomicBool::new(false),
            subscriptions: DashSet::new(),
            last_message: std::sync::atomic::AtomicU64::new(0),
        });

        // Spawn IO task
        let io_state = state.clone();
        let api_key = self.config.api_key.clone();
        let reconnect_config = self.config.reconnect.clone();

        tokio::spawn(async move {
            run_io_task(
                write,
                read,
                cmd_rx,
                event_tx,
                io_state,
                api_key,
                reconnect_config,
            ).await;
        });

        // Create handle
        let handle = WsHandle { cmd_tx, state };

        // Wait for authentication
        handle.wait_for_auth().await?;

        // Create event stream
        let stream = Box::pin(futures::stream::unfold(event_rx, |mut rx| async move {
            rx.recv().await.map(|batch| (batch, rx))
        }));

        Ok((handle, stream))
    }

    fn build_url(&self) -> String {
        let host = match self.config.feed {
            Feed::RealTime => "socket.massive.com",
            Feed::Delayed => "delayed.massive.com",
        };
        format!("wss://{}/{}", host, self.config.market.as_path())
    }

    fn create_event_channel(&self) -> (mpsc::Sender<WsMessageBatch>, mpsc::Receiver<WsMessageBatch>) {
        mpsc::channel(self.config.dispatch.capacity)
    }
}

impl WsHandle {
    /// Subscribe to topics
    pub async fn subscribe(&self, topics: &[Subscription]) -> Result<(), MassiveError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx.send(WsCommand::Subscribe(topics.to_vec(), tx)).await
            .map_err(|_| MassiveError::Closed)?;
        rx.await.map_err(|_| MassiveError::Closed)?
    }

    /// Unsubscribe from topics
    pub async fn unsubscribe(&self, topics: &[Subscription]) -> Result<(), MassiveError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx.send(WsCommand::Unsubscribe(topics.to_vec(), tx)).await
            .map_err(|_| MassiveError::Closed)?;
        rx.await.map_err(|_| MassiveError::Closed)?
    }

    /// Close the connection gracefully
    pub async fn close(&self) -> Result<(), MassiveError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(WsCommand::Close(tx)).await;
        let _ = rx.await;
        Ok(())
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.state.authenticated.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Get current subscriptions
    pub fn subscriptions(&self) -> Vec<Subscription> {
        self.state.subscriptions.iter().map(|s| s.clone()).collect()
    }

    async fn wait_for_auth(&self) -> Result<(), MassiveError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(10);

        while start.elapsed() < timeout {
            if self.is_authenticated() {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Err(MassiveError::Ws(WsError::AuthFailed("Timeout waiting for auth".into())))
    }
}

/// IO task that handles WebSocket read/write
async fn run_io_task(
    mut write: impl futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
    mut read: impl futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    mut cmd_rx: mpsc::Receiver<WsCommand>,
    event_tx: mpsc::Sender<WsMessageBatch>,
    state: Arc<WsState>,
    api_key: ApiKey,
    _reconnect: ReconnectConfig,
) {
    // Send authentication
    let auth_msg = WsAuthMessage {
        action: "auth".to_string(),
        params: api_key.expose().to_string(),
    };

    if let Err(e) = write.send(Message::Text(serde_json::to_string(&auth_msg).unwrap())).await {
        error!(error = %e, "Failed to send auth message");
        return;
    }

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let received_at = std::time::Instant::now();
                        state.last_message.store(
                            received_at.elapsed().as_millis() as u64,
                            std::sync::atomic::Ordering::Release
                        );

                        match parse_ws_message(&text) {
                            Ok(events) => {
                                // Check for auth success
                                for event in &events {
                                    if let WsEvent::Status(status) = event {
                                        if status.status == "auth_success" {
                                            state.authenticated.store(true, std::sync::atomic::Ordering::Release);
                                            info!("WebSocket authenticated");
                                        }
                                    }
                                }

                                let batch = WsMessageBatch { events, received_at };

                                // Non-blocking send with backpressure handling
                                if event_tx.try_send(batch).is_err() {
                                    warn!("Event buffer full, dropping message");
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to parse WebSocket message");
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = write.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket connection closed");
                        break;
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "WebSocket error");
                        break;
                    }
                    _ => {}
                }
            }

            // Handle commands
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(WsCommand::Subscribe(topics, reply)) => {
                        let params = topics.iter()
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(",");

                        let msg = WsSubscribeMessage {
                            action: "subscribe".to_string(),
                            params,
                        };

                        let result = write.send(Message::Text(serde_json::to_string(&msg).unwrap())).await
                            .map_err(|e| MassiveError::Ws(WsError::Connection(e)));

                        if result.is_ok() {
                            for topic in topics {
                                state.subscriptions.insert(topic);
                            }
                        }

                        let _ = reply.send(result);
                    }
                    Some(WsCommand::Unsubscribe(topics, reply)) => {
                        let params = topics.iter()
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(",");

                        let msg = WsSubscribeMessage {
                            action: "unsubscribe".to_string(),
                            params,
                        };

                        let result = write.send(Message::Text(serde_json::to_string(&msg).unwrap())).await
                            .map_err(|e| MassiveError::Ws(WsError::Connection(e)));

                        if result.is_ok() {
                            for topic in topics {
                                state.subscriptions.remove(&topic);
                            }
                        }

                        let _ = reply.send(result);
                    }
                    Some(WsCommand::Close(reply)) => {
                        let _ = write.send(Message::Close(None)).await;
                        let _ = reply.send(());
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}

/// Parse WebSocket message (handles both single events and arrays)
fn parse_ws_message(text: &str) -> Result<Vec<WsEvent>, serde_json::Error> {
    let trimmed = text.trim();

    if trimmed.starts_with('[') {
        // Array of events
        serde_json::from_str(trimmed)
    } else {
        // Single event
        let event: WsEvent = serde_json::from_str(trimmed)?;
        Ok(vec![event])
    }
}
```

#### 5.2.2 WebSocket Models (`src/ws/models/events.rs`)

```rust
use crate::util::{Symbol, UnixMs};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Unified WebSocket event enum
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "ev")]
pub enum WsEvent {
    /// Connection/authentication status
    #[serde(rename = "status")]
    Status(WsStatusEvent),

    /// Trade event
    #[serde(rename = "T")]
    Trade(WsTradeEvent),

    /// Quote event (NBBO)
    #[serde(rename = "Q")]
    Quote(WsQuoteEvent),

    /// Second aggregate
    #[serde(rename = "A")]
    SecondAggregate(WsAggregateEvent),

    /// Minute aggregate
    #[serde(rename = "AM")]
    MinuteAggregate(WsAggregateEvent),

    /// Limit Up/Limit Down
    #[serde(rename = "LULD")]
    LimitUpLimitDown(WsLuldEvent),

    /// Fair Market Value
    #[serde(rename = "FMV")]
    FairMarketValue(WsFmvEvent),

    /// Unknown event type (forward compatibility)
    #[serde(other)]
    Unknown,
}

/// Status/control message
#[derive(Debug, Clone, Deserialize)]
pub struct WsStatusEvent {
    pub status: String,
    pub message: Option<String>,
}

/// Trade event
#[derive(Debug, Clone, Deserialize)]
pub struct WsTradeEvent {
    /// Ticker symbol
    pub sym: Symbol,
    /// Exchange ID
    pub x: u8,
    /// Trade ID
    pub i: String,
    /// Tape (1=NYSE, 2=AMEX, 3=NASDAQ)
    pub z: u8,
    /// Price
    pub p: f64,
    /// Size
    pub s: u64,
    /// Trade conditions
    #[serde(default)]
    pub c: Vec<i32>,
    /// SIP timestamp (Unix ms)
    pub t: i64,
    /// Sequence number
    pub q: u64,
    /// TRF ID
    pub trfi: Option<u8>,
    /// TRF timestamp
    pub trft: Option<i64>,
}

/// Quote event (NBBO)
#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteEvent {
    /// Ticker symbol
    pub sym: Symbol,
    /// Bid exchange
    pub bx: u8,
    /// Bid price
    pub bp: f64,
    /// Bid size
    pub bs: u64,
    /// Ask exchange
    pub ax: u8,
    /// Ask price
    pub ap: f64,
    /// Ask size (renamed from "as" which is a keyword)
    #[serde(rename = "as")]
    pub ask_size: u64,
    /// Quote condition
    pub c: Option<i32>,
    /// Timestamp
    pub t: i64,
}

/// Aggregate event (second or minute)
#[derive(Debug, Clone, Deserialize)]
pub struct WsAggregateEvent {
    /// Ticker symbol
    pub sym: Symbol,
    /// Volume in window
    pub v: u64,
    /// Accumulated volume today
    pub av: u64,
    /// Official open price
    pub op: f64,
    /// VWAP for window
    pub vw: f64,
    /// Open price (window)
    pub o: f64,
    /// Close price (window)
    pub c: f64,
    /// High price (window)
    pub h: f64,
    /// Low price (window)
    pub l: f64,
    /// VWAP today
    pub a: f64,
    /// Average trade size
    pub z: u64,
    /// Window start (Unix ms)
    pub s: i64,
    /// Window end (Unix ms)
    pub e: i64,
    /// OTC ticker flag
    #[serde(default)]
    pub otc: bool,
}

/// Limit Up/Limit Down event
#[derive(Debug, Clone, Deserialize)]
pub struct WsLuldEvent {
    pub sym: Symbol,
    pub high_price: f64,
    pub low_price: f64,
    pub indicators: Vec<i32>,
    pub tape: u8,
    pub t: i64,
}

/// Fair Market Value event
#[derive(Debug, Clone, Deserialize)]
pub struct WsFmvEvent {
    pub sym: Symbol,
    pub fmv: f64,
    pub t: i64,
}
```

#### 5.2.3 Subscription Types (`src/ws/protocol.rs`)

```rust
use serde::Serialize;
use smol_str::SmolStr;

/// Subscription topic
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subscription(SmolStr);

impl Subscription {
    /// Trade subscription: T.{symbol}
    pub fn trade(symbol: &str) -> Self {
        Self(SmolStr::new(format!("T.{}", symbol)))
    }

    /// Quote subscription: Q.{symbol}
    pub fn quote(symbol: &str) -> Self {
        Self(SmolStr::new(format!("Q.{}", symbol)))
    }

    /// Second aggregate: A.{symbol}
    pub fn second_agg(symbol: &str) -> Self {
        Self(SmolStr::new(format!("A.{}", symbol)))
    }

    /// Minute aggregate: AM.{symbol}
    pub fn minute_agg(symbol: &str) -> Self {
        Self(SmolStr::new(format!("AM.{}", symbol)))
    }

    /// All trades: T.*
    pub fn all_trades() -> Self {
        Self(SmolStr::new_static("T.*"))
    }

    /// All quotes: Q.*
    pub fn all_quotes() -> Self {
        Self(SmolStr::new_static("Q.*"))
    }

    /// Raw subscription string
    pub fn raw(s: impl Into<SmolStr>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Authentication message
#[derive(Debug, Clone, Serialize)]
pub struct WsAuthMessage {
    pub action: String,
    pub params: String,
}

/// Subscribe/unsubscribe message
#[derive(Debug, Clone, Serialize)]
pub struct WsSubscribeMessage {
    pub action: String,
    pub params: String,
}
```

#### 5.2.4 Reconnection Logic (`src/ws/reconnect.rs`)

```rust
use std::time::Duration;

/// Reconnection configuration
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Enable automatic reconnection
    pub enabled: bool,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Maximum retry attempts (None = unlimited)
    pub max_retries: Option<u32>,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_retries: None,
            backoff_multiplier: 2.0,
        }
    }
}

impl ReconnectConfig {
    /// Calculate delay for given attempt number
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32 - 1);

        Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f64) as u64)
    }

    /// Check if should retry
    pub fn should_retry(&self, attempt: u32) -> bool {
        self.enabled && self.max_retries.map_or(true, |max| attempt < max)
    }
}
```

#### 5.2.5 Dispatch/Backpressure (`src/ws/dispatch.rs`)

```rust
/// Dispatch configuration for backpressure handling
#[derive(Debug, Clone)]
pub struct DispatchConfig {
    /// Channel buffer capacity
    pub capacity: usize,
    /// Overflow policy when buffer is full
    pub overflow: OverflowPolicy,
    /// Fanout mode
    pub fanout: FanoutMode,
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            overflow: OverflowPolicy::DropOldest,
            fanout: FanoutMode::SingleConsumer,
        }
    }
}

/// Policy when buffer is full
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Drop oldest messages to make room
    DropOldest,
    /// Drop newest (incoming) messages
    DropNewest,
    /// Treat overflow as fatal error
    ErrorAndClose,
}

/// Fanout mode for multiple consumers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanoutMode {
    /// Single consumer (mpsc channel)
    SingleConsumer,
    /// Multiple consumers (broadcast channel)
    Broadcast,
}
```

### 5.3 Acceptance Criteria - Phase 3

- [ ] WebSocket connects to correct endpoint based on Feed/Market
- [ ] Authentication handshake completes successfully
- [ ] Subscribe/unsubscribe work dynamically
- [ ] Single JSON events parse correctly
- [ ] Array JSON events (batched) parse correctly
- [ ] Unknown event types don't crash parsing (forward compatibility)
- [ ] Backpressure handles buffer overflow per policy
- [ ] Reconnection with exponential backoff works
- [ ] State (subscriptions) restored after reconnect
- [ ] IO task never blocks on slow consumers

### 5.4 Milestone Checklist

- [ ] M3.1: `WsClient` connection establishment
- [ ] M3.2: Authentication handshake
- [ ] M3.3: Subscription management
- [ ] M3.4: Message parsing (single + batched)
- [ ] M3.5: `WsEvent` enum with all event types
- [ ] M3.6: Backpressure handling
- [ ] M3.7: Automatic reconnection
- [ ] M3.8: Integration tests with mock WS server

---

## 6. Phase 4: Code Generation & Full API Coverage

### 6.1 Objectives

- Set up OpenAPI spec ingestion pipeline
- Generate REST models from spec
- Generate request builders from spec
- Ensure 100% endpoint coverage
- Add checksum verification for spec changes

### 6.2 Code Generation Strategy

#### 6.2.1 Build Script (`build.rs`)

```rust
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=openapi/spec.json");
    println!("cargo:rerun-if-changed=openapi/checksum.sha256");

    #[cfg(feature = "codegen")]
    {
        generate_from_openapi();
    }
}

#[cfg(feature = "codegen")]
fn generate_from_openapi() {
    use std::fs;
    use sha2::{Sha256, Digest};

    let spec_path = Path::new("openapi/spec.json");
    let checksum_path = Path::new("openapi/checksum.sha256");
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Verify checksum
    let spec_content = fs::read(spec_path).expect("Failed to read OpenAPI spec");
    let computed_hash = format!("{:x}", Sha256::digest(&spec_content));
    let expected_hash = fs::read_to_string(checksum_path)
        .expect("Failed to read checksum")
        .trim()
        .to_string();

    if computed_hash != expected_hash {
        panic!(
            "OpenAPI spec checksum mismatch!\n\
             Expected: {}\n\
             Computed: {}\n\
             Run `cargo xtask update-spec` to regenerate.",
            expected_hash, computed_hash
        );
    }

    // Parse OpenAPI spec
    let spec: openapiv3::OpenAPI = serde_json::from_slice(&spec_content)
        .expect("Failed to parse OpenAPI spec");

    // Generate models
    let models_code = generate_models(&spec);
    fs::write(Path::new(&out_dir).join("generated_models.rs"), models_code)
        .expect("Failed to write generated models");

    // Generate requests
    let requests_code = generate_requests(&spec);
    fs::write(Path::new(&out_dir).join("generated_requests.rs"), requests_code)
        .expect("Failed to write generated requests");
}

#[cfg(feature = "codegen")]
fn generate_models(spec: &openapiv3::OpenAPI) -> String {
    // Model generation logic
    // ...
    todo!()
}

#[cfg(feature = "codegen")]
fn generate_requests(spec: &openapiv3::OpenAPI) -> String {
    // Request generation logic
    // ...
    todo!()
}
```

#### 6.2.2 Endpoint Categories

| Category | Endpoints | Priority |
|----------|-----------|----------|
| Aggregates | `/v2/aggs/*` | P0 |
| Trades | `/v3/trades/*`, `/v2/last/trade/*` | P0 |
| Quotes | `/v3/quotes/*`, `/v2/last/nbbo/*` | P0 |
| Snapshots | `/v2/snapshot/*` | P0 |
| Reference | `/v3/reference/*` | P1 |
| Options | `/v3/snapshot/options/*`, `/v3/options/*` | P1 |
| Forex | `/v2/aggs/forex/*`, `/v1/conversion/*` | P1 |
| Crypto | `/v2/aggs/crypto/*`, `/v1/open-close/crypto/*` | P1 |
| Indices | `/v1/summaries/*` | P2 |
| Futures | `/v1/summaries/*` (futures) | P2 |

### 6.3 Milestone Checklist

- [ ] M4.1: OpenAPI spec acquisition and versioning
- [ ] M4.2: Checksum-based change detection
- [ ] M4.3: Model generation from schemas
- [ ] M4.4: Request builder generation from paths
- [ ] M4.5: 100% endpoint coverage verification
- [ ] M4.6: Backwards compatibility aliases

---

## 7. Phase 5: Performance Optimization & HFT Hardening

### 7.1 Objectives

- Achieve <10μs message deserialization
- Support 100k+ messages/second throughput
- Minimize allocations in hot paths
- Implement optional SIMD JSON parsing
- Add metrics and observability hooks

### 7.2 Optimization Targets

#### 7.2.1 JSON Parsing Optimization

```rust
#[cfg(feature = "simd-json")]
pub fn parse_ws_message_fast(bytes: &mut [u8]) -> Result<Vec<WsEvent>, MassiveError> {
    use simd_json::prelude::*;

    let value = simd_json::to_borrowed_value(bytes)
        .map_err(|e| MassiveError::Deserialize {
            source: serde_json::Error::custom(e.to_string()),
            body_snippet: String::from_utf8_lossy(&bytes[..bytes.len().min(100)]).to_string(),
        })?;

    // Fast path for common event types
    // ...
}
```

#### 7.2.2 Memory Pool for Messages

```rust
use std::sync::Arc;
use parking_lot::Mutex;

/// Object pool for reducing allocations
pub struct MessagePool {
    trades: Mutex<Vec<Box<WsTradeEvent>>>,
    quotes: Mutex<Vec<Box<WsQuoteEvent>>>,
    aggregates: Mutex<Vec<Box<WsAggregateEvent>>>,
}

impl MessagePool {
    pub fn acquire_trade(&self) -> PooledTrade {
        // ...
    }

    pub fn release_trade(&self, trade: Box<WsTradeEvent>) {
        // ...
    }
}
```

#### 7.2.3 Metrics Trait

```rust
/// Trait for custom metrics collection
pub trait MetricsSink: Send + Sync + 'static {
    /// Increment a counter
    fn counter(&self, name: &'static str, value: u64, tags: &[(&'static str, &str)]);

    /// Set a gauge value
    fn gauge(&self, name: &'static str, value: i64, tags: &[(&'static str, &str)]);

    /// Record a histogram value
    fn histogram(&self, name: &'static str, value: f64, tags: &[(&'static str, &str)]);
}

/// No-op metrics sink (default)
pub struct NoopMetrics;

impl MetricsSink for NoopMetrics {
    fn counter(&self, _: &'static str, _: u64, _: &[(&'static str, &str)]) {}
    fn gauge(&self, _: &'static str, _: i64, _: &[(&'static str, &str)]) {}
    fn histogram(&self, _: &'static str, _: f64, _: &[(&'static str, &str)]) {}
}
```

### 7.3 Benchmarks

```rust
// benches/json_parsing.rs
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn bench_trade_parsing(c: &mut Criterion) {
    let trade_json = r#"{"ev":"T","sym":"AAPL","x":4,"i":"123","z":3,"p":150.25,"s":100,"c":[0],"t":1703001234567,"q":12345}"#;

    let mut group = c.benchmark_group("trade_parsing");
    group.throughput(Throughput::Bytes(trade_json.len() as u64));

    group.bench_function("serde_json", |b| {
        b.iter(|| {
            let _: WsEvent = serde_json::from_str(trade_json).unwrap();
        })
    });

    #[cfg(feature = "simd-json")]
    group.bench_function("simd_json", |b| {
        let mut bytes = trade_json.as_bytes().to_vec();
        b.iter(|| {
            let _: WsEvent = simd_json::from_slice(&mut bytes.clone()).unwrap();
        })
    });

    group.finish();
}

fn bench_batch_parsing(c: &mut Criterion) {
    // Array of 100 trades
    let batch_json = generate_trade_batch(100);

    let mut group = c.benchmark_group("batch_parsing");
    group.throughput(Throughput::Elements(100));

    group.bench_function("parse_100_trades", |b| {
        b.iter(|| {
            parse_ws_message(&batch_json).unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, bench_trade_parsing, bench_batch_parsing);
criterion_main!(benches);
```

### 7.4 Milestone Checklist

- [ ] M5.1: Baseline benchmarks established
- [ ] M5.2: SIMD JSON parsing (feature-gated)
- [ ] M5.3: Memory pooling for hot paths
- [ ] M5.4: Metrics hooks implementation
- [ ] M5.5: Tracing integration
- [ ] M5.6: Performance targets validated

---

## 8. Phase 6: Documentation, Testing & Release

### 8.1 Documentation Requirements

- Comprehensive rustdoc for all public items
- README with quick start examples
- API reference guide
- HFT usage patterns guide
- Migration guide from Python client

### 8.2 Testing Strategy

#### Unit Tests
- JSON parsing (all event types)
- Pagination logic
- Error handling
- Configuration validation

#### Integration Tests
- REST client with wiremock
- WebSocket client with mock server
- End-to-end with live API (optional, CI-gated)

#### Property-Based Tests
- Timestamp conversions
- Subscription string building
- Query parameter serialization

#### Performance Tests
- Message throughput
- Latency percentiles (p50, p95, p99)
- Memory allocation profiles

### 8.3 Release Checklist

- [ ] Version follows semver
- [ ] CHANGELOG.md updated
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Examples verified working
- [ ] Benchmarks show acceptable performance
- [ ] Security audit (no exposed secrets)
- [ ] License files present
- [ ] crates.io metadata complete

### 8.4 Milestone Checklist

- [ ] M6.1: Complete rustdoc coverage
- [ ] M6.2: README and examples
- [ ] M6.3: Integration test suite
- [ ] M6.4: Property-based tests
- [ ] M6.5: Performance test suite
- [ ] M6.6: CI/CD pipeline finalized
- [ ] M6.7: Initial release (v0.1.0)

---

## 9. Risk Assessment & Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| OpenAPI spec unavailable/incomplete | High | Medium | Fall back to manual endpoint implementation; document gaps |
| API breaking changes | High | Low | Checksum verification; version pinning; backwards compat aliases |
| Performance targets not met | High | Low | Early benchmarking; SIMD fallback; profiling |
| WebSocket protocol changes | Medium | Low | Forward-compatible parsing; version negotiation |
| Rate limiting during development | Low | Medium | Use test API keys; implement proper backoff |
| Dependency security vulnerabilities | Medium | Low | Regular `cargo audit`; minimal dependencies |

---

## 10. Appendices

### Appendix A: Cargo.toml Template

```toml
[package]
name = "massive-rs"
version = "0.1.0"
edition = "2021"
rust-version = "1.75.0"
license = "MIT OR Apache-2.0"
description = "High-performance Rust client for Massive.com market data APIs"
repository = "https://github.com/your-org/massive-rs"
documentation = "https://docs.rs/massive-rs"
readme = "README.md"
keywords = ["trading", "market-data", "hft", "websocket", "financial"]
categories = ["api-bindings", "asynchronous", "finance"]

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
reqwest = { version = "0.12", default-features = false, features = ["json"] }
tokio-tungstenite = { version = "0.24", optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
bytes = "1"
smol_str = { version = "0.3", features = ["serde"] }
dashmap = { version = "6", optional = true }
futures = "0.3"
tracing = "0.1"
url = "2"
secrecy = "0.10"
chrono = { version = "0.4", features = ["serde"] }

# Optional
simd-json = { version = "0.14", optional = true }
rust_decimal = { version = "1", optional = true }

[dev-dependencies]
tokio-test = "0.4"
wiremock = "0.6"
criterion = { version = "0.5", features = ["async_tokio"] }
proptest = "1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[features]
default = ["rustls", "gzip", "ws"]
rustls = ["reqwest/rustls-tls", "tokio-tungstenite?/rustls-tls-native-roots"]
native-tls = ["reqwest/native-tls", "tokio-tungstenite?/native-tls"]
gzip = ["reqwest/gzip"]
ws = ["dep:tokio-tungstenite", "dep:dashmap"]
simd-json = ["dep:simd-json"]
decimal = ["dep:rust_decimal"]
blocking = ["tokio/rt-multi-thread"]
codegen = []

[[bench]]
name = "json_parsing"
harness = false

[[bench]]
name = "ws_throughput"
harness = false

[[example]]
name = "rest_aggregates"
required-features = []

[[example]]
name = "ws_realtime_trades"
required-features = ["ws"]
```

### Appendix B: Exchange ID Reference

| ID | Exchange | Type |
|----|----------|------|
| 1 | NYSE American (AMEX) | Exchange |
| 4 | NYSE | Exchange |
| 7 | NYSE Arca | Exchange |
| 11 | NASDAQ | Exchange |
| 12 | FINRA NYSE TRF | TRF |
| 15 | FINRA Nasdaq TRF | TRF |
| 19 | IEX | Exchange |
| 21 | MEMX | Exchange |

### Appendix C: WebSocket Channel Prefixes

| Prefix | Event Type | Description |
|--------|------------|-------------|
| `T.` | Trade | Real-time tick-level trades |
| `Q.` | Quote | Real-time NBBO quotes |
| `A.` | Second Aggregate | Per-second OHLCV bars |
| `AM.` | Minute Aggregate | Per-minute OHLCV bars |
| `LULD.` | Limit Up/Down | Circuit breaker events |
| `FMV.` | Fair Market Value | Proprietary price estimate |

---

## Summary

This implementation plan provides a comprehensive roadmap for building the `massive-rs` crate. The six-phase approach ensures:

1. **Solid foundation** with proper error handling and configuration
2. **Complete REST coverage** with pagination and retry logic
3. **HFT-ready WebSocket** with backpressure handling
4. **Spec-driven completeness** via OpenAPI code generation
5. **Performance optimization** for latency-sensitive applications
6. **Production quality** through comprehensive testing and documentation

The plan prioritizes correctness and safety first, with performance optimizations layered on top. All public APIs are designed to be ergonomic for Rust developers while maintaining strict adherence to the Massive.com API specifications.

---

*Document generated: December 2025*
*Last updated: Version 1.0*
