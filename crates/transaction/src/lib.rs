//! Transaction types, validation, and lifecycle.
//!
//! Defines transaction domain models and validation interfaces.
pub mod error;
pub mod traits;
pub mod types;
pub mod validator;
pub use error::TransactionResult;
