# massive-rs

[![Crates.io](https://img.shields.io/crates/v/massive-rs.svg)](https://crates.io/crates/massive-rs)
[![Documentation](https://docs.rs/massive-rs/badge.svg)](https://docs.rs/massive-rs)
[![License](https://img.shields.io/crates/l/massive-rs.svg)](LICENSE)

High-performance Rust client for the [Massive.com](https://massive.com) (formerly Polygon.io) market data APIs.

## Features

- **REST API Client**: Typed requests with automatic pagination and retry logic
- **WebSocket Streaming**: Real-time market data with backpressure handling
- **HFT-Ready**: Optimized for low-latency, high-throughput applications
- **Type Safety**: Strongly-typed models for all API responses
- **Resilient**: Automatic reconnection, exponential backoff, rate limit handling
- **Observable**: Built-in metrics hooks for monitoring

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
massive-rs = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### REST API Example

```rust
use massive_rs::rest::RestClient;
use massive_rs::rest::endpoints::{GetAggsRequest, Timespan};
use massive_rs::config::RestConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client (reads MASSIVE_API_KEY from environment)
    let client = RestClient::new(RestConfig::default())?;

    // Fetch daily aggregate bars for AAPL
    let request = GetAggsRequest::new("AAPL")
        .multiplier(1)
        .timespan(Timespan::Day)
        .from("2024-01-01")
        .to("2024-01-31");

    let response = client.execute(request).await?;

    for bar in response.results {
        println!(
            "Date: {}, O: {:.2}, H: {:.2}, L: {:.2}, C: {:.2}, V: {}",
            bar.timestamp, bar.open, bar.high, bar.low, bar.close, bar.volume as u64
        );
    }

    Ok(())
}
```

### WebSocket Streaming Example

```rust
use massive_rs::ws::{WsClient, Subscription};
use massive_rs::config::WsConfig;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create WebSocket client
    let client = WsClient::new(WsConfig::default())?;
    let (handle, mut stream) = client.connect().await?;

    // Subscribe to real-time trades
    handle.subscribe(&[
        Subscription::trade("AAPL"),
        Subscription::trade("MSFT"),
    ]).await?;

    // Process incoming events
    while let Some(result) = stream.next().await {
        let batch = result?;
        for event in batch.events {
            match event {
                massive_rs::ws::WsEvent::Trade(trade) => {
                    println!("{}: ${:.2} x {}", trade.sym, trade.p, trade.s);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
```

### Paginated Results

The REST client automatically handles pagination for endpoints that support it:

```rust
use massive_rs::rest::RestClient;
use massive_rs::rest::endpoints::GetTradesRequest;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RestClient::new(Default::default())?;

    // Stream all trades with automatic pagination
    let request = GetTradesRequest::new("AAPL")
        .timestamp_gte("2024-01-01")
        .limit(50000);

    let mut stream = client.stream(request);

    while let Some(trade) = stream.next().await {
        let trade = trade?;
        println!("Trade: ${:.2} x {}", trade.price, trade.size);
    }

    Ok(())
}
```

## API Coverage

### REST Endpoints

| Category | Endpoints | Status |
|----------|-----------|--------|
| Market Data | Aggregates, Previous Close, Open/Close | Implemented |
| Trades | Historical Trades, Last Trade | Implemented |
| Quotes | Historical Quotes, Last Quote | Implemented |
| Snapshots | Ticker Snapshot, All Tickers, Gainers/Losers | Implemented |
| Reference | Tickers, Ticker Details | Implemented |

### WebSocket Channels

| Channel | Event Type | Status |
|---------|------------|--------|
| `T.*` | Trades | Implemented |
| `Q.*` | Quotes (NBBO) | Implemented |
| `A.*` | Second Aggregates | Implemented |
| `AM.*` | Minute Aggregates | Implemented |
| `LULD.*` | Limit Up/Limit Down | Implemented |
| `FMV.*` | Fair Market Value | Implemented |

### Asset Classes

| Asset Class | REST | WebSocket |
|-------------|------|-----------|
| Stocks | Full | Full |
| Options | Models | Planned |
| Forex | Models | Planned |
| Crypto | Models | Planned |
| Indices | Planned | Planned |
| Futures | Planned | Planned |

## Feature Flags

```toml
[features]
default = ["rustls", "gzip", "ws"]

# TLS backends
rustls = ["reqwest/rustls-tls", "tokio-tungstenite/rustls-tls-native-roots"]
native-tls = ["reqwest/native-tls", "tokio-tungstenite/native-tls"]

# HTTP compression
gzip = ["reqwest/gzip"]

# WebSocket support
ws = ["dep:tokio-tungstenite", "dep:dashmap"]

# Performance optimizations
simd-json = ["dep:simd-json"]  # SIMD-accelerated JSON parsing

# Precision numerics
decimal = ["dep:rust_decimal"]  # Exact decimal arithmetic
```

## Configuration

### REST Client

```rust
use massive_rs::config::RestConfig;
use massive_rs::auth::ApiKey;
use std::time::Duration;

let config = RestConfig {
    api_key: ApiKey::new("your-api-key"),
    base_url: "https://api.massive.com".parse().unwrap(),
    connect_timeout: Duration::from_secs(10),
    request_timeout: Duration::from_secs(30),
    max_retries: 3,
    ..Default::default()
};
```

### WebSocket Client

```rust
use massive_rs::config::{WsConfig, Feed, Market, ReconnectConfig, DispatchConfig};
use std::time::Duration;

let config = WsConfig {
    feed: Feed::RealTime,
    market: Market::Stocks,
    api_key: ApiKey::from_env().unwrap(),
    reconnect: ReconnectConfig {
        enabled: true,
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(60),
        max_retries: None, // Unlimited
        ..Default::default()
    },
    dispatch: DispatchConfig {
        capacity: 10_000,
        overflow: OverflowPolicy::DropOldest,
        ..Default::default()
    },
    ..Default::default()
};
```

## Performance

The crate is optimized for HFT and algorithmic trading applications:

- **Sub-10us message parsing** with standard serde_json
- **Sub-1us message parsing** with SIMD-JSON feature
- **100k+ messages/second** throughput
- **Lock-free atomic counters** for thread-safe statistics
- **Zero-copy parsing** where possible

### Benchmarks

Run benchmarks with:

```bash
cargo bench --bench json_parsing
```

## Observability

### Built-in Statistics

```rust
use massive_rs::metrics::ClientStats;
use std::sync::Arc;

let stats = Arc::new(ClientStats::new());

// Use stats in your client...

// Get current snapshot
let snapshot = stats.snapshot();
println!("Messages received: {}", snapshot.messages_received);
println!("Messages dropped: {}", snapshot.messages_dropped);
println!("Parse errors: {}", snapshot.parse_errors);
```

### Custom Metrics

Implement `MetricsSink` to integrate with your monitoring system:

```rust
use massive_rs::metrics::MetricsSink;

struct MyPrometheusMetrics { /* ... */ }

impl MetricsSink for MyPrometheusMetrics {
    fn counter(&self, name: &'static str, value: u64, tags: &[(&'static str, &str)]) {
        // Record to Prometheus...
    }

    fn gauge(&self, name: &'static str, value: i64, tags: &[(&'static str, &str)]) {
        // Record to Prometheus...
    }

    fn histogram(&self, name: &'static str, value: f64, tags: &[(&'static str, &str)]) {
        // Record to Prometheus...
    }
}
```

## Error Handling

All operations return `Result<T, MassiveError>`:

```rust
use massive_rs::error::MassiveError;

match client.execute(request).await {
    Ok(response) => { /* handle success */ }
    Err(MassiveError::RateLimited { retry_after, .. }) => {
        // Wait and retry
        if let Some(duration) = retry_after {
            tokio::time::sleep(duration).await;
        }
    }
    Err(MassiveError::Api(api_error)) => {
        eprintln!("API error: {}", api_error);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `MASSIVE_API_KEY` | API key for authentication |

## Examples

See the `examples/` directory for more complete examples:

- `rest_aggregates.rs` - Fetching aggregate bars
- `ws_realtime_trades.rs` - Real-time trade streaming

Run examples with:

```bash
MASSIVE_API_KEY=your-key cargo run --example rest_aggregates
MASSIVE_API_KEY=your-key cargo run --example ws_realtime_trades
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
