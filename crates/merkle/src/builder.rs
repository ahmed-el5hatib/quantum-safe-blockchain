//! Fluent builder for [`MerkleTree`].
//!
//! `TreeBuilder` lets callers incrementally configure a tree before building it: choose the hash
//! provider, add leaves (either raw data or pre-computed hashes), and toggle the
//! duplicate-last-leaf strategy.

use cryptography::core::traits::HashFunction;

use crate::CoreError;
use crate::CoreResult;
use crate::MerkleTree;

/// Fluent builder for a [`MerkleTree`].
///
/// # Examples
///
/// ```
/// use cryptography::providers::Sha256Provider;
/// use merkle::TreeBuilder;
///
/// let tree = TreeBuilder::new(Sha256Provider)
///     .with_leaves(vec![b"a", b"b", b"c", b"d"])
///     .build()
///     .unwrap();
/// assert_eq!(tree.leaf_count(), 4);
/// ```
pub struct TreeBuilder<H: HashFunction> {
    hasher: H,
    /// Raw leaf data (hashed at build time) or pre-computed leaf hashes.
    leaves: Vec<Vec<u8>>,
    /// Whether `leaves` already contains hashes (true) or raw data (false).
    prehashed: bool,
    /// Whether to duplicate the last leaf when a level has an odd node count.
    duplicate_last_leaf: bool,
}

impl<H: HashFunction> TreeBuilder<H> {
    /// Creates a new builder backed by `hasher`.
    pub fn new(hasher: H) -> Self {
        Self {
            hasher,
            leaves: Vec::new(),
            prehashed: false,
            duplicate_last_leaf: true,
        }
    }

    /// Alias for [`Self::new`], emphasising that the builder is bound to a hash provider.
    pub fn with_hasher(hasher: H) -> Self {
        Self::new(hasher)
    }

    /// Adds raw leaf data. Each leaf is hashed with the provider at build time.
    pub fn with_leaves<L: AsRef<[u8]>>(mut self, leaves: Vec<L>) -> Self {
        self.prehashed = false;
        self.leaves = leaves.into_iter().map(|l| l.as_ref().to_vec()).collect();
        self
    }

    /// Adds a single raw leaf.
    pub fn with_leaf<L: AsRef<[u8]>>(mut self, leaf: L) -> Self {
        self.prehashed = false;
        self.leaves.push(leaf.as_ref().to_vec());
        self
    }

    /// Sets the leaves directly as pre-computed hashes (no additional hashing is performed).
    ///
    /// Use this when the inputs are already digests, e.g. transaction hashes.
    pub fn with_leaf_hashes(mut self, hashes: Vec<Vec<u8>>) -> Self {
        self.prehashed = true;
        self.leaves = hashes;
        self
    }

    /// Enables or disables the duplicate-last-leaf strategy for odd-sized levels.
    ///
    /// When disabled and an odd number of leaves (or an odd level) is encountered, building
    /// returns [`CoreError::InvalidTree`].
    pub fn with_duplicate_last_leaf(mut self, enabled: bool) -> Self {
        self.duplicate_last_leaf = enabled;
        self
    }

    /// Returns `true` if the stored leaves are already hashes (not raw data).
    pub fn is_prehashed(&self) -> bool {
        self.prehashed
    }

    /// Builds the [`MerkleTree`].
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyTree`] if no leaves were supplied, or
    /// [`CoreError::InvalidTree`] if an odd level is encountered while duplicate-last-leaf is
    /// disabled.
    pub fn build(self) -> CoreResult<MerkleTree<H>> {
        if self.leaves.is_empty() {
            return Err(CoreError::EmptyTree("no leaves supplied to builder".into()));
        }
        let leaf_hashes = if self.prehashed {
            self.leaves
        } else {
            self.leaves
                .iter()
                .map(|data| self.hasher.hash(data).into_inner())
                .collect()
        };
        MerkleTree::build_from_hashes_with(leaf_hashes, self.hasher, self.duplicate_last_leaf)
    }
}
