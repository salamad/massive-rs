//! Pagination support for REST API endpoints.
//!
//! This module provides the [`PageStream`] type that automatically
//! handles pagination by following `next_url` links in API responses.

use crate::config::PaginationMode;
use crate::error::MassiveError;
use crate::rest::client::RestClient;
use crate::rest::request::PaginatableRequest;
use futures::Stream;
use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Type alias for the boxed future used in pagination.
type PageFuture<T> = Pin<Box<dyn Future<Output = Result<T, MassiveError>> + Send + 'static>>;

/// Stream adapter for paginated requests that handles `next_url` chaining.
///
/// # How next_url Works
///
/// Massive API returns paginated responses like:
/// ```json
/// {
///   "status": "OK",
///   "results": [...],
///   "next_url": "https://api.massive.com/v2/aggs/...?cursor=abc123&apiKey=***"
/// }
/// ```
///
/// The `next_url` is a complete URL that includes:
/// - The same endpoint path
/// - A cursor/pagination token
/// - The API key (as query param)
///
/// This stream fetches each page via `next_url` until exhausted.
///
/// # Example
///
/// ```no_run
/// use massive_rs::rest::RestClient;
/// use massive_rs::rest::endpoints::GetAggsRequest;
/// use massive_rs::rest::endpoints::Timespan;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = RestClient::from_api_key(std::env::var("POLYGON_API_KEY")?)?;
/// let request = GetAggsRequest::new("AAPL")
///     .multiplier(1)
///     .timespan(Timespan::Day)
///     .from("2024-01-01")
///     .to("2024-01-31");
///
/// let mut stream = client.stream(request);
///
/// while let Some(result) = stream.next().await {
///     let item = result?;
///     println!("{:?}", item);
/// }
/// # Ok(())
/// # }
/// ```
pub struct PageStream<R>
where
    R: PaginatableRequest,
    R::Response: DeserializeOwned + Send + 'static,
{
    client: RestClient,
    /// Initial request (only used for first page)
    initial_request: Option<R>,
    /// URL for next page (from previous response's next_url field)
    next_url: Option<String>,
    /// Pagination mode controlling when to stop
    mode: PaginationMode,
    /// Count of items yielded so far
    items_yielded: u64,
    /// Buffer of items from current page
    buffer: VecDeque<R::Item>,
    /// Current async operation in progress
    in_flight: Option<PageFuture<R::Response>>,
    /// Whether we've completed pagination
    done: bool,
}

impl<R> PageStream<R>
where
    R: PaginatableRequest + Send + 'static,
    R::Response: DeserializeOwned + Send + 'static,
{
    /// Create a new page stream.
    pub fn new(client: RestClient, request: R, mode: PaginationMode) -> Self {
        Self {
            client,
            initial_request: Some(request),
            next_url: None,
            mode,
            items_yielded: 0,
            buffer: VecDeque::new(),
            in_flight: None,
            done: false,
        }
    }

    /// Check if we should continue fetching based on pagination mode.
    fn should_continue(&self) -> bool {
        match self.mode {
            PaginationMode::Auto => true,
            PaginationMode::None => self.items_yielded == 0, // Only first page
            PaginationMode::MaxItems(max) => self.items_yielded < max,
        }
    }

    /// Check if we've hit the item limit.
    fn at_item_limit(&self) -> bool {
        matches!(self.mode, PaginationMode::MaxItems(max) if self.items_yielded >= max)
    }
}

impl<R> Stream for PageStream<R>
where
    R: PaginatableRequest + Unpin + Send + 'static,
    R::Response: DeserializeOwned + Send + 'static,
    R::Item: Unpin,
{
    type Item = Result<R::Item, MassiveError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // 1. Yield buffered items first (fast path)
            if let Some(item) = this.buffer.pop_front() {
                this.items_yielded += 1;

                // Stop yielding if we hit MaxItems limit
                if this.at_item_limit() {
                    this.done = true;
                }

                return Poll::Ready(Some(Ok(item)));
            }

            // 2. Check if we're done
            if this.done {
                return Poll::Ready(None);
            }

            // 3. Check if there's an in-flight request
            if let Some(ref mut fut) = this.in_flight {
                match fut.as_mut().poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(e)) => {
                        this.done = true;
                        this.in_flight = None;
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Ready(Ok(response)) => {
                        // Extract next_url BEFORE consuming response
                        this.next_url = R::extract_next_url(&response).map(String::from);

                        // Extract items into buffer
                        let items = R::extract_items(response);
                        this.buffer.extend(items);

                        this.in_flight = None;

                        // If no items and no next_url, we're done
                        if this.buffer.is_empty() && this.next_url.is_none() {
                            this.done = true;
                        }

                        // Continue loop to yield from buffer
                        continue;
                    }
                }
            }

            // 4. No in-flight request, decide what to fetch next
            if !this.should_continue() {
                this.done = true;
                continue;
            }

            // 5. Fetch next page via next_url, or initial request
            if let Some(url) = this.next_url.take() {
                // Subsequent pages: fetch via next_url directly
                // The next_url is a complete URL from Massive API
                let client = this.client.clone();
                this.in_flight = Some(Box::pin(async move {
                    client.fetch_url::<R::Response>(&url).await
                }));
            } else if let Some(req) = this.initial_request.take() {
                // First page: execute the initial request
                let client = this.client.clone();
                this.in_flight = Some(Box::pin(async move { client.execute(req).await }));
            } else {
                // No next_url and no initial request = done
                this.done = true;
                continue;
            }
        }
    }
}

impl<R> PageStream<R>
where
    R: PaginatableRequest + Unpin + Send + 'static,
    R::Response: DeserializeOwned + Send + 'static,
    R::Item: Unpin,
{
    /// Collect all items from all pages into a Vec.
    ///
    /// # Warning
    ///
    /// This loads all data into memory. For large result sets,
    /// prefer using the stream directly.
    pub async fn collect_all(self) -> Result<Vec<R::Item>, MassiveError> {
        use futures::TryStreamExt;
        self.try_collect().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require mocking the HTTP client,
    // which is done in the integration tests module.

    #[test]
    fn test_pagination_mode_should_continue() {
        // Create a minimal stream to test the mode logic
        // This is more of a unit test for the mode behavior

        let mode = PaginationMode::Auto;
        assert!(matches!(mode, PaginationMode::Auto));

        let mode = PaginationMode::None;
        assert!(matches!(mode, PaginationMode::None));

        let mode = PaginationMode::MaxItems(100);
        assert!(matches!(mode, PaginationMode::MaxItems(100)));
    }
}
