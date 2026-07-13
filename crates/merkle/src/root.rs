//! Merkle root type and conversions to/from the blockchain data model.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::CoreError;
use crate::CoreResult;

/// The root hash of a [`MerkleTree`](crate::MerkleTree).
///
/// The root is a single, fixed-size commitment to the entire set of leaves. If any leaf changes,
/// the root changes with overwhelming probability. Roots are compared for equality to verify that
/// two trees (or a proof and a trusted root) commit to the same data.
///
/// This type integrates directly with the blockchain data model: it converts losslessly to and
/// from `blockchain_core::MerkleRoot`, which is stored in a block header.
///
/// # Examples
///
/// ```
/// use merkle::MerkleRoot;
///
/// let root = MerkleRoot::new(vec![0x01; 32]);
/// assert_eq!(root.as_bytes(), &[0x01; 32]);
/// assert_eq!(root.to_hex(), "01".repeat(32));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleRoot(pub Vec<u8>);

impl MerkleRoot {
    /// Creates a new root from raw hash bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the root hash as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns `true` if the root hash is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the length of the root hash in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Encodes the root hash as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Decodes a hex string into a root hash.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidTree`] if the string is not valid hex.
    pub fn from_hex(s: &str) -> CoreResult<Self> {
        hex::decode(s)
            .map(Self)
            .map_err(|e| CoreError::InvalidTree(format!("invalid merkle root hex: {e}")))
    }
}

impl fmt::Display for MerkleRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<blockchain_core::MerkleRoot> for MerkleRoot {
    fn from(value: blockchain_core::MerkleRoot) -> Self {
        Self(value.0)
    }
}

impl From<MerkleRoot> for blockchain_core::MerkleRoot {
    fn from(value: MerkleRoot) -> Self {
        Self(value.0)
    }
}
