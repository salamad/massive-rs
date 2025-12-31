//! REST API Integration Tests
//!
//! This module contains comprehensive integration tests for all REST API endpoints.
//! Tests require the `MASSIVE_API_KEY` environment variable to be set.
//!
//! Run with: `cargo test --test rest_integration --features ws`

mod common;

use common::{
    create_rest_client, has_api_key, test_date, test_date_range, TEST_TICKER, TEST_TICKER_ALT,
};
use futures::StreamExt;
use massive_rs::config::PaginationMode;
use massive_rs::rest::endpoints::{
    GetAggsRequest, GetAllTickersSnapshotRequest, GetDailyOpenCloseRequest,
    GetGainersLosersRequest, GetLastQuoteRequest, GetLastTradeRequest, GetMarketStatusRequest,
    GetPreviousCloseRequest, GetQuotesRequest, GetRsiRequest, GetTickerDetailsRequest,
    GetTickerSnapshotRequest, GetTickersRequest, GetTradesRequest, IndicatorTimespan, MarketType,
    Order, SeriesType, Sort, Timespan,
};

// ============================================================================
// Market Data Endpoints
// ============================================================================

/// Test GetAggsRequest - Aggregate bars (OHLCV data)
#[tokio::test]
async fn test_get_aggregates() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let (from, to) = test_date_range();

    let request = GetAggsRequest::new(TEST_TICKER)
        .multiplier(1)
        .timespan(Timespan::Day)
        .from(&from)
        .to(&to)
        .adjusted(true)
        .sort(Sort::Asc)
        .limit(50);

    let response = client.execute(request).await;

    match response {
        Ok(aggs) => {
            assert!(
                aggs.status.as_deref() == Some("OK") || aggs.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status, got: {:?}",
                aggs.status
            );
            assert!(
                !aggs.results.is_empty(),
                "Expected at least one aggregate bar"
            );

            // Verify first bar has valid OHLCV data
            let first = &aggs.results[0];
            assert!(first.open > 0.0, "Open price should be positive");
            assert!(first.high > 0.0, "High price should be positive");
            assert!(first.low > 0.0, "Low price should be positive");
            assert!(first.close > 0.0, "Close price should be positive");
            assert!(first.high >= first.low, "High should be >= Low");
            assert!(first.high >= first.open, "High should be >= Open");
            assert!(first.high >= first.close, "High should be >= Close");
            assert!(first.low <= first.open, "Low should be <= Open");
            assert!(first.low <= first.close, "Low should be <= Close");

            println!(
                "Successfully fetched {} aggregate bars for {}",
                aggs.results.len(),
                TEST_TICKER
            );
        }
        Err(e) => {
            panic!("Failed to fetch aggregates: {:?}", e);
        }
    }
}

/// Test different timespan values for aggregates
#[tokio::test]
async fn test_get_aggregates_different_timespans() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let (from, to) = test_date_range();

    let timespans = [
        (Timespan::Minute, 5),
        (Timespan::Hour, 1),
        (Timespan::Day, 1),
        (Timespan::Week, 1),
    ];

    for (timespan, multiplier) in timespans {
        let request = GetAggsRequest::new(TEST_TICKER)
            .multiplier(multiplier)
            .timespan(timespan)
            .from(&from)
            .to(&to)
            .limit(10);

        let response = client.execute(request).await;

        assert!(
            response.is_ok(),
            "Failed for timespan {:?}: {:?}",
            timespan,
            response.err()
        );
        println!("Successfully tested timespan {:?}", timespan);
    }
}

/// Test GetPreviousCloseRequest - Previous day's OHLC data
#[tokio::test]
async fn test_get_previous_close() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetPreviousCloseRequest::new(TEST_TICKER).adjusted(true);

    let response = client.execute(request).await;

    match response {
        Ok(prev_close) => {
            assert!(
                prev_close.status.as_deref() == Some("OK")
                    || prev_close.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            if !prev_close.results.is_empty() {
                let bar = &prev_close.results[0];
                assert!(bar.close > 0.0, "Close price should be positive");
                println!("Previous close for {}: ${:.2}", TEST_TICKER, bar.close);
            }
        }
        Err(e) => {
            panic!("Failed to fetch previous close: {:?}", e);
        }
    }
}

