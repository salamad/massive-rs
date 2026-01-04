//! News endpoints.
//!
//! This module provides endpoints for news articles and sentiment data.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::GetNewsRequest;
//!
//! // Get news for Apple
//! let request = GetNewsRequest::new()
//!     .ticker("AAPL")
//!     .limit(10);
//! ```

use crate::rest::request::{PaginatableRequest, QueryBuilder, RestRequest};
use reqwest::Method;
use serde::Deserialize;
use std::borrow::Cow;

// ============================================================================
// News Types
// ============================================================================

/// Sentiment classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sentiment {
    /// Positive sentiment.
    Positive,
    /// Negative sentiment.
    Negative,
    /// Neutral sentiment.
    #[default]
    Neutral,
}

/// News publisher information.
#[derive(Debug, Clone, Deserialize)]
pub struct Publisher {
    /// Publisher name.
    pub name: String,
    /// Publisher homepage URL.
    pub homepage_url: Option<String>,
    /// Publisher logo URL.
    pub logo_url: Option<String>,
    /// Publisher favicon URL.
    pub favicon_url: Option<String>,
}

/// Ticker insight within a news article.
#[derive(Debug, Clone, Deserialize)]
pub struct NewsInsight {
    /// Ticker symbol.
    pub ticker: String,
    /// Sentiment for this ticker.
    pub sentiment: Option<Sentiment>,
    /// Reasoning for the sentiment.
    pub sentiment_reasoning: Option<String>,
}

/// News article.
#[derive(Debug, Clone, Deserialize)]
pub struct NewsArticle {
    /// Unique article ID.
    pub id: String,
    /// Article title.
    pub title: String,
    /// Article description/summary.
    pub description: Option<String>,
    /// Author name.
    pub author: Option<String>,
    /// Publication timestamp (ISO 8601).
    pub published_utc: String,
    /// URL to the full article.
    pub article_url: String,
    /// Tickers mentioned in the article.
    #[serde(default)]
    pub tickers: Vec<String>,
    /// AMP URL if available.
    pub amp_url: Option<String>,
    /// Image URL.
    pub image_url: Option<String>,
    /// Publisher information.
    pub publisher: Publisher,
    /// Ticker-specific insights.
    #[serde(default)]
    pub insights: Vec<NewsInsight>,
    /// Keywords/tags.
    #[serde(default)]
    pub keywords: Vec<String>,
}

impl NewsArticle {
    /// Get sentiment for a specific ticker.
    pub fn sentiment_for(&self, ticker: &str) -> Option<Sentiment> {
        self.insights
            .iter()
            .find(|i| i.ticker == ticker)
            .and_then(|i| i.sentiment)
    }

    /// Check if article mentions a specific ticker.
    pub fn mentions(&self, ticker: &str) -> bool {
        self.tickers.iter().any(|t| t == ticker)
    }

    /// Get all positive sentiment tickers.
    pub fn positive_tickers(&self) -> Vec<&str> {
        self.insights
            .iter()
            .filter(|i| matches!(i.sentiment, Some(Sentiment::Positive)))
            .map(|i| i.ticker.as_str())
            .collect()
    }

    /// Get all negative sentiment tickers.
    pub fn negative_tickers(&self) -> Vec<&str> {
        self.insights
            .iter()
            .filter(|i| matches!(i.sentiment, Some(Sentiment::Negative)))
            .map(|i| i.ticker.as_str())
            .collect()
    }
}

/// Response from news endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Next URL for pagination.
    pub next_url: Option<String>,
    /// News results.
    #[serde(default)]
    pub results: Vec<NewsArticle>,
}

/// Request for news articles.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetNewsRequest;
///
/// let request = GetNewsRequest::new()
///     .ticker("AAPL")
///     .published_after("2024-01-01")
///     .limit(20);
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetNewsRequest {
    /// Filter by ticker.
    pub ticker: Option<String>,
    /// Filter by multiple tickers.
    pub tickers: Option<Vec<String>>,
    /// Published after date (inclusive).
    pub published_after: Option<String>,
    /// Published before date (inclusive).
    pub published_before: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Sort order.
    pub order: Option<String>,
    /// Sort by field.
    pub sort: Option<String>,
}

impl GetNewsRequest {
    /// Create a new news request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by a single ticker.
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by multiple tickers.
    pub fn tickers(mut self, tickers: Vec<String>) -> Self {
        self.tickers = Some(tickers);
        self
    }

