//! Shared test helpers for the validation engine integration tests.
//!
//! These helpers build *honestly valid* blocks and transactions (their hashes and Merkle roots are
//! computed through the same public commitment helpers the engine uses), and small mutators that
//! corrupt a specific field so each validation rule can be exercised in isolation.

#![allow(dead_code)]

use std::sync::Arc;

use blockchain_core::{
    Block, BlockHash, BlockHeader, BlockMetadata, Difficulty, HashDigest, Height, MerkleRoot,
    Nonce, OutPoint, PreviousHash, Timestamp, Transaction, TransactionInput, TransactionOutput,
    Version,
};
use cryptography::core::traits::HashFunction;
use cryptography::providers::Sha256Provider;

use validation::{
    compute_block_hash, compute_merkle_root, ContextBuilder, ValidationConfig, ValidationContext,
};

/// A shared SHA-256 provider (used only in test code; the engine itself never names an algorithm).
pub fn sha256() -> Arc<dyn HashFunction> {
    Arc::new(Sha256Provider)
}

/// Builds a single transaction with one input and one output.
pub fn make_transaction(value: u64, sig_len: usize) -> Transaction {
    let input = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; sig_len.max(1)],
        sequence: 0,
    };
    let output = TransactionOutput {
        value,
        script_pubkey: vec![0xCC; 20],
    };
    Transaction::new(1, vec![input], vec![output], 0)
}

/// Builds a chain of `count` distinct transactions.
pub fn make_transactions(count: usize) -> Vec<Transaction> {
    (0..count)
        .map(|i| make_transaction(100 + i as u64, 64))
        .collect()
}

fn empty_root(hasher: &Arc<dyn HashFunction>) -> Vec<u8> {
    vec![0u8; hasher.digest_size()]
}

/// Builds a valid block. For genesis, pass `height = 0`, `prev_hash = None`, and `transactions =
/// vec![]`. Otherwise provide the previous block's hash and height.
#[allow(clippy::too_many_arguments)]
pub fn make_block(
    hasher: &Arc<dyn HashFunction>,
    height: u64,
    timestamp: u64,
    prev_hash: Option<Vec<u8>>,
    _prev_height: Option<u64>,
    difficulty: u32,
    transactions: Vec<Transaction>,
) -> Block {
    let merkle_root = compute_merkle_root(transactions.as_slice(), hasher, &empty_root(hasher))
        .expect("merkle root computation must succeed");

    let previous = match prev_hash {
        Some(bytes) => PreviousHash::new(HashDigest::new(bytes)),
        None => PreviousHash::zero(),
    };

    let header = BlockHeader::new(
        Version::new(1),
        Height::new(height),
        Timestamp::new(timestamp),
        previous,
        MerkleRoot::new(merkle_root),
        Difficulty::new(difficulty),
        Nonce::new(0),
    );

    let block_hash = BlockHash::new(HashDigest::new(
        compute_block_hash(&header, hasher.as_ref()).expect("block hash must compute"),
    ));

    let metadata = BlockMetadata::new(0, transactions.len());
    Block::new(header, transactions, metadata, block_hash)
}

/// Builds a valid genesis block.
pub fn make_genesis(hasher: &Arc<dyn HashFunction>, timestamp: u64) -> Block {
    make_block(hasher, 0, timestamp, None, None, 1, vec![])
}

/// Builds a context for validating a block that follows `prev_hash` at `prev_height`.
pub fn block_context(
    hasher: &Arc<dyn HashFunction>,
    clock: u64,
    prev_hash: Option<Vec<u8>>,
    prev_height: Option<u64>,
    is_genesis: bool,
) -> ValidationContext {
    let mut builder = ContextBuilder::new(hasher.clone()).with_clock(clock);
    if let Some(h) = prev_hash {
        builder = builder.with_prev_block_hash(h);
    }
    if let Some(h) = prev_height {
        builder = builder.with_prev_height(h);
    }
    builder.with_genesis(is_genesis).build()
}

/// Builds a default transaction-validation context.
pub fn transaction_context(hasher: &Arc<dyn HashFunction>) -> ValidationContext {
    ValidationContext::new(hasher.clone())
}

/// Returns a config with a single rule override applied (used for enable/disable tests).
pub fn config_with_rule(id: &str, enabled: bool) -> ValidationConfig {
    let mut cfg = ValidationConfig::default_with_digest_size(32);
    cfg.set_rule_enabled(id.to_string(), enabled);
    cfg
}

/// Recomputes and rewrites the block hash after a header mutation (keeps hash consistency valid).
pub fn fix_block_hash(block: &mut Block, hasher: &Arc<dyn HashFunction>) {
    let bytes =
        compute_block_hash(&block.header, hasher.as_ref()).expect("block hash must compute");
    block.block_hash = BlockHash::new(HashDigest::new(bytes));
}

/// Recomputes and rewrites the header Merkle root after a transaction-set mutation.
pub fn fix_merkle_root(block: &mut Block, hasher: &Arc<dyn HashFunction>) {
    let root = compute_merkle_root(&block.transactions, hasher, &empty_root(hasher))
        .expect("merkle root must compute");
    block.header.merkle_root = MerkleRoot::new(root);
}
