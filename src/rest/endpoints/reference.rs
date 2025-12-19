//! Reference data endpoints (tickers, markets, exchanges).
//!
//! This module contains request types for fetching reference data
//! from the Massive API.

use crate::models::Ticker;
use crate::rest::models::ListEnvelope;
use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

/// Market type filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketType {
    /// US equities
    Stocks,
    /// Options contracts
    Options,
    /// Cryptocurrency
    Crypto,
    /// Foreign exchange
    Forex,
    /// OTC securities
    Otc,
    /// Indices
    Indices,
}

impl std::fmt::Display for MarketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketType::Stocks => write!(f, "stocks"),
            MarketType::Options => write!(f, "options"),
            MarketType::Crypto => write!(f, "crypto"),
            MarketType::Forex => write!(f, "fx"),
            MarketType::Otc => write!(f, "otc"),
            MarketType::Indices => write!(f, "indices"),
        }
    }
}

/// Request for listing tickers.
///
/// Returns a list of tickers matching the filter criteria.
///
/// # Example
///
/// ```
/// use massive_rs::rest::endpoints::{GetTickersRequest, MarketType};
///
/// let request = GetTickersRequest::default()
///     .market(MarketType::Stocks)
///     .search("AAPL")
///     .active(true)
///     .limit(100);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetTickersRequest {
    /// Filter by ticker symbol prefix
    pub ticker: Option<String>,
    /// Search by ticker or company name
    pub search: Option<String>,
    /// Filter by market type
    pub market: Option<MarketType>,
    /// Filter by exchange
    pub exchange: Option<String>,
    /// Filter by CUSIP
    pub cusip: Option<String>,
    /// Filter by CIK
    pub cik: Option<String>,
    /// Date for which to check ticker status
    pub date: Option<String>,
    /// Filter by active status
    pub active: Option<bool>,
    /// Maximum results per page
    pub limit: Option<u32>,
    /// Pagination cursor
    pub cursor: Option<String>,
}

impl GetTickersRequest {
    /// Filter by ticker symbol prefix.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Search by ticker or company name.
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// Filter by market type.
    pub fn market(mut self, market: MarketType) -> Self {
        self.market = Some(market);
        self
    }

    /// Filter by exchange.
    pub fn exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = Some(exchange.into());
        self
    }

    /// Filter by active status.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Set maximum results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the pagination cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}

impl RestRequest for GetTickersRequest {
    type Response = ListEnvelope<Ticker>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v3/reference/tickers".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.clone());
        params.push_opt_param("search", self.search.clone());
        params.push_opt_param("market", self.market.map(|m| m.to_string()));
        params.push_opt_param("exchange", self.exchange.clone());
        params.push_opt_param("cusip", self.cusip.clone());
        params.push_opt_param("cik", self.cik.clone());
        params.push_opt_param("date", self.date.clone());
        params.push_opt_param("active", self.active);
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("cursor", self.cursor.clone());
        params
    }
}

impl PaginatableRequest for GetTickersRequest {
    type Item = Ticker;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

/// Request for ticker details.
///
/// Returns detailed information about a specific ticker.
#[derive(Debug, Clone)]
pub struct GetTickerDetailsRequest {
    /// Ticker symbol
    pub ticker: String,
    /// Date for which to get details
    pub date: Option<String>,
}

impl GetTickerDetailsRequest {
    /// Create a new ticker details request.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            date: None,
        }
    }

    /// Set the date for historical data.
    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }
}

/// Response from ticker details endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerDetailsResponse {
    /// Status string
    pub status: String,
    /// Request ID
    pub request_id: String,
    /// Ticker details
    pub results: TickerDetails,
}

