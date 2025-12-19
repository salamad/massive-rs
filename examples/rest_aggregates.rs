//! Example: Fetching aggregate bars (OHLCV) from the REST API.
//!
//! This example demonstrates how to use the REST client to fetch
//! historical aggregate data for a stock.

use massive_rs::config::RestConfig;
use massive_rs::rest::RestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create client (reads MASSIVE_API_KEY from environment)
    let config = RestConfig::default();
    let _client = RestClient::new(config)?;

    println!("REST client created successfully!");
    println!("Note: Full API implementation is in Phase 2");

    // Example usage (when implemented):
    // let bars = client.get_aggs("AAPL", 1, Timespan::Day, "2024-01-01", "2024-01-31").await?;
    // for bar in bars {
    //     println!("{}: O={} H={} L={} C={} V={}",
    //         bar.timestamp, bar.open, bar.high, bar.low, bar.close, bar.volume);
    // }

    Ok(())
}