/// Test GetDailyOpenCloseRequest - Daily open/close for a specific date
#[tokio::test]
async fn test_get_daily_open_close() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    // Use a recent trading day - a few days ago to ensure data is available
    let recent_date = chrono::Utc::now() - chrono::Duration::days(5);
    let date = recent_date.format("%Y-%m-%d").to_string();

    let request = GetDailyOpenCloseRequest::new(TEST_TICKER, &date).adjusted(true);

    let response = client.execute(request).await;

    match response {
        Ok(daily) => {
            assert_eq!(daily.status, "OK", "Expected OK status");
            assert_eq!(daily.symbol, TEST_TICKER, "Symbol should match");
            assert!(daily.open > 0.0, "Open price should be positive");
            assert!(daily.close > 0.0, "Close price should be positive");
            assert!(daily.high >= daily.low, "High should be >= Low");

            println!(
                "Daily open/close for {} on {}: Open=${:.2}, Close=${:.2}",
                TEST_TICKER, date, daily.open, daily.close
            );
        }
        Err(e) => {
            // NOT_FOUND is acceptable for weekends/holidays or delayed data availability
            let error_str = format!("{:?}", e);
            if error_str.contains("NOT_FOUND") || error_str.contains("Data not found") {
                println!(
                    "Daily open/close for {} on {} not available (may be weekend/holiday): {:?}",
                    TEST_TICKER, date, e
                );
            } else {
                panic!("Failed to fetch daily open/close: {:?}", e);
            }
        }
    }
}

// ============================================================================
// Trade Endpoints
// ============================================================================

