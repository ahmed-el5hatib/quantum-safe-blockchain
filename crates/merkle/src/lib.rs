//! # QSB Merkle Tree Engine
//!
//! A production-quality, deterministic, and **generic** Merkle Tree library for the Quantum
//! Safe Blockchain (QSB) ecosystem.
//!
//! This engine is milestone 4.2 of the QSB project. It is intentionally independent of any
//! blockchain logic: it only depends on the abstract [`HashFunction`] trait from the
//! cryptography crate, which means it can be reused across the entire ecosystem for:
//!
//! - **Block transactions** — committing the set of transactions in a block to a single root.
//! - **State commitments** — summarizing world state for light-client verification.
//! - **Storage verification** — proving inclusion of keys/values in a verified store.
//! - **Network synchronization** — compactly proving a peer has the same data set.
//! - **Proof generation** — producing and verifying inclusion/non-inclusion proofs.
//!
//! ## Merkle Tree Concepts
//!
//! A Merkle tree (a *hash tree*) is a binary tree in which every leaf is the cryptographic hash
//! of a data block, and every internal node is the cryptographic hash of the concatenation of its
//! two children. The single node at the top is the **Merkle root**. Because a cryptographic hash
//! is a fixed-size, deterministic fingerprint of its input, the root is a compact commitment to
//! *every* leaf: changing even a single bit of any leaf changes the root with overwhelming
//! probability (the *avalanche effect*).
//!
//! ```text
//!              root = H(H(ab) || H(cd))
//!                  /                  \
//!          H(ab) = H(a || b)    H(cd) = H(c || d)
//!            /        \            /        \
//!           a          b          c          d        <- leaf hashes
//! ```
//!
//! ## Proof Generation
//!
//! To prove that a leaf `L` is part of a tree with a known root, a *Merkle proof* supplies, for
//! every level between the leaf and the root, the **sibling hash** (the hash of the other child
//! sharing the same parent) together with the side on which that sibling sits. The verifier
//! starts from the leaf hash and, one level at a time, recomputes the parent hash:
//!
//! - If the sibling is on the **left**, `parent = H(sibling || current)`.
//! - If the sibling is on the **right**, `parent = H(current || sibling)`.
//!
//! After processing all steps the recomputed value must equal the trusted root. The proof size is
//! `O(log n)` hashes, independent of the total number of leaves.
//!
//! ## Proof Verification
//!
//! Verification is fully **stateless**: it needs only the leaf hash, the proof steps, the hashing
//! provider, and the expected root. It does *not* require the full tree, which is what makes
//! Merkle proofs so useful for light clients and cross-process checks.
//!
//! ## Algorithm Complexity
//!
//! | Operation            | Complexity      |
//! |----------------------|-----------------|
//! | Build                | `O(n)` hashes   |
//! | Root retrieval       | `O(1)`          |
//! | Proof generation     | `O(log n)`      |
//! | Proof verification   | `O(log n)`      |
//! | Contains (leaf)      | `O(n)` (leaves) |
//!
//! Memory usage for an `n`-leaf tree is `O(n)` nodes (the tree is stored as an arena of nodes plus
//! a per-level index table).
//!
//! ## Security Considerations
//!
//! - **Determinism**: The same leaves always produce the same root. Do not feed the same logical
//!   leaf through different hash providers and expect equality.
//! - **Second pre-image / collision resistance** depend entirely on the underlying
//!   [`HashFunction`]. SHA-256 is used by default in tests; migrate to SHA-3 or BLAKE3 providers
//!   for stronger post-quantum margins without changing any tree logic.
//! - **Sibling ordering matters**: the proof records which side each sibling is on. A verifier that
//!   ignores ordering (e.g. `H(a || b)` == `H(b || a)`) would be vulnerable to forgery; this engine
//!   always encodes explicit direction.
//! - **Duplicate-last-leaf strategy**: when a level has an odd number of nodes, the last node is
//!   duplicated and hashed with itself (`H(x || x)`). This is the Bitcoin-style strategy. It is
//!   enabled by default and can be disabled (in which case building an odd-sized tree returns
//!   [`CoreError::InvalidTree`]).
//! - **Poisoned inputs**: proofs and trees are validated on deserialization; malformed or
//!   corrupted structures surface as [`CoreError::InvalidProof`] / [`CoreError::InvalidTree`]
//!   rather than panicking.
//!
//! ## Future Extensibility
//!
//! - New hash algorithms require **only** a new [`HashFunction`] provider; no tree code changes.
//! - The tree is generic over the provider `H`, so it works equally for SHA-256, SHA-3, BLAKE3,
//!   or a future post-quantum hash.
//! - Conversions to/from `blockchain_core::MerkleRoot` and an implementation of the
//!   `blockchain_core::MerkleTreeT` trait are provided so the engine drops directly into the
//!   blockchain data model.
//!
//! [`HashFunction`]: cryptography::core::traits::HashFunction
//! [`CoreError::InvalidTree`]: blockchain_core::CoreError::InvalidTree
//! [`CoreError::InvalidProof`]: blockchain_core::CoreError::InvalidProof

pub mod builder;
pub mod node;
pub mod proof;
pub mod root;
pub mod tree;

pub use blockchain_core::{CoreError, CoreResult};

pub use builder::TreeBuilder;
pub use node::MerkleNode;
pub use proof::{MerkleProof, ProofStep, ProofVerifier};
pub use root::MerkleRoot;
pub use tree::MerkleTree;

/// Result alias used by the Merkle engine; errors are unified with the blockchain error model.
pub type MerkleResult<T> = CoreResult<T>;
