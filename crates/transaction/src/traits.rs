//! Transaction abstraction trait.
//!
//! A small, crypto-agnostic view over any transaction. Future validation, consensus, and storage
//! engines should program against this trait rather than the concrete [`Transaction`](crate::Transaction)
//! type, so the transaction representation can evolve without breaking consumers.

use crate::{
    TransactionHash, TransactionId, TransactionInput, TransactionOutput, TransactionType,
    TransactionVersion,
};

/// A read-only view over the essential fields of a transaction.
///
/// Implemented by [`Transaction`](crate::Transaction). Higher-level components (validation,
/// merkle commitment, storage) depend on this trait, never on the concrete struct.
pub trait TransactionT {
    /// Returns the unique transaction identifier (equal to the transaction hash).
    fn id(&self) -> &TransactionId;
    /// Returns the transaction hash.
    fn hash(&self) -> &TransactionHash;
    /// Returns the transaction version.
    fn version(&self) -> TransactionVersion;
    /// Returns the transaction type.
    fn transaction_type(&self) -> TransactionType;
    /// Returns the transaction inputs.
    fn inputs(&self) -> &[TransactionInput];
    /// Returns the transaction outputs.
    fn outputs(&self) -> &[TransactionOutput];
}