/// Test GetTradesRequest - Historical trades
#[tokio::test]
async fn test_get_trades() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let date = test_date();

    let request = GetTradesRequest::new(TEST_TICKER)
        .timestamp_gte(&date)
        .order("asc")
        .limit(100);

    let response = client.execute(request).await;

    match response {
        Ok(trades) => {
            assert!(
                trades.status.as_deref() == Some("OK")
                    || trades.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            if !trades.results.is_empty() {
                let first_trade = &trades.results[0];
                assert!(first_trade.price > 0.0, "Trade price should be positive");
                assert!(first_trade.size > 0, "Trade size should be positive");

                // Test the value() method
                let value = first_trade.value();
                assert!(value > 0.0, "Trade value should be positive");

                println!(
                    "Fetched {} trades, first trade: ${:.2} x {} (value: ${:.2})",
                    trades.results.len(),
                    first_trade.price,
                    first_trade.size,
                    value
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch trades: {:?}", e);
        }
    }
}

/// Test GetLastTradeRequest - Last trade for a ticker
#[tokio::test]
async fn test_get_last_trade() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetLastTradeRequest::new(TEST_TICKER);

    let response = client.execute(request).await;

    match response {
        Ok(last_trade) => {
            assert!(
                last_trade.status.as_deref() == Some("OK")
                    || last_trade.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            if let Some(trade) = last_trade.results {
                assert!(trade.price > 0.0, "Trade price should be positive");
                println!("Last trade for {}: ${:.2}", TEST_TICKER, trade.price);
            }
        }
        Err(e) => {
            panic!("Failed to fetch last trade: {:?}", e);
        }
    }
}

// ============================================================================
// Quote Endpoints
// ============================================================================

/// Test GetQuotesRequest - Historical quotes (NBBO)
#[tokio::test]
async fn test_get_quotes() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let date = test_date();

    let request = GetQuotesRequest::new(TEST_TICKER)
        .timestamp_gte(&date)
        .order("asc")
        .limit(100);

    let response = client.execute(request).await;

    match response {
        Ok(quotes) => {
            assert!(
                quotes.status.as_deref() == Some("OK")
                    || quotes.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            if !quotes.results.is_empty() {
                let first_quote = &quotes.results[0];
                assert!(
                    first_quote.bid_price >= 0.0,
                    "Bid price should be non-negative"
                );
                assert!(
                    first_quote.ask_price >= 0.0,
                    "Ask price should be non-negative"
                );

                // Test quote helper methods
                let spread = first_quote.spread();
                let mid = first_quote.mid_price();

                println!(
                    "Fetched {} quotes, first: Bid=${:.2}, Ask=${:.2}, Spread=${:.4}, Mid=${:.2}",
                    quotes.results.len(),
                    first_quote.bid_price,
                    first_quote.ask_price,
                    spread,
                    mid
                );

                // Verify helper method calculations
                if first_quote.ask_price > 0.0 && first_quote.bid_price > 0.0 {
                    assert!(
                        (mid - (first_quote.bid_price + first_quote.ask_price) / 2.0).abs() < 0.001,
                        "Mid price calculation should be correct"
                    );
                }
            }
        }
        Err(e) => {
            panic!("Failed to fetch quotes: {:?}", e);
        }
    }
}

/// Test GetLastQuoteRequest - Last quote (NBBO) for a ticker
#[tokio::test]
async fn test_get_last_quote() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetLastQuoteRequest::new(TEST_TICKER);

    let response = client.execute(request).await;

    match response {
        Ok(last_quote) => {
            assert!(
                last_quote.status.as_deref() == Some("OK")
                    || last_quote.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            if let Some(quote) = last_quote.results {
                assert!(quote.bid_price >= 0.0, "Bid price should be non-negative");
                assert!(quote.ask_price >= 0.0, "Ask price should be non-negative");

                println!(
                    "Last quote for {}: Bid=${:.2}, Ask=${:.2}",
                    TEST_TICKER, quote.bid_price, quote.ask_price
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch last quote: {:?}", e);
        }
    }
}

/// Test Quote helper methods (is_crossed, is_locked)
#[tokio::test]
async fn test_quote_helper_methods() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetLastQuoteRequest::new(TEST_TICKER);

    let response = client.execute(request).await;

    if let Ok(last_quote) = response {
        if let Some(quote) = last_quote.results {
            // Test is_crossed and is_locked
            let is_crossed = quote.is_crossed();
            let is_locked = quote.is_locked();

            // For a normal market, the quote should not be crossed or locked
            println!(
                "Quote status - Crossed: {}, Locked: {}",
                is_crossed, is_locked
            );
        }
    }
}

// ============================================================================
// Snapshot Endpoints
// ============================================================================

/// Test GetTickerSnapshotRequest - Single ticker snapshot
#[tokio::test]
async fn test_get_ticker_snapshot() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetTickerSnapshotRequest::new("stocks", TEST_TICKER);

    let response = client.execute(request).await;

    match response {
        Ok(snapshot) => {
            assert!(
                snapshot.status.as_deref() == Some("OK")
                    || snapshot.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            if let Some(ticker_snapshot) = snapshot.ticker {
                assert_eq!(ticker_snapshot.ticker, TEST_TICKER, "Ticker should match");

                if let Some(day) = ticker_snapshot.day {
                    assert!(day.open > 0.0, "Open should be positive");
                    assert!(day.close > 0.0, "Close should be positive");
                    println!(
                        "Snapshot for {}: Open=${:.2}, Close=${:.2}",
                        TEST_TICKER, day.open, day.close
                    );
                }

                if let Some(change_perc) = ticker_snapshot.todays_change_perc {
                    println!("Today's change: {:.2}%", change_perc);
                }
            }
        }
        Err(e) => {
            panic!("Failed to fetch ticker snapshot: {:?}", e);
        }
    }
}

/// Test GetAllTickersSnapshotRequest - Multiple tickers snapshot
#[tokio::test]
async fn test_get_all_tickers_snapshot() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request =
        GetAllTickersSnapshotRequest::new("stocks").tickers(&[TEST_TICKER, TEST_TICKER_ALT]);

    let response = client.execute(request).await;

    match response {
        Ok(snapshot) => {
            assert!(
                snapshot.status.as_deref() == Some("OK")
                    || snapshot.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            println!("Got {} ticker snapshots", snapshot.tickers.len());

            for ticker in &snapshot.tickers {
                println!(
                    "  - {} (change: {:?}%)",
                    ticker.ticker, ticker.todays_change_perc
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch all tickers snapshot: {:?}", e);
        }
    }
}

/// Test GetGainersLosersRequest - Top gainers
#[tokio::test]
async fn test_get_gainers() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetGainersLosersRequest::gainers("stocks");

    let response = client.execute(request).await;

    match response {
        Ok(gainers) => {
            assert!(
                gainers.status.as_deref() == Some("OK")
                    || gainers.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            println!("Top gainers ({} tickers):", gainers.tickers.len());
            for ticker in gainers.tickers.iter().take(5) {
                println!(
                    "  - {} (+{:.2}%)",
                    ticker.ticker,
                    ticker.todays_change_perc.unwrap_or(0.0)
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch gainers: {:?}", e);
        }
    }
}

/// Test GetGainersLosersRequest - Top losers
#[tokio::test]
async fn test_get_losers() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetGainersLosersRequest::losers("stocks");

    let response = client.execute(request).await;

    match response {
        Ok(losers) => {
            assert!(
                losers.status.as_deref() == Some("OK")
                    || losers.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status"
            );

            println!("Top losers ({} tickers):", losers.tickers.len());
            for ticker in losers.tickers.iter().take(5) {
                println!(
                    "  - {} ({:.2}%)",
                    ticker.ticker,
                    ticker.todays_change_perc.unwrap_or(0.0)
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch losers: {:?}", e);
        }
    }
}

// ============================================================================
// Reference Data Endpoints
// ============================================================================

/// Test GetTickersRequest - List tickers
#[tokio::test]
async fn test_get_tickers() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetTickersRequest::default()
        .market(MarketType::Stocks)
        .active(true)
        .search("Apple")
        .limit(10);

    let response = client.execute(request).await;

    match response {
        Ok(tickers) => {
            assert!(
                tickers.status.as_deref() == Some("OK"),
                "Expected OK status, got: {:?}",
                tickers.status
            );
            assert!(!tickers.results.is_empty(), "Expected at least one ticker");

            println!("Found {} tickers matching 'Apple':", tickers.results.len());
            for ticker in &tickers.results {
                println!(
                    "  - {} ({}): {}",
                    ticker.ticker, &ticker.market, &ticker.name
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch tickers: {:?}", e);
        }
    }
}

/// Test GetTickersRequest with market type filters
#[tokio::test]
async fn test_get_tickers_by_market() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let markets = [MarketType::Stocks, MarketType::Crypto, MarketType::Forex];

    for market in markets {
        let request = GetTickersRequest::default().market(market).limit(5);

        let response = client.execute(request).await;

        match response {
            Ok(tickers) => {
                println!(
                    "Market {:?}: found {} tickers",
                    market,
                    tickers.results.len()
                );
            }
            Err(e) => {
                // Some markets might require different subscription tiers
                println!("Market {:?}: {:?}", market, e);
            }
        }
    }
}

/// Test GetTickerDetailsRequest - Ticker details
#[tokio::test]
async fn test_get_ticker_details() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetTickerDetailsRequest::new(TEST_TICKER);

    let response = client.execute(request).await;

    match response {
        Ok(details) => {
            assert_eq!(details.status, "OK", "Expected OK status");
            assert_eq!(details.results.ticker, TEST_TICKER, "Ticker should match");
            assert!(!details.results.name.is_empty(), "Name should not be empty");
            assert!(details.results.active, "Ticker should be active");

            println!("Ticker details for {}:", TEST_TICKER);
            println!("  Name: {}", details.results.name);
            println!("  Market: {}", details.results.market);
            println!("  Locale: {}", details.results.locale);
            println!("  Primary Exchange: {:?}", details.results.primary_exchange);
            println!("  Type: {:?}", details.results.ticker_type);
            println!("  Active: {}", details.results.active);
            if let Some(market_cap) = details.results.market_cap {
                println!("  Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
            }
            if let Some(description) = &details.results.description {
                println!(
                    "  Description: {}...",
                    description.chars().take(100).collect::<String>()
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch ticker details: {:?}", e);
        }
    }
}

// ============================================================================
// Pagination Tests
// ============================================================================

/// Test streaming paginated results
#[tokio::test]
async fn test_pagination_streaming() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let (from, to) = test_date_range();

    let request = GetAggsRequest::new(TEST_TICKER)
        .multiplier(1)
        .timespan(Timespan::Day)
        .from(&from)
        .to(&to)
        .limit(10); // Small limit to encourage pagination

    let mut stream = client.stream_with_mode(request, PaginationMode::MaxItems(25));
    let mut count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(_bar) => {
                count += 1;
            }
            Err(e) => {
                panic!("Error during pagination: {:?}", e);
            }
        }
    }

    println!("Streamed {} bars with pagination", count);
    assert!(count > 0, "Expected at least one result from pagination");
}

/// Test trade pagination
#[tokio::test]
async fn test_trade_pagination() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let date = test_date();

    let request = GetTradesRequest::new(TEST_TICKER)
        .timestamp_gte(&date)
        .limit(50);

    let mut stream = client.stream_with_mode(request, PaginationMode::MaxItems(100));
    let mut count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(_trade) => {
                count += 1;
            }
            Err(e) => {
                panic!("Error during trade pagination: {:?}", e);
            }
        }
    }

    println!("Streamed {} trades with pagination", count);
    assert!(count > 0, "Expected at least one trade from pagination");
}

/// Test ticker pagination
#[tokio::test]
async fn test_ticker_pagination() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetTickersRequest::default()
        .market(MarketType::Stocks)
        .active(true)
        .limit(10);

    let mut stream = client.stream_with_mode(request, PaginationMode::MaxItems(30));
    let mut count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(_ticker) => {
                count += 1;
            }
            Err(e) => {
                panic!("Error during ticker pagination: {:?}", e);
            }
        }
    }

    println!("Streamed {} tickers with pagination", count);
    assert!(count > 0, "Expected at least one ticker from pagination");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test handling of invalid ticker symbols
#[tokio::test]
async fn test_invalid_ticker() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let (from, to) = test_date_range();

    // Use a ticker that definitely doesn't exist
    let request = GetAggsRequest::new("INVALIDTICKER12345")
        .multiplier(1)
        .timespan(Timespan::Day)
        .from(&from)
        .to(&to);

    let response = client.execute(request).await;

    // API should return OK but with empty results, or a specific error
    match response {
        Ok(aggs) => {
            println!(
                "Invalid ticker returned {} results (status: {:?})",
                aggs.results.len(),
                aggs.status
            );
        }
        Err(e) => {
            println!("Invalid ticker error (expected): {:?}", e);
        }
    }
}

/// Test handling of invalid date range
#[tokio::test]
async fn test_invalid_date_range() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    // Future dates should return no data
    let request = GetAggsRequest::new(TEST_TICKER)
        .multiplier(1)
        .timespan(Timespan::Day)
        .from("2099-01-01")
        .to("2099-12-31");

    let response = client.execute(request).await;

    match response {
        Ok(aggs) => {
            assert!(
                aggs.results.is_empty(),
                "Future dates should return empty results"
            );
            println!("Future date range returned empty results as expected");
        }
        Err(e) => {
            println!("Future date range error: {:?}", e);
        }
    }
}

