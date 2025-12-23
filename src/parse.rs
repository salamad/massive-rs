//! High-performance JSON parsing utilities.
//!
//! This module provides optimized JSON parsing for WebSocket messages,
//! with optional SIMD acceleration when the `simd-json` feature is enabled.
//!
//! # Performance Characteristics
//!
//! - Standard serde_json: ~1-2μs per trade message
//! - SIMD-JSON (with feature): ~0.3-0.5μs per trade message
//!
//! # Usage
//!
//! The parsing functions automatically use SIMD when available:
//!
//! ```
//! use massive_rs::parse::parse_ws_events;
//!
//! let json = r#"[{"ev":"T","sym":"AAPL","x":4,"i":"123","z":3,"p":150.0,"s":100,"t":1234567890,"q":1}]"#;
//! let events = parse_ws_events(json).unwrap();
//! assert_eq!(events.len(), 1);
//! ```

use crate::error::MassiveError;
use crate::ws::models::events::WsEvent;

/// Parse WebSocket events from a JSON string.
///
/// This function handles both single events and arrays of events,
/// as the Massive API may send either format.
///
/// # Arguments
///
/// * `text` - The JSON string to parse
///
/// # Returns
///
/// A vector of parsed events, or an error if parsing fails.
///
/// # Example
///
/// ```
/// use massive_rs::parse::parse_ws_events;
///
/// // Single event
/// let text = r#"{"ev":"status","status":"connected"}"#;
/// let events = parse_ws_events(text).unwrap();
/// assert_eq!(events.len(), 1);
///
/// // Array of events
/// let text = r#"[{"ev":"status","status":"connected"}]"#;
/// let events = parse_ws_events(text).unwrap();
/// assert_eq!(events.len(), 1);
/// ```
pub fn parse_ws_events(text: &str) -> Result<Vec<WsEvent>, MassiveError> {
    let trimmed = text.trim();

    if trimmed.starts_with('[') {
        parse_array(trimmed)
    } else {
        parse_single(trimmed)
    }
}

/// Parse WebSocket events from a mutable byte slice.
///
/// This variant is optimized for SIMD-JSON which requires mutable input.
/// When SIMD-JSON is not enabled, it converts to string and uses standard parsing.
///
/// # Arguments
///
/// * `bytes` - Mutable byte slice containing JSON (will be modified by SIMD-JSON)
///
/// # Safety
///
/// The input bytes may be modified in place when using SIMD-JSON.
#[cfg(feature = "simd-json")]
pub fn parse_ws_events_bytes(bytes: &mut [u8]) -> Result<Vec<WsEvent>, MassiveError> {
    // SIMD-JSON requires mutable input
    let trimmed_len = trim_ascii_bytes(bytes);
    let trimmed = &mut bytes[..trimmed_len];

    if trimmed.first() == Some(&b'[') {
        simd_json::from_slice(trimmed).map_err(|e| MassiveError::Deserialize {
            source: serde_json::from_str::<()>("").unwrap_err(), // Placeholder error
            body_snippet: format!(
                "simd-json error: {} | {}",
                e,
                String::from_utf8_lossy(&trimmed[..trimmed.len().min(100)])
            ),
        })
    } else {
        let event: WsEvent =
            simd_json::from_slice(trimmed).map_err(|e| MassiveError::Deserialize {
                source: serde_json::from_str::<()>("").unwrap_err(), // Placeholder error
                body_snippet: format!(
                    "simd-json error: {} | {}",
                    e,
                    String::from_utf8_lossy(&trimmed[..trimmed.len().min(100)])
                ),
            })?;
        Ok(vec![event])
    }
}

/// Parse WebSocket events from a mutable byte slice (non-SIMD fallback).
#[cfg(not(feature = "simd-json"))]
pub fn parse_ws_events_bytes(bytes: &mut [u8]) -> Result<Vec<WsEvent>, MassiveError> {
    let text = std::str::from_utf8(bytes).map_err(|e| MassiveError::Deserialize {
        source: serde_json::Error::custom(e.to_string()),
        body_snippet: String::from_utf8_lossy(&bytes[..bytes.len().min(100)]).to_string(),
    })?;
    parse_ws_events(text)
}

/// Parse a JSON array of events.
fn parse_array(text: &str) -> Result<Vec<WsEvent>, MassiveError> {
    serde_json::from_str(text).map_err(|e| MassiveError::Deserialize {
        source: e,
        body_snippet: text[..text.len().min(100)].to_string(),
    })
}

/// Parse a single JSON event.
fn parse_single(text: &str) -> Result<Vec<WsEvent>, MassiveError> {
    let event: WsEvent = serde_json::from_str(text).map_err(|e| MassiveError::Deserialize {
        source: e,
        body_snippet: text[..text.len().min(100)].to_string(),
    })?;
    Ok(vec![event])
}

