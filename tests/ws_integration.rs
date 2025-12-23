//! WebSocket API Integration Tests
//!
//! This module contains comprehensive integration tests for WebSocket functionality.
//! Tests require the `POLYGON_API_KEY` environment variable to be set.
//!
//! Run with: `cargo test --test ws_integration --features ws`

#![cfg(feature = "ws")]

mod common;

use common::{
    create_ws_config, create_ws_config_for_market, has_api_key, TEST_TICKER, TEST_TICKER_ALT,
};
use futures::StreamExt;
use massive_rs::config::{Market, OverflowPolicy, ReconnectConfig};
use massive_rs::ws::{ConnectionState, Subscription, WsClient};
use std::time::Duration;
use tokio::time::timeout;

/// Default timeout for WebSocket tests
const WS_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Short timeout for quick operations
const WS_SHORT_TIMEOUT: Duration = Duration::from_secs(10);

// ============================================================================
// Connection Tests
// ============================================================================

/// Test basic WebSocket connection and authentication
#[tokio::test]
async fn test_ws_connect_and_authenticate() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let result = timeout(WS_TEST_TIMEOUT, client.connect()).await;

    match result {
        Ok(Ok((handle, _stream))) => {
            assert!(
                handle.is_authenticated(),
                "Connection should be authenticated"
            );
            assert_eq!(
                handle.connection_state(),
                ConnectionState::Connected,
                "Connection state should be Connected"
            );

            println!("Successfully connected and authenticated");

            // Clean up
            let _ = handle.close().await;
        }
        Ok(Err(e)) => {
            panic!("WebSocket connection failed: {:?}", e);
        }
        Err(_) => {
            panic!("Connection timed out after {:?}", WS_TEST_TIMEOUT);
        }
    }
}

/// Test WebSocket connection to different markets
#[tokio::test]
async fn test_ws_connect_to_stocks_market() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config_for_market(Market::Stocks);
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let result = timeout(WS_TEST_TIMEOUT, client.connect()).await;

    match result {
        Ok(Ok((handle, _stream))) => {
            assert!(handle.is_authenticated());
            println!("Successfully connected to stocks market");
            let _ = handle.close().await;
        }
        Ok(Err(e)) => {
            // Stocks market might require specific subscription
            println!("Stocks market connection result: {:?}", e);
        }
        Err(_) => {
            panic!("Connection timed out");
        }
    }
}

// ============================================================================
// Subscription Tests
// ============================================================================

/// Test subscribing to a single ticker's trades
#[tokio::test]
async fn test_ws_subscribe_trade() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Subscribe to trades
    let subscription = Subscription::trade(TEST_TICKER);
    let result = timeout(WS_SHORT_TIMEOUT, handle.subscribe(&[subscription.clone()])).await;

    match result {
        Ok(Ok(())) => {
            let subs = handle.subscriptions();
            assert!(
                subs.contains(&subscription),
                "Subscription should be tracked"
            );
            println!("Successfully subscribed to trades for {}", TEST_TICKER);
        }
        Ok(Err(e)) => {
            panic!("Subscription failed: {:?}", e);
        }
        Err(_) => {
            panic!("Subscription timed out");
        }
    }

    let _ = handle.close().await;
}

/// Test subscribing to quotes
#[tokio::test]
async fn test_ws_subscribe_quote() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    let subscription = Subscription::quote(TEST_TICKER);
    let result = timeout(WS_SHORT_TIMEOUT, handle.subscribe(&[subscription.clone()])).await;

    match result {
        Ok(Ok(())) => {
            let subs = handle.subscriptions();
            assert!(
                subs.contains(&subscription),
                "Subscription should be tracked"
            );
            println!("Successfully subscribed to quotes for {}", TEST_TICKER);
        }
        Ok(Err(e)) => {
            panic!("Subscription failed: {:?}", e);
        }
        Err(_) => {
            panic!("Subscription timed out");
        }
    }

    let _ = handle.close().await;
}

