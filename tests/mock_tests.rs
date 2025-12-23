//! Mock-based Unit Tests
//!
//! This module contains unit tests using wiremock to simulate API responses,
//! testing error handling, edge cases, and various scenarios without requiring
//! a real API key.
//!
//! Run with: `cargo test --test mock_tests`

use massive_rs::auth::{ApiKey, AuthMode};
use massive_rs::config::{PaginationMode, RestConfig};
use massive_rs::error::MassiveError;
use massive_rs::rest::endpoints::{
    GetAggsRequest, GetDailyOpenCloseRequest, GetLastQuoteRequest, GetLastTradeRequest,
    GetPreviousCloseRequest, GetTickerDetailsRequest, GetTickersRequest, Timespan,
};
use massive_rs::rest::RestClient;
use url::Url;
use wiremock::matchers::{header, method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Create a REST client configured for the mock server.
fn create_mock_client(mock_server: &MockServer) -> RestClient {
    let config = RestConfig {
        base_url: Url::parse(&mock_server.uri()).expect("valid URL"),
        api_key: ApiKey::new("test-api-key"),
        auth_mode: AuthMode::HeaderBearer,
        ..Default::default()
    };

    RestClient::new(config).expect("Failed to create REST client")
}

/// Create a REST client with query param auth.
fn create_mock_client_query_auth(mock_server: &MockServer) -> RestClient {
    let config = RestConfig {
        base_url: Url::parse(&mock_server.uri()).expect("valid URL"),
        api_key: ApiKey::new("test-api-key"),
        auth_mode: AuthMode::QueryParam,
        ..Default::default()
    };

    RestClient::new(config).expect("Failed to create REST client")
}

// ============================================================================
// Successful Response Tests
// ============================================================================

/// Test successful aggregates response parsing
#[tokio::test]
async fn test_mock_aggregates_success() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "ticker": "AAPL",
        "queryCount": 2,
        "resultsCount": 2,
        "adjusted": true,
        "results": [
            {
                "o": 150.0,
                "h": 155.0,
                "l": 148.0,
                "c": 153.0,
                "v": 1000000,
                "vw": 151.5,
                "t": 1703001234567,
                "n": 5000
            },
            {
                "o": 153.0,
                "h": 158.0,
                "l": 152.0,
                "c": 156.0,
                "v": 1200000,
                "vw": 155.0,
                "t": 1703087634567,
                "n": 6000
            }
        ]
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/v2/aggs/ticker/.*/range/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetAggsRequest::new("AAPL")
        .multiplier(1)
        .timespan(Timespan::Day)
        .from("2024-01-01")
        .to("2024-01-31");

    let response = client.execute(request).await;

    let aggs = response.expect("Should parse aggregates");
    assert_eq!(aggs.status, Some("OK".to_string()));
    assert_eq!(aggs.ticker, Some("AAPL".to_string()));
    assert_eq!(aggs.results.len(), 2);
    assert_eq!(aggs.results[0].open, 150.0);
    assert_eq!(aggs.results[0].high, 155.0);
    assert_eq!(aggs.results[0].low, 148.0);
    assert_eq!(aggs.results[0].close, 153.0);
}

/// Test successful previous close response
#[tokio::test]
async fn test_mock_previous_close_success() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "request_id": "test-123",
        "results": [
            {
                "o": 150.0,
                "h": 155.0,
                "l": 148.0,
                "c": 153.0,
                "v": 1000000,
                "t": 1703001234567
            }
        ]
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/v2/aggs/ticker/.*/prev"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetPreviousCloseRequest::new("AAPL");

    let response = client.execute(request).await;

    let prev = response.expect("Should parse previous close");
    assert_eq!(prev.status, Some("OK".to_string()));
    assert_eq!(prev.results.len(), 1);
    assert_eq!(prev.results[0].close, 153.0);
}

/// Test successful daily open/close response
#[tokio::test]
async fn test_mock_daily_open_close_success() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "from": "2024-01-15",
        "symbol": "AAPL",
        "open": 150.0,
        "high": 155.0,
        "low": 148.0,
        "close": 153.0,
        "volume": 1000000,
        "afterHours": 153.50,
        "preMarket": 149.50
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/v1/open-close/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetDailyOpenCloseRequest::new("AAPL", "2024-01-15");

    let response = client.execute(request).await;

    let daily = response.expect("Should parse daily open/close");
    assert_eq!(daily.status, "OK");
    assert_eq!(daily.symbol, "AAPL");
    assert_eq!(daily.open, 150.0);
    assert_eq!(daily.close, 153.0);
    assert_eq!(daily.after_hours, Some(153.50));
    assert_eq!(daily.pre_market, Some(149.50));
}