/// Trim ASCII whitespace from a byte slice, returning the trimmed length.
#[cfg(feature = "simd-json")]
fn trim_ascii_bytes(bytes: &[u8]) -> usize {
    let start = bytes
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|&b| !b.is_ascii_whitespace())
        .map_or(start, |p| p + 1);
    end - start
}

/// Estimate the number of events in a JSON message.
///
/// This is a fast heuristic that counts array elements without full parsing.
/// Useful for pre-allocating buffers.
pub fn estimate_event_count(text: &str) -> usize {
    let trimmed = text.trim();
    if !trimmed.starts_with('[') {
        return 1;
    }

    // Count opening braces at depth 1 (array elements)
    let mut depth = 0;
    let mut count = 0;

    for c in trimmed.chars() {
        match c {
            '[' | '{' => {
                if depth == 1 && c == '{' {
                    count += 1;
                }
                depth += 1;
            }
            ']' | '}' => depth -= 1,
            _ => {}
        }
    }

    count.max(1)
}

/// Check if the message appears to be a status/control message.
///
/// Status messages typically have "ev":"status" and can be processed
/// separately from market data.
pub fn is_status_message(text: &str) -> bool {
    text.contains(r#""ev":"status""#) || text.contains(r#""ev": "status""#)
}

/// Extract the event type from a JSON message without full parsing.
///
/// Returns the event type string if found, or None if not.
pub fn extract_event_type(text: &str) -> Option<&str> {
    // Look for "ev":"X" or "ev": "X" patterns
    let patterns = [r#""ev":""#, r#""ev": ""#];

    for pattern in patterns {
        if let Some(start) = text.find(pattern) {
            let ev_start = start + pattern.len();
            if let Some(end) = text[ev_start..].find('"') {
                return Some(&text[ev_start..ev_start + end]);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_event() {
        let text = r#"{"ev":"status","status":"connected","message":"Connected"}"#;
        let events = parse_ws_events(text).unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::Status(s) => {
                assert_eq!(s.status, "connected");
            }
            _ => panic!("Expected status event"),
        }
    }

    #[test]
    fn test_parse_array_events() {
        let text = r#"[{"ev":"status","status":"auth_success"}]"#;
        let events = parse_ws_events(text).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let text = r#"  { "ev": "status", "status": "connected" }  "#;
        let events = parse_ws_events(text).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_estimate_event_count_single() {
        let text = r#"{"ev":"T","sym":"AAPL","p":150.0}"#;
        assert_eq!(estimate_event_count(text), 1);
    }

    #[test]
    fn test_estimate_event_count_array() {
        let text = r#"[{"ev":"T"},{"ev":"T"},{"ev":"T"}]"#;
        assert_eq!(estimate_event_count(text), 3);
    }

    #[test]
    fn test_is_status_message() {
        assert!(is_status_message(r#"{"ev":"status","status":"connected"}"#));
        assert!(is_status_message(
            r#"{"ev": "status", "status": "auth_success"}"#
        ));
        assert!(!is_status_message(r#"{"ev":"T","sym":"AAPL"}"#));
    }

    #[test]
    fn test_extract_event_type() {
        assert_eq!(extract_event_type(r#"{"ev":"T","sym":"AAPL"}"#), Some("T"));
        assert_eq!(extract_event_type(r#"{"ev": "status"}"#), Some("status"));
        assert_eq!(
            extract_event_type(r#"{"ev":"AM","sym":"AAPL"}"#),
            Some("AM")
        );
        assert_eq!(extract_event_type(r#"{"foo":"bar"}"#), None);
    }

    #[test]
    fn test_parse_bytes() {
        let mut bytes = br#"{"ev":"status","status":"connected"}"#.to_vec();
        let events = parse_ws_events_bytes(&mut bytes).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_trade_event() {
        let text = r#"{"ev":"T","sym":"AAPL","x":4,"i":"123","z":3,"p":150.25,"s":100,"c":[0],"t":1703001234567,"q":12345}"#;
        let events = parse_ws_events(text).unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::Trade(t) => {
                assert_eq!(t.sym.as_str(), "AAPL");
                assert_eq!(t.p, 150.25);
                assert_eq!(t.s, 100);
            }
            _ => panic!("Expected trade event"),
        }
    }

    #[test]
    fn test_parse_quote_event() {
        let text = r#"{"ev":"Q","sym":"AAPL","bx":4,"bp":150.0,"bs":100,"ax":7,"ap":150.10,"as":200,"t":1703001234567}"#;
        let events = parse_ws_events(text).unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            WsEvent::Quote(q) => {
                assert_eq!(q.sym.as_str(), "AAPL");
                assert_eq!(q.bp, 150.0);
                assert_eq!(q.ap, 150.10);
            }
            _ => panic!("Expected quote event"),
        }
    }
}