    /// Filter by publish date (after).
    pub fn published_after(mut self, date: impl Into<String>) -> Self {
        self.published_after = Some(date.into());
        self
    }

    /// Filter by publish date (before).
    pub fn published_before(mut self, date: impl Into<String>) -> Self {
        self.published_before = Some(date.into());
        self
    }

    /// Set result limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set sort order.
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Set sort field.
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }
}

impl RestRequest for GetNewsRequest {
    type Response = NewsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        "/v2/reference/news".into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        params.push_opt_param("ticker", self.ticker.as_ref());
        if let Some(ref tickers) = self.tickers {
            params.push((Cow::Borrowed("ticker"), tickers.join(",")));
        }
        params.push_opt_param("published_utc.gte", self.published_after.as_ref());
        params.push_opt_param("published_utc.lte", self.published_before.as_ref());
        params.push_opt_param("limit", self.limit);
        params.push_opt_param("order", self.order.as_ref());
        params.push_opt_param("sort", self.sort.as_ref());
        params
    }
}

impl PaginatableRequest for GetNewsRequest {
    type Item = NewsArticle;

    fn extract_items(response: Self::Response) -> Vec<Self::Item> {
        response.results
    }

    fn extract_next_url(response: &Self::Response) -> Option<&str> {
        response.next_url.as_deref()
    }
}

// ============================================================================
// Related Companies
// ============================================================================

/// Related company/ticker.
#[derive(Debug, Clone, Deserialize)]
pub struct RelatedCompany {
    /// Ticker symbol.
    pub ticker: String,
}

/// Response from related companies endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RelatedCompaniesResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Count of results.
    pub count: Option<u64>,
    /// Related companies.
    #[serde(default)]
    pub results: Vec<RelatedCompany>,
}

/// Request for related companies.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetRelatedCompaniesRequest;
///
/// let request = GetRelatedCompaniesRequest::new("AAPL");
/// ```
#[derive(Debug, Clone)]
pub struct GetRelatedCompaniesRequest {
    /// Ticker to find related companies for.
    pub ticker: String,
}

impl GetRelatedCompaniesRequest {
    /// Create a new request for related companies.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
        }
    }
}

impl RestRequest for GetRelatedCompaniesRequest {
    type Response = RelatedCompaniesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/v1/related-companies/{}", self.ticker).into()
    }
}

// ============================================================================
// Ticker Events
// ============================================================================

/// Ticker event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TickerEventType {
    /// Stock split.
    #[default]
    StockSplit,
    /// Stock dividend.
    StockDividend,
    /// Spin-off.
    SpinOff,
    /// Ticker change.
    TickerChange,
    /// Company name change.
    CompanyChange,
    /// Unknown event type.
    #[serde(other)]
    Unknown,
}

/// Ticker change details.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerChangeDetails {
    /// Old ticker.
    pub ticker: Option<String>,
}

/// Ticker event.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerEvent {
    /// Event type.
    #[serde(rename = "type")]
    pub event_type: TickerEventType,
    /// Event date.
    pub date: Option<String>,
    /// Ticker change details (if applicable).
    pub ticker_change: Option<TickerChangeDetails>,
}

/// Ticker events result.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerEventsResult {
    /// Ticker name.
    pub name: Option<String>,
    /// Composite FIGI.
    pub composite_figi: Option<String>,
    /// CIK.
    pub cik: Option<String>,
    /// Events list.
    #[serde(default)]
    pub events: Vec<TickerEvent>,
}

/// Response from ticker events endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TickerEventsResponse {
    /// Status string.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Events results.
    pub results: Option<TickerEventsResult>,
}

/// Request for ticker events.
///
/// # Example
///
/// ```
/// use massive_rs::rest::GetTickerEventsRequest;
///
/// let request = GetTickerEventsRequest::new("AAPL");
/// ```
#[derive(Debug, Clone)]
pub struct GetTickerEventsRequest {
    /// Ticker symbol.
    pub ticker: String,
    /// Event types filter.
    pub types: Option<Vec<String>>,
}

impl GetTickerEventsRequest {
    /// Create a new request for ticker events.
    pub fn new(ticker: impl Into<String>) -> Self {
        Self {
            ticker: ticker.into(),
            types: None,
        }
    }

    /// Filter by event types.
    pub fn types(mut self, types: Vec<String>) -> Self {
        self.types = Some(types);
        self
    }
}

