//! Dynamic hasher adapter and canonical commitment helpers.
//!
//! The cryptography crate's [`HashFunction`] is a *trait object* at the boundary (`Arc<dyn
//! HashFunction>`). The `merkle` crate, however, is generic over a concrete `H: HashFunction`. To
//! bridge the two without ever naming a concrete algorithm, this module provides
//! [`DynHashFunction`] — a thin `HashFunction` implementation that delegates to a shared,
//! type-erased provider — plus the canonical byte encodings used to commit blocks and transactions.

use std::fmt;
use std::sync::Arc;

use blockchain_core::{Block, BlockHeader, Transaction};
use cryptography::core::traits::HashFunction;
use cryptography::HashDigest;

use crate::error::{ValidationError, ValidationResult};

/// A `HashFunction` implementation backed by a type-erased, reference-counted provider.
///
/// This lets the generic `merkle` engine be driven by whatever provider the caller injected into the
/// [`ValidationContext`](crate::ValidationContext), preserving full crypto-agility.
#[derive(Clone)]
pub struct DynHashFunction {
    inner: Arc<dyn HashFunction>,
}

impl DynHashFunction {
    /// Wraps a shared provider.
    pub fn new(hasher: &Arc<dyn HashFunction>) -> Self {
        Self {
            inner: Arc::clone(hasher),
        }
    }
}

impl fmt::Debug for DynHashFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynHashFunction")
            .field("algorithm", &self.inner.algorithm_name())
            .finish()
    }
}

impl HashFunction for DynHashFunction {
    fn hash(&self, data: &[u8]) -> HashDigest {
        self.inner.hash(data)
    }

    fn hash_stream(
        &self,
        reader: &mut dyn std::io::Read,
    ) -> cryptography::CryptoResult<HashDigest> {
        self.inner.hash_stream(reader)
    }

    fn digest_size(&self) -> usize {
        self.inner.digest_size()
    }

    fn algorithm_name(&self) -> &str {
        self.inner.algorithm_name()
    }
}

/// Serializes any `Serialize` value to `Vec<u8>`, surfacing serialization failures as
/// [`ValidationError::Internal`] rather than panicking.
pub fn serialize<T: serde::Serialize>(value: &T) -> ValidationResult<Vec<u8>> {
    bincode::serialize(value)
        .map_err(|e| ValidationError::Internal(format!("serialization failed: {e}")))
}

/// Canonical byte encoding of a block header (used for block-hash commitment).
pub fn block_header_bytes(header: &BlockHeader) -> ValidationResult<Vec<u8>> {
    serialize(header)
}

/// Canonical byte encoding of a transaction (used for transaction-hash commitment).
///
/// The encoding is the deterministic `bincode` serialization of the on-chain
/// `blockchain_core::Transaction`. Because the block carries this transaction type, the validation
/// engine defines — and is the single authority for — its canonical commitment. The same encoding
/// must be used by any future block builder so that Merkle roots stay consistent network-wide.
pub fn transaction_bytes(tx: &Transaction) -> ValidationResult<Vec<u8>> {
    serialize(tx)
}

/// Recomputes the block hash from its header using `hasher`.
///
/// This mirrors `blockchain_core::Block::hash_with` exactly (hash over the bincode-encoded header),
/// so an honestly-built block passes hash consistency regardless of which provider is configured.
pub fn compute_block_hash(
    header: &BlockHeader,
    hasher: &dyn HashFunction,
) -> ValidationResult<Vec<u8>> {
    let bytes = block_header_bytes(header)?;
    Ok(hasher.hash(&bytes).into_inner())
}

/// Recomputes a transaction's canonical hash using `hasher`.
pub fn compute_transaction_hash(
    tx: &Transaction,
    hasher: &dyn HashFunction,
) -> ValidationResult<Vec<u8>> {
    let bytes = transaction_bytes(tx)?;
    Ok(hasher.hash(&bytes).into_inner())
}

/// Recomputes the Merkle root committed by a block's transaction set.
///
/// Each transaction is hashed (via [`compute_transaction_hash`]) and those digests become the leaves
/// of a `merkle` tree built with `hasher`. A block with zero transactions commits `empty_root`
/// instead (the genesis convention), since an empty tree has no root.
pub fn compute_merkle_root(
    transactions: &[Transaction],
    hasher: &Arc<dyn HashFunction>,
    empty_root: &[u8],
) -> ValidationResult<Vec<u8>> {
    if transactions.is_empty() {
        return Ok(empty_root.to_vec());
    }

    let leaf_hashes: Vec<Vec<u8>> = transactions
        .iter()
        .map(|tx| compute_transaction_hash(tx, hasher.as_ref()))
        .collect::<ValidationResult<Vec<Vec<u8>>>>()?;

    let tree = merkle::MerkleTree::build_from_hashes(leaf_hashes, DynHashFunction::new(hasher))
        .map_err(|e| {
            ValidationError::InvalidMerkleRoot(format!("merkle construction failed: {e}"))
        })?;

    Ok(tree.root_hash_bytes())
}

/// Convenience helper that hashes a block for callers that only need the digest bytes.
pub fn block_hash(block: &Block, hasher: &Arc<dyn HashFunction>) -> ValidationResult<Vec<u8>> {
    compute_block_hash(&block.header, hasher.as_ref())
}
