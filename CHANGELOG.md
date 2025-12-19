# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-12-19

### Added

#### Core Infrastructure
- `ApiKey` type with secure handling (redacted debug output)
- `AuthMode` enum supporting header bearer and query param authentication
- `MassiveError` unified error type with comprehensive error variants
- `WsError` for WebSocket-specific errors
- `RestConfig` and `WsConfig` for client configuration
- `UnixMs` and `UnixNs` timestamp types with chrono integration

#### REST Client
- `RestClient` with connection pooling and automatic retry logic
- `RestRequest` trait for typed endpoint requests
- `PaginatableRequest` trait for paginated endpoints
- `PageStream` for automatic `next_url` pagination handling
- `PaginationMode`: Auto, None, and MaxItems modes
- Exponential backoff retry for 502/503/504 errors
- Rate limiting detection with `Retry-After` header parsing

#### REST Endpoints
- `GetAggsRequest`: Aggregate bars (OHLCV) with timespan/multiplier
- `GetPreviousCloseRequest`: Previous day's OHLC data
- `GetDailyOpenCloseRequest`: Specific date open/close data
- `GetTradesRequest`: Historical trades with pagination
- `GetLastTradeRequest`: Last trade for a ticker
- `GetQuotesRequest`: Historical quotes with pagination
- `GetLastQuoteRequest`: Last NBBO for a ticker
- `GetTickersRequest`: List and search tickers
- `GetTickerDetailsRequest`: Detailed ticker information
- `GetTickerSnapshotRequest`: Real-time ticker snapshot
- `GetAllTickersSnapshotRequest`: All tickers snapshot
- `GetGainersLosersRequest`: Top gainers/losers

#### WebSocket Client
- `WsClient` with automatic reconnection
- `WsHandle` for connection control and subscription management
- `WsState` with atomic counters for connection statistics
- `ConnectionState` enum for monitoring connection lifecycle
- Backpressure handling with configurable `OverflowPolicy`
- `Subscription` type for channel subscriptions (T.*, Q.*, A.*, AM.*)

#### WebSocket Events
- `WsEvent` enum for all event types
- `WsTradeEvent`: Real-time trade data
- `WsQuoteEvent`: Real-time NBBO quotes
- `WsAggregateEvent`: Second and minute aggregates
- `WsLuldEvent`: Limit Up/Limit Down events
- `WsFmvEvent`: Fair Market Value events
- `WsStatusEvent`: Connection status events

#### Models
- `AggregateBar`: OHLCV data with calculations
- `Trade`: Trade data with conditions
- `Quote`: NBBO quote with spread calculations
- `Ticker`: Ticker metadata
- Options models: `OptionContract`, `Greeks`, `OptionQuote`, `OptionSnapshot`
- Forex models: `CurrencyPair`, `ForexQuote`, `ForexBar`, `CurrencyConversion`
- Crypto models: `CryptoPair`, `CryptoTrade`, `CryptoQuote`, `CryptoBar`

#### Performance & Observability
- `MetricsSink` trait for custom metrics integration
- `NoopMetrics` and `TracingMetrics` implementations
- `ClientStats` with atomic counters for statistics
- `parse_ws_events()` and `parse_ws_events_bytes()` for optimized parsing
- SIMD-JSON support (feature-gated)
- Fast event type extraction without full parsing

#### Build Infrastructure
- OpenAPI spec placeholder with checksum verification
- Build script for future code generation support
- Comprehensive benchmark suite

### Feature Flags
- `rustls` (default): rustls TLS backend
- `native-tls`: Native TLS backend
- `gzip` (default): HTTP compression
- `ws` (default): WebSocket support
- `simd-json`: SIMD-accelerated JSON parsing
- `decimal`: Exact decimal arithmetic
- `codegen`: OpenAPI code generation

[0.1.0]: https://github.com/your-org/massive-rs/releases/tag/v0.1.0