/// Detailed ticker information.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerDetails {
    /// Ticker symbol
    pub ticker: String,
    /// Company name
    pub name: String,
    /// Market type
    pub market: String,
    /// Locale (us, global)
    pub locale: String,
    /// Primary exchange
    pub primary_exchange: Option<String>,
    /// Asset type
    #[serde(rename = "type")]
    pub ticker_type: Option<String>,
    /// Whether the ticker is active
    pub active: bool,
    /// Currency code
    pub currency_name: Option<String>,
    /// CIK number
    pub cik: Option<String>,
    /// Composite FIGI
    pub composite_figi: Option<String>,
    /// Share class FIGI
    pub share_class_figi: Option<String>,
    /// Market cap
    pub market_cap: Option<f64>,
    /// Phone number
    pub phone_number: Option<String>,
    /// Company address
    pub address: Option<Address>,
    /// Company description
    pub description: Option<String>,
    /// SIC code
    pub sic_code: Option<String>,
    /// SIC description
    pub sic_description: Option<String>,
    /// Number of employees
    pub total_employees: Option<u64>,
    /// List date
    pub list_date: Option<String>,
    /// Company homepage URL
    pub homepage_url: Option<String>,
    /// Branding info
    pub branding: Option<Branding>,
    /// Share class shares outstanding
    pub share_class_shares_outstanding: Option<u64>,
    /// Weighted shares outstanding
    pub weighted_shares_outstanding: Option<u64>,
    /// Round lot size
    pub round_lot: Option<u32>,
}

/// Company address.
#[derive(Debug, Clone, Deserialize)]
pub struct Address {
    /// Street address
    pub address1: Option<String>,
    /// City
    pub city: Option<String>,
    /// State
    pub state: Option<String>,
    /// Postal code
    pub postal_code: Option<String>,
}

/// Company branding info.
#[derive(Debug, Clone, Deserialize)]
pub struct Branding {
    /// Logo URL
    pub logo_url: Option<String>,
    /// Icon URL
    pub icon_url: Option<String>,
}

impl RestRequest for GetTickerDetailsRequest {
    type Response = TickerDetailsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v3/reference/tickers/{}", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("date", self.date.clone());
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_type_display() {
        assert_eq!(MarketType::Stocks.to_string(), "stocks");
        assert_eq!(MarketType::Options.to_string(), "options");
        assert_eq!(MarketType::Crypto.to_string(), "crypto");
        assert_eq!(MarketType::Forex.to_string(), "fx");
        assert_eq!(MarketType::Otc.to_string(), "otc");
        assert_eq!(MarketType::Indices.to_string(), "indices");
    }

    #[test]
    fn test_get_tickers_request_path() {
        let req = GetTickersRequest::default();
        assert_eq!(req.path(), "/v3/reference/tickers");
    }

    #[test]
    fn test_get_tickers_request_query() {
        let req = GetTickersRequest::default()
            .market(MarketType::Stocks)
            .active(true)
            .search("Apple")
            .limit(50);

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();

        assert_eq!(query_map.get("market").unwrap(), "stocks");
        assert_eq!(query_map.get("active").unwrap(), "true");
        assert_eq!(query_map.get("search").unwrap(), "Apple");
        assert_eq!(query_map.get("limit").unwrap(), "50");
    }

    #[test]
    fn test_get_ticker_details_request() {
        let req = GetTickerDetailsRequest::new("AAPL");
        assert_eq!(req.path(), "/v3/reference/tickers/AAPL");

        let req_with_date = GetTickerDetailsRequest::new("AAPL").date("2024-01-15");
        let query = req_with_date.query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].1, "2024-01-15");
    }

    #[test]
    fn test_ticker_details_response_deserialize() {
        let json = r#"{
            "status": "OK",
            "request_id": "abc123",
            "results": {
                "ticker": "AAPL",
                "name": "Apple Inc.",
                "market": "stocks",
                "locale": "us",
                "primary_exchange": "XNAS",
                "type": "CS",
                "active": true,
                "currency_name": "usd",
                "cik": "0000320193"
            }
        }"#;

        let response: TickerDetailsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "OK");
        assert_eq!(response.results.ticker, "AAPL");
        assert_eq!(response.results.name, "Apple Inc.");
        assert!(response.results.active);
    }
}
