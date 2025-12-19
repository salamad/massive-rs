//! Example: Streaming real-time trades via WebSocket.
//!
//! This example demonstrates how to connect to the WebSocket API
//! and subscribe to real-time trade data.

use futures::StreamExt;
use massive_rs::config::WsConfig;
use massive_rs::ws::{Subscription, WsClient, WsEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create WebSocket client (reads MASSIVE_API_KEY from environment)
    let config = WsConfig::default();
    let client = WsClient::new(config)?;

    println!("Connecting to WebSocket...");

    // Connect
    let (handle, mut stream) = client.connect().await?;

    println!("Connected and authenticated!");

    // Subscribe to AAPL trades
    handle.subscribe(&[Subscription::trade("AAPL")]).await?;

    println!("Subscribed to T.AAPL");
    println!("Waiting for trades (press Ctrl+C to exit)...");

    // Process events
    while let Some(result) = stream.next().await {
        match result {
            Ok(batch) => {
                for event in batch.events {
                    match event {
                        WsEvent::Trade(trade) => {
                            println!(
                                "Trade: {} @ ${:.2} x {} (exchange {})",
                                trade.sym, trade.p, trade.s, trade.x
                            );
                        }
                        WsEvent::Status(status) => {
                            println!("Status: {}", status.status);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    // Clean up
    handle.close().await?;

    Ok(())
}
