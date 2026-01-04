//! Query parameter filter builders for range comparisons.
//!
//! Many Massive API endpoints support range filtering on fields using suffixes
//! like `.gt`, `.gte`, `.lt`, `.lte`, and `.any_of`. This module provides
//! type-safe builders for constructing these filters.
//!
//! # Example
//!
//! ```
//! use massive_rs::rest::filters::{RangeFilter, SortOrder};
//!
//! // Build a date range filter
//! let date_filter = RangeFilter::new()
//!     .gte("2024-01-01".to_string())
//!     .lte("2024-12-31".to_string());
//!
//! // Convert to query parameters
//! let params = date_filter.to_query_params("date");
//! assert_eq!(params.len(), 2);
//! ```

use std::fmt;

/// A filterable field supporting range comparisons (.gt, .gte, .lt, .lte, .any_of).
///
/// This type is generic over the filter value type `T`, allowing it to work with
/// strings, numbers, dates, and other comparable types.
///
/// # Supported Filter Operations
///
/// | Method | API Suffix | Description |
/// |--------|------------|-------------|
/// | `eq()` | (none) | Exact equality |
/// | `gt()` | `.gt` | Greater than |
/// | `gte()` | `.gte` | Greater than or equal |
/// | `lt()` | `.lt` | Less than |
/// | `lte()` | `.lte` | Less than or equal |
/// | `any_of()` | `.any_of` | Match any of the values |
///
/// # Example
///
/// ```
/// use massive_rs::rest::filters::RangeFilter;
///
/// // Numeric range filter
/// let price_filter: RangeFilter<f64> = RangeFilter::new()
///     .gte(100.0)
///     .lt(200.0);
///
/// // Date range filter
/// let date_filter: RangeFilter<String> = RangeFilter::new()
///     .gte("2024-01-01".to_string())
///     .lte("2024-03-31".to_string());
///
/// // Multiple exact values
/// let ticker_filter: RangeFilter<String> = RangeFilter::new()
///     .any_of(vec!["AAPL".to_string(), "MSFT".to_string(), "GOOG".to_string()]);
/// ```
#[derive(Debug, Clone)]
pub struct RangeFilter<T> {
    /// Exact equality value
    pub eq: Option<T>,
    /// Greater than value
    pub gt: Option<T>,
    /// Greater than or equal value
    pub gte: Option<T>,
    /// Less than value
    pub lt: Option<T>,
    /// Less than or equal value
    pub lte: Option<T>,
    /// Match any of these values
    pub any_of: Option<Vec<T>>,
}

impl<T> Default for RangeFilter<T> {
    fn default() -> Self {
        Self {
            eq: None,
            gt: None,
            gte: None,
            lt: None,
            lte: None,
            any_of: None,
        }
    }
}

impl<T: Clone> RangeFilter<T> {
    /// Create a new empty range filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set exact equality filter.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter = RangeFilter::new().eq("AAPL".to_string());
    /// ```
    pub fn eq(mut self, value: T) -> Self {
        self.eq = Some(value);
        self
    }

    /// Set greater than filter.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter: RangeFilter<i32> = RangeFilter::new().gt(100);
    /// ```
    pub fn gt(mut self, value: T) -> Self {
        self.gt = Some(value);
        self
    }

    /// Set greater than or equal filter.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter = RangeFilter::new().gte("2024-01-01".to_string());
    /// ```
    pub fn gte(mut self, value: T) -> Self {
        self.gte = Some(value);
        self
    }

    /// Set less than filter.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter: RangeFilter<f64> = RangeFilter::new().lt(100.0);
    /// ```
    pub fn lt(mut self, value: T) -> Self {
        self.lt = Some(value);
        self
    }

    /// Set less than or equal filter.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter = RangeFilter::new().lte("2024-12-31".to_string());
    /// ```
    pub fn lte(mut self, value: T) -> Self {
        self.lte = Some(value);
        self
    }

    /// Set filter to match any of the given values.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter = RangeFilter::new()
    ///     .any_of(vec!["AAPL".to_string(), "MSFT".to_string()]);
    /// ```
    pub fn any_of(mut self, values: Vec<T>) -> Self {
        self.any_of = Some(values);
        self
    }

    /// Check if the filter has no conditions set.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let empty: RangeFilter<String> = RangeFilter::new();
    /// assert!(empty.is_empty());
    ///
    /// let with_value = RangeFilter::new().eq("test".to_string());
    /// assert!(!with_value.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.eq.is_none()
            && self.gt.is_none()
            && self.gte.is_none()
            && self.lt.is_none()
            && self.lte.is_none()
            && self.any_of.is_none()
    }

    /// Create a range filter for values between two bounds (inclusive).
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter = RangeFilter::between("2024-01-01".to_string(), "2024-12-31".to_string());
    /// assert!(filter.gte.is_some());
    /// assert!(filter.lte.is_some());
    /// ```
    pub fn between(from: T, to: T) -> Self {
        Self::new().gte(from).lte(to)
    }
}

