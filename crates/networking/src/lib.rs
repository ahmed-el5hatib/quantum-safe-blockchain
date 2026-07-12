//! libp2p networking layer.
//!
//! Provides trait-based interfaces for P2P networking.
pub mod discovery;
pub mod error;
pub mod protocol;
pub mod traits;
pub use error::NetworkResult;
