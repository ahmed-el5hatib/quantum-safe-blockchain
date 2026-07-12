//! Node orchestration and lifecycle management.
//!
//! Coordinates all subsystems into a running node.
pub mod config;
pub mod error;
pub mod service;
pub use error::NodeResult;
