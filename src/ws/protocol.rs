//! WebSocket protocol messages and subscription types.
//!
//! This module defines the message formats used for WebSocket
//! communication, including authentication and subscription messages.

use serde::Serialize;
use smol_str::SmolStr;

/// Subscription topic for WebSocket streams.
///
/// Subscriptions follow a `{type}.{symbol}` format, where the type
/// determines what kind of events are received.
///
/// # Example
///
/// ```
/// use massive_rs::ws::Subscription;
///
/// // Subscribe to Apple trades
/// let trade_sub = Subscription::trade("AAPL");
///
/// // Subscribe to all trades
/// let all_trades = Subscription::all_trades();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subscription(SmolStr);

impl Subscription {
    /// Trade subscription: `T.{symbol}`
    ///
    /// Receives real-time tick-level trade events.
    pub fn trade(symbol: &str) -> Self {
        Self(SmolStr::new(format!("T.{}", symbol)))
    }

    /// Quote subscription: `Q.{symbol}`
    ///
    /// Receives real-time NBBO (National Best Bid/Offer) quotes.
    pub fn quote(symbol: &str) -> Self {
        Self(SmolStr::new(format!("Q.{}", symbol)))
    }

    /// Second aggregate subscription: `A.{symbol}`
    ///
    /// Receives per-second OHLCV bars.
    pub fn second_agg(symbol: &str) -> Self {
        Self(SmolStr::new(format!("A.{}", symbol)))
    }

    /// Minute aggregate subscription: `AM.{symbol}`
    ///
    /// Receives per-minute OHLCV bars.
    pub fn minute_agg(symbol: &str) -> Self {
        Self(SmolStr::new(format!("AM.{}", symbol)))
    }

    /// Subscribe to all trades: `T.*`
    pub fn all_trades() -> Self {
        Self(SmolStr::new_static("T.*"))
    }

    /// Subscribe to all quotes: `Q.*`
    pub fn all_quotes() -> Self {
        Self(SmolStr::new_static("Q.*"))
    }

    /// Subscribe to all second aggregates: `A.*`
    pub fn all_second_aggs() -> Self {
        Self(SmolStr::new_static("A.*"))
    }

    /// Subscribe to all minute aggregates: `AM.*`
    pub fn all_minute_aggs() -> Self {
        Self(SmolStr::new_static("AM.*"))
    }

    /// Create from a raw subscription string.
    ///
    /// Use this for custom or less common subscription types.
    pub fn raw(s: impl Into<SmolStr>) -> Self {
        Self(s.into())
    }

    /// Get the subscription string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Subscription {
    fn from(s: &str) -> Self {
        Self::raw(s)
    }
}

impl From<String> for Subscription {
    fn from(s: String) -> Self {
        Self::raw(s)
    }
}

/// Authentication message sent to the WebSocket server.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WsAuthMessage {
    /// Action type (always "auth")
    pub action: String,
    /// API key
    pub params: String,
}

impl WsAuthMessage {
    /// Create a new authentication message.
    pub fn new(api_key: &str) -> Self {
        Self {
            action: "auth".to_string(),
            params: api_key.to_string(),
        }
    }
}

/// Subscribe/unsubscribe message sent to the WebSocket server.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WsSubscribeMessage {
    /// Action type ("subscribe" or "unsubscribe")
    pub action: String,
    /// Comma-separated list of subscriptions
    pub params: String,
}

impl WsSubscribeMessage {
    /// Create a subscribe message.
    pub fn subscribe(topics: &[Subscription]) -> Self {
        Self {
            action: "subscribe".to_string(),
            params: topics
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(","),
        }
    }

    /// Create an unsubscribe message.
    pub fn unsubscribe(topics: &[Subscription]) -> Self {
        Self {
            action: "unsubscribe".to_string(),
            params: topics
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(","),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_trade() {
        let sub = Subscription::trade("AAPL");
        assert_eq!(sub.as_str(), "T.AAPL");
        assert_eq!(format!("{}", sub), "T.AAPL");
    }

    #[test]
    fn test_subscription_quote() {
        let sub = Subscription::quote("MSFT");
        assert_eq!(sub.as_str(), "Q.MSFT");
    }

    #[test]
    fn test_subscription_aggregates() {
        let second = Subscription::second_agg("TSLA");
        assert_eq!(second.as_str(), "A.TSLA");

        let minute = Subscription::minute_agg("TSLA");
        assert_eq!(minute.as_str(), "AM.TSLA");
    }

    #[test]
    fn test_subscription_wildcard() {
        assert_eq!(Subscription::all_trades().as_str(), "T.*");
        assert_eq!(Subscription::all_quotes().as_str(), "Q.*");
        assert_eq!(Subscription::all_second_aggs().as_str(), "A.*");
        assert_eq!(Subscription::all_minute_aggs().as_str(), "AM.*");
    }

    #[test]
    fn test_subscription_raw() {
        let sub = Subscription::raw("LULD.AAPL");
        assert_eq!(sub.as_str(), "LULD.AAPL");
    }

    #[test]
    fn test_subscription_from_str() {
        let sub: Subscription = "T.GOOG".into();
        assert_eq!(sub.as_str(), "T.GOOG");
    }

    #[test]
    fn test_subscription_equality() {
        let a = Subscription::trade("AAPL");
        let b = Subscription::trade("AAPL");
        let c = Subscription::trade("MSFT");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_auth_message_serialize() {
        let msg = WsAuthMessage::new("my-api-key");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"action\":\"auth\""));
        assert!(json.contains("\"params\":\"my-api-key\""));
    }

    #[test]
    fn test_subscribe_message() {
        let subs = vec![Subscription::trade("AAPL"), Subscription::trade("MSFT")];
        let msg = WsSubscribeMessage::subscribe(&subs);
        assert_eq!(msg.action, "subscribe");
        assert_eq!(msg.params, "T.AAPL,T.MSFT");
    }

    #[test]
    fn test_unsubscribe_message() {
        let subs = vec![Subscription::quote("GOOG")];
        let msg = WsSubscribeMessage::unsubscribe(&subs);
        assert_eq!(msg.action, "unsubscribe");
        assert_eq!(msg.params, "Q.GOOG");
    }
}
