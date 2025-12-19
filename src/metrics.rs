//! Metrics and observability hooks.
//!
//! This module provides traits for custom metrics collection, allowing
//! integration with external monitoring systems like Prometheus, StatsD, etc.
//!
//! # Example
//!
//! ```
//! use massive_rs::metrics::{MetricsSink, NoopMetrics};
//!
//! // Use the no-op metrics sink (default)
//! let metrics = NoopMetrics;
//! metrics.counter("messages_received", 1, &[("ticker", "AAPL")]);
//!
//! // Or implement your own sink
//! struct MyMetrics;
//!
//! impl MetricsSink for MyMetrics {
//!     fn counter(&self, name: &'static str, value: u64, tags: &[(&'static str, &str)]) {
//!         println!("counter: {} = {} {:?}", name, value, tags);
//!     }
//!     fn gauge(&self, name: &'static str, value: i64, tags: &[(&'static str, &str)]) {
//!         println!("gauge: {} = {} {:?}", name, value, tags);
//!     }
//!     fn histogram(&self, name: &'static str, value: f64, tags: &[(&'static str, &str)]) {
//!         println!("histogram: {} = {} {:?}", name, value, tags);
//!     }
//! }
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Trait for custom metrics collection.
///
/// Implement this trait to integrate with your metrics system
/// (Prometheus, StatsD, CloudWatch, etc.).
pub trait MetricsSink: Send + Sync + 'static {
    /// Increment a counter.
    ///
    /// Counters track occurrences of events. They only go up.
    fn counter(&self, name: &'static str, value: u64, tags: &[(&'static str, &str)]);

    /// Set a gauge value.
    ///
    /// Gauges track values that can go up and down.
    fn gauge(&self, name: &'static str, value: i64, tags: &[(&'static str, &str)]);

    /// Record a histogram value.
    ///
    /// Histograms track distribution of values (latencies, sizes, etc.).
    fn histogram(&self, name: &'static str, value: f64, tags: &[(&'static str, &str)]);

    /// Record a timing value in microseconds.
    fn timing(&self, name: &'static str, micros: u64, tags: &[(&'static str, &str)]) {
        self.histogram(name, micros as f64, tags);
    }
}

/// No-op metrics sink (default).
///
/// This sink discards all metrics. Use it when you don't need metrics
/// collection or for testing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetrics;

impl MetricsSink for NoopMetrics {
    #[inline]
    fn counter(&self, _: &'static str, _: u64, _: &[(&'static str, &str)]) {}

    #[inline]
    fn gauge(&self, _: &'static str, _: i64, _: &[(&'static str, &str)]) {}

    #[inline]
    fn histogram(&self, _: &'static str, _: f64, _: &[(&'static str, &str)]) {}
}

/// Logging metrics sink.
///
/// This sink logs all metrics using the `tracing` framework.
/// Useful for debugging and development.
#[derive(Debug, Clone, Copy, Default)]
pub struct TracingMetrics;

impl MetricsSink for TracingMetrics {
    fn counter(&self, name: &'static str, value: u64, tags: &[(&'static str, &str)]) {
        tracing::trace!(metric_type = "counter", name, value, ?tags);
    }

    fn gauge(&self, name: &'static str, value: i64, tags: &[(&'static str, &str)]) {
        tracing::trace!(metric_type = "gauge", name, value, ?tags);
    }

    fn histogram(&self, name: &'static str, value: f64, tags: &[(&'static str, &str)]) {
        tracing::trace!(metric_type = "histogram", name, value, ?tags);
    }
}

/// Built-in counters for tracking client statistics.
#[derive(Debug, Default)]
pub struct ClientStats {
    /// Total messages received
    pub messages_received: AtomicU64,
    /// Total messages dropped due to backpressure
    pub messages_dropped: AtomicU64,
    /// Total bytes received
    pub bytes_received: AtomicU64,
    /// Total parse errors
    pub parse_errors: AtomicU64,
    /// Total reconnections
    pub reconnections: AtomicU64,
    /// Total requests sent
    pub requests_sent: AtomicU64,
    /// Total request errors
    pub request_errors: AtomicU64,
    /// Total rate limit hits
    pub rate_limits: AtomicU64,
}

impl ClientStats {
    /// Create new client statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment messages received counter.
    #[inline]
    pub fn inc_messages_received(&self, count: u64) {
        self.messages_received.fetch_add(count, Ordering::Relaxed);
    }

    /// Increment messages dropped counter.
    #[inline]
    pub fn inc_messages_dropped(&self, count: u64) {
        self.messages_dropped.fetch_add(count, Ordering::Relaxed);
    }