/// Test subscribing to minute aggregates
#[tokio::test]
async fn test_ws_subscribe_minute_agg() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    let subscription = Subscription::minute_agg(TEST_TICKER);
    let result = timeout(WS_SHORT_TIMEOUT, handle.subscribe(&[subscription.clone()])).await;

    match result {
        Ok(Ok(())) => {
            let subs = handle.subscriptions();
            assert!(
                subs.contains(&subscription),
                "Subscription should be tracked"
            );
            println!(
                "Successfully subscribed to minute aggregates for {}",
                TEST_TICKER
            );
        }
        Ok(Err(e)) => {
            panic!("Subscription failed: {:?}", e);
        }
        Err(_) => {
            panic!("Subscription timed out");
        }
    }

    let _ = handle.close().await;
}

/// Test subscribing to multiple tickers at once
#[tokio::test]
async fn test_ws_subscribe_multiple_tickers() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    let subscriptions = vec![
        Subscription::trade(TEST_TICKER),
        Subscription::trade(TEST_TICKER_ALT),
        Subscription::quote(TEST_TICKER),
        Subscription::quote(TEST_TICKER_ALT),
    ];

    let result = timeout(WS_SHORT_TIMEOUT, handle.subscribe(&subscriptions)).await;

    match result {
        Ok(Ok(())) => {
            let subs = handle.subscriptions();
            for sub in &subscriptions {
                assert!(subs.contains(sub), "Subscription {} should be tracked", sub);
            }
            println!(
                "Successfully subscribed to {} subscriptions",
                subscriptions.len()
            );
        }
        Ok(Err(e)) => {
            panic!("Subscription failed: {:?}", e);
        }
        Err(_) => {
            panic!("Subscription timed out");
        }
    }

    let _ = handle.close().await;
}

/// Test wildcard subscriptions
#[tokio::test]
async fn test_ws_subscribe_wildcard() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Subscribe to all trades
    let subscription = Subscription::all_trades();
    let result = timeout(WS_SHORT_TIMEOUT, handle.subscribe(&[subscription.clone()])).await;

    match result {
        Ok(Ok(())) => {
            println!("Successfully subscribed to all trades (wildcard)");
        }
        Ok(Err(e)) => {
            // Wildcard subscriptions might be restricted
            println!("Wildcard subscription result: {:?}", e);
        }
        Err(_) => {
            panic!("Subscription timed out");
        }
    }

    let _ = handle.close().await;
}

// ============================================================================
// Unsubscription Tests
// ============================================================================

/// Test unsubscribing from a subscription
#[tokio::test]
async fn test_ws_unsubscribe() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Subscribe first
    let subscription = Subscription::trade(TEST_TICKER);
    timeout(WS_SHORT_TIMEOUT, handle.subscribe(&[subscription.clone()]))
        .await
        .expect("Subscribe timed out")
        .expect("Subscribe failed");

    assert!(
        handle.subscriptions().contains(&subscription),
        "Should be subscribed"
    );

    // Unsubscribe
    let result = timeout(
        WS_SHORT_TIMEOUT,
        handle.unsubscribe(&[subscription.clone()]),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            assert!(
                !handle.subscriptions().contains(&subscription),
                "Should be unsubscribed"
            );
            println!("Successfully unsubscribed from trades for {}", TEST_TICKER);
        }
        Ok(Err(e)) => {
            panic!("Unsubscribe failed: {:?}", e);
        }
        Err(_) => {
            panic!("Unsubscribe timed out");
        }
    }

    let _ = handle.close().await;
}

/// Test subscribing and unsubscribing multiple times
#[tokio::test]
async fn test_ws_subscribe_unsubscribe_cycle() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    let subscription = Subscription::trade(TEST_TICKER);

    // Subscribe -> Unsubscribe -> Subscribe -> Unsubscribe
    for i in 0..3 {
        // Subscribe
        timeout(WS_SHORT_TIMEOUT, handle.subscribe(&[subscription.clone()]))
            .await
            .expect("Subscribe timed out")
            .expect("Subscribe failed");

        assert!(
            handle.subscriptions().contains(&subscription),
            "Cycle {}: Should be subscribed",
            i
        );

        // Unsubscribe
        timeout(
            WS_SHORT_TIMEOUT,
            handle.unsubscribe(&[subscription.clone()]),
        )
        .await
        .expect("Unsubscribe timed out")
        .expect("Unsubscribe failed");

        assert!(
            !handle.subscriptions().contains(&subscription),
            "Cycle {}: Should be unsubscribed",
            i
        );

        println!("Cycle {} complete", i);
    }

    let _ = handle.close().await;
}

