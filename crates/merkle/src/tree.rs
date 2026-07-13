//! The Merkle tree, its builder, and all tree operations.
//!
//! [`MerkleTree`] stores the full tree as an arena of [`MerkleNode`]s plus a per-level index table.
//! This makes building `O(n)`, root retrieval `O(1)`, and proof generation `O(log n)`.

use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use cryptography::core::traits::HashFunction;

use crate::CoreError;
use crate::CoreResult;
use crate::MerkleNode;
use crate::MerkleProof;
use crate::MerkleRoot;
use crate::ProofStep;

/// Serialization format version for [`MerkleTree`] export/import.
const MERKLE_FORMAT_VERSION: u8 = 1;

/// A generic, deterministic Merkle tree.
///
/// The tree is parameterized over the hash provider `H: HashFunction`, so the exact hashing
/// algorithm is decoupled from the tree logic. The same code backs SHA-256, SHA-3, BLAKE3, or any
/// future post-quantum hash provider.
///
/// # Construction
///
/// Use [`MerkleTree::build`] to hash raw data, [`MerkleTree::build_from_hashes`] when leaves are
/// already hashes (e.g. transaction hashes), or the [`TreeBuilder`](crate::TreeBuilder) for a
/// fluent API.
///
/// # Examples
///
/// ```
/// use cryptography::providers::Sha256Provider;
/// use merkle::MerkleTree;
///
/// let hasher = Sha256Provider;
/// let tree = MerkleTree::build(
///     vec![b"t1".as_ref(), b"t2".as_ref(), b"t3".as_ref()],
///     hasher,
/// ).unwrap();
/// assert_eq!(tree.leaf_count(), 3);
/// assert!(tree.height() >= 1);
/// ```
pub struct MerkleTree<H: HashFunction> {
    hasher: H,
    /// Flat arena of all nodes (leaves followed by internal nodes, bottom-up).
    nodes: Vec<MerkleNode>,
    /// `levels[k]` holds the arena indices of the nodes at depth `k` (level 0 = leaves).
    levels: Vec<Vec<usize>>,
    /// Number of leaves (level-0 nodes).
    leaf_count: usize,
    /// Arena index of the single root node.
    root_index: usize,
    /// Whether the duplicate-last-leaf strategy is used for odd levels.
    duplicate_last_leaf: bool,
}

impl<H: HashFunction> Clone for MerkleTree<H>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            hasher: self.hasher.clone(),
            nodes: self.nodes.clone(),
            levels: self.levels.clone(),
            leaf_count: self.leaf_count,
            root_index: self.root_index,
            duplicate_last_leaf: self.duplicate_last_leaf,
        }
    }
}

impl<H: HashFunction + std::fmt::Debug> std::fmt::Debug for MerkleTree<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MerkleTree")
            .field("leaf_count", &self.leaf_count)
            .field("height", &self.height())
            .field("root", &self.root_hash_bytes())
            .field("duplicate_last_leaf", &self.duplicate_last_leaf)
            .finish()
    }
}

// =============================================================================
// Builder-style / associated constructors
// =============================================================================

impl<H: HashFunction> MerkleTree<H> {
    /// Builds a tree by hashing each leaf's raw data with `hasher`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyTree`] if `leaves` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use cryptography::providers::Sha256Provider;
    /// use merkle::MerkleTree;
    ///
    /// let tree = MerkleTree::build(vec![b"a".as_ref(), b"b".as_ref()], Sha256Provider).unwrap();
    /// assert_eq!(tree.leaf_count(), 2);
    /// ```
    pub fn build<L: AsRef<[u8]>>(leaves: Vec<L>, hasher: H) -> CoreResult<Self> {
        let leaf_hashes: Vec<Vec<u8>> = leaves
            .iter()
            .map(|l| hasher.hash(l.as_ref()).into_inner())
            .collect();
        Self::build_from_hashes(leaf_hashes, hasher)
    }

    /// Builds a tree directly from pre-computed leaf hashes (no additional hashing).
    ///
    /// Use this when leaves are already digests — for example transaction hashes committed by a
    /// block header. The duplicate-last-leaf strategy is enabled.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyTree`] if `hashes` is empty.
    pub fn build_from_hashes(hashes: Vec<Vec<u8>>, hasher: H) -> CoreResult<Self> {
        Self::build_from_hashes_with(hashes, hasher, true)
    }

