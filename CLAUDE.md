# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build --all-features        # Build with all features
cargo clippy --all-features       # Lint (must pass with no warnings)
cargo fmt --check                 # Check formatting
cargo test --all-features         # Run all tests
cargo test <test_name>            # Run a single test
cargo test --lib                  # Run library tests only
cargo doc --all-features --no-deps # Build documentation
cargo bench --bench json_parsing  # Run JSON parsing benchmarks
```

## Project Overview

massive-rs is a high-performance Rust client for the Massive.com market data API (formerly Polygon.io). It provides both REST and WebSocket clients optimized for low-latency, high-throughput trading applications.

## Architecture

### Module Structure

- **`auth`**: API key handling with `secrecy` crate for secure storage (never appears in debug output)
- **`config`**: `RestConfig` and `WsConfig` with builder patterns for client configuration
- **`error`**: Unified `MassiveError` type covering transport, HTTP, parsing, and WebSocket errors
- **`rest`**: REST client with typed requests, automatic pagination, and retry logic
- **`ws`**: WebSocket client with reconnection, backpressure handling, and subscription management (feature-gated)
- **`models`**: Data types for stocks, options, forex, and crypto responses
- **`metrics`**: `MetricsSink` trait for custom observability integration, `ClientStats` for built-in counters
- **`parse`**: High-performance JSON parsing helpers with optional SIMD acceleration

### Key Design Patterns

**Request Traits**: REST endpoints implement `RestRequest` (defines method, path, query, response type) and optionally `PaginatableRequest` (for streaming paginated results via `PageStream`). See `src/rest/request.rs`.

**WebSocket Events**: `WsEvent` enum with variants for Trade, Quote, Aggregate, Status, LULD, FMV, and Unknown (for forward compatibility). Events are delivered in batches via `WsMessageBatch`.

**Configuration**: Both clients read `MASSIVE_API_KEY` from environment by default. Auth modes: `HeaderBearer` (default) or `QueryParam`.

### Feature Flags

- `default = ["rustls", "gzip", "ws"]`
- `ws`: Enables WebSocket client (requires `tokio-tungstenite`, `dashmap`)
- `simd-json`: SIMD-accelerated JSON parsing
- `decimal`: Exact decimal arithmetic via `rust_decimal`

### Adding New Endpoints

1. Create request struct in appropriate file under `src/rest/endpoints/`
2. Implement `RestRequest` trait (method, path, query, Response type)
3. For paginated endpoints, also implement `PaginatableRequest`
4. Add response model to `src/rest/models/` if new types needed
5. Re-export from `src/rest/endpoints/mod.rs`

### Code Style

- `#![deny(missing_docs)]` enforced - all public items need doc comments
- `#![warn(clippy::all)]` - clippy warnings treated seriously
- Uses `smol_str` for small-string optimization on ticker symbols
- Timestamps use `chrono` with `UnixMs`/`UnixNs` wrapper types in `src/util.rs`