/// Test successful last trade response
#[tokio::test]
async fn test_mock_last_trade_success() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "request_id": "test-123",
        "results": {
            "i": "trade-1",
            "c": [0, 12],
            "x": 4,
            "p": 153.25,
            "t": 1703001234567890123,
            "s": 100,
            "z": 3
        }
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/v2/last/trade/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    let last_trade = response.expect("Should parse last trade");
    assert_eq!(last_trade.status, Some("OK".to_string()));
    let trade = last_trade.results.expect("Should have trade result");
    assert_eq!(trade.price, 153.25);
    assert_eq!(trade.size, 100);
    assert_eq!(trade.conditions, vec![0, 12]);
}

/// Test successful last quote response
#[tokio::test]
async fn test_mock_last_quote_success() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "request_id": "test-123",
        "results": {
            "X": 4,
            "P": 153.50,
            "S": 200,
            "x": 7,
            "p": 153.40,
            "s": 300,
            "c": [1],
            "t": 1703001234567890123,
            "z": 3
        }
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/v2/last/nbbo/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastQuoteRequest::new("AAPL");

    let response = client.execute(request).await;

    let last_quote = response.expect("Should parse last quote");
    assert_eq!(last_quote.status, Some("OK".to_string()));
    let quote = last_quote.results.expect("Should have quote result");
    assert_eq!(quote.ask_price, 153.50);
    assert_eq!(quote.bid_price, 153.40);

    // Test helper methods
    assert!((quote.spread() - 0.10).abs() < 0.001);
    assert!((quote.mid_price() - 153.45).abs() < 0.001);
    assert!(!quote.is_crossed());
    assert!(!quote.is_locked());
}

/// Test successful ticker details response
#[tokio::test]
async fn test_mock_ticker_details_success() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "request_id": "test-123",
        "results": {
            "ticker": "AAPL",
            "name": "Apple Inc.",
            "market": "stocks",
            "locale": "us",
            "primary_exchange": "XNAS",
            "type": "CS",
            "active": true,
            "currency_name": "usd",
            "cik": "0000320193",
            "market_cap": 2500000000000,
            "description": "Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide."
        }
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/v3/reference/tickers/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetTickerDetailsRequest::new("AAPL");

    let response = client.execute(request).await;

    let details = response.expect("Should parse ticker details");
    assert_eq!(details.status, "OK");
    assert_eq!(details.results.ticker, "AAPL");
    assert_eq!(details.results.name, "Apple Inc.");
    assert!(details.results.active);
    assert_eq!(details.results.market_cap, Some(2500000000000.0));
}

// ============================================================================
// Error Response Tests
// ============================================================================

/// Test handling of 401 Unauthorized error
#[tokio::test]
async fn test_mock_unauthorized_error() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "ERROR",
        "request_id": "test-123",
        "error": "Invalid API key"
    }"#;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    assert!(response.is_err());
    let error = response.unwrap_err();
    match error {
        MassiveError::HttpStatus { status, .. } => {
            assert_eq!(status, 401);
        }
        MassiveError::Api(api_err) => {
            assert!(api_err.error.is_some() || api_err.message.is_some());
        }
        _ => panic!("Unexpected error type: {:?}", error),
    }
}

/// Test handling of 403 Forbidden error
#[tokio::test]
async fn test_mock_forbidden_error() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "ERROR",
        "request_id": "test-123",
        "error": "Access denied. Please upgrade your subscription."
    }"#;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    assert!(response.is_err());
    let error = response.unwrap_err();
    match error {
        MassiveError::HttpStatus { status, .. } => {
            assert_eq!(status, 403);
        }
        MassiveError::Api(api_err) => {
            assert!(api_err.error.is_some() || api_err.message.is_some());
        }
        _ => panic!("Unexpected error type: {:?}", error),
    }
}

/// Test handling of 404 Not Found error
#[tokio::test]
async fn test_mock_not_found_error() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "NOT_FOUND",
        "request_id": "test-123",
        "error": "Ticker not found"
    }"#;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("INVALIDTICKER");

    let response = client.execute(request).await;

    assert!(response.is_err());
    let error = response.unwrap_err();
    match error {
        MassiveError::HttpStatus { status, .. } => {
            assert_eq!(status, 404);
        }
        MassiveError::Api(_) => {
            // Also acceptable
        }
        _ => panic!("Unexpected error type: {:?}", error),
    }
}

