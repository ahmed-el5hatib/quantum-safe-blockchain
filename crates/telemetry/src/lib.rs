//! Metrics and telemetry collection.
//!
//! Provides trait-based interfaces for metrics collection and export.
pub mod error;
pub mod exporter;
pub mod metrics;
pub use error::TelemetryResult;
