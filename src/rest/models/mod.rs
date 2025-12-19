//! REST API response models.
//!
//! This module contains types for parsing REST API responses,
//! including envelope types and generated models.

mod envelope;

pub use envelope::{ApiEnvelope, ListEnvelope};