// ============================================================================
// Model Method Tests (using live data)
// ============================================================================

/// Test AggregateBar helper methods
#[tokio::test]
async fn test_aggregate_bar_methods() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let (from, to) = test_date_range();

    let request = GetAggsRequest::new(TEST_TICKER)
        .multiplier(1)
        .timespan(Timespan::Day)
        .from(&from)
        .to(&to)
        .limit(20);

    let response = client.execute(request).await;

    if let Ok(aggs) = response {
        for (i, bar) in aggs.results.iter().take(5).enumerate() {
            let range = bar.range();
            let body = bar.body();
            let is_bullish = bar.is_bullish();
            let is_bearish = bar.is_bearish();

            println!(
                "Bar {}: Range={:.2}, Body={:.2}, Bullish={}, Bearish={}",
                i, range, body, is_bullish, is_bearish
            );

            assert!(range >= 0.0, "Range should be non-negative");
            assert!(body.abs() <= range + 0.001, "Body should be <= range");

            // A bar should be bullish XOR bearish (or neither if doji)
            assert!(
                !(is_bullish && is_bearish),
                "Bar cannot be both bullish and bearish"
            );
        }
    }
}

// ============================================================================
// Concurrent Request Tests
// ============================================================================

/// Test multiple concurrent requests
#[tokio::test]
async fn test_concurrent_requests() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    // Create multiple requests
    let request1 = GetLastTradeRequest::new(TEST_TICKER);
    let request2 = GetLastQuoteRequest::new(TEST_TICKER);
    let request3 = GetPreviousCloseRequest::new(TEST_TICKER);

    // Execute concurrently
    let (result1, result2, result3) = tokio::join!(
        client.execute(request1),
        client.execute(request2),
        client.execute(request3)
    );

    // All should succeed
    assert!(result1.is_ok(), "Last trade request should succeed");
    assert!(result2.is_ok(), "Last quote request should succeed");
    assert!(result3.is_ok(), "Previous close request should succeed");

    println!("All three concurrent requests succeeded");
}