impl RestRequest for GetTickerEventsRequest {
    type Response = TickerEventsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> Cow<'static, str> {
        format!("/vX/reference/tickers/{}/events", self.ticker).into()
    }

    fn query(&self) -> Vec<(Cow<'static, str>, String)> {
        let mut params = Vec::new();
        if let Some(ref types) = self.types {
            params.push((Cow::Borrowed("types"), types.join(",")));
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_article_helpers() {
        let article = NewsArticle {
            id: "1".to_string(),
            title: "Test Article".to_string(),
            description: Some("Description".to_string()),
            author: Some("Author".to_string()),
            published_utc: "2024-01-15T12:00:00Z".to_string(),
            article_url: "https://example.com/article".to_string(),
            tickers: vec!["AAPL".to_string(), "MSFT".to_string()],
            amp_url: None,
            image_url: None,
            publisher: Publisher {
                name: "Test Publisher".to_string(),
                homepage_url: None,
                logo_url: None,
                favicon_url: None,
            },
            insights: vec![
                NewsInsight {
                    ticker: "AAPL".to_string(),
                    sentiment: Some(Sentiment::Positive),
                    sentiment_reasoning: Some("Strong earnings".to_string()),
                },
                NewsInsight {
                    ticker: "MSFT".to_string(),
                    sentiment: Some(Sentiment::Negative),
                    sentiment_reasoning: None,
                },
            ],
            keywords: vec![],
        };

        assert!(article.mentions("AAPL"));
        assert!(article.mentions("MSFT"));
        assert!(!article.mentions("GOOG"));

        assert_eq!(article.sentiment_for("AAPL"), Some(Sentiment::Positive));
        assert_eq!(article.sentiment_for("MSFT"), Some(Sentiment::Negative));
        assert_eq!(article.sentiment_for("GOOG"), None);

        assert_eq!(article.positive_tickers(), vec!["AAPL"]);
        assert_eq!(article.negative_tickers(), vec!["MSFT"]);
    }

    #[test]
    fn test_get_news_request() {
        let req = GetNewsRequest::new()
            .ticker("AAPL")
            .published_after("2024-01-01")
            .limit(20);

        assert_eq!(req.path(), "/v2/reference/news");

        let query = req.query();
        let query_map: std::collections::HashMap<_, _> = query.into_iter().collect();
        assert_eq!(query_map.get("ticker").unwrap(), "AAPL");
        assert_eq!(query_map.get("published_utc.gte").unwrap(), "2024-01-01");
        assert_eq!(query_map.get("limit").unwrap(), "20");
    }

    #[test]
    fn test_get_related_companies_request() {
        let req = GetRelatedCompaniesRequest::new("AAPL");

        assert_eq!(req.path(), "/v1/related-companies/AAPL");
    }

    #[test]
    fn test_get_ticker_events_request() {
        let req = GetTickerEventsRequest::new("AAPL")
            .types(vec!["stock_split".to_string(), "dividend".to_string()]);

        assert_eq!(req.path(), "/vX/reference/tickers/AAPL/events");

        let query = req.query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].1, "stock_split,dividend");
    }

    #[test]
    fn test_sentiment_deserialize() {
        let json = r#"{"ticker": "AAPL", "sentiment": "positive"}"#;
        let insight: NewsInsight = serde_json::from_str(json).unwrap();
        assert_eq!(insight.sentiment, Some(Sentiment::Positive));

        let json = r#"{"ticker": "AAPL", "sentiment": "negative"}"#;
        let insight: NewsInsight = serde_json::from_str(json).unwrap();
        assert_eq!(insight.sentiment, Some(Sentiment::Negative));
    }

    #[test]
    fn test_news_article_deserialize() {
        let json = r#"{
            "id": "abc123",
            "title": "Breaking News",
            "published_utc": "2024-01-15T12:00:00Z",
            "article_url": "https://example.com",
            "tickers": ["AAPL"],
            "publisher": {
                "name": "Test News"
            },
            "insights": [
                {"ticker": "AAPL", "sentiment": "positive"}
            ]
        }"#;

        let article: NewsArticle = serde_json::from_str(json).unwrap();
        assert_eq!(article.id, "abc123");
        assert_eq!(article.title, "Breaking News");
        assert_eq!(article.tickers.len(), 1);
        assert_eq!(article.insights.len(), 1);
    }

    #[test]
    fn test_ticker_event_type_deserialize() {
        let json = r#"{"type": "stock_split", "date": "2024-01-15"}"#;
        let event: TickerEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, TickerEventType::StockSplit);

        let json = r#"{"type": "spin_off"}"#;
        let event: TickerEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, TickerEventType::SpinOff);
    }
}