impl<T: ToString> RangeFilter<T> {
    /// Convert to query parameters with the given field name.
    ///
    /// Returns a vector of (key, value) pairs suitable for query string building.
    ///
    /// # Example
    ///
    /// ```
    /// use massive_rs::rest::filters::RangeFilter;
    ///
    /// let filter = RangeFilter::new()
    ///     .gte("2024-01-01".to_string())
    ///     .lte("2024-12-31".to_string());
    ///
    /// let params = filter.to_query_params("date");
    /// assert_eq!(params.len(), 2);
    /// assert!(params.contains(&("date.gte".to_string(), "2024-01-01".to_string())));
    /// assert!(params.contains(&("date.lte".to_string(), "2024-12-31".to_string())));
    /// ```
    pub fn to_query_params(&self, field: &str) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref v) = self.eq {
            params.push((field.to_string(), v.to_string()));
        }
        if let Some(ref v) = self.gt {
            params.push((format!("{}.gt", field), v.to_string()));
        }
        if let Some(ref v) = self.gte {
            params.push((format!("{}.gte", field), v.to_string()));
        }
        if let Some(ref v) = self.lt {
            params.push((format!("{}.lt", field), v.to_string()));
        }
        if let Some(ref v) = self.lte {
            params.push((format!("{}.lte", field), v.to_string()));
        }
        if let Some(ref vals) = self.any_of {
            let joined = vals
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push((format!("{}.any_of", field), joined));
        }

        params
    }

    /// Append filter parameters to an existing query builder.
    ///
    /// This is useful when building complex queries with multiple filters.
    pub fn append_to_params(&self, field: &str, params: &mut Vec<(String, String)>) {
        params.extend(self.to_query_params(field));
    }
}

/// Sort order specification for API requests.
///
/// Most Massive API endpoints support sorting results in ascending or descending order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortOrder {
    /// Sort in ascending order (smallest to largest, oldest to newest).
    Asc,
    /// Sort in descending order (largest to smallest, newest to oldest).
    #[default]
    Desc,
}

impl SortOrder {
    /// Get the string representation for API requests.
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A sort specification combining a field name and order.
///
/// # Example
///
/// ```
/// use massive_rs::rest::filters::{SortSpec, SortOrder};
///
/// let sort = SortSpec::new("timestamp", SortOrder::Desc);
/// assert_eq!(sort.to_query_value(), "timestamp.desc");
///
/// let default_sort = SortSpec::desc("date");
/// assert_eq!(default_sort.to_query_value(), "date.desc");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortSpec {
    /// The field to sort by.
    pub field: String,
    /// The sort order.
    pub order: SortOrder,
}

impl SortSpec {
    /// Create a new sort specification.
    pub fn new(field: impl Into<String>, order: SortOrder) -> Self {
        Self {
            field: field.into(),
            order,
        }
    }

    /// Create an ascending sort specification.
    pub fn asc(field: impl Into<String>) -> Self {
        Self::new(field, SortOrder::Asc)
    }

    /// Create a descending sort specification.
    pub fn desc(field: impl Into<String>) -> Self {
        Self::new(field, SortOrder::Desc)
    }

    /// Convert to the API query value format (`field.order`).
    pub fn to_query_value(&self) -> String {
        format!("{}.{}", self.field, self.order)
    }
}

impl fmt::Display for SortSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_query_value())
    }
}

/// Builder for multi-field sort specifications.
///
/// Some endpoints support sorting by multiple fields with a priority order.
///
/// # Example
///
/// ```
/// use massive_rs::rest::filters::{SortBuilder, SortOrder};
///
/// let sort = SortBuilder::new()
///     .add("date", SortOrder::Desc)
///     .add("ticker", SortOrder::Asc)
///     .build();
///
/// assert_eq!(sort, "date.desc,ticker.asc");
/// ```
#[derive(Debug, Clone, Default)]
pub struct SortBuilder {
    specs: Vec<SortSpec>,
}

impl SortBuilder {
    /// Create a new empty sort builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a sort specification.
    pub fn add(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.specs.push(SortSpec::new(field, order));
        self
    }

    /// Add an ascending sort specification.
    pub fn asc(self, field: impl Into<String>) -> Self {
        self.add(field, SortOrder::Asc)
    }

    /// Add a descending sort specification.
    pub fn desc(self, field: impl Into<String>) -> Self {
        self.add(field, SortOrder::Desc)
    }

    /// Build the final comma-separated sort string.
    pub fn build(&self) -> String {
        self.specs
            .iter()
            .map(|s| s.to_query_value())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Check if no sort specifications have been added.
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_filter_empty() {
        let filter: RangeFilter<String> = RangeFilter::new();
        assert!(filter.is_empty());
        assert!(filter.to_query_params("field").is_empty());
    }

    #[test]
    fn test_range_filter_eq() {
        let filter = RangeFilter::new().eq("AAPL".to_string());
        let params = filter.to_query_params("ticker");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("ticker".to_string(), "AAPL".to_string()));
    }