/// Test concurrent requests for multiple tickers
#[tokio::test]
async fn test_concurrent_multi_ticker() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let tickers = [TEST_TICKER, TEST_TICKER_ALT, "GOOG", "AMZN", "META"];

    let futures: Vec<_> = tickers
        .iter()
        .map(|ticker| {
            let client = client.clone();
            let ticker = ticker.to_string();
            async move {
                let request = GetLastTradeRequest::new(&ticker);
                let result = client.execute(request).await;
                (ticker, result)
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    let success_count = results.iter().filter(|(_, result)| result.is_ok()).count();

    println!(
        "Concurrent multi-ticker: {}/{} succeeded",
        success_count,
        tickers.len()
    );

    for (ticker, result) in &results {
        match result {
            Ok(_) => println!("  {} - OK", ticker),
            Err(e) => println!("  {} - Error: {:?}", ticker, e),
        }
    }

    assert!(success_count > 0, "At least some requests should succeed");
}

// ============================================================================
// Market Status Endpoints
// ============================================================================

/// Test GetMarketStatusRequest - Current market status
#[tokio::test]
async fn test_get_market_status() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetMarketStatusRequest;

    let response = client.execute(request).await;

    match response {
        Ok(status) => {
            // Verify we got valid market status
            assert!(
                !status.market.is_empty(),
                "Market status should not be empty"
            );
            assert!(
                !status.server_time.is_empty(),
                "Server time should not be empty"
            );

            println!("Market Status:");
            println!("  Overall market: {}", status.market);
            println!("  After hours: {}", status.after_hours);
            println!("  Early hours: {}", status.early_hours);
            println!("  Server time: {}", status.server_time);
            println!("  Exchanges:");
            println!("    NYSE: {}", status.exchanges.nyse);
            println!("    NASDAQ: {}", status.exchanges.nasdaq);
            println!("    OTC: {}", status.exchanges.otc);
            println!("  Currencies:");
            println!("    Crypto: {}", status.currencies.crypto);
            println!("    Forex: {}", status.currencies.fx);

            if let Some(indices) = &status.indices_groups {
                println!("  Indices Groups:");
                if let Some(dow) = &indices.dow_jones {
                    println!("    Dow Jones: {}", dow);
                }
                if let Some(sp) = &indices.s_and_p {
                    println!("    S&P: {}", sp);
                }
                if let Some(nasdaq) = &indices.nasdaq {
                    println!("    NASDAQ: {}", nasdaq);
                }
            }
        }
        Err(e) => {
            panic!("Failed to fetch market status: {:?}", e);
        }
    }
}

/// Test market status fields are valid values
#[tokio::test]
async fn test_market_status_valid_values() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let response = client.execute(GetMarketStatusRequest).await;

    if let Ok(status) = response {
        // Valid market status values
        let valid_statuses = ["open", "closed", "extended-hours"];

        // Check that market status is one of the expected values
        let market_status_valid = valid_statuses.contains(&status.market.as_str())
            || status.market.starts_with("extended"); // Some variations like "extended-hours"

        println!(
            "Market status '{}' valid: {}",
            status.market, market_status_valid
        );

        // Exchange statuses should be "open" or "closed"
        let exchange_valid = ["open", "closed"];
        assert!(
            exchange_valid.contains(&status.exchanges.nyse.as_str()),
            "NYSE status should be 'open' or 'closed', got: {}",
            status.exchanges.nyse
        );
        assert!(
            exchange_valid.contains(&status.exchanges.nasdaq.as_str()),
            "NASDAQ status should be 'open' or 'closed', got: {}",
            status.exchanges.nasdaq
        );
    }
}

