//! Merkle tree node representation.
//!
//! A [`MerkleNode`] is a single vertex of the tree. The whole tree is stored as an *arena* — a flat
//! vector of nodes — and each internal node references its two children by index. This avoids
//! recursive ownership (`Box`) and makes serialization, proof generation, and parent lookups cheap
//! and allocation-friendly.

use serde::{Deserialize, Serialize};

/// A single node in a [`MerkleTree`](crate::MerkleTree).
///
/// Leaf nodes have no children (`left` and `right` are both `None`). Internal nodes reference
/// their left and right children by index into the tree's node arena.
///
/// # Examples
///
/// ```
/// use merkle::MerkleNode;
///
/// // A leaf node carries only its hash.
/// let leaf = MerkleNode::leaf(vec![0xaa; 32]);
/// assert!(leaf.is_leaf());
/// assert!(!leaf.is_internal());
///
/// // An internal node references its two children.
/// let internal = MerkleNode::internal(vec![0xbb; 32], 0, 1);
/// assert!(internal.is_internal());
/// assert_eq!(internal.left(), Some(0));
/// assert_eq!(internal.right(), Some(1));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleNode {
    /// The hash stored at this node (the leaf hash, or the parent hash of the children).
    hash: Vec<u8>,
    /// Index of the left child node in the tree arena, if this is an internal node.
    left: Option<usize>,
    /// Index of the right child node in the tree arena, if this is an internal node.
    right: Option<usize>,
}

impl MerkleNode {
    /// Creates a new leaf node carrying `hash`.
    pub fn leaf(hash: Vec<u8>) -> Self {
        Self {
            hash,
            left: None,
            right: None,
        }
    }

    /// Creates a new internal node carrying `hash` with the given child indices.
    pub fn internal(hash: Vec<u8>, left: usize, right: usize) -> Self {
        Self {
            hash,
            left: Some(left),
            right: Some(right),
        }
    }

    /// Returns the hash stored at this node.
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    /// Returns the index of the left child, if this is an internal node.
    pub fn left(&self) -> Option<usize> {
        self.left
    }

    /// Returns the index of the right child, if this is an internal node.
    pub fn right(&self) -> Option<usize> {
        self.right
    }

    /// Returns `true` if this node is a leaf (has no children).
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    /// Returns `true` if this node is internal (has two children).
    pub fn is_internal(&self) -> bool {
        !self.is_leaf()
    }
}