    #[test]
    fn test_range_filter_gt() {
        let filter: RangeFilter<i32> = RangeFilter::new().gt(100);
        let params = filter.to_query_params("price");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("price.gt".to_string(), "100".to_string()));
    }

    #[test]
    fn test_range_filter_gte() {
        let filter: RangeFilter<f64> = RangeFilter::new().gte(99.99);
        let params = filter.to_query_params("price");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("price.gte".to_string(), "99.99".to_string()));
    }

    #[test]
    fn test_range_filter_lt() {
        let filter: RangeFilter<i64> = RangeFilter::new().lt(1000);
        let params = filter.to_query_params("volume");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("volume.lt".to_string(), "1000".to_string()));
    }

    #[test]
    fn test_range_filter_lte() {
        let filter = RangeFilter::new().lte("2024-12-31".to_string());
        let params = filter.to_query_params("date");
        assert_eq!(params.len(), 1);
        assert_eq!(
            params[0],
            ("date.lte".to_string(), "2024-12-31".to_string())
        );
    }

    #[test]
    fn test_range_filter_any_of() {
        let filter = RangeFilter::new().any_of(vec![
            "AAPL".to_string(),
            "MSFT".to_string(),
            "GOOG".to_string(),
        ]);
        let params = filter.to_query_params("ticker");
        assert_eq!(params.len(), 1);
        assert_eq!(
            params[0],
            ("ticker.any_of".to_string(), "AAPL,MSFT,GOOG".to_string())
        );
    }

    #[test]
    fn test_range_filter_combined() {
        let filter = RangeFilter::new()
            .gte("2024-01-01".to_string())
            .lte("2024-12-31".to_string());

        let params = filter.to_query_params("date");
        assert_eq!(params.len(), 2);
        assert!(params.contains(&("date.gte".to_string(), "2024-01-01".to_string())));
        assert!(params.contains(&("date.lte".to_string(), "2024-12-31".to_string())));
    }

    #[test]
    fn test_range_filter_between() {
        let filter = RangeFilter::between(10, 20);
        assert_eq!(filter.gte, Some(10));
        assert_eq!(filter.lte, Some(20));
    }

    #[test]
    fn test_range_filter_append_to_params() {
        let filter = RangeFilter::new().gt(100);
        let mut params = vec![("ticker".to_string(), "AAPL".to_string())];
        filter.append_to_params("price", &mut params);

        assert_eq!(params.len(), 2);
        assert!(params.contains(&("ticker".to_string(), "AAPL".to_string())));
        assert!(params.contains(&("price.gt".to_string(), "100".to_string())));
    }

    #[test]
    fn test_sort_order_as_str() {
        assert_eq!(SortOrder::Asc.as_str(), "asc");
        assert_eq!(SortOrder::Desc.as_str(), "desc");
    }

    #[test]
    fn test_sort_order_display() {
        assert_eq!(format!("{}", SortOrder::Asc), "asc");
        assert_eq!(format!("{}", SortOrder::Desc), "desc");
    }

    #[test]
    fn test_sort_order_default() {
        assert_eq!(SortOrder::default(), SortOrder::Desc);
    }

    #[test]
    fn test_sort_spec_new() {
        let spec = SortSpec::new("date", SortOrder::Desc);
        assert_eq!(spec.field, "date");
        assert_eq!(spec.order, SortOrder::Desc);
        assert_eq!(spec.to_query_value(), "date.desc");
    }

    #[test]
    fn test_sort_spec_asc() {
        let spec = SortSpec::asc("ticker");
        assert_eq!(spec.to_query_value(), "ticker.asc");
    }

    #[test]
    fn test_sort_spec_desc() {
        let spec = SortSpec::desc("timestamp");
        assert_eq!(spec.to_query_value(), "timestamp.desc");
    }

    #[test]
    fn test_sort_spec_display() {
        let spec = SortSpec::new("volume", SortOrder::Desc);
        assert_eq!(format!("{}", spec), "volume.desc");
    }

    #[test]
    fn test_sort_builder_empty() {
        let builder = SortBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.build(), "");
    }

    #[test]
    fn test_sort_builder_single() {
        let sort = SortBuilder::new().desc("date").build();
        assert_eq!(sort, "date.desc");
    }

    #[test]
    fn test_sort_builder_multiple() {
        let sort = SortBuilder::new()
            .desc("date")
            .asc("ticker")
            .add("volume", SortOrder::Desc)
            .build();
        assert_eq!(sort, "date.desc,ticker.asc,volume.desc");
    }

    #[test]
    fn test_sort_builder_not_empty() {
        let builder = SortBuilder::new().desc("date");
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_range_filter_is_clone() {
        let filter = RangeFilter::new().eq("test".to_string());
        let cloned = filter.clone();
        assert_eq!(cloned.eq, Some("test".to_string()));
    }

    #[test]
    fn test_range_filter_is_debug() {
        let filter = RangeFilter::new().eq("test".to_string());
        let debug_str = format!("{:?}", filter);
        assert!(debug_str.contains("test"));
    }
}