// ============================================================================
// Technical Indicator Endpoints
// ============================================================================

/// Test GetRsiRequest - RSI indicator
#[tokio::test]
async fn test_get_rsi() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetRsiRequest::new(TEST_TICKER)
        .timespan(IndicatorTimespan::Day)
        .window(14)
        .series_type(SeriesType::Close)
        .order(Order::Desc)
        .limit(30);

    let response = client.execute(request).await;

    match response {
        Ok(rsi) => {
            assert!(
                rsi.status.as_deref() == Some("OK") || rsi.status.as_deref() == Some("DELAYED"),
                "Expected OK or DELAYED status, got: {:?}",
                rsi.status
            );

            assert!(
                !rsi.results.values.is_empty(),
                "Expected at least one RSI value"
            );

            println!("RSI values for {} (14-day):", TEST_TICKER);
            for (i, value) in rsi.results.values.iter().take(5).enumerate() {
                let condition = if value.is_oversold() {
                    "OVERSOLD"
                } else if value.is_overbought() {
                    "OVERBOUGHT"
                } else {
                    "neutral"
                };
                println!(
                    "  {}: timestamp={}, RSI={:.2} ({})",
                    i, value.timestamp, value.value, condition
                );
            }

            // Verify RSI values are in valid range (0-100)
            for value in &rsi.results.values {
                assert!(
                    value.value >= 0.0 && value.value <= 100.0,
                    "RSI value should be between 0 and 100, got: {}",
                    value.value
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch RSI: {:?}", e);
        }
    }
}

/// Test RSI with different timespans
#[tokio::test]
async fn test_get_rsi_different_timespans() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let timespans = [
        IndicatorTimespan::Hour,
        IndicatorTimespan::Day,
        IndicatorTimespan::Week,
    ];

    for timespan in timespans {
        let request = GetRsiRequest::new(TEST_TICKER)
            .timespan(timespan)
            .window(14)
            .limit(5);

        let response = client.execute(request).await;

        match response {
            Ok(rsi) => {
                println!(
                    "RSI timespan {:?}: {} values",
                    timespan,
                    rsi.results.values.len()
                );
            }
            Err(e) => {
                // Some timespans might have limited data
                println!("RSI timespan {:?}: {:?}", timespan, e);
            }
        }
    }
}

