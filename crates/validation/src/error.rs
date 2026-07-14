//! Error types for the QSB Validation Engine.
//!
//! Every validator and validation rule in this crate returns a [`ValidationResult`], which is a
//! `Result<T, ValidationError>`. The error variants map directly onto the structural and semantic
//! failures that a block or transaction can exhibit. They are deliberately granular so that callers
//! (consensus, mempool, RPC) can react to the precise cause of a rejection.

use thiserror::Error;

/// The error type produced by every validator and validation rule in the engine.
///
/// All variants carry a human-readable message. They are derived from the canonical failure modes
/// enumerated in the milestone specification: invalid blocks, headers, merkle roots, hashes,
/// transactions, timestamps, heights, duplicate inputs/outputs, invalid amounts, and generic rule
/// failures.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The block as a whole is invalid (transaction count mismatch, structural inconsistency).
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    /// The block header is malformed (bad version, difficulty, or hash-length mismatch).
    #[error("invalid header: {0}")]
    InvalidHeader(String),

    /// The committed Merkle root does not match the transactions in the block.
    #[error("invalid merkle root: {0}")]
    InvalidMerkleRoot(String),

    /// A hash (block hash or transaction hash) does not match its recomputed value.
    #[error("invalid hash: {0}")]
    InvalidHash(String),

    /// A transaction is structurally or semantically invalid.
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    /// The block timestamp is outside the acceptable window.
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// The block height is inconsistent with the chain tip or genesis constraints.
    #[error("invalid height: {0}")]
    InvalidHeight(String),

    /// Two inputs spend the same previous output (double spend within a transaction).
    #[error("duplicate transaction input: {0}")]
    DuplicateInput(String),

    /// A transaction contains identical outputs.
    #[error("duplicate transaction output: {0}")]
    DuplicateOutput(String),

    /// A transaction amount violates the value rules (zero or overflowing total).
    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    /// A generic validation rule failed; wraps the rule's own message.
    #[error("validation rule failed: {0}")]
    ValidationRuleFailed(String),

    /// The block is missing the required previous-block hash (non-genesis block).
    #[error("missing previous hash: {0}")]
    MissingPreviousHash(String),

    /// A genesis-specific constraint was violated.
    #[error("genesis constraint violation: {0}")]
    GenesisConstraint(String),

    /// The validation configuration itself is invalid (e.g. contradictory limits).
    #[error("configuration error: {0}")]
    Configuration(String),

    /// An internal, non-recoverable failure (serialization, merkle construction).
    #[error("internal validation error: {0}")]
    Internal(String),
}

/// The canonical result type returned by validators and validation rules.
///
/// Per the milestone design principles, every validator must return `Result<T, ValidationError>`.
/// This alias is the single, unambiguous spelling of that contract.
pub type ValidationResult<T = ()> = Result<T, ValidationError>;
