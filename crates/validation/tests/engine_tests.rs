//! Integration tests for the Validation Engine API, configuration, and reporting.
//!
//! Covers the engine entry points, rule enable/disable through configuration, `fail_fast`
//! short-circuiting, the fluent builders, and the structure of [`ValidationReport`].

mod common;

use blockchain_core::BlockMetadata;
use blockchain_core::HashDigest;

use validation::block::default_block_rules;
use validation::transaction::default_transaction_rules;
use validation::{
    EngineBuilder, ValidationConfig, ValidationEngine, ValidationPipeline, ValidationReport,
    ValidationStatus,
};

use common::*;

fn engine() -> ValidationEngine {
    ValidationEngine::with_defaults()
}

#[test]
fn engine_default_has_expected_rule_counts() {
    let e = engine();
    assert_eq!(e.block_pipeline().rules().len(), 8);
    assert_eq!(e.transaction_pipeline().rules().len(), 9);
}

#[test]
fn disabling_a_rule_makes_corrupted_block_pass() {
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
    block.block_hash = blockchain_core::BlockHash::new(HashDigest::new(vec![0u8; 32]));

    let mut cfg = ValidationConfig::default_with_digest_size(hasher.digest_size());
    cfg.set_rule_enabled("block.hash_consistency".to_string(), false);
    let ctx = block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false).with_config(cfg);

    let report = engine().validate_block(&block, &ctx);
    assert_eq!(
        report.status,
        ValidationStatus::Passed,
        "errors: {:?}",
        report.errors
    );
    assert_eq!(report.executed_rules(), 7);
}

#[test]
fn enabling_disabled_rule_by_default_is_respected() {
    // The genesis constraints rule is a no-op for non-genesis blocks; ensure toggling does not break.
    let hasher = sha256();
    let block = make_genesis(&hasher, 1_700_000_000);
    let mut cfg = ValidationConfig::default_with_digest_size(hasher.digest_size());
    cfg.set_rule_enabled("block.genesis_constraints".to_string(), false);
    let ctx = block_context(&hasher, 1_700_000_100, None, None, true).with_config(cfg);

    let report = engine().validate_block(&block, &ctx);
    assert_eq!(report.status, ValidationStatus::Passed);
}

#[test]
fn fail_fast_stops_after_first_failure() {
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
    // Corrupt both the hash and the merkle root.
    block.block_hash = blockchain_core::BlockHash::new(HashDigest::new(vec![0u8; 32]));
    block.header.merkle_root = blockchain_core::MerkleRoot::new(vec![0xFF; hasher.digest_size()]);

    let ctx_full = block_context(&hasher, 1_700_000_100, Some(prev.clone()), Some(0), false);
    let report_full = engine().validate_block(&block, &ctx_full);
    assert!(report_full.executed_rules() >= 8);

    let mut cfg = ValidationConfig::default_with_digest_size(hasher.digest_size());
    cfg.fail_fast = true;
    let ctx_fast =
        block_context(&hasher, 1_700_000_100, Some(prev), Some(0), false).with_config(cfg);
    let report_fast = engine().validate_block(&block, &ctx_fast);

    assert_eq!(report_fast.status, ValidationStatus::Failed);
    assert!(report_fast.executed_rules() < report_full.executed_rules());
}

#[test]
fn builder_with_custom_rules() {
    let e = EngineBuilder::default()
        .block_rules(default_block_rules())
        .transaction_rules(default_transaction_rules())
        .build();
    assert_eq!(e.block_pipeline().rules().len(), 8);
    assert_eq!(e.transaction_pipeline().rules().len(), 9);
}

#[test]
fn pipeline_runs_rules_and_aggregates() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let pipeline: ValidationPipeline<blockchain_core::Transaction> =
        ValidationPipeline::with_rules(default_transaction_rules());
    let ctx = transaction_context(&hasher);

    let report: ValidationReport = pipeline.run(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Passed);
    assert_eq!(report.executed_rules(), 9);
    assert!(report.execution_time.as_nanos() > 0);
}

#[test]
fn report_metadata_count_mismatch_recorded() {
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
    assert!(!report.is_valid());
    assert!(!report.errors.is_empty());
    assert!(report.failed_rules() >= 1);
    assert!(report.passed_rules() + report.failed_rules() == report.executed_rules());
}

#[test]
fn validator_trait_is_implemented() {
    use validation::Validator;
    let hasher = sha256();
    let block = make_genesis(&hasher, 1_700_000_000);
    let ctx = block_context(&hasher, 1_700_000_100, None, None, true);
    let e = engine();
    let report = Validator::validate(&e, &block, &ctx);
    assert_eq!(report.status, ValidationStatus::Passed);
}

#[test]
fn rule_id_is_stable_string() {
    let rule = default_block_rules().into_iter().next().unwrap();
    assert!(!rule.id().as_str().is_empty());
}
