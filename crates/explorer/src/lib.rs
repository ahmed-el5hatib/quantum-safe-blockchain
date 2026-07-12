//! Block explorer web interface.
//!
//! Provides trait-based interfaces for block exploration.
pub mod api;
pub mod error;
pub mod traits;
pub mod web;
pub use error::ExplorerResult;
