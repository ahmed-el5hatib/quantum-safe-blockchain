//! Storage backend abstractions and implementations.
//!
//! Provides trait-based interfaces for all storage operations.
pub mod error;
pub mod rocks;
pub mod sled;
pub mod traits;
pub use error::StorageResult;