/// Test handling of 429 Rate Limit error
#[tokio::test]
async fn test_mock_rate_limit_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_string(r#"{"status": "ERROR", "error": "Rate limit exceeded"}"#),
        )
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    assert!(response.is_err());
    let error = response.unwrap_err();
    match error {
        MassiveError::RateLimited { retry_after, .. } => {
            assert_eq!(retry_after, Some(std::time::Duration::from_secs(60)));
        }
        _ => panic!("Expected RateLimited error, got: {:?}", error),
    }
}

/// Test handling of 500 Internal Server Error
#[tokio::test]
async fn test_mock_server_error() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "ERROR",
        "request_id": "test-123",
        "error": "Internal server error"
    }"#;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    assert!(response.is_err());
    let error = response.unwrap_err();
    match error {
        MassiveError::HttpStatus { status, .. } => {
            assert_eq!(status, 500);
        }
        MassiveError::Api(_) => {
            // Also acceptable
        }
        _ => panic!("Unexpected error type: {:?}", error),
    }
}

// ============================================================================
// Authentication Tests
// ============================================================================

/// Test Bearer token authentication header
#[tokio::test]
async fn test_mock_bearer_auth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"status": "OK", "results": null}"#),
        )
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;
    assert!(response.is_ok(), "Bearer auth should succeed");
}

/// Test query parameter authentication
#[tokio::test]
async fn test_mock_query_param_auth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param("apiKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"status": "OK", "results": null}"#),
        )
        .mount(&mock_server)
        .await;

    let client = create_mock_client_query_auth(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;
    assert!(response.is_ok(), "Query param auth should succeed");
}

// ============================================================================
// Response Parsing Tests
// ============================================================================

/// Test handling of empty results
#[tokio::test]
async fn test_mock_empty_results() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "ticker": "INVALIDTICKER",
        "queryCount": 0,
        "resultsCount": 0,
        "adjusted": true,
        "results": []
    }"#;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetAggsRequest::new("INVALIDTICKER")
        .multiplier(1)
        .timespan(Timespan::Day)
        .from("2024-01-01")
        .to("2024-01-31");

    let response = client.execute(request).await;

    let aggs = response.expect("Should parse empty results");
    assert_eq!(aggs.status, Some("OK".to_string()));
    assert!(aggs.results.is_empty());
}

/// Test handling of null results field
#[tokio::test]
async fn test_mock_null_results() {
    let mock_server = MockServer::start().await;

    let response_json = r#"{
        "status": "OK",
        "request_id": "test-123",
        "results": null
    }"#;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    let last_trade = response.expect("Should handle null results");
    assert!(last_trade.results.is_none());
}

/// Test handling of malformed JSON
#[tokio::test]
async fn test_mock_malformed_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ invalid json }"))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    assert!(response.is_err());
    match response.unwrap_err() {
        MassiveError::Deserialize { body_snippet, .. } => {
            assert!(body_snippet.contains("invalid"));
        }
        other => panic!("Expected Deserialize error, got: {:?}", other),
    }
}

/// Test handling of extra fields in response
#[tokio::test]
async fn test_mock_extra_fields() {
    let mock_server = MockServer::start().await;

    // Response with extra unexpected fields
    let response_json = r#"{
        "status": "OK",
        "request_id": "test-123",
        "results": {
            "i": "trade-1",
            "p": 153.25,
            "s": 100,
            "extra_field": "unexpected",
            "another_extra": 12345
        },
        "unknown_top_level": true
    }"#;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;

    // Should succeed - extra fields should be ignored
    let last_trade = response.expect("Should handle extra fields");
    assert_eq!(last_trade.results.unwrap().price, 153.25);
}

// ============================================================================
// Pagination Tests
// ============================================================================

