//! JSON-RPC server and API definitions.
//!
//! Provides trait-based interfaces for RPC services.
pub mod error;
pub mod methods;
pub mod server;
pub mod traits;
pub use error::RpcResult;