/// Test RSI with different window sizes
#[tokio::test]
async fn test_get_rsi_different_windows() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let windows = [7, 14, 21]; // Common RSI periods

    for window in windows {
        let request = GetRsiRequest::new(TEST_TICKER)
            .timespan(IndicatorTimespan::Day)
            .window(window)
            .limit(5);

        let response = client.execute(request).await;

        match response {
            Ok(rsi) => {
                if !rsi.results.values.is_empty() {
                    println!(
                        "RSI window {}: latest value = {:.2}",
                        window, rsi.results.values[0].value
                    );
                }
            }
            Err(e) => {
                panic!("Failed to fetch RSI with window {}: {:?}", window, e);
            }
        }
    }
}

/// Test RSI with series type variations
#[tokio::test]
async fn test_get_rsi_series_types() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let series_types = [
        SeriesType::Close,
        SeriesType::Open,
        SeriesType::High,
        SeriesType::Low,
    ];

    for series_type in series_types {
        let request = GetRsiRequest::new(TEST_TICKER)
            .timespan(IndicatorTimespan::Day)
            .window(14)
            .series_type(series_type)
            .limit(5);

        let response = client.execute(request).await;

        match response {
            Ok(rsi) => {
                if !rsi.results.values.is_empty() {
                    println!(
                        "RSI series {:?}: latest = {:.2}",
                        series_type, rsi.results.values[0].value
                    );
                }
            }
            Err(e) => {
                println!("RSI series {:?}: {:?}", series_type, e);
            }
        }
    }
}