/// Test pagination with next_url
#[tokio::test]
async fn test_mock_pagination() {
    let mock_server = MockServer::start().await;
    let mock_uri = mock_server.uri();

    // First page response
    let page1_json = format!(
        r#"{{
        "status": "OK",
        "ticker": "AAPL",
        "results": [
            {{"o": 150.0, "h": 155.0, "l": 148.0, "c": 153.0, "v": 1000000, "t": 1703001234567}}
        ],
        "next_url": "{}/v2/aggs/ticker/AAPL/range/1/day/2024-01-01/2024-01-31?cursor=page2"
    }}"#,
        mock_uri
    );

    // Second page response
    let page2_json = r#"{
        "status": "OK",
        "ticker": "AAPL",
        "results": [
            {"o": 153.0, "h": 158.0, "l": 152.0, "c": 156.0, "v": 1200000, "t": 1703087634567}
        ]
    }"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/v2/aggs/ticker/AAPL/range/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1_json))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(query_param("cursor", "page2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page2_json))
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetAggsRequest::new("AAPL")
        .multiplier(1)
        .timespan(Timespan::Day)
        .from("2024-01-01")
        .to("2024-01-31");

    // Test single page execution
    let response = client.execute(request.clone()).await;
    let aggs = response.expect("Should get first page");
    assert_eq!(aggs.results.len(), 1);
    assert!(aggs.next_url.is_some());
}

// ============================================================================
// Request Building Tests
// ============================================================================

/// Test that query parameters are correctly built
#[tokio::test]
async fn test_mock_query_params() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param("adjusted", "true"))
        .and(query_param("sort", "asc"))
        .and(query_param("limit", "50"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"status": "OK", "results": []}"#),
        )
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetAggsRequest::new("AAPL")
        .multiplier(1)
        .timespan(Timespan::Day)
        .from("2024-01-01")
        .to("2024-01-31")
        .adjusted(true)
        .sort(massive_rs::rest::endpoints::Sort::Asc)
        .limit(50);

    let response = client.execute(request).await;
    assert!(response.is_ok());
}

/// Test ticker list request query parameters
#[tokio::test]
async fn test_mock_tickers_query_params() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers"))
        .and(query_param("market", "stocks"))
        .and(query_param("active", "true"))
        .and(query_param("limit", "100"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"status": "OK", "results": []}"#),
        )
        .mount(&mock_server)
        .await;

    let client = create_mock_client(&mock_server);
    let request = GetTickersRequest::default()
        .market(massive_rs::rest::endpoints::MarketType::Stocks)
        .active(true)
        .limit(100);

    let response = client.execute(request).await;
    assert!(response.is_ok());
}

// ============================================================================
// Client Configuration Tests
// ============================================================================

/// Test client with custom user agent
#[tokio::test]
async fn test_mock_custom_user_agent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("User-Agent", "my-custom-agent/1.0"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"status": "OK", "results": null}"#),
        )
        .mount(&mock_server)
        .await;

    let config = RestConfig {
        base_url: Url::parse(&mock_server.uri()).expect("valid URL"),
        api_key: ApiKey::new("test-api-key"),
        user_agent: Some("my-custom-agent/1.0".to_string()),
        ..Default::default()
    };

    let client = RestClient::new(config).expect("Failed to create REST client");
    let request = GetLastTradeRequest::new("AAPL");

    let response = client.execute(request).await;
    assert!(response.is_ok());
}

// ============================================================================
// Error Type Tests
// ============================================================================

/// Test MassiveError implements Error trait
#[test]
fn test_error_trait_impl() {
    let error = MassiveError::Timeout;
    let _: &dyn std::error::Error = &error;
}

/// Test error display messages
#[test]
fn test_error_display() {
    let timeout_error = MassiveError::Timeout;
    assert!(!format!("{}", timeout_error).is_empty());

    let rate_limit_error = MassiveError::RateLimited {
        retry_after: Some(std::time::Duration::from_secs(60)),
        request_id: Some("test-123".to_string()),
    };
    let display = format!("{}", rate_limit_error);
    assert!(display.contains("Rate limit") || display.contains("rate"));
}

// ============================================================================
// Model Method Tests
// ============================================================================

/// Test AggregateBar helper methods
#[test]
fn test_aggregate_bar_methods() {
    use massive_rs::models::AggregateBar;

    let bar = AggregateBar {
        ticker: None,
        open: 100.0,
        high: 110.0,
        low: 95.0,
        close: 105.0,
        volume: 1000000.0,
        vwap: Some(102.5),
        timestamp: 1703001234567,
        transactions: Some(5000),
        otc: false,
    };

    // Test range
    assert_eq!(bar.range(), 15.0);

    // Test body
    assert_eq!(bar.body(), 5.0);

    // Test bullish/bearish
    assert!(bar.is_bullish());
    assert!(!bar.is_bearish());

    // Test doji
    let doji = AggregateBar {
        ticker: None,
        open: 100.0,
        high: 101.0,
        low: 99.0,
        close: 100.0,
        volume: 1000.0,
        vwap: None,
        timestamp: 1703001234567,
        transactions: None,
        otc: false,
    };
    assert!(doji.is_doji(0.01));

    // Test bearish bar
    let bearish = AggregateBar {
        ticker: None,
        open: 105.0,
        high: 110.0,
        low: 95.0,
        close: 100.0,
        volume: 1000000.0,
        vwap: None,
        timestamp: 1703001234567,
        transactions: None,
        otc: false,
    };
    assert!(bearish.is_bearish());
    assert!(!bearish.is_bullish());
}