    /// Increment bytes received counter.
    #[inline]
    pub fn inc_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment parse errors counter.
    #[inline]
    pub fn inc_parse_errors(&self) {
        self.parse_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment reconnections counter.
    #[inline]
    pub fn inc_reconnections(&self) {
        self.reconnections.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment requests sent counter.
    #[inline]
    pub fn inc_requests_sent(&self) {
        self.requests_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment request errors counter.
    #[inline]
    pub fn inc_request_errors(&self) {
        self.request_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment rate limits counter.
    #[inline]
    pub fn inc_rate_limits(&self) {
        self.rate_limits.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current snapshot of all statistics.
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_dropped: self.messages_dropped.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            parse_errors: self.parse_errors.load(Ordering::Relaxed),
            reconnections: self.reconnections.load(Ordering::Relaxed),
            requests_sent: self.requests_sent.load(Ordering::Relaxed),
            request_errors: self.request_errors.load(Ordering::Relaxed),
            rate_limits: self.rate_limits.load(Ordering::Relaxed),
        }
    }

    /// Reset all counters to zero.
    pub fn reset(&self) {
        self.messages_received.store(0, Ordering::Relaxed);
        self.messages_dropped.store(0, Ordering::Relaxed);
        self.bytes_received.store(0, Ordering::Relaxed);
        self.parse_errors.store(0, Ordering::Relaxed);
        self.reconnections.store(0, Ordering::Relaxed);
        self.requests_sent.store(0, Ordering::Relaxed);
        self.request_errors.store(0, Ordering::Relaxed);
        self.rate_limits.store(0, Ordering::Relaxed);
    }
}

/// Point-in-time snapshot of client statistics.
#[derive(Debug, Clone, Copy, Default)]
pub struct StatsSnapshot {
    /// Total messages received
    pub messages_received: u64,
    /// Total messages dropped
    pub messages_dropped: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total parse errors
    pub parse_errors: u64,
    /// Total reconnections
    pub reconnections: u64,
    /// Total requests sent
    pub requests_sent: u64,
    /// Total request errors
    pub request_errors: u64,
    /// Total rate limit hits
    pub rate_limits: u64,
}

/// Timer for measuring operation latencies.
///
/// Records the duration between creation and drop.
pub struct LatencyTimer<'a, M: MetricsSink> {
    metrics: &'a M,
    name: &'static str,
    tags: Vec<(&'static str, &'static str)>,
    start: Instant,
}

impl<'a, M: MetricsSink> LatencyTimer<'a, M> {
    /// Create a new latency timer.
    pub fn new(metrics: &'a M, name: &'static str) -> Self {
        Self {
            metrics,
            name,
            tags: Vec::new(),
            start: Instant::now(),
        }
    }

    /// Add a tag to the timer.
    pub fn tag(mut self, key: &'static str, value: &'static str) -> Self {
        self.tags.push((key, value));
        self
    }

    /// Stop the timer and record the latency.
    pub fn stop(self) {
        let elapsed = self.start.elapsed();
        let micros = elapsed.as_micros() as u64;
        let tags: Vec<(&'static str, &str)> =
            self.tags.iter().map(|(k, v)| (*k, *v as &str)).collect();
        self.metrics.timing(self.name, micros, &tags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_noop_metrics() {
        let metrics = NoopMetrics;
        metrics.counter("test", 1, &[("tag", "value")]);
        metrics.gauge("test", -5, &[]);
        metrics.histogram("test", 1.5, &[("foo", "bar")]);
    }

    #[test]
    fn test_client_stats() {
        let stats = ClientStats::new();
        stats.inc_messages_received(10);
        stats.inc_bytes_received(1000);
        stats.inc_parse_errors();

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.messages_received, 10);
        assert_eq!(snapshot.bytes_received, 1000);
        assert_eq!(snapshot.parse_errors, 1);
    }

    #[test]
    fn test_client_stats_reset() {
        let stats = ClientStats::new();
        stats.inc_messages_received(100);
        stats.inc_reconnections();

        stats.reset();
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.messages_received, 0);
        assert_eq!(snapshot.reconnections, 0);
    }

    #[test]
    fn test_stats_snapshot_default() {
        let snapshot = StatsSnapshot::default();
        assert_eq!(snapshot.messages_received, 0);
        assert_eq!(snapshot.messages_dropped, 0);
    }

    #[test]
    fn test_client_stats_thread_safe() {
        let stats = Arc::new(ClientStats::new());
        let stats2 = stats.clone();

        let handle = std::thread::spawn(move || {
            for _ in 0..1000 {
                stats2.inc_messages_received(1);
            }
        });

        for _ in 0..1000 {
            stats.inc_messages_received(1);
        }

        handle.join().unwrap();
        assert_eq!(stats.snapshot().messages_received, 2000);
    }
}
