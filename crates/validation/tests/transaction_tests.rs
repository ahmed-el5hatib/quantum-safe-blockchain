//! Integration tests for transaction validation.
//!
//! Covers a valid transaction and every corruption mode from the milestone: structure, version,
//! missing inputs/outputs, duplicate inputs, duplicate outputs, amount rules, hash integrity, and
//! signature placeholder validation.

mod common;

use blockchain_core::{HashDigest, OutPoint, Transaction, TransactionInput, TransactionOutput};

use validation::error::ValidationError;
use validation::{ValidationEngine, ValidationStatus};

use common::*;

fn engine() -> ValidationEngine {
    ValidationEngine::with_defaults()
}

#[test]
fn valid_transaction_passes() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(
        report.status,
        ValidationStatus::Passed,
        "errors: {:?}",
        report.errors
    );
    assert_eq!(report.executed_rules(), 9);
    assert!(report.is_valid());
}

#[test]
fn empty_payload_fails_structure() {
    let hasher = sha256();
    let tx = Transaction::new(1, vec![], vec![], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTransaction(_))));
}

#[test]
fn unsupported_version_fails() {
    let hasher = sha256();
    let input = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(99, vec![input], vec![output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTransaction(_))));
}

#[test]
fn missing_inputs_fails() {
    let hasher = sha256();
    let empty_hash = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![]),
            output_index: 0,
        },
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(1, vec![empty_hash], vec![output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTransaction(_))));
}

#[test]
fn no_outputs_fails() {
    let hasher = sha256();
    let input = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let tx = Transaction::new(1, vec![input], vec![], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTransaction(_))));
}

#[test]
fn duplicate_inputs_fails() {
    let hasher = sha256();
    let outpoint = OutPoint {
        transaction_hash: HashDigest::new(vec![0xAB; 32]),
        output_index: 7,
    };
    let input1 = TransactionInput {
        previous_output: outpoint.clone(),
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let input2 = TransactionInput {
        previous_output: outpoint,
        signature: vec![0x02; 32],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(1, vec![input1, input2], vec![output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::DuplicateInput(_))));
}

#[test]
fn distinct_inputs_pass() {
    let hasher = sha256();
    let input1 = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let input2 = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xCD; 32]),
            output_index: 1,
        },
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(1, vec![input1, input2], vec![output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(
        report.status,
        ValidationStatus::Passed,
        "errors: {:?}",
        report.errors
    );
}

#[test]
fn duplicate_outputs_fails() {
    let hasher = sha256();
    let input = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(1, vec![input], vec![output.clone(), output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::DuplicateOutput(_))));
}

#[test]
fn zero_output_value_fails() {
    let hasher = sha256();
    let input = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; 32],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 0,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(1, vec![input], vec![output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidAmount(_))));
}

#[test]
fn empty_signature_placeholder_fails() {
    let hasher = sha256();
    let input = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(1, vec![input], vec![output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTransaction(_))));
}

#[test]
fn oversized_signature_placeholder_fails() {
    let hasher = sha256();
    let input = TransactionInput {
        previous_output: OutPoint {
            transaction_hash: HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; 10_000],
        sequence: 0,
    };
    let output = TransactionOutput {
        value: 100,
        script_pubkey: vec![0xCC; 20],
    };
    let tx = Transaction::new(1, vec![input], vec![output], 0);
    let ctx = transaction_context(&hasher);

    let report = engine().validate_transaction(&tx, &ctx);
    assert_eq!(report.status, ValidationStatus::Failed);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidTransaction(_))));
}

#[test]
fn check_transaction_returns_result() {
    let hasher = sha256();
    let tx = make_transaction(100, 64);
    let ctx = transaction_context(&hasher);
    assert!(engine().check_transaction(&tx, &ctx).is_ok());

    let bad = Transaction::new(1, vec![], vec![], 0);
    assert!(engine().check_transaction(&bad, &ctx).is_err());
}