/// Test RSI with timestamp filters
#[tokio::test]
async fn test_get_rsi_with_date_range() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();
    let (from, to) = test_date_range();

    let request = GetRsiRequest::new(TEST_TICKER)
        .timespan(IndicatorTimespan::Day)
        .window(14)
        .timestamp_gte(&from)
        .timestamp_lte(&to)
        .order(Order::Asc)
        .limit(50);

    let response = client.execute(request).await;

    match response {
        Ok(rsi) => {
            println!(
                "RSI for {} from {} to {}: {} values",
                TEST_TICKER,
                from,
                to,
                rsi.results.values.len()
            );

            // Verify all values are in valid range
            for value in &rsi.results.values {
                assert!(
                    value.value >= 0.0 && value.value <= 100.0,
                    "RSI should be 0-100"
                );
            }
        }
        Err(e) => {
            panic!("Failed to fetch RSI with date range: {:?}", e);
        }
    }
}

/// Test RSI pagination
#[tokio::test]
async fn test_get_rsi_pagination() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetRsiRequest::new(TEST_TICKER)
        .timespan(IndicatorTimespan::Day)
        .window(14)
        .limit(10); // Small limit to encourage pagination

    let mut stream = client.stream_with_mode(request, PaginationMode::MaxItems(25));
    let mut count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => {
                count += 1;
                // Verify each RSI value is valid
                assert!(
                    value.value >= 0.0 && value.value <= 100.0,
                    "RSI should be 0-100"
                );
            }
            Err(e) => {
                panic!("Error during RSI pagination: {:?}", e);
            }
        }
    }

    println!("Streamed {} RSI values with pagination", count);
    assert!(count > 0, "Expected at least one RSI value from pagination");
}

/// Test RSI helper methods with live data
#[tokio::test]
async fn test_rsi_helper_methods() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetRsiRequest::new(TEST_TICKER)
        .timespan(IndicatorTimespan::Day)
        .window(14)
        .limit(100); // Get enough data to likely find different conditions

    let response = client.execute(request).await;

    if let Ok(rsi) = response {
        let mut oversold_count = 0;
        let mut overbought_count = 0;
        let mut neutral_count = 0;

        for value in &rsi.results.values {
            if value.is_oversold() {
                oversold_count += 1;
            } else if value.is_overbought() {
                overbought_count += 1;
            } else if value.is_neutral() {
                neutral_count += 1;
            }
        }

        println!(
            "RSI conditions analysis for {} values:",
            rsi.results.values.len()
        );
        println!("  Oversold (< 30): {}", oversold_count);
        println!("  Overbought (> 70): {}", overbought_count);
        println!("  Neutral (30-70): {}", neutral_count);

        // Verify totals add up
        assert_eq!(
            oversold_count + overbought_count + neutral_count,
            rsi.results.values.len(),
            "All values should be categorized"
        );
    }
}

/// Test RSI for alternative ticker
#[tokio::test]
async fn test_get_rsi_alt_ticker() {
    if !has_api_key() {
        eprintln!("Skipping test: MASSIVE_API_KEY not set");
        return;
    }

    let client = create_rest_client();

    let request = GetRsiRequest::new(TEST_TICKER_ALT)
        .timespan(IndicatorTimespan::Day)
        .window(14)
        .limit(10);

    let response = client.execute(request).await;

    match response {
        Ok(rsi) => {
            assert!(
                !rsi.results.values.is_empty(),
                "Expected RSI values for {}",
                TEST_TICKER_ALT
            );
            println!(
                "RSI for {}: {} values, latest = {:.2}",
                TEST_TICKER_ALT,
                rsi.results.values.len(),
                rsi.results.values[0].value
            );
        }
        Err(e) => {
            panic!("Failed to fetch RSI for {}: {:?}", TEST_TICKER_ALT, e);
        }
    }
}