    /// Builds a tree directly from pre-computed leaf hashes, controlling the duplicate-last-leaf
    /// strategy via `duplicate_last_leaf`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyTree`] if `hashes` is empty, or [`CoreError::InvalidTree`] if an
    /// odd level is encountered while `duplicate_last_leaf` is `false`.
    pub fn build_from_hashes_with(
        hashes: Vec<Vec<u8>>,
        hasher: H,
        duplicate_last_leaf: bool,
    ) -> CoreResult<Self> {
        Self::build_internal(hashes, hasher, duplicate_last_leaf)
    }

    /// Internal constructor. `leaf_hashes` must be non-empty.
    fn build_internal(
        leaf_hashes: Vec<Vec<u8>>,
        hasher: H,
        duplicate_last_leaf: bool,
    ) -> CoreResult<Self> {
        if leaf_hashes.is_empty() {
            return Err(CoreError::EmptyTree(
                "cannot build a merkle tree from zero leaves".into(),
            ));
        }

        let mut nodes: Vec<MerkleNode> = Vec::with_capacity(leaf_hashes.len() * 2);
        let mut levels: Vec<Vec<usize>> = Vec::with_capacity(leaf_hashes.len().next_power_of_two());

        let mut current_level: Vec<usize> = Vec::with_capacity(leaf_hashes.len());
        for hash in leaf_hashes {
            let idx = nodes.len();
            nodes.push(MerkleNode::leaf(hash));
            current_level.push(idx);
        }
        levels.push(current_level.clone());

        let leaf_count = current_level.len();
        let mut current = current_level;

        while current.len() > 1 {
            if !current.len().is_multiple_of(2) && !duplicate_last_leaf {
                return Err(CoreError::InvalidTree(format!(
                    "odd number of nodes ({}) at a tree level while duplicate-last-leaf is disabled",
                    current.len()
                )));
            }

            let mut next_level: Vec<usize> = Vec::with_capacity(current.len() / 2 + 1);
            let mut i = 0;
            while i < current.len() {
                let left_idx = current[i];
                let (right_idx, right_hash) = if i + 1 < current.len() {
                    (current[i + 1], nodes[current[i + 1]].hash().to_vec())
                } else {
                    // Duplicate-last-leaf strategy: hash the last node with itself.
                    (current[i], nodes[current[i]].hash().to_vec())
                };
                let mut combined = nodes[left_idx].hash().to_vec();
                combined.extend_from_slice(&right_hash);
                let parent_hash = hasher.hash(&combined).into_inner();
                let parent_idx = nodes.len();
                nodes.push(MerkleNode::internal(parent_hash, left_idx, right_idx));
                next_level.push(parent_idx);
                i += 2;
            }
            levels.push(next_level.clone());
            current = next_level;
        }

        let root_index = *current
            .first()
            .expect("a non-empty tree always has exactly one root node");

        Ok(Self {
            hasher,
            nodes,
            levels,
            leaf_count,
            root_index,
            duplicate_last_leaf,
        })
    }

    // =============================================================================
    // Tree operations
    // =============================================================================

    /// Returns the [`MerkleRoot`] of the tree.
    pub fn root(&self) -> MerkleRoot {
        MerkleRoot(self.root_hash_bytes())
    }

    /// Returns the raw root hash bytes.
    pub fn root_hash_bytes(&self) -> Vec<u8> {
        self.nodes[self.root_index].hash().to_vec()
    }

    /// Returns the tree height (number of edges from a leaf to the root).
    ///
    /// A single-leaf tree has height `0`; a two-leaf tree has height `1`.
    pub fn height(&self) -> usize {
        self.levels.len().saturating_sub(1)
    }

    /// Returns the tree depth, an alias for [`Self::height`].
    pub fn depth(&self) -> usize {
        self.height()
    }

    /// Returns the number of leaves in the tree.
    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    /// Returns `true` if the tree uses the duplicate-last-leaf strategy for odd levels.
    pub fn uses_duplicate_last_leaf(&self) -> bool {
        self.duplicate_last_leaf
    }

    /// Returns the hash of the leaf at `index`, if it exists.
    pub fn leaf(&self, index: usize) -> Option<&[u8]> {
        self.levels
            .first()
            .and_then(|leaves| leaves.get(index))
            .map(|&idx| self.nodes[idx].hash())
    }

    /// Returns `true` if a leaf with hash `leaf_hash` exists in the tree.
    pub fn contains(&self, leaf_hash: &[u8]) -> bool {
        self.levels
            .first()
            .is_some_and(|leaves| leaves.iter().any(|&i| self.nodes[i].hash() == leaf_hash))
    }

