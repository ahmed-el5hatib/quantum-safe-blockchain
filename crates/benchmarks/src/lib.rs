//! Performance benchmarks.
//!
//! Provides trait-based interfaces for benchmarking.
pub mod crypto;
pub mod error;
pub mod latency;
pub mod tps;
pub use error::BenchmarkResult;
