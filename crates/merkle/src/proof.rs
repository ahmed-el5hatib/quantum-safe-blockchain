//! Merkle proof types and verification.
//!
//! A [`MerkleProof`] is a compact, `O(log n)` witness that a specific leaf participates in a tree
//! with a known root. It contains the leaf's index and hash, the ordered list of sibling hashes
//! ([`ProofStep`]), and the root hash that was trusted at proof-generation time.

use serde::{Deserialize, Serialize};

use crate::CoreError;
use crate::CoreResult;
use crate::MerkleRoot;
use cryptography::core::traits::HashFunction;

/// A single step in a [`MerkleProof`].
///
/// Each step records the **sibling hash** for one level of the tree, together with the side on
/// which that sibling sits *relative to the node being recomputed*.
///
/// - [`ProofStep::Left`] means the sibling is on the **left**, so the parent is
///   `H(sibling || current)`.
/// - [`ProofStep::Right`] means the sibling is on the **right**, so the parent is
///   `H(current || sibling)`.
///
/// Encoding the direction explicitly is what prevents forgery: a verifier that ignored ordering
/// would accept `H(a || b)` as equal to `H(b || a)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofStep {
    /// The sibling hash sits on the left: `parent = H(sibling || current)`.
    Left {
        /// The sibling node's hash.
        hash: Vec<u8>,
    },
    /// The sibling hash sits on the right: `parent = H(current || sibling)`.
    Right {
        /// The sibling node's hash.
        hash: Vec<u8>,
    },
}

impl ProofStep {
    /// Returns the sibling hash carried by this step.
    pub fn hash(&self) -> &[u8] {
        match self {
            ProofStep::Left { hash } | ProofStep::Right { hash } => hash,
        }
    }

    /// Returns `true` if the sibling sits on the left.
    pub fn is_left(&self) -> bool {
        matches!(self, ProofStep::Left { .. })
    }

    /// Returns `true` if the sibling sits on the right.
    pub fn is_right(&self) -> bool {
        matches!(self, ProofStep::Right { .. })
    }
}

/// A Merkle inclusion proof for a single leaf.
///
/// The proof is fully self-describing: given a [`HashFunction`] and the expected root, [`Self::verify`]
/// (or [`ProofVerifier`](crate::ProofVerifier)) can confirm inclusion without the original tree.
///
/// # Examples
///
/// ```
/// use cryptography::providers::Sha256Provider;
/// use cryptography::core::traits::HashFunction;
/// use merkle::{MerkleTree, MerkleProof, ProofVerifier};
///
/// let hasher = Sha256Provider;
/// let tree = MerkleTree::build(
///     vec![b"a".as_ref(), b"b".as_ref(), b"c".as_ref(), b"d".as_ref()],
///     hasher.clone(),
/// ).unwrap();
/// let root = tree.root();
///
/// let proof: MerkleProof = tree.proof(2).unwrap();
/// let verifier = ProofVerifier::new(hasher.clone());
/// assert!(verifier.verify(&proof));
/// assert!(proof.verify_against(&hasher, root.as_bytes()));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Index of the proven leaf in the original leaf ordering.
    pub leaf_index: usize,
    /// Hash of the proven leaf.
    pub leaf_hash: Vec<u8>,
    /// Sibling hashes from the leaf up to (but not including) the root, one per level.
    pub steps: Vec<ProofStep>,
    /// The trusted root hash recorded when this proof was generated.
    pub root_hash: Vec<u8>,
}

impl MerkleProof {
    /// Serializes the proof to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidProof`] if serialization fails.
    pub fn to_json(&self) -> CoreResult<String> {
        serde_json::to_string(self)
            .map_err(|e| CoreError::InvalidProof(format!("json encode: {e}")))
    }

    /// Deserializes a proof from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidProof`] if the input is not a valid proof.
    pub fn from_json(data: &str) -> CoreResult<Self> {
        serde_json::from_str(data).map_err(|e| CoreError::InvalidProof(format!("json decode: {e}")))
    }

