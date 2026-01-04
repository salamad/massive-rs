# massive-rs

[![Crates.io](https://img.shields.io/crates/v/massive-rs.svg)](https://crates.io/crates/massive-rs)
[![Documentation](https://docs.rs/massive-rs/badge.svg)](https://docs.rs/massive-rs)
[![License](https://img.shields.io/crates/l/massive-rs.svg)](LICENSE)

High-performance Rust client for the [Massive.com](https://massive.com) (formerly Polygon.io) market data APIs. Designed for low-latency, high-throughput quantitative trading applications.

## Features

- **REST API Client**: Typed requests with automatic pagination, retry logic, and rate limit handling
- **WebSocket Streaming**: Real-time market data with automatic reconnection and backpressure handling
- **Complete API Coverage**: Stocks, options, futures, forex, crypto, indices, and economic data
- **HFT-Ready**: Optimized for sub-microsecond parsing with optional SIMD acceleration
- **Type Safety**: Strongly-typed models with helper methods for financial calculations
- **Resilient**: Automatic reconnection, exponential backoff, configurable overflow policies
- **Observable**: Built-in metrics hooks for Prometheus, StatsD, or custom monitoring
- **Secure**: API keys are protected using the `secrecy` crate and never appear in logs

## Table of Contents

- [Quick Start](#quick-start)
- [REST API](#rest-api)
  - [Market Data](#market-data)
  - [Trades & Quotes](#trades--quotes)
  - [Snapshots](#snapshots)
  - [Reference Data](#reference-data)
  - [Technical Indicators](#technical-indicators)
  - [Options](#options)
  - [Futures](#futures)
  - [Forex](#forex)
  - [Crypto](#crypto)
  - [Fundamentals](#fundamentals)
  - [Corporate Actions](#corporate-actions)
  - [Economic Data](#economic-data)
  - [News](#news)
  - [Partner Data](#partner-data)
- [WebSocket Streaming](#websocket-streaming)
- [Pagination](#pagination)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Observability](#observability)
- [Performance](#performance)
- [Feature Flags](#feature-flags)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
massive-rs = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

Set your API key:

```bash
export MASSIVE_API_KEY=your-api-key
```

### Basic REST Example

```rust
use massive_rs::rest::RestClient;
use massive_rs::rest::endpoints::{GetAggsRequest, Timespan};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client (reads MASSIVE_API_KEY from environment)
    let client = RestClient::new(Default::default())?;

    // Fetch daily bars for AAPL
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

### Basic WebSocket Example

```rust
use massive_rs::ws::{WsClient, Subscription};
use massive_rs::config::WsConfig;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = WsClient::new(WsConfig::default())?;
    let (handle, mut stream) = client.connect().await?;

    // Subscribe to real-time trades
    handle.subscribe(&[
        Subscription::trade("AAPL"),
        Subscription::trade("MSFT"),
    ]).await?;

    // Process events
    while let Some(result) = stream.next().await {
        let batch = result?;
        for event in batch.events {
            if let massive_rs::ws::WsEvent::Trade(trade) = event {
                println!("{}: ${:.2} x {}", trade.sym, trade.p, trade.s);
            }
        }
    }

    Ok(())
}
```

---

## REST API

### Market Data

#### Aggregate Bars (OHLCV)

```rust
use massive_rs::rest::endpoints::{GetAggsRequest, Timespan, Sort};

// Daily bars
let request = GetAggsRequest::new("AAPL")
    .multiplier(1)
    .timespan(Timespan::Day)
    .from("2024-01-01")
    .to("2024-12-31")
    .adjusted(true)
    .sort(Sort::Asc)
    .limit(5000);

let response = client.execute(request).await?;

// Bar helper methods
for bar in response.results {
    println!("Range: {:.2}", bar.range());           // high - low
    println!("Body: {:.2}", bar.body());             // |close - open|
    println!("Change: {:.2}%", bar.change_percent()); // % change
    println!("VWAP: {:.2}", bar.vwap());
    println!("Bullish: {}", bar.is_bullish());
    println!("Doji: {}", bar.is_doji(0.1));
}
```

**Timespans**: `Second`, `Minute`, `Hour`, `Day`, `Week`, `Month`, `Quarter`, `Year`

#### Previous Close

```rust
use massive_rs::rest::endpoints::GetPreviousCloseRequest;

let request = GetPreviousCloseRequest::new("AAPL").adjusted(true);
let response = client.execute(request).await?;
```

#### Daily Open/Close

```rust
use massive_rs::rest::endpoints::GetDailyOpenCloseRequest;

let request = GetDailyOpenCloseRequest::new("AAPL", "2024-01-15").adjusted(true);
let response = client.execute(request).await?;
```

---

### Trades & Quotes

#### Historical Trades

```rust
use massive_rs::rest::endpoints::GetTradesRequest;

let request = GetTradesRequest::new("AAPL")
    .timestamp_gte("2024-01-15T09:30:00Z")
    .timestamp_lte("2024-01-15T16:00:00Z")
    .limit(50000);

let response = client.execute(request).await?;

for trade in response.results {
    println!("Price: ${:.2}, Size: {}, Value: ${:.2}",
        trade.price, trade.size, trade.value());
}
```

#### Last Trade

```rust
use massive_rs::rest::endpoints::GetLastTradeRequest;

let request = GetLastTradeRequest::new("AAPL");
let response = client.execute(request).await?;
```

#### Historical Quotes (NBBO)

```rust
use massive_rs::rest::endpoints::GetQuotesRequest;

let request = GetQuotesRequest::new("AAPL")
    .timestamp_gte("2024-01-15T09:30:00Z")
    .limit(50000);

let response = client.execute(request).await?;

for quote in response.results {
    println!("Bid: ${:.2} x {}, Ask: ${:.2} x {}",
        quote.bid_price, quote.bid_size, quote.ask_price, quote.ask_size);
    println!("Spread: ${:.4} ({:.2} bps)", quote.spread(), quote.spread_bps());
    println!("Mid: ${:.4}", quote.mid_price());
    println!("Crossed: {}", quote.is_crossed());
}
```

#### Last Quote (NBBO)

```rust
use massive_rs::rest::endpoints::GetLastQuoteRequest;

let request = GetLastQuoteRequest::new("AAPL");
let response = client.execute(request).await?;
```

---

### Snapshots

#### Single Ticker Snapshot

```rust
use massive_rs::rest::endpoints::GetTickerSnapshotRequest;

let request = GetTickerSnapshotRequest::new("AAPL");
let response = client.execute(request).await?;

if let Some(ticker) = response.ticker {
    println!("Today's change: {:.2}%", ticker.todays_change_percent);
    println!("Day volume: {}", ticker.day.volume as u64);
}
```

#### All Tickers Snapshot

```rust
use massive_rs::rest::endpoints::GetAllTickersSnapshotRequest;

let request = GetAllTickersSnapshotRequest::new()
    .tickers(vec!["AAPL".into(), "MSFT".into(), "GOOGL".into()]);

let response = client.execute(request).await?;
```

#### Gainers & Losers

```rust
use massive_rs::rest::endpoints::{GetGainersLosersRequest, GainersLosersDirection};

// Top gainers
let gainers = GetGainersLosersRequest::new(GainersLosersDirection::Gainers);

// Top losers
let losers = GetGainersLosersRequest::new(GainersLosersDirection::Losers);
```

#### Unified Snapshots (Multi-Asset)

```rust
use massive_rs::rest::endpoints::GetUnifiedSnapshotRequest;

let request = GetUnifiedSnapshotRequest::new()
    .ticker_any_of(vec!["AAPL".into(), "O:AAPL251219C00200000".into()]);

let response = client.execute(request).await?;

for result in response.results {
    if result.is_stock() {
        println!("Stock: {}", result.ticker);
    } else if result.is_option() {
        println!("Option: {}", result.ticker);
    }
}
```

#### Grouped Daily Bars

```rust
use massive_rs::rest::endpoints::GetGroupedDailyRequest;

// All US stocks for a specific date
let request = GetGroupedDailyRequest::us_stocks("2024-01-15").adjusted(true);

// All crypto pairs
let request = GetGroupedDailyRequest::crypto("2024-01-15");
```

---

### Reference Data

#### Tickers List

```rust
use massive_rs::rest::endpoints::{GetTickersRequest, MarketType};

let request = GetTickersRequest::new()
    .market(MarketType::Stocks)
    .active(true)
    .search("Apple")
    .limit(100);

let response = client.execute(request).await?;
```

#### Ticker Details

```rust
use massive_rs::rest::endpoints::GetTickerDetailsRequest;

let request = GetTickerDetailsRequest::new("AAPL");
let response = client.execute(request).await?;

if let Some(details) = response.results {
    println!("Name: {}", details.name);
    println!("Market Cap: ${}", details.market_cap.unwrap_or(0.0));
    println!("Shares Outstanding: {}", details.share_class_shares_outstanding.unwrap_or(0));
}
```

#### Exchanges

```rust
use massive_rs::rest::endpoints::GetExchangesRequest;

let request = GetExchangesRequest::new().asset_class("stocks");
let response = client.execute(request).await?;
```

#### Conditions

```rust
use massive_rs::rest::endpoints::GetConditionsRequest;

let request = GetConditionsRequest::new().asset_class("stocks");
let response = client.execute(request).await?;
```

#### Ticker Types

```rust
use massive_rs::rest::endpoints::GetTickerTypesRequest;

let request = GetTickerTypesRequest::new().asset_class("stocks");
let response = client.execute(request).await?;
```

#### Market Holidays

```rust
use massive_rs::rest::endpoints::GetMarketHolidaysRequest;

let request = GetMarketHolidaysRequest;
let response = client.execute(request).await?;

for holiday in response {
    println!("{}: {} ({})", holiday.date, holiday.name, holiday.status);
    if holiday.is_early_close() {
        println!("  Early close at: {}", holiday.close.unwrap_or_default());
    }
}
```

#### Market Status

```rust
use massive_rs::rest::endpoints::GetMarketStatusRequest;

let response = client.execute(GetMarketStatusRequest).await?;
println!("Market is: {}", response.market);
println!("After hours: {}", response.after_hours);
println!("Early hours: {}", response.early_hours);
```

---

### Technical Indicators

#### RSI (Relative Strength Index)

```rust
use massive_rs::rest::endpoints::{GetRsiRequest, IndicatorTimespan, SeriesType};

let request = GetRsiRequest::new("AAPL")
    .timespan(IndicatorTimespan::Day)
    .window(14)
    .series_type(SeriesType::Close)
    .timestamp_gte("2024-01-01")
    .limit(100);

let response = client.execute(request).await?;

for value in response.results.values {
    println!("RSI: {:.2}", value.value);
    println!("Overbought: {}", value.is_overbought(70.0));
    println!("Oversold: {}", value.is_oversold(30.0));
}
```

#### SMA (Simple Moving Average)

```rust
use massive_rs::rest::endpoints::{GetIndicatorRequest, Sma, IndicatorTimespan};

let request = GetIndicatorRequest::<Sma>::new("AAPL")
    .timespan(IndicatorTimespan::Day)
    .window(50);

let response = client.execute(request).await?;
```

#### EMA (Exponential Moving Average)

```rust
use massive_rs::rest::endpoints::{GetIndicatorRequest, Ema};

let request = GetIndicatorRequest::<Ema>::new("AAPL").window(20);
let response = client.execute(request).await?;
```

#### MACD

```rust
use massive_rs::rest::endpoints::GetMacdRequest;

let request = GetMacdRequest::new("AAPL")
    .short_window(12)
    .long_window(26)
    .signal_window(9);

let response = client.execute(request).await?;

for value in response.results.values {
    println!("MACD: {:.4}, Signal: {:.4}, Histogram: {:.4}",
        value.value, value.signal, value.histogram);
    println!("Bullish crossover: {}", value.is_bullish_crossover());
    println!("Bearish crossover: {}", value.is_bearish_crossover());
}
```

---

### Options

#### Options Contracts List

```rust
use massive_rs::rest::endpoints::{GetOptionsContractsRequest, ContractType};

let request = GetOptionsContractsRequest::new()
    .underlying_ticker("AAPL")
    .contract_type(ContractType::Call)
    .expiration_date_gte("2024-06-01")
    .expiration_date_lte("2024-12-31")
    .strike_price_gte(150.0)
    .strike_price_lte(200.0);

let response = client.execute(request).await?;
```

#### Single Options Contract

```rust
use massive_rs::rest::endpoints::GetOptionsContractRequest;

let request = GetOptionsContractRequest::new("O:AAPL251219C00200000");
let response = client.execute(request).await?;

if let Some(contract) = response.results {
    println!("Strike: ${}", contract.strike_price);
    println!("Expiration: {}", contract.expiration_date);
    println!("Days to expiry: {}", contract.days_to_expiration().unwrap_or(0));
    println!("In the money: {}", contract.is_in_the_money(195.0));
}
```

#### Options Chain

```rust
use massive_rs::rest::endpoints::GetOptionsChainRequest;

let request = GetOptionsChainRequest::new("AAPL")
    .expiration_date("2024-12-20")
    .strike_price_gte(180.0)
    .strike_price_lte(220.0);

let response = client.execute(request).await?;

// Get unique strikes
let strikes = response.unique_strikes();

// Filter by type
let calls = response.calls();
let puts = response.puts();
```

---

### Futures

#### Futures Contracts

```rust
use massive_rs::rest::endpoints::GetFuturesContractsRequest;

let request = GetFuturesContractsRequest::new()
    .product_code("ES")  // E-mini S&P 500
    .active(true);

let response = client.execute(request).await?;

for contract in response.results {
    println!("{}: expires {}", contract.ticker, contract.expiration_date);
    println!("Days to maturity: {}", contract.days_to_maturity.unwrap_or(0));
    println!("Is front month: {}", contract.is_front_month());
}
```

#### Futures Products

```rust
use massive_rs::rest::endpoints::GetFuturesProductsRequest;

let request = GetFuturesProductsRequest::new().asset_class("equity");
let response = client.execute(request).await?;
```

#### Futures Schedules

```rust
use massive_rs::rest::endpoints::GetFuturesSchedulesRequest;

let request = GetFuturesSchedulesRequest::new();
let response = client.execute(request).await?;
```

#### Futures Snapshots

```rust
use massive_rs::rest::endpoints::GetFuturesSnapshotRequest;

let request = GetFuturesSnapshotRequest::new()
    .ticker_any_of(vec!["ESH4".into(), "NQH4".into()]);

let response = client.execute(request).await?;
```

---

### Forex

#### Forex Quote

```rust
use massive_rs::rest::endpoints::GetForexQuoteRequest;

let request = GetForexQuoteRequest::new("EUR", "USD");
let response = client.execute(request).await?;

if let Some(quote) = response.last {
    println!("EUR/USD: {:.5}", quote.ask);
    println!("Spread: {:.1} pips", quote.spread_pips(false));  // false = not JPY pair
}
```

#### Currency Conversion

```rust
use massive_rs::rest::endpoints::GetForexConversionRequest;

let request = GetForexConversionRequest::new("EUR", "USD")
    .amount(1000.0)
    .precision(4);

let response = client.execute(request).await?;

println!("1000 EUR = {:.2} USD", response.converted);
```

#### Forex Snapshot

```rust
use massive_rs::rest::endpoints::GetForexSnapshotRequest;

let request = GetForexSnapshotRequest::new()
    .tickers(vec!["C:EURUSD".into(), "C:GBPUSD".into()]);

let response = client.execute(request).await?;
```

#### Forex Gainers/Losers

```rust
use massive_rs::rest::endpoints::{GetForexGainersLosersRequest, GainersLosersDirection};

let request = GetForexGainersLosersRequest::new(GainersLosersDirection::Gainers);
let response = client.execute(request).await?;
```

---

### Crypto

#### Crypto Snapshot

```rust
use massive_rs::rest::endpoints::GetCryptoSnapshotRequest;

let request = GetCryptoSnapshotRequest::new()
    .tickers(vec!["X:BTCUSD".into(), "X:ETHUSD".into()]);

let response = client.execute(request).await?;
```

#### Crypto Open/Close

```rust
use massive_rs::rest::endpoints::GetCryptoOpenCloseRequest;

let request = GetCryptoOpenCloseRequest::new("BTC", "USD", "2024-01-15");
let response = client.execute(request).await?;
```

#### Crypto L2 Order Book

```rust
use massive_rs::rest::endpoints::GetCryptoL2BookRequest;

let request = GetCryptoL2BookRequest::new("X:BTCUSD");
let response = client.execute(request).await?;

if let Some(book) = response.data {
    println!("Best bid: ${:.2} x {:.4}", book.bids[0].price, book.bids[0].size);
    println!("Best ask: ${:.2} x {:.4}", book.asks[0].price, book.asks[0].size);
    println!("Spread: ${:.2}", book.spread());
    println!("Volume imbalance: {:.2}", book.volume_imbalance());
}
```

#### Crypto Exchanges

```rust
use massive_rs::rest::endpoints::GetCryptoExchangesRequest;

let response = client.execute(GetCryptoExchangesRequest).await?;
```

---

### Fundamentals

#### Balance Sheets

```rust
use massive_rs::rest::endpoints::{GetBalanceSheetsRequest, FinancialTimeframe};

let request = GetBalanceSheetsRequest::new("AAPL")
    .timeframe(FinancialTimeframe::Quarterly)
    .limit(4);

let response = client.execute(request).await?;

for bs in response.results {
    println!("Period: {}", bs.fiscal_period);
    println!("Total Assets: ${}", bs.total_assets());
    println!("Current Ratio: {:.2}", bs.current_ratio().unwrap_or(0.0));
    println!("Debt/Equity: {:.2}", bs.debt_to_equity().unwrap_or(0.0));
}
```

#### Income Statements

```rust
use massive_rs::rest::endpoints::GetIncomeStatementsRequest;

let request = GetIncomeStatementsRequest::new("AAPL")
    .timeframe(FinancialTimeframe::Annual)
    .limit(5);

let response = client.execute(request).await?;

for inc in response.results {
    println!("Revenue: ${}", inc.revenues.basic.unwrap_or(0.0));
    println!("Gross Margin: {:.1}%", inc.gross_margin().unwrap_or(0.0) * 100.0);
    println!("Net Margin: {:.1}%", inc.net_margin().unwrap_or(0.0) * 100.0);
    println!("Operating Margin: {:.1}%", inc.operating_margin().unwrap_or(0.0) * 100.0);
}
```

#### Cash Flow Statements

```rust
use massive_rs::rest::endpoints::GetCashFlowStatementsRequest;

let request = GetCashFlowStatementsRequest::new("AAPL")
    .timeframe(FinancialTimeframe::Quarterly);

let response = client.execute(request).await?;
```

#### Short Interest

```rust
use massive_rs::rest::endpoints::GetShortInterestRequest;

let request = GetShortInterestRequest::new("AAPL");
let response = client.execute(request).await?;

for si in response.results {
    println!("Short interest: {}", si.short_volume);
    println!("Days to cover: {:.1}", si.days_to_cover().unwrap_or(0.0));
    println!("Short ratio: {:.2}%", si.short_interest_ratio().unwrap_or(0.0) * 100.0);
}
```

#### Short Volume

```rust
use massive_rs::rest::endpoints::GetShortVolumeRequest;

let request = GetShortVolumeRequest::new("AAPL")
    .date_gte("2024-01-01")
    .date_lte("2024-01-31");

let response = client.execute(request).await?;
```

---

### Corporate Actions

#### Dividends

```rust
use massive_rs::rest::endpoints::GetDividendsRequest;

let request = GetDividendsRequest::new()
    .ticker("AAPL")
    .ex_dividend_date_gte("2024-01-01")
    .limit(10);

let response = client.execute(request).await?;

for div in response.results {
    println!("Ex-date: {}, Amount: ${:.2}", div.ex_dividend_date, div.cash_amount);
    println!("Frequency: {}", div.frequency.unwrap_or(0));
    println!("Is special: {}", div.is_special());

    // Calculate yield
    if let Some(yield_pct) = div.dividend_yield(175.0) {
        println!("Yield at $175: {:.2}%", yield_pct * 100.0);
    }
}
```

#### Stock Splits

```rust
use massive_rs::rest::endpoints::GetSplitsRequest;

let request = GetSplitsRequest::new()
    .ticker("AAPL")
    .execution_date_gte("2020-01-01");

let response = client.execute(request).await?;

for split in response.results {
    println!("{}: {} for {}", split.ticker, split.split_to, split.split_from);
    println!("Ratio: {:.4}", split.split_ratio());
    println!("Is reverse split: {}", split.is_reverse_split());
}
```

---

### Economic Data

#### Treasury Yields

```rust
use massive_rs::rest::endpoints::GetTreasuryYieldsRequest;

let request = GetTreasuryYieldsRequest::new()
    .date_gte("2024-01-01")
    .limit(30);

let response = client.execute(request).await?;

for yields in response.results {
    println!("2Y: {:.2}%, 10Y: {:.2}%",
        yields.yield_2y.unwrap_or(0.0),
        yields.yield_10y.unwrap_or(0.0));

    // Spread analysis
    if let Some(spread) = yields.spread_2s10s() {
        println!("2s10s spread: {:.2} bps", spread * 100.0);
        println!("Curve inverted: {}", yields.is_2s10s_inverted().unwrap_or(false));
    }
}
```

#### Inflation Data

```rust
use massive_rs::rest::endpoints::GetInflationRequest;

let request = GetInflationRequest::new().date_gte("2023-01-01");
let response = client.execute(request).await?;

for data in response.results {
    println!("CPI: {:.1}%", data.value);
    println!("Above 2% target: {}", data.is_above_target().unwrap_or(false));

    // Real yield calculation
    if let Some(real_yield) = data.real_yield(4.5) {
        println!("Real yield (nominal 4.5%): {:.2}%", real_yield);
    }
}
```

#### Fed Funds Rate

```rust
use massive_rs::rest::endpoints::GetFedFundsRateRequest;

let request = GetFedFundsRateRequest::new().date_gte("2024-01-01");
let response = client.execute(request).await?;

for rate in response.results {
    println!("Effective rate: {:.2}%", rate.rate);
    if let Some(midpoint) = rate.target_midpoint() {
        println!("Target midpoint: {:.3}%", midpoint);
    }
}
```

---

### News

#### News Articles

```rust
use massive_rs::rest::endpoints::GetNewsRequest;

let request = GetNewsRequest::new()
    .ticker("AAPL")
    .published_after("2024-01-01")
    .limit(20);

let response = client.execute(request).await?;

for article in response.results {
    println!("Title: {}", article.title);
    println!("Published: {}", article.published_utc);

    // Sentiment analysis
    if let Some(sentiment) = article.sentiment_for("AAPL") {
        println!("Sentiment: {:?}", sentiment);
    }

    // Check mentioned tickers
    println!("Mentions AAPL: {}", article.mentions("AAPL"));
    println!("Positive tickers: {:?}", article.positive_tickers());
}
```

#### Related Companies

```rust
use massive_rs::rest::endpoints::GetRelatedCompaniesRequest;

let request = GetRelatedCompaniesRequest::new("AAPL");
let response = client.execute(request).await?;
```

#### Ticker Events

```rust
use massive_rs::rest::endpoints::GetTickerEventsRequest;

let request = GetTickerEventsRequest::new("AAPL")
    .types(vec!["stock_split".into(), "dividend".into()]);

let response = client.execute(request).await?;
```

---

### Partner Data

#### Benzinga Earnings

```rust
use massive_rs::rest::endpoints::GetEarningsRequest;

let request = GetEarningsRequest::new()
    .ticker("AAPL")
    .date_gte("2024-01-01");

let response = client.execute(request).await?;

for earnings in response.results {
    println!("EPS Actual: ${:.2}", earnings.eps_actual.unwrap_or(0.0));
    println!("EPS Estimate: ${:.2}", earnings.eps_estimate.unwrap_or(0.0));
    println!("Beat: {}", earnings.beat_eps());
    println!("Surprise %: {:.1}%", earnings.eps_surprise_percent().unwrap_or(0.0) * 100.0);
}
```

#### Analyst Ratings

```rust
use massive_rs::rest::endpoints::GetAnalystRatingsRequest;

let request = GetAnalystRatingsRequest::new()
    .ticker("AAPL")
    .limit(10);

let response = client.execute(request).await?;

for rating in response.results {
    println!("Analyst: {}", rating.analyst.as_deref().unwrap_or("Unknown"));
    println!("Rating: {:?} -> {:?}", rating.rating_prior, rating.rating_current);
    println!("Is upgrade: {}", rating.is_upgrade());
    println!("Price target: ${}", rating.price_target.unwrap_or(0.0));
}
```

#### ETF Profiles

```rust
use massive_rs::rest::endpoints::GetEtfProfilesRequest;

let request = GetEtfProfilesRequest::new().ticker("SPY");
let response = client.execute(request).await?;
```

#### ETF Holdings

```rust
use massive_rs::rest::endpoints::GetEtfHoldingsRequest;

let request = GetEtfHoldingsRequest::new("SPY").limit(100);
let response = client.execute(request).await?;
```

---

## WebSocket Streaming

### Connection Management

```rust
use massive_rs::ws::{WsClient, Subscription};
use massive_rs::config::{WsConfig, Feed, Market};

// Configure for specific market
let config = WsConfig::new("your-api-key")
    .with_feed(Feed::RealTime)
    .with_market(Market::Stocks);

let client = WsClient::new(config)?;
let (handle, stream) = client.connect().await?;

// Check connection status
println!("Authenticated: {}", handle.is_authenticated());
println!("State: {:?}", handle.connection_state());

// Get statistics
let stats = handle.stats();
println!("Messages received: {}", stats.message_count);
println!("Reconnections: {}", stats.reconnect_count);

// Graceful shutdown
handle.close().await?;
```

### Subscription Types

```rust
use massive_rs::ws::Subscription;

// Stocks
Subscription::trade("AAPL")           // T.AAPL
Subscription::quote("AAPL")           // Q.AAPL
Subscription::second_agg("AAPL")      // A.AAPL
Subscription::minute_agg("AAPL")      // AM.AAPL
Subscription::luld("AAPL")            // LULD.AAPL
Subscription::order_imbalance("AAPL") // NOI.AAPL

// Wildcards
Subscription::all_trades()            // T.*
Subscription::all_quotes()            // Q.*
Subscription::all_second_aggs()       // A.*
Subscription::all_minute_aggs()       // AM.*

// Options
Subscription::options_trade("O:AAPL251219C00200000")
Subscription::options_quote("O:AAPL251219C00200000")
Subscription::all_options_trades()    // T.O:*
Subscription::all_options_quotes()    // Q.O:*

// Forex
Subscription::forex_quote("EUR", "USD")      // C.EURUSD
Subscription::forex_minute_agg("EUR", "USD") // CA.EURUSD
Subscription::all_forex_quotes()             // C.*

// Crypto
Subscription::crypto_trade("BTC", "USD")      // XT.BTC-USD
Subscription::crypto_quote("BTC", "USD")      // XQ.BTC-USD
Subscription::crypto_minute_agg("BTC", "USD") // XA.BTC-USD
Subscription::crypto_l2("BTC", "USD")         // XL2.BTC-USD
Subscription::all_crypto_trades()             // XT.*

// Indices
Subscription::index_value("SPX")              // V.I:SPX
Subscription::index_minute_agg("SPX")         // AM.I:SPX
Subscription::all_index_values()              // V.I:*

// Custom
Subscription::raw("CUSTOM.CHANNEL")
```

### Event Types

```rust
use massive_rs::ws::WsEvent;

match event {
    WsEvent::Trade(t) => {
        println!("{} ${:.2} x {} @ {:?}", t.sym, t.p, t.s, t.conditions);
    }
    WsEvent::Quote(q) => {
        println!("{} Bid: ${:.2} x {} Ask: ${:.2} x {}",
            q.sym, q.bp, q.bs, q.ap, q.as_);
    }
    WsEvent::SecondAggregate(a) | WsEvent::MinuteAggregate(a) => {
        println!("{} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{}",
            a.sym, a.o, a.h, a.l, a.c, a.v);
    }
    WsEvent::LimitUpLimitDown(l) => {
        println!("{} Limit Up: ${:.2} Limit Down: ${:.2}",
            l.sym, l.up, l.down);
    }
    WsEvent::OrderImbalance(o) => {
        println!("{} Imbalance: {} @ ${:.2}", o.sym, o.imbalance, o.price);
    }
    WsEvent::IndexValue(v) => {
        println!("Index {}: {:.2}", v.sym, v.value);
    }
    WsEvent::CryptoTrade(t) => {
        println!("Crypto {} ${:.2} x {:.4}", t.pair, t.p, t.s);
    }
    WsEvent::ForexQuote(q) => {
        println!("Forex {}/{} Bid: {:.5} Ask: {:.5}",
            q.from, q.to, q.bid, q.ask);
    }
    WsEvent::Status(s) => {
        println!("Status: {:?} - {:?}", s.status, s.message);
    }
    WsEvent::Unknown => {
        // Forward compatibility for new event types
    }
    _ => {}
}
```

---

## Pagination

### Automatic Streaming

```rust
use futures::StreamExt;

let request = GetTradesRequest::new("AAPL")
    .timestamp_gte("2024-01-15")
    .limit(50000);

// Automatically fetches all pages
let mut stream = client.stream(request);
let mut count = 0;

while let Some(trade) = stream.next().await {
    let trade = trade?;
    count += 1;
}

println!("Total trades: {}", count);
```

### Custom Pagination Mode

```rust
use massive_rs::config::PaginationMode;

// Limit to specific number of items
let stream = client.stream_with_mode(request, PaginationMode::MaxItems(10_000));

// Single page only
let stream = client.stream_with_mode(request, PaginationMode::None);
```

### Collect All Results

```rust
use futures::StreamExt;

let items: Vec<_> = client.stream(request)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
```

---

## Configuration

### REST Client Configuration

```rust
use massive_rs::config::{RestConfig, PaginationMode};
use massive_rs::auth::{ApiKey, AuthMode};
use std::time::Duration;

let config = RestConfig {
    api_key: ApiKey::new("your-api-key"),
    base_url: "https://api.massive.com".parse().unwrap(),
    auth_mode: AuthMode::HeaderBearer,  // or AuthMode::QueryParam
    connect_timeout: Duration::from_secs(10),
    request_timeout: Duration::from_secs(30),
    pagination: PaginationMode::Auto,
    max_retries: 3,
    trace: false,
    user_agent: Some("my-trading-app/1.0".into()),
};

let client = RestClient::new(config)?;
```

### WebSocket Client Configuration

```rust
use massive_rs::config::{
    WsConfig, Feed, Market, ReconnectConfig,
    DispatchConfig, OverflowPolicy, FanoutMode
};
use std::time::Duration;

let config = WsConfig {
    feed: Feed::RealTime,           // or Feed::Delayed
    market: Market::Stocks,         // Stocks, Options, Futures, Forex, Crypto, Indices
    api_key: ApiKey::new("your-key"),
    connect_timeout: Duration::from_secs(10),
    idle_timeout: Duration::from_secs(30),
    ping_interval: Duration::from_secs(15),
    reconnect: ReconnectConfig {
        enabled: true,
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(60),
        max_retries: None,          // Unlimited
        backoff_multiplier: 2.0,
    },
    dispatch: DispatchConfig {
        capacity: 10_000,           // Buffer size
        overflow: OverflowPolicy::DropOldest,  // or DropNewest, ErrorAndClose
        fanout: FanoutMode::SingleConsumer,    // or Broadcast
    },
};
```

---

## Error Handling

```rust
use massive_rs::error::MassiveError;

match client.execute(request).await {
    Ok(response) => {
        // Success
    }
    Err(MassiveError::Auth(msg)) => {
        // API key is empty or invalid
        eprintln!("Authentication error: {}", msg);
    }
    Err(MassiveError::RateLimited { retry_after, request_id }) => {
        // Rate limit hit - wait and retry
        if let Some(duration) = retry_after {
            tokio::time::sleep(duration).await;
        }
    }
    Err(MassiveError::Api(api_error)) => {
        // API returned an error response
        eprintln!("API error: {} ({})", api_error.message.unwrap_or_default(),
            api_error.request_id.unwrap_or_default());
    }
    Err(MassiveError::HttpStatus { status, body, request_id }) => {
        // HTTP error that couldn't be parsed as API error
        eprintln!("HTTP {}: {:?}", status, body);
    }
    Err(MassiveError::Timeout) => {
        // Request timed out
    }
    Err(MassiveError::Deserialize { source, body_snippet }) => {
        // Failed to parse response
        eprintln!("Parse error: {} in {}", source, body_snippet);
    }
    Err(MassiveError::Ws(ws_error)) => {
        // WebSocket-specific error
        eprintln!("WebSocket error: {}", ws_error);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

---

## Observability

### Built-in Statistics

```rust
use massive_rs::metrics::ClientStats;
use std::sync::Arc;

let stats = Arc::new(ClientStats::new());

// After processing...
let snapshot = stats.snapshot();
println!("Messages received: {}", snapshot.messages_received);
println!("Messages dropped: {}", snapshot.messages_dropped);
println!("Bytes received: {}", snapshot.bytes_received);
println!("Parse errors: {}", snapshot.parse_errors);
println!("Reconnections: {}", snapshot.reconnections);
println!("Requests sent: {}", snapshot.requests_sent);
println!("Request errors: {}", snapshot.request_errors);
println!("Rate limits: {}", snapshot.rate_limits);

// Reset counters
stats.reset();
```

### Custom Metrics Integration

```rust
use massive_rs::metrics::MetricsSink;

struct PrometheusMetrics {
    // Your Prometheus registry
}

impl MetricsSink for PrometheusMetrics {
    fn counter(&self, name: &'static str, value: u64, tags: &[(&'static str, &str)]) {
        // Record to Prometheus counter
    }

    fn gauge(&self, name: &'static str, value: i64, tags: &[(&'static str, &str)]) {
        // Record to Prometheus gauge
    }

    fn histogram(&self, name: &'static str, value: f64, tags: &[(&'static str, &str)]) {
        // Record to Prometheus histogram
    }
}
```

### Tracing Metrics

```rust
use massive_rs::metrics::TracingMetrics;

// Outputs metrics via tracing framework
let metrics = TracingMetrics;
```

---

## Performance

The crate is optimized for HFT and algorithmic trading:

| Metric | Standard | With SIMD |
|--------|----------|-----------|
| Trade parsing | ~1-2 μs | ~0.3-0.5 μs |
| Quote parsing | ~1-2 μs | ~0.3-0.5 μs |
| Batch (100 msgs) | ~100 μs | ~30 μs |
| Throughput | 100k+ msg/s | 300k+ msg/s |

### Benchmarks

```bash
cargo bench --bench json_parsing
```

### High-Performance Parsing

```rust
use massive_rs::parse::{parse_ws_events, parse_ws_events_bytes};

// Standard parsing
let events = parse_ws_events(json_text)?;

// Zero-copy byte parsing (faster)
let events = parse_ws_events_bytes(json_bytes)?;

// Quick event type extraction (no full parse)
let event_type = extract_event_type(json_text);

// Estimate batch size (heuristic)
let count = estimate_event_count(json_text);
```

---

## Feature Flags

```toml
[features]
default = ["rustls", "gzip", "ws"]

# TLS backends (choose one)
rustls = ["reqwest/rustls-tls", "tokio-tungstenite/rustls-tls-native-roots"]
native-tls = ["reqwest/native-tls", "tokio-tungstenite/native-tls"]

# HTTP compression
gzip = ["reqwest/gzip"]

# WebSocket support
ws = ["dep:tokio-tungstenite", "dep:dashmap"]

# Performance optimizations
simd-json = ["dep:simd-json"]  # SIMD-accelerated JSON parsing

# Precision numerics
decimal = ["dep:rust_decimal"]  # Exact decimal arithmetic for financial calcs
```

### Minimal Build (REST only)

```toml
[dependencies]
massive-rs = { version = "0.1", default-features = false, features = ["rustls", "gzip"] }
```

### Maximum Performance

```toml
[dependencies]
massive-rs = { version = "0.1", features = ["simd-json"] }
```

---

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `MASSIVE_API_KEY` | API key for authentication | Yes (or provide via config) |

---

## Examples

See the `examples/` directory:

```bash
# REST examples
MASSIVE_API_KEY=your-key cargo run --example rest_aggregates
MASSIVE_API_KEY=your-key cargo run --example rest_options_chain

# WebSocket examples
MASSIVE_API_KEY=your-key cargo run --example ws_realtime_trades
MASSIVE_API_KEY=your-key cargo run --example ws_multi_asset
```

---

## API Coverage Summary

### REST Endpoints (56 total)

| Category | Endpoints |
|----------|-----------|
| Market Data | Aggregates, Previous Close, Daily Open/Close |
| Trades | Historical Trades, Last Trade |
| Quotes | Historical Quotes, Last Quote |
| Snapshots | Ticker, All Tickers, Gainers/Losers, Unified, Grouped Daily |
| Reference | Tickers, Details, Exchanges, Conditions, Types, Holidays, Market Status |
| Indicators | RSI, SMA, EMA, MACD |
| Options | Contracts List, Contract Details, Chain |
| Futures | Contracts, Products, Schedules, Snapshots |
| Forex | Quote, Conversion, Snapshot, Gainers/Losers |
| Crypto | Snapshot, Open/Close, L2 Book, Exchanges, Gainers/Losers |
| Fundamentals | Balance Sheets, Income Statements, Cash Flow, Short Interest/Volume |
| Corporate Actions | Dividends, Stock Splits |
| Economy | Treasury Yields, Inflation, Fed Funds Rate |
| News | Articles, Related Companies, Ticker Events |
| Partner | Earnings, Analyst Ratings, ETF Profiles/Holdings |

### WebSocket Channels (16 event types)

| Asset Class | Channels |
|-------------|----------|
| Stocks | Trades, Quotes, Second/Minute Aggs, LULD, Order Imbalance, FMV |
| Options | Trades, Quotes, Second/Minute Aggs |
| Forex | Quotes, Minute Aggs |
| Crypto | Trades, Quotes, Aggs, L2 Book |
| Indices | Values, Second/Minute Aggs |

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