    /// Returns all leaf hashes in order.
    pub fn leaves(&self) -> Vec<Vec<u8>> {
        self.levels
            .first()
            .map(|leaves| {
                leaves
                    .iter()
                    .map(|&i| self.nodes[i].hash().to_vec())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Generates a [`MerkleProof`] for the leaf at `index`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidLeaf`] if `index` is out of bounds.
    pub fn proof(&self, index: usize) -> CoreResult<MerkleProof> {
        if index >= self.leaf_count {
            return Err(CoreError::InvalidLeaf(format!(
                "leaf index {index} out of bounds (leaf_count = {})",
                self.leaf_count
            )));
        }

        let leaf_hash = self.nodes[self.levels[0][index]].hash().to_vec();
        let mut steps: Vec<ProofStep> = Vec::with_capacity(self.levels.len().saturating_sub(1));
        let mut current = index;

        for level in &self.levels[..self.levels.len().saturating_sub(1)] {
            let sibling_index = if current.is_multiple_of(2) {
                current + 1
            } else {
                current - 1
            };
            let sibling_hash = if sibling_index < level.len() {
                self.nodes[level[sibling_index]].hash().to_vec()
            } else {
                // Odd level: the last node is duplicated, so it is its own sibling.
                self.nodes[level[current]].hash().to_vec()
            };
            let step = if current.is_multiple_of(2) {
                ProofStep::Right { hash: sibling_hash }
            } else {
                ProofStep::Left { hash: sibling_hash }
            };
            steps.push(step);
            current /= 2;
        }

        Ok(MerkleProof {
            leaf_index: index,
            leaf_hash,
            steps,
            root_hash: self.root_hash_bytes(),
        })
    }

    /// Verifies that `leaf_hash` at `index` is committed by this tree's root using `proof`.
    ///
    /// This is a convenience that reuses the tree's own hash provider. For fully stateless
    /// verification (without the tree), prefer [`ProofVerifier`](crate::ProofVerifier).
    pub fn verify_proof(&self, index: usize, leaf_hash: &[u8], proof: &MerkleProof) -> bool
    where
        H: Clone,
    {
        if index != proof.leaf_index {
            return false;
        }
        if leaf_hash != proof.leaf_hash.as_slice() {
            return false;
        }
        if let Some(stored) = self.leaf(index) {
            if stored != leaf_hash {
                return false;
            }
        } else {
            return false;
        }
        proof.verify_against(&self.hasher, &self.root_hash_bytes())
    }

    /// Returns a reference to the hash provider used by this tree.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
}

// =============================================================================
// Serialization
// =============================================================================

/// Serializable snapshot of a [`MerkleTree`].
///
/// The hash provider itself is not serialized; it must be supplied again when reconstructing the
/// tree (see [`MerkleTree::from_json`] / [`MerkleTree::from_binary`]).
#[derive(Serialize, Deserialize)]
struct MerkleTreeData {
    version: u8,
    algorithm: String,
    nodes: Vec<MerkleNode>,
    levels: Vec<Vec<usize>>,
    leaf_count: usize,
    root_index: usize,
    duplicate_last_leaf: bool,
}

impl<H: HashFunction> MerkleTree<H> {
    fn to_data(&self) -> MerkleTreeData {
        MerkleTreeData {
            version: MERKLE_FORMAT_VERSION,
            algorithm: self.hasher.algorithm_name().to_string(),
            nodes: self.nodes.clone(),
            levels: self.levels.clone(),
            leaf_count: self.leaf_count,
            root_index: self.root_index,
            duplicate_last_leaf: self.duplicate_last_leaf,
        }
    }

    /// Serializes the tree to a JSON string.
    ///
    /// # Panics
    ///
    /// Panics only if the in-memory structure cannot be serialized, which should not happen for
    /// a well-formed tree.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.to_data())
            .expect("a well-formed merkle tree is always serializable to JSON")
    }

    /// Serializes the tree to a compact `bincode` binary form.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidTree`] if serialization fails.
    pub fn to_binary(&self) -> CoreResult<Vec<u8>> {
        bincode::serialize(&self.to_data())
            .map_err(|e| CoreError::InvalidTree(format!("binary encode: {e}")))
    }

    /// Reconstructs a tree from a [`MerkleTreeData`] snapshot, validating structural consistency.
    fn from_data(data: MerkleTreeData, hasher: H) -> CoreResult<Self> {
        if data.version != MERKLE_FORMAT_VERSION {
            return Err(CoreError::InvalidTree(format!(
                "unsupported merkle format version {} (expected {MERKLE_FORMAT_VERSION})",
                data.version
            )));
        }
        if data.nodes.is_empty() {
            return Err(CoreError::EmptyTree(
                "deserialized merkle tree has no nodes".into(),
            ));
        }
        if data.root_index >= data.nodes.len() {
            return Err(CoreError::InvalidTree("root index out of bounds".into()));
        }
        let level0_len = data.levels.first().map_or(0, Vec::len);
        if level0_len != data.leaf_count {
            return Err(CoreError::InvalidTree(
                "leaf count does not match the number of level-0 nodes".into(),
            ));
        }
        if data.levels.last().map_or(0, Vec::len) != 1 {
            return Err(CoreError::InvalidTree(
                "top level must contain exactly one root node".into(),
            ));
        }
        for level in &data.levels {
            for &idx in level {
                if idx >= data.nodes.len() {
                    return Err(CoreError::InvalidTree(
                        "level references missing node".into(),
                    ));
                }
            }
        }

        Ok(Self {
            hasher,
            nodes: data.nodes,
            levels: data.levels,
            leaf_count: data.leaf_count,
            root_index: data.root_index,
            duplicate_last_leaf: data.duplicate_last_leaf,
        })
    }

    /// Reconstructs a tree from a JSON string using the supplied `hasher`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidTree`] / [`CoreError::InvalidProof`] on malformed input, or
    /// [`CoreError::EmptyTree`] if the snapshot has no nodes.
    pub fn from_json(data: &str, hasher: H) -> CoreResult<Self> {
        let parsed: MerkleTreeData = serde_json::from_str(data)
            .map_err(|e| CoreError::InvalidTree(format!("json decode: {e}")))?;
        Self::from_data(parsed, hasher)
    }

    /// Reconstructs a tree from its `bincode` binary form using the supplied `hasher`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidTree`] / [`CoreError::InvalidProof`] on malformed input.
    pub fn from_binary(data: &[u8], hasher: H) -> CoreResult<Self> {
        let parsed: MerkleTreeData = bincode::deserialize(data)
            .map_err(|e| CoreError::InvalidTree(format!("binary decode: {e}")))?;
        Self::from_data(parsed, hasher)
    }
}

impl<H> Serialize for MerkleTree<H>
where
    H: HashFunction,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_data().serialize(serializer)
    }
}

