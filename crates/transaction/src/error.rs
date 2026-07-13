//! Transaction error types and result aliases.
//!
//! The transaction domain model intentionally reuses the unified blockchain error type
//! ([`CoreError`]) so that errors compose cleanly across the ecosystem. The transaction-specific
//! variants (`InvalidTransaction`, `MissingInputs`, `MissingOutputs`, `InvalidAmount`,
//! `DuplicateInput`, `DuplicateOutput`, `InvalidVersion`, `TransactionSerialization`) are defined
//! on [`CoreError`] in `blockchain-core`.

pub use blockchain_core::{CoreError, CoreResult};

/// Alias mirroring [`CoreError`], provided for ergonomic use as `TransactionError`.
pub type TransactionError = CoreError;

/// Result type returned by transaction domain operations.
pub type TransactionResult<T> = CoreResult<T>;
