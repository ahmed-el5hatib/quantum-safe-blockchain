//! Per-rule unit tests.
//!
//! Each validation rule is exercised in isolation — both the passing and the failing path — to
//! guarantee the atomic, independently-testable units the milestone requires.

mod common;

use blockchain_core::HashDigest;
use blockchain_core::{
    BlockMetadata, Difficulty, Height, MerkleRoot, PreviousHash, Timestamp, Transaction,
    TransactionInput, TransactionOutput, Version,
};

use validation::block::{
    BlockHashConsistencyRule, BlockMetadataRule, GenesisConstraintsRule, HeaderFormatRule,
    HeightConsistencyRule, MerkleRootRule, PreviousHashRule, TimestampValidityRule,
};
use validation::rule::ValidationRule;
use validation::transaction::{
    AmountRule, DuplicateInputsRule, DuplicateOutputsRule, HashIntegrityRule, InputsExistRule,
    OutputsExistRule, SignaturePlaceholderRule, TransactionStructureRule, TransactionVersionRule,
};
use validation::{ValidationConfig, ValidationContext, ValidationResult};

use common::*;

fn block_result<R: ValidationRule<BlockchainBlock>>(
    rule: &R,
    block: &BlockchainBlock,
    ctx: &ValidationContext,
) -> ValidationResult<()> {
    rule.validate(block, ctx, &mut Vec::new())
}

type BlockchainBlock = blockchain_core::Block;

fn tx_result<R: ValidationRule<blockchain_core::Transaction>>(
    rule: &R,
    tx: &blockchain_core::Transaction,
    ctx: &ValidationContext,
) -> ValidationResult<()> {
    rule.validate(tx, ctx, &mut Vec::new())
}

// =============================================================================
// Block rules
// =============================================================================

#[test]
fn header_format_rule_passes_and_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev.clone()), Some(0), false);
    assert!(block_result(&HeaderFormatRule, &block, &ctx).is_ok());

    let mut bad = block.clone();
    bad.header.version = Version::new(9);
    fix_block_hash(&mut bad, &hasher);
    assert!(block_result(&HeaderFormatRule, &bad, &ctx).is_err());

    let mut zero_diff = block.clone();
    zero_diff.header.difficulty = Difficulty::new(0);
    fix_block_hash(&mut zero_diff, &hasher);
    assert!(block_result(&HeaderFormatRule, &zero_diff, &ctx).is_err());
}

#[test]
fn block_hash_consistency_rule() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    assert!(block_result(&BlockHashConsistencyRule, &block, &ctx).is_ok());

    let mut bad = block;
    bad.block_hash = blockchain_core::BlockHash::new(HashDigest::new(vec![0u8; 32]));
    assert!(block_result(&BlockHashConsistencyRule, &bad, &ctx).is_err());
}

#[test]
fn merkle_root_rule() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(3),
    );
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    assert!(block_result(&MerkleRootRule, &block, &ctx).is_ok());

    let mut bad = block;
    bad.header.merkle_root = MerkleRoot::new(vec![0xAB; hasher.digest_size()]);
    fix_block_hash(&mut bad, &hasher);
    assert!(block_result(&MerkleRootRule, &bad, &ctx).is_err());
}

#[test]
fn timestamp_rule() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let clock = 1_700_000_000u64;
    let mut block = make_block(
        &hasher,
        1,
        clock,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    let ctx = block_context(&hasher, clock, Some(prev.clone()), Some(0), false);
    assert!(block_result(&TimestampValidityRule, &block, &ctx).is_ok());

    block.header.timestamp = Timestamp::new(clock + 1_000_000);
    fix_block_hash(&mut block, &hasher);
    assert!(block_result(&TimestampValidityRule, &block, &ctx).is_err());
}

#[test]
fn height_consistency_rule() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev.clone()), Some(0), false);
    assert!(block_result(&HeightConsistencyRule, &block, &ctx).is_ok());

    let mut bad = block;
    bad.header.height = Height::new(7);
    fix_block_hash(&mut bad, &hasher);
    assert!(block_result(&HeightConsistencyRule, &bad, &ctx).is_err());
}

#[test]
fn previous_hash_rule() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev.clone()), Some(0), false);
    assert!(block_result(&PreviousHashRule, &block, &ctx).is_ok());

    let mut bad = block;
    bad.header.previous_hash = PreviousHash::zero();
    fix_block_hash(&mut bad, &hasher);
    assert!(block_result(&PreviousHashRule, &bad, &ctx).is_err());
}

#[test]
fn genesis_constraints_rule() {
    let hasher = sha256();
    let genesis = make_genesis(&hasher, 1_700_000_000);
    let ctx = block_context(&hasher, 1_700_000_100, None, None, true);
    assert!(block_result(&GenesisConstraintsRule, &genesis, &ctx).is_ok());

    let mut bad = genesis;
    bad.transactions = make_transactions(1);
    bad.metadata = BlockMetadata::new(0, bad.transactions.len());
    fix_merkle_root(&mut bad, &hasher);
    fix_block_hash(&mut bad, &hasher);
    assert!(block_result(&GenesisConstraintsRule, &bad, &ctx).is_err());

    // Non-genesis context: rule should be a no-op (pass).
    let non_genesis_ctx = block_context(
        &hasher,
        1_700_000_100,
        Some(vec![0x11u8; 32]),
        Some(0),
        false,
    );
    assert!(block_result(&GenesisConstraintsRule, &bad, &non_genesis_ctx).is_ok());
}

