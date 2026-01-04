//! Utility types and helpers.
//!
//! This module provides common types used throughout the crate, including
//! timestamp types, symbol handling, and query parameter building.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;

/// Unix timestamp in milliseconds.
///
/// This type represents timestamps as returned by the Massive API,
/// which uses Unix epoch milliseconds for most timestamp fields.
///
/// # Example
///
/// ```
/// use massive_rs::util::UnixMs;
///
/// let ts = UnixMs::now();
/// if let Some(datetime) = ts.as_datetime() {
///     println!("Current time: {}", datetime);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UnixMs(pub i64);

impl UnixMs {
    /// Create a timestamp for the current time.
    ///
    /// Returns 0 if the system clock is before Unix epoch (should never happen
    /// on properly configured systems).
    pub fn now() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
        )
    }

    /// Create from a raw millisecond value.
    pub fn from_millis(ms: i64) -> Self {
        Self(ms)
    }

    /// Get the raw millisecond value.
    pub fn as_millis(&self) -> i64 {
        self.0
    }

    /// Convert to a chrono DateTime.
    ///
    /// Returns `None` if the timestamp is out of range for chrono's DateTime
    /// (e.g., corrupted or invalid data).
    pub fn as_datetime(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::from_timestamp_millis(self.0)
    }

    /// Convert to a chrono DateTime, using Unix epoch as fallback.
    ///
    /// This is useful when you need a non-optional DateTime and can tolerate
    /// the fallback for invalid timestamps.
    pub fn as_datetime_or_epoch(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp_millis(self.0).unwrap_or_default()
    }

    /// Create from a chrono DateTime.
    pub fn from_datetime(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt.timestamp_millis())
    }
}

impl Serialize for UnixMs {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for UnixMs {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        i64::deserialize(d).map(UnixMs)
    }
}

impl From<i64> for UnixMs {
    fn from(ms: i64) -> Self {
        Self(ms)
    }
}

impl From<UnixMs> for i64 {
    fn from(ts: UnixMs) -> Self {
        ts.0
    }
}

/// Unix timestamp in nanoseconds.
///
/// This type is used for high-precision timestamps such as SIP timestamps
/// that require nanosecond resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UnixNs(pub i64);

impl UnixNs {
    /// Create a timestamp for the current time.
    ///
    /// Returns 0 if the system clock is before Unix epoch (should never happen
    /// on properly configured systems).
    pub fn now() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as i64)
                .unwrap_or(0),
        )
    }

    /// Create from a raw nanosecond value.
    pub fn from_nanos(ns: i64) -> Self {
        Self(ns)
    }

    /// Get the raw nanosecond value.
    pub fn as_nanos(&self) -> i64 {
        self.0
    }

    /// Convert to milliseconds (truncating).
    pub fn as_millis(&self) -> i64 {
        self.0 / 1_000_000
    }

    /// Convert to UnixMs (truncating).
    pub fn to_unix_ms(&self) -> UnixMs {
        UnixMs(self.as_millis())
    }
}

impl Serialize for UnixNs {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for UnixNs {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        i64::deserialize(d).map(UnixNs)
    }
}

impl From<i64> for UnixNs {
    fn from(ns: i64) -> Self {
        Self(ns)
    }
}

/// Type alias for ticker symbols using small-string optimization.
///
/// Most ticker symbols are short (1-5 characters), so using SmolStr
/// avoids heap allocation for these common cases.
pub type Symbol = SmolStr;

/// Create a Symbol from a string slice.
///
/// # Example
///
/// ```
/// use massive_rs::util::symbol;
///
/// let sym = symbol("AAPL");
/// assert_eq!(sym.as_str(), "AAPL");
/// ```
pub fn symbol(s: &str) -> Symbol {
    SmolStr::new(s)
}