// ============================================================================
// Event Receiving Tests
// ============================================================================

/// Test receiving events after subscription
#[tokio::test]
async fn test_ws_receive_events() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, mut stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Subscribe to a high-volume ticker
    let subscription = Subscription::trade(TEST_TICKER);
    timeout(WS_SHORT_TIMEOUT, handle.subscribe(&[subscription]))
        .await
        .expect("Subscribe timed out")
        .expect("Subscribe failed");

    // Wait for some events (during market hours, or just status messages)
    let receive_duration = Duration::from_secs(5);
    let mut event_count = 0;

    let receive_result = timeout(receive_duration, async {
        while let Some(result) = stream.next().await {
            match result {
                Ok(batch) => {
                    event_count += batch.events.len();
                    println!("Received batch with {} events", batch.events.len());
                    for event in &batch.events {
                        println!("  Event: {:?}", event);
                    }
                    // Exit after receiving some events
                    if event_count >= 5 {
                        break;
                    }
                }
                Err(e) => {
                    println!("Error receiving event: {:?}", e);
                    break;
                }
            }
        }
    })
    .await;

    // It's okay if we timeout - we might not receive events outside market hours
    match receive_result {
        Ok(()) => {
            println!("Received {} events total", event_count);
        }
        Err(_) => {
            println!(
                "Event receive timed out after {:?} (received {} events)",
                receive_duration, event_count
            );
        }
    }

    let _ = handle.close().await;
}

/// Test that event batches include timing information
#[tokio::test]
async fn test_ws_event_batch_timing() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, mut stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Wait for at least one batch (status message on connect)
    let result = timeout(Duration::from_secs(5), stream.next()).await;

    match result {
        Ok(Some(Ok(batch))) => {
            // Verify batch has timing info
            assert!(
                batch.received_at.elapsed() < Duration::from_secs(10),
                "Batch should have been received recently"
            );
            println!("Batch received at: {:?}", batch.received_at);
            println!("Latency hint: {:?}", batch.latency_hint_ns);
        }
        Ok(Some(Err(e))) => {
            println!("Error in batch: {:?}", e);
        }
        Ok(None) => {
            println!("Stream ended");
        }
        Err(_) => {
            println!("No events received within timeout");
        }
    }

    let _ = handle.close().await;
}

// ============================================================================
// Connection State Tests
// ============================================================================

/// Test connection state monitoring
#[tokio::test]
async fn test_ws_connection_state() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Check initial state
    let state = handle.connection_state();
    assert_eq!(state, ConnectionState::Connected, "Should be connected");

    // Check authentication
    assert!(handle.is_authenticated(), "Should be authenticated");

    // Get stats
    let stats = handle.stats();
    println!("Connection stats:");
    println!("  Message count: {}", stats.message_count);
    println!("  Last message age: {:?}", stats.last_message_age);
    println!("  Reconnect count: {}", stats.reconnect_count);
    println!("  Subscription count: {}", stats.subscription_count);

    assert_eq!(stats.reconnect_count, 0, "Should not have reconnected");

    let _ = handle.close().await;
}

/// Test that subscriptions are tracked properly
#[tokio::test]
async fn test_ws_subscription_tracking() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Initially no subscriptions
    assert_eq!(
        handle.subscriptions().len(),
        0,
        "Should start with no subscriptions"
    );

    // Add subscriptions
    let subs = vec![
        Subscription::trade(TEST_TICKER),
        Subscription::quote(TEST_TICKER),
    ];

    timeout(WS_SHORT_TIMEOUT, handle.subscribe(&subs))
        .await
        .expect("Subscribe timed out")
        .expect("Subscribe failed");

    assert_eq!(
        handle.subscriptions().len(),
        2,
        "Should have 2 subscriptions"
    );

    // Stats should reflect subscription count
    let stats = handle.stats();
    assert_eq!(stats.subscription_count, 2);

    let _ = handle.close().await;
}

// ============================================================================
// Graceful Closure Tests
// ============================================================================