impl<'de, H> Deserialize<'de> for MerkleTree<H>
where
    H: HashFunction + Default,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = MerkleTreeData::deserialize(deserializer)?;
        if data.version != MERKLE_FORMAT_VERSION {
            return Err(serde::de::Error::custom(format!(
                "unsupported merkle format version {}",
                data.version
            )));
        }
        if data.nodes.is_empty() {
            return Err(serde::de::Error::custom(
                "deserialized merkle tree has no nodes",
            ));
        }
        Ok(Self {
            hasher: H::default(),
            nodes: data.nodes,
            levels: data.levels,
            leaf_count: data.leaf_count,
            root_index: data.root_index,
            duplicate_last_leaf: data.duplicate_last_leaf,
        })
    }
}

// =============================================================================
// Integration with the blockchain data model
// =============================================================================

impl<H> blockchain_core::traits::MerkleTreeT for MerkleTree<H>
where
    H: HashFunction + Clone,
{
    fn root(&self) -> blockchain_core::MerkleRoot {
        blockchain_core::MerkleRoot::new(self.root_hash_bytes())
    }

    fn verify(&self, index: usize, leaf: &[u8], proof: &[blockchain_core::MerkleRoot]) -> bool {
        if index >= self.leaf_count {
            return false;
        }
        let stored = match self.leaf(index) {
            Some(h) => h,
            None => return false,
        };
        if stored != leaf {
            return false;
        }

        let mut current = leaf.to_vec();
        let mut idx = index;
        for sibling in proof {
            let mut combined = Vec::with_capacity(current.len() * 2);
            if idx.is_multiple_of(2) {
                combined.extend_from_slice(&current);
                combined.extend_from_slice(sibling.as_bytes());
            } else {
                combined.extend_from_slice(sibling.as_bytes());
                combined.extend_from_slice(&current);
            }
            current = self.hasher.hash(&combined).into_inner();
            idx /= 2;
        }
        current == self.root_hash_bytes()
    }
}