/// Test Quote helper methods
#[test]
fn test_quote_methods() {
    use massive_rs::rest::endpoints::Quote;

    let quote = Quote {
        ticker: None,
        ask_exchange: Some(4),
        ask_price: 100.10,
        ask_size: 100,
        bid_exchange: Some(7),
        bid_price: 100.00,
        bid_size: 200,
        conditions: vec![],
        sip_timestamp: None,
        participant_timestamp: None,
        indicators: vec![],
        sequence_number: None,
        tape: None,
    };

    assert!((quote.spread() - 0.10).abs() < 0.001);
    assert!((quote.mid_price() - 100.05).abs() < 0.001);
    assert!(!quote.is_crossed());
    assert!(!quote.is_locked());

    // Test crossed market
    let crossed = Quote {
        ask_price: 99.0,
        bid_price: 100.0,
        ..quote.clone()
    };
    assert!(crossed.is_crossed());

    // Test locked market (need same prices)
    let locked = Quote {
        ask_price: 100.0,
        bid_price: 100.0,
        ..quote
    };
    assert!(locked.is_locked());
}

/// Test Trade value calculation
#[test]
fn test_trade_value() {
    use massive_rs::rest::endpoints::Trade;

    let trade = Trade {
        ticker: None,
        id: Some("test-trade".to_string()),
        conditions: vec![],
        exchange: Some(4),
        price: 150.50,
        sip_timestamp: None,
        participant_timestamp: None,
        trf_timestamp: None,
        size: 100,
        tape: None,
        sequence_number: None,
        reporting_facility: None,
    };

    assert_eq!(trade.value(), 15050.0);
}

// ============================================================================
// ApiKey Tests
// ============================================================================

/// Test ApiKey debug redaction
#[test]
fn test_api_key_redaction() {
    let key = ApiKey::new("super-secret-key");
    let debug_output = format!("{:?}", key);
    assert!(!debug_output.contains("super"));
    assert!(!debug_output.contains("secret"));
    assert!(debug_output.contains("***"));
}

/// Test ApiKey is_empty
#[test]
fn test_api_key_empty() {
    let empty = ApiKey::new("");
    assert!(empty.is_empty());

    let non_empty = ApiKey::new("some-key");
    assert!(!non_empty.is_empty());
}

// ============================================================================
// Configuration Tests
// ============================================================================

/// Test RestConfig defaults
#[test]
fn test_rest_config_defaults() {
    // Create without env var
    let config = RestConfig::new("test-key");

    assert_eq!(config.auth_mode, AuthMode::HeaderBearer);
    assert_eq!(config.connect_timeout, std::time::Duration::from_secs(10));
    assert_eq!(config.request_timeout, std::time::Duration::from_secs(30));
    assert_eq!(config.pagination, PaginationMode::Auto);
    assert_eq!(config.max_retries, 3);
    assert!(!config.trace);
}

/// Test RestConfig builder methods
#[test]
fn test_rest_config_builder() {
    let config = RestConfig::new("test-key")
        .with_auth_mode(AuthMode::QueryParam)
        .with_connect_timeout(std::time::Duration::from_secs(20))
        .with_request_timeout(std::time::Duration::from_secs(60))
        .with_pagination(PaginationMode::MaxItems(1000))
        .with_trace(true)
        .with_user_agent("my-app/1.0");

    assert_eq!(config.auth_mode, AuthMode::QueryParam);
    assert_eq!(config.connect_timeout, std::time::Duration::from_secs(20));
    assert_eq!(config.request_timeout, std::time::Duration::from_secs(60));
    assert_eq!(config.pagination, PaginationMode::MaxItems(1000));
    assert!(config.trace);
    assert_eq!(config.user_agent, Some("my-app/1.0".to_string()));
}