/// Test graceful connection closure
#[tokio::test]
async fn test_ws_graceful_close() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Close the connection
    let result = timeout(WS_SHORT_TIMEOUT, handle.close()).await;

    match result {
        Ok(Ok(())) => {
            println!("Connection closed gracefully");
        }
        Ok(Err(e)) => {
            panic!("Close failed: {:?}", e);
        }
        Err(_) => {
            panic!("Close timed out");
        }
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

/// Test different WebSocket configurations
#[tokio::test]
async fn test_ws_custom_configuration() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    use massive_rs::auth::ApiKey;
    use massive_rs::config::{DispatchConfig, Feed, WsConfig};

    let api_key = common::get_api_key();

    // Custom configuration
    let config = WsConfig {
        feed: Feed::Delayed,
        market: Market::Stocks,
        api_key: ApiKey::new(&api_key),
        connect_timeout: Duration::from_secs(20),
        idle_timeout: Duration::from_secs(60),
        ping_interval: Duration::from_secs(30),
        reconnect: ReconnectConfig {
            enabled: true,
            max_retries: Some(2),
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
        },
        dispatch: DispatchConfig {
            capacity: 5000,
            overflow: OverflowPolicy::DropNewest,
            ..Default::default()
        },
    };

    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let result = timeout(WS_TEST_TIMEOUT, client.connect()).await;

    match result {
        Ok(Ok((handle, _stream))) => {
            assert!(handle.is_authenticated());
            println!("Successfully connected with custom configuration");
            let _ = handle.close().await;
        }
        Ok(Err(e)) => {
            panic!("Connection with custom config failed: {:?}", e);
        }
        Err(_) => {
            panic!("Connection timed out");
        }
    }
}

/// Test WebSocket client builder
#[tokio::test]
async fn test_ws_client_builder() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::builder()
        .config(config)
        .build()
        .expect("Failed to build WebSocket client");

    let result = timeout(WS_TEST_TIMEOUT, client.connect()).await;

    match result {
        Ok(Ok((handle, _stream))) => {
            assert!(handle.is_authenticated());
            println!("Successfully connected using builder pattern");
            let _ = handle.close().await;
        }
        Ok(Err(e)) => {
            panic!("Builder connection failed: {:?}", e);
        }
        Err(_) => {
            panic!("Connection timed out");
        }
    }
}

// ============================================================================
// Handle Debug and Clone Tests
// ============================================================================

/// Test WsHandle debug output
#[tokio::test]
async fn test_ws_handle_debug() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    let debug_output = format!("{:?}", handle);
    println!("Handle debug output: {}", debug_output);

    assert!(
        debug_output.contains("WsHandle"),
        "Debug output should contain WsHandle"
    );
    assert!(
        debug_output.contains("authenticated"),
        "Debug output should contain authenticated"
    );

    let _ = handle.close().await;
}

/// Test WsHandle clone functionality
#[tokio::test]
async fn test_ws_handle_clone() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();
    let client = WsClient::new(config).expect("Failed to create WebSocket client");

    let (handle, _stream) = timeout(WS_TEST_TIMEOUT, client.connect())
        .await
        .expect("Connection timed out")
        .expect("Connection failed");

    // Clone the handle
    let handle_clone = handle.clone();

    // Both handles should see the same state
    assert_eq!(handle.is_authenticated(), handle_clone.is_authenticated());
    assert_eq!(handle.connection_state(), handle_clone.connection_state());

    // Subscribe using the clone
    let subscription = Subscription::trade(TEST_TICKER);
    timeout(
        WS_SHORT_TIMEOUT,
        handle_clone.subscribe(&[subscription.clone()]),
    )
    .await
    .expect("Subscribe timed out")
    .expect("Subscribe failed");

    // Original should see the subscription too
    assert!(
        handle.subscriptions().contains(&subscription),
        "Original handle should see subscription"
    );

    println!("Handle clone works correctly");

    let _ = handle.close().await;
}

// ============================================================================
// Subscription Type Tests
// ============================================================================

