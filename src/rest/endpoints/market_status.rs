//! Market status endpoint.
//!
//! This module contains the request type for fetching the current
//! trading status of exchanges and markets.

use crate::rest::request::RestRequest;
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

/// Request for current market status.
///
/// Returns the current trading status for various exchanges and overall
/// financial markets, including indicators for pre-market and after-hours
/// sessions.
///
/// # Example
///
/// ```ignore
/// use massive_rs::rest::endpoints::GetMarketStatusRequest;
///
/// let client = RestClient::new(api_key);
/// let status = client.execute(GetMarketStatusRequest).await?;
/// println!("Market is: {}", status.market);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GetMarketStatusRequest;

impl RestRequest for GetMarketStatusRequest {
    type Response = MarketStatusResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v1/marketstatus/now".into()
    }
}

/// Response from the market status endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketStatusResponse {
    /// Whether the market is currently in after-hours trading.
    pub after_hours: bool,
    /// Status of currency markets.
    pub currencies: CurrencyStatus,
    /// Whether the market is currently in early/pre-market hours.
    pub early_hours: bool,
    /// Status of major stock exchanges.
    pub exchanges: ExchangeStatus,
    /// Status of various index groups.
    #[serde(default)]
    pub indices_groups: Option<IndicesGroupStatus>,
    /// Overall market status (e.g., "open", "closed", "extended-hours").
    pub market: String,
    /// Current server time in RFC3339 format.
    pub server_time: String,
}

/// Status of currency markets.
#[derive(Debug, Clone, Deserialize)]
pub struct CurrencyStatus {
    /// Crypto market status.
    pub crypto: String,
    /// Forex market status.
    pub fx: String,
}

/// Status of major stock exchanges.
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeStatus {
    /// NASDAQ exchange status.
    pub nasdaq: String,
    /// NYSE exchange status.
    pub nyse: String,
    /// OTC market status.
    pub otc: String,
}

/// Status of various index groups.
#[derive(Debug, Clone, Deserialize)]
pub struct IndicesGroupStatus {
    /// CCCY index status.
    #[serde(default)]
    pub cccy: Option<String>,
    /// CGI index status.
    #[serde(default)]
    pub cgi: Option<String>,
    /// Dow Jones index status.
    #[serde(default)]
    pub dow_jones: Option<String>,
    /// FTSE Russell index status.
    #[serde(default)]
    pub ftse_russell: Option<String>,
    /// MSCI index status.
    #[serde(default)]
    pub msci: Option<String>,
    /// Morningstar index status.
    #[serde(default)]
    pub mstar: Option<String>,
    /// Morningstar C index status.
    #[serde(default)]
    pub mstarc: Option<String>,
    /// NASDAQ index status.
    #[serde(default)]
    pub nasdaq: Option<String>,
    /// S&P index status.
    #[serde(default)]
    pub s_and_p: Option<String>,
    /// Societe Generale index status.
    #[serde(default)]
    pub societe_generale: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_status_request_path() {
        let req = GetMarketStatusRequest;
        assert_eq!(req.path(), "/v1/marketstatus/now");
        assert_eq!(req.method(), Method::GET);
    }

    #[test]
    fn test_market_status_request_query_empty() {
        let req = GetMarketStatusRequest;
        assert!(req.query().is_empty());
    }

    #[test]
    fn test_market_status_response_deserialize() {
        let json = r#"{
            "afterHours": true,
            "currencies": {
                "crypto": "open",
                "fx": "open"
            },
            "earlyHours": false,
            "exchanges": {
                "nasdaq": "closed",
                "nyse": "closed",
                "otc": "closed"
            },
            "market": "extended-hours",
            "serverTime": "2024-01-15T20:30:00-05:00"
        }"#;

        let response: MarketStatusResponse = serde_json::from_str(json).unwrap();
        assert!(response.after_hours);
        assert!(!response.early_hours);
        assert_eq!(response.market, "extended-hours");
        assert_eq!(response.currencies.crypto, "open");
        assert_eq!(response.currencies.fx, "open");
        assert_eq!(response.exchanges.nasdaq, "closed");
        assert_eq!(response.exchanges.nyse, "closed");
        assert_eq!(response.exchanges.otc, "closed");
        assert_eq!(response.server_time, "2024-01-15T20:30:00-05:00");
    }

    #[test]
    fn test_market_status_response_with_indices() {
        let json = r#"{
            "afterHours": false,
            "currencies": {
                "crypto": "open",
                "fx": "open"
            },
            "earlyHours": false,
            "exchanges": {
                "nasdaq": "open",
                "nyse": "open",
                "otc": "open"
            },
            "indicesGroups": {
                "dow_jones": "open",
                "nasdaq": "open",
                "s_and_p": "open"
            },
            "market": "open",
            "serverTime": "2024-01-15T10:30:00-05:00"
        }"#;

        let response: MarketStatusResponse = serde_json::from_str(json).unwrap();
        assert!(!response.after_hours);
        assert_eq!(response.market, "open");

        let indices = response.indices_groups.unwrap();
        assert_eq!(indices.dow_jones.as_deref(), Some("open"));
        assert_eq!(indices.nasdaq.as_deref(), Some("open"));
        assert_eq!(indices.s_and_p.as_deref(), Some("open"));
    }
}
