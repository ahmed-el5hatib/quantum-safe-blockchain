//! Integration tests for block validation.
//!
//! Covers valid blocks (genesis and non-genesis), every corruption mode enumerated in the milestone
//! (header, hash, Merkle root, timestamp, height, previous hash, genesis constraints, metadata), and
//! the structure of the resulting [`ValidationReport`].

mod common;

use blockchain_core::{BlockMetadata, Difficulty, MerkleRoot, PreviousHash, Version};

use validation::error::ValidationError;
use validation::{ValidationEngine, ValidationStatus};

use common::*;

fn engine() -> ValidationEngine {
    ValidationEngine::with_defaults()
}

#[test]
fn valid_genesis_block_passes() {
    let hasher = sha256();
    let block = make_genesis(&hasher, 1_700_000_000);
    let ctx = block_context(&hasher, 1_700_000_100, None, None, true);

    let report = engine().validate_block(&block, &ctx);
    assert_eq!(
        report.status,
        ValidationStatus::Passed,
        "errors: {:?}",
        report.errors
    );
    assert!(report.is_valid());
    assert!(report.errors.is_empty());
    assert_eq!(report.executed_rules(), 8);
    assert_eq!(report.passed_rules(), 8);
    assert!(report.execution_time.as_nanos() > 0);
}

#[test]
fn valid_non_genesis_block_passes() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(5),
    );
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);

    let report = engine().validate_block(&block, &ctx);
    assert_eq!(
        report.status,
        ValidationStatus::Passed,
        "errors: {:?}",
        report.errors
    );
    assert_eq!(report.executed_rules(), 8);
}

#[test]
fn corrupted_block_hash_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(3),
    );
    block.block_hash =
        blockchain_core::BlockHash::new(blockchain_core::HashDigest::new(vec![0x99; 32]));

    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidHash(_))));
}

#[test]
fn corrupted_merkle_root_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(4),
    );
    block.header.merkle_root = MerkleRoot::new(vec![0xFF; 32]);
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidMerkleRoot(_))));
}

#[test]
fn unsupported_version_fails_header_format() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    block.header.version = Version::new(2);
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidHeader(_))));
}

#[test]
fn zero_difficulty_fails_header_format() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    block.header.difficulty = Difficulty::new(0);
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidHeader(_))));
}

#[test]
fn future_timestamp_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let clock = 1_700_000_000u64;
    let mut block = make_block(
        &hasher,
        1,
        clock + 100_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, clock, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTimestamp(_))));
}

#[test]
fn timestamp_before_minimum_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        1,
        1,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_000, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTimestamp(_))));
}

#[test]
fn height_not_prev_plus_one_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        5,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidHeight(_))));
}

#[test]
fn non_genesis_zero_height_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        0,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(2),
    );
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidHeight(_))));
}

#[test]
fn missing_previous_hash_fails() {
    let hasher = sha256();
    let mut block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(vec![0x11u8; 32]),
        Some(0),
        1,
        make_transactions(2),
    );
    block.header.previous_hash = PreviousHash::zero();
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(
        &hasher,
        1_700_000_100,
        Some(vec![0x11u8; 32]),
        Some(0),
        false,
    );
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::MissingPreviousHash(_))));
}

#[test]
fn genesis_with_transactions_fails_constraints() {
    let hasher = sha256();
    let mut block = make_genesis(&hasher, 1_700_000_000);
    block.transactions = make_transactions(1);
    block.metadata = BlockMetadata::new(0, block.transactions.len());
    fix_merkle_root(&mut block, &hasher);
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_100, None, None, true);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::GenesisConstraint(_))));
}

#[test]
fn genesis_with_nonzero_previous_hash_fails() {
    let hasher = sha256();
    let mut block = make_genesis(&hasher, 1_700_000_000);
    block.header.previous_hash =
        PreviousHash::new(blockchain_core::HashDigest::new(vec![0x22u8; 32]));
    fix_block_hash(&mut block, &hasher);

    let ctx = block_context(&hasher, 1_700_000_100, None, None, true);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::MissingPreviousHash(_))));
}

#[test]
fn metadata_count_mismatch_fails() {
    let hasher = sha256();
    let prev = vec![0x11u8; 32];
    let mut block = make_block(
        &hasher,
        1,
        1_700_000_000,
        Some(prev.clone()),
        Some(0),
        1,
        make_transactions(3),
    );
    block.metadata = BlockMetadata::new(0, 999);

    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false);
    let report = engine().validate_block(&block, &ctx);

    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidBlock(_))));
}

#[test]
fn check_block_returns_result() {
    let hasher = sha256();
    let block = make_genesis(&hasher, 1_700_000_000);
    let ctx = block_context(&hasher, 1_700_000_100, None, None, true);
    assert!(engine().check_block(&block, &ctx).is_ok());

    let mut bad = make_genesis(&hasher, 1_700_000_000);
    bad.block_hash =
        blockchain_core::BlockHash::new(blockchain_core::HashDigest::new(vec![0u8; 32]));
    assert!(engine().check_block(&bad, &ctx).is_err());
}