    /// Serializes the proof to a compact binary form using `bincode`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidProof`] if serialization fails.
    pub fn to_binary(&self) -> CoreResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| CoreError::InvalidProof(format!("binary encode: {e}")))
    }

    /// Deserializes a proof from its `bincode` binary form.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidProof`] if the input is not a valid proof.
    pub fn from_binary(data: &[u8]) -> CoreResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| CoreError::InvalidProof(format!("binary decode: {e}")))
    }

    /// Verifies this proof against a trusted root using the given hash provider.
    ///
    /// Recomputes the root from the leaf hash and proof steps and compares it to `expected_root`.
    /// Returns `false` (rather than erroring) for an invalid proof, since an untrusted caller may
    /// supply a malformed proof.
    pub fn verify_against<H: HashFunction>(&self, hasher: &H, expected_root: &[u8]) -> bool {
        if expected_root.is_empty() || self.root_hash.is_empty() {
            return false;
        }
        let computed = self.recompute(hasher);
        computed == expected_root
    }

    /// Verifies this proof against the root it was generated with.
    ///
    /// This is a convenience wrapper around [`Self::verify_against`] using `self.root_hash`.
    pub fn verify<H: HashFunction>(&self, hasher: &H) -> bool {
        self.verify_against(hasher, &self.root_hash)
    }

    /// Recomputes the candidate root by walking the proof steps from the leaf hash upward.
    fn recompute<H: HashFunction>(&self, hasher: &H) -> Vec<u8> {
        let mut current = self.leaf_hash.clone();
        for step in &self.steps {
            let mut combined = Vec::with_capacity(current.len() * 2);
            match step {
                ProofStep::Left { hash } => {
                    combined.extend_from_slice(hash);
                    combined.extend_from_slice(&current);
                }
                ProofStep::Right { hash } => {
                    combined.extend_from_slice(&current);
                    combined.extend_from_slice(hash);
                }
            }
            current = hasher.hash(&combined).into_inner();
        }
        current
    }
}

impl From<MerkleProof> for MerkleRoot {
    fn from(value: MerkleProof) -> Self {
        Self(value.root_hash)
    }
}

/// Stateless verifier for [`MerkleProof`]s.
///
/// `ProofVerifier` owns a hash provider and validates proofs against a trusted root. It is the
/// recommended entry point for verification because it keeps the hashing provider explicit and
/// avoids accidentally trusting the wrong algorithm.
///
/// # Examples
///
/// ```
/// use cryptography::providers::Sha256Provider;
/// use merkle::{MerkleTree, ProofVerifier};
///
/// let hasher = Sha256Provider;
/// let tree = MerkleTree::build(vec![b"x".as_ref(), b"y".as_ref()], hasher.clone()).unwrap();
/// let proof = tree.proof(0).unwrap();
/// let verifier = ProofVerifier::new(hasher);
/// assert!(verifier.verify(&proof));
/// ```
pub struct ProofVerifier<H: HashFunction> {
    hasher: H,
}

impl<H: HashFunction> ProofVerifier<H> {
    /// Creates a new verifier backed by `hasher`.
    pub fn new(hasher: H) -> Self {
        Self { hasher }
    }

    /// Verifies `proof` against the root it was generated with.
    pub fn verify(&self, proof: &MerkleProof) -> bool {
        proof.verify_against(&self.hasher, &proof.root_hash)
    }

    /// Verifies `proof` against an explicit `expected_root`.
    pub fn verify_against(&self, proof: &MerkleProof, expected_root: &[u8]) -> bool {
        proof.verify_against(&self.hasher, expected_root)
    }

    /// Verifies that `leaf_hash` at `leaf_index` is committed by `expected_root`.
    ///
    /// In addition to recomputing the root, this checks that the proof's recorded leaf index and
    /// hash match the supplied values, guarding against a proof being replayed for the wrong leaf.
    pub fn verify_leaf(
        &self,
        leaf_index: usize,
        leaf_hash: &[u8],
        proof: &MerkleProof,
        expected_root: &[u8],
    ) -> bool {
        leaf_index == proof.leaf_index
            && leaf_hash == proof.leaf_hash.as_slice()
            && proof.verify_against(&self.hasher, expected_root)
    }

    /// Returns a reference to the hash provider used by this verifier.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
}