#[test]
fn block_metadata_rule() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(3),
    );
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    assert!(block_result(&BlockMetadataRule, &block, &ctx).is_ok());

    let mut bad = block;
    bad.metadata = BlockMetadata::new(0, 12345);
    assert!(block_result(&BlockMetadataRule, &bad, &ctx).is_err());
}

// =============================================================================
// Transaction rules
// =============================================================================

fn make_custom_tx(
    version: u32,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    lock_time: u64,
) -> Transaction {
    Transaction::new(version, inputs, outputs, lock_time)
}

#[test]
fn transaction_structure_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&TransactionStructureRule, &tx, &ctx).is_ok());

    let empty = Transaction::new(1, vec![], vec![], 0);
    assert!(tx_result(&TransactionStructureRule, &empty, &ctx).is_err());
}

#[test]
fn transaction_version_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&TransactionVersionRule, &tx, &ctx).is_ok());

    let bad = make_custom_tx(
        0,
        vec![TransactionInput {
            previous_output: blockchain_core::OutPoint {
                transaction_hash: HashDigest::new(vec![0xAB; 32]),
                output_index: 0,
            },
            signature: vec![0x01; 32],
            sequence: 0,
        }],
        vec![TransactionOutput {
            value: 100,
            script_pubkey: vec![0xCC; 20],
        }],
        0,
    );
    assert!(tx_result(&TransactionVersionRule, &bad, &ctx).is_err());
}

#[test]
fn inputs_exist_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&InputsExistRule, &tx, &ctx).is_ok());

    let bad = make_custom_tx(
        1,
        vec![TransactionInput {
            previous_output: blockchain_core::OutPoint {
                transaction_hash: HashDigest::new(vec![]),
                output_index: 0,
            },
            signature: vec![0x01; 32],
            sequence: 0,
        }],
        vec![TransactionOutput {
            value: 100,
            script_pubkey: vec![0xCC; 20],
        }],
        0,
    );
    assert!(tx_result(&InputsExistRule, &bad, &ctx).is_err());
}

#[test]
fn outputs_exist_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&OutputsExistRule, &tx, &ctx).is_ok());

    let bad = make_custom_tx(
        1,
        vec![TransactionInput {
            previous_output: blockchain_core::OutPoint {
                transaction_hash: HashDigest::new(vec![0xAB; 32]),
                output_index: 0,
            },
            signature: vec![0x01; 32],
            sequence: 0,
        }],
        vec![],
        0,
    );
    assert!(tx_result(&OutputsExistRule, &bad, &ctx).is_err());
}

#[test]
fn duplicate_inputs_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&DuplicateInputsRule, &tx, &ctx).is_ok());

    let op = blockchain_core::OutPoint {
        transaction_hash: HashDigest::new(vec![0xAB; 32]),
        output_index: 0,
    };
    let bad = make_custom_tx(
        1,
        vec![
            TransactionInput {
                previous_output: op.clone(),
                signature: vec![0x01; 32],
                sequence: 0,
            },
            TransactionInput {
                previous_output: op,
                signature: vec![0x02; 32],
                sequence: 0,
            },
        ],
        vec![TransactionOutput {
            value: 100,
            script_pubkey: vec![0xCC; 20],
        }],
        0,
    );
    assert!(tx_result(&DuplicateInputsRule, &bad, &ctx).is_err());
}

#[test]
fn duplicate_outputs_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&DuplicateOutputsRule, &tx, &ctx).is_ok());

    let out = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let bad = make_custom_tx(
        1,
        vec![TransactionInput {
            previous_output: blockchain_core::OutPoint {
                transaction_hash: HashDigest::new(vec![0xAB; 32]),
                output_index: 0,
            },
            signature: vec![0x01; 32],
            sequence: 0,
        }],
        vec![out.clone(), out],
        0,
    );
    assert!(tx_result(&DuplicateOutputsRule, &bad, &ctx).is_err());
}

#[test]
fn amount_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&AmountRule, &tx, &ctx).is_ok());

    let zero = make_custom_tx(
        1,
        vec![TransactionInput {
            previous_output: blockchain_core::OutPoint {
                transaction_hash: HashDigest::new(vec![0xAB; 32]),
                output_index: 0,
            },
            signature: vec![0x01; 32],
            sequence: 0,
        }],
        vec![TransactionOutput {
            value: 0,
            script_pubkey: vec![0xCC; 20],
        }],
        0,
    );
    assert!(tx_result(&AmountRule, &zero, &ctx).is_err());
}

#[test]
fn hash_integrity_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&HashIntegrityRule, &tx, &ctx).is_ok());
}

#[test]
fn signature_placeholder_rule() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(tx_result(&SignaturePlaceholderRule, &tx, &ctx).is_ok());

    let empty_sig = make_custom_tx(
        1,
        vec![TransactionInput {
            previous_output: blockchain_core::OutPoint {
                transaction_hash: HashDigest::new(vec![0xAB; 32]),
                output_index: 0,
            },
            signature: vec![],
            sequence: 0,
        }],
        vec![TransactionOutput {
            value: 100,
            script_pubkey: vec![0xCC; 20],
        }],
        0,
    );
    assert!(tx_result(&SignaturePlaceholderRule, &empty_sig, &ctx).is_err());
}

// Reference `ValidationConfig` so the import is used by at least one test path.
#[allow(dead_code)]
fn _cfg() -> ValidationConfig {
    ValidationConfig::default_with_digest_size(32)
}