/// Query parameter builder for REST requests.
///
/// This builder helps construct query parameters for API requests,
/// with support for optional values.
///
/// # Example
///
/// ```
/// use massive_rs::util::QueryParams;
///
/// let mut params = QueryParams::new();
/// params.push("limit", 100);
/// params.push_opt("adjusted", Some(true));
/// params.push_opt::<String>("cursor", None);
///
/// let pairs = params.into_pairs();
/// assert_eq!(pairs.len(), 2);
/// ```
#[derive(Debug, Default, Clone)]
pub struct QueryParams(Vec<(String, String)>);

impl QueryParams {
    /// Create a new empty query parameter builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a required parameter.
    pub fn push(&mut self, key: impl Into<String>, value: impl ToString) {
        self.0.push((key.into(), value.to_string()));
    }

    /// Add an optional parameter (only if value is Some).
    pub fn push_opt<T: ToString>(&mut self, key: impl Into<String>, value: Option<T>) {
        if let Some(v) = value {
            self.push(key, v);
        }
    }

    /// Check if there are any parameters.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Convert to a vector of key-value pairs.
    pub fn into_pairs(self) -> Vec<(String, String)> {
        self.0
    }

    /// Get an iterator over the parameters.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl IntoIterator for QueryParams {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<(String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Format a date as YYYY-MM-DD for API requests.
pub fn format_date(date: chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Parse a date from YYYY-MM-DD format.
pub fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_unix_ms_now() {
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let ts = UnixMs::now();

        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        assert!(ts.0 >= before);
        assert!(ts.0 <= after);
    }

    #[test]
    fn test_unix_ms_datetime_roundtrip() {
        let original = chrono::Utc::now();
        let ts = UnixMs::from_datetime(original);
        let recovered = ts.as_datetime().expect("valid timestamp should convert");

        // Should be within 1ms due to truncation
        let diff = (original.timestamp_millis() - recovered.timestamp_millis()).abs();
        assert!(diff <= 1);
    }

    #[test]
    fn test_unix_ms_as_datetime_or_epoch() {
        let ts = UnixMs::from_millis(1703001234567);
        let dt = ts.as_datetime_or_epoch();
        assert_eq!(dt.timestamp_millis(), 1703001234567);

        // Invalid timestamp should return epoch
        let invalid = UnixMs::from_millis(i64::MAX);
        let epoch = invalid.as_datetime_or_epoch();
        assert_eq!(epoch.timestamp(), 0);
    }

    #[test]
    fn test_unix_ms_serde() {
        let ts = UnixMs::from_millis(1703001234567);
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(json, "1703001234567");

        let parsed: UnixMs = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.0, ts.0);
    }

    #[test]
    fn test_unix_ns_to_ms() {
        let ns = UnixNs::from_nanos(1703001234567890123);
        let ms = ns.to_unix_ms();
        assert_eq!(ms.0, 1703001234567);
    }

    #[test]
    fn test_symbol() {
        let sym = symbol("AAPL");
        assert_eq!(sym.as_str(), "AAPL");

        // SmolStr should inline short strings
        let sym2 = symbol("A");
        assert_eq!(sym2.len(), 1);
    }

    #[test]
    fn test_query_params() {
        let mut params = QueryParams::new();
        assert!(params.is_empty());

        params.push("limit", 100);
        params.push("adjusted", true);
        params.push_opt("sort", Some("asc"));
        params.push_opt::<String>("cursor", None);

        assert_eq!(params.len(), 3);

        let pairs = params.into_pairs();
        assert_eq!(pairs[0], ("limit".to_string(), "100".to_string()));
        assert_eq!(pairs[1], ("adjusted".to_string(), "true".to_string()));
        assert_eq!(pairs[2], ("sort".to_string(), "asc".to_string()));
    }

    #[test]
    fn test_query_params_iter() {
        let mut params = QueryParams::new();
        params.push("a", "1");
        params.push("b", "2");

        let collected: Vec<_> = params.iter().collect();
        assert_eq!(collected, vec![("a", "1"), ("b", "2")]);
    }

    #[test]
    fn test_format_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(format_date(date), "2024-01-15");
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2024-01-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);

        assert!(parse_date("invalid").is_none());
    }
}