/// Test all subscription type constructors
#[test]
fn test_subscription_types() {
    // Trade subscription
    let trade = Subscription::trade("AAPL");
    assert_eq!(trade.as_str(), "T.AAPL");
    assert_eq!(format!("{}", trade), "T.AAPL");

    // Quote subscription
    let quote = Subscription::quote("MSFT");
    assert_eq!(quote.as_str(), "Q.MSFT");

    // Second aggregate
    let second = Subscription::second_agg("GOOG");
    assert_eq!(second.as_str(), "A.GOOG");

    // Minute aggregate
    let minute = Subscription::minute_agg("AMZN");
    assert_eq!(minute.as_str(), "AM.AMZN");

    // Wildcards
    assert_eq!(Subscription::all_trades().as_str(), "T.*");
    assert_eq!(Subscription::all_quotes().as_str(), "Q.*");
    assert_eq!(Subscription::all_second_aggs().as_str(), "A.*");
    assert_eq!(Subscription::all_minute_aggs().as_str(), "AM.*");

    // Raw subscription
    let raw = Subscription::raw("CUSTOM.TEST");
    assert_eq!(raw.as_str(), "CUSTOM.TEST");

    // From string
    let from_str: Subscription = "T.META".into();
    assert_eq!(from_str.as_str(), "T.META");

    println!("All subscription type tests passed");
}

/// Test subscription equality and hashing
#[test]
fn test_subscription_equality() {
    let a = Subscription::trade("AAPL");
    let b = Subscription::trade("AAPL");
    let c = Subscription::trade("MSFT");
    let d = Subscription::quote("AAPL");

    assert_eq!(a, b, "Same subscriptions should be equal");
    assert_ne!(a, c, "Different tickers should not be equal");
    assert_ne!(a, d, "Different types should not be equal");

    // Test that they can be used in HashSet
    let mut set = std::collections::HashSet::new();
    set.insert(a.clone());
    assert!(set.contains(&b));
    assert!(!set.contains(&c));

    println!("Subscription equality tests passed");
}

// ============================================================================
// Connection State Enum Tests
// ============================================================================

/// Test ConnectionState enum
#[test]
fn test_connection_state_enum() {
    let connecting = ConnectionState::Connecting;
    let authenticating = ConnectionState::Authenticating;
    let connected = ConnectionState::Connected;
    let reconnecting = ConnectionState::Reconnecting(1);
    let disconnected = ConnectionState::Disconnected;

    // Test Debug
    assert_eq!(format!("{:?}", connecting), "Connecting");
    assert_eq!(format!("{:?}", authenticating), "Authenticating");
    assert_eq!(format!("{:?}", connected), "Connected");
    assert_eq!(format!("{:?}", reconnecting), "Reconnecting(1)");
    assert_eq!(format!("{:?}", disconnected), "Disconnected");

    // Test equality
    assert_eq!(connecting, ConnectionState::Connecting);
    assert_ne!(connecting, connected);

    // Test clone
    let cloned = connected.clone();
    assert_eq!(connected, cloned);

    println!("ConnectionState enum tests passed");
}

// ============================================================================
// Multiple Connection Tests
// ============================================================================

/// Test multiple concurrent WebSocket connections
#[tokio::test]
async fn test_ws_multiple_connections() {
    if !has_api_key() {
        eprintln!("Skipping test: POLYGON_API_KEY not set");
        return;
    }

    common::init_test_logging();

    let config = create_ws_config();

    // Create two connections
    let client1 = WsClient::new(config.clone()).expect("Failed to create WebSocket client 1");
    let client2 = WsClient::new(config).expect("Failed to create WebSocket client 2");

    let result = timeout(
        WS_TEST_TIMEOUT,
        futures::future::join(client1.connect(), client2.connect()),
    )
    .await;

    match result {
        Ok((Ok((handle1, _stream1)), Ok((handle2, _stream2)))) => {
            assert!(handle1.is_authenticated());
            assert!(handle2.is_authenticated());
            println!("Successfully created two concurrent connections");

            let _ = handle1.close().await;
            let _ = handle2.close().await;
        }
        Ok((Err(e1), Err(e2))) => {
            println!("Both connections failed: {:?}, {:?}", e1, e2);
        }
        Ok((Ok((handle, _)), Err(e))) | Ok((Err(e), Ok((handle, _)))) => {
            println!("One connection succeeded, one failed: {:?}", e);
            let _ = handle.close().await;
        }
        Err(_) => {
            panic!("Connections timed out");
        }
    }
}
