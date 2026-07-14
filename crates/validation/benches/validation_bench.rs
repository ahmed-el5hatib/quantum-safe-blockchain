//! Performance benchmarks for the QSB Validation Engine.
//!
//! Run with `cargo bench -p validation`. Covers block validation, transaction validation, the
//! validation pipeline, and individual rule execution.

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use blockchain_core::{
    Block, BlockHash, BlockHeader, BlockMetadata, Difficulty, Height, MerkleRoot, Nonce,
    PreviousHash, Timestamp, Transaction, TransactionInput, TransactionOutput, Version,
};
use cryptography::core::traits::HashFunction;
use cryptography::providers::Sha256Provider;
use validation::{
    compute_block_hash, compute_merkle_root, default_block_rules, default_transaction_rules,
    ValidationContext, ValidationEngine, ValidationPipeline,
};

/// Builds a valid transaction for benchmarking.
fn make_transaction(value: u64, sig_len: usize) -> Transaction {
    let input = TransactionInput {
        previous_output: blockchain_core::OutPoint {
            transaction_hash: blockchain_core::HashDigest::new(vec![0xAB; 32]),
            output_index: 0,
        },
        signature: vec![0x01; sig_len],
        sequence: 0,
    };
    let output = TransactionOutput {
        value,
        script_pubkey: vec![0xCC; 20],
    };
    Transaction::new(1, vec![input], vec![output], 0)
}

/// Builds a valid block (with `tx_count` transactions) for benchmarking, hashed with `hasher`.
fn make_block(hasher: &Arc<dyn HashFunction>, tx_count: usize) -> Block {
    let transactions: Vec<Transaction> = (0..tx_count)
        .map(|i| make_transaction(100 + i as u64, 64))
        .collect();

    let empty_root = vec![0u8; hasher.digest_size()];
    let merkle_root = compute_merkle_root(&transactions, hasher, &empty_root)
        .expect("merkle root computation must succeed");

    let header = BlockHeader::new(
        Version::new(1),
        Height::new(1),
        Timestamp::new(1_700_000_000),
        PreviousHash::new(blockchain_core::HashDigest::new(vec![0x11; 32])),
        MerkleRoot::new(merkle_root),
        Difficulty::new(1),
        Nonce::new(0),
    );

    let block_hash = BlockHash::new(blockchain_core::HashDigest::new(
        compute_block_hash(&header, hasher.as_ref()).expect("block hash must compute"),
    ));

    let metadata = BlockMetadata::new(0, transactions.len());
    Block::new(header, transactions, metadata, block_hash)
}

fn bench_block_validation(c: &mut Criterion) {
    let hasher: Arc<dyn HashFunction> = Arc::new(Sha256Provider);
    let engine = ValidationEngine::with_defaults();

    let mut group = c.benchmark_group("block_validation");
    for &tx_count in &[1usize, 10, 100, 1000] {
        let block = make_block(&hasher, tx_count);
        let ctx = ValidationContext::builder(hasher.clone())
            .with_clock(1_700_000_100)
            .with_prev_height(0)
            .with_prev_block_hash(vec![0x11; 32])
            .build();
        group.bench_with_input(
            BenchmarkId::from_parameter(tx_count),
            &(&block, &ctx),
            |b, (block, ctx)| {
                b.iter(|| {
                    let report = engine.validate_block(block, ctx);
                    criterion::black_box(report.is_valid())
                });
            },
        );
    }
    group.finish();
}

fn bench_transaction_validation(c: &mut Criterion) {
    let hasher: Arc<dyn HashFunction> = Arc::new(Sha256Provider);
    let engine = ValidationEngine::with_defaults();
    let tx = make_transaction(100, 64);
    let ctx = ValidationContext::new(hasher.clone());

    c.bench_function("transaction_validation", |b| {
        b.iter(|| {
            let report = engine.validate_transaction(&tx, &ctx);
            criterion::black_box(report.is_valid())
        });
    });
}

fn bench_pipeline(c: &mut Criterion) {
    let hasher: Arc<dyn HashFunction> = Arc::new(Sha256Provider);
    let block_pipeline: ValidationPipeline<Block> =
        ValidationPipeline::with_rules(default_block_rules());
    let tx_pipeline: ValidationPipeline<Transaction> =
        ValidationPipeline::with_rules(default_transaction_rules());

    let block = make_block(&hasher, 100);
    let ctx = ValidationContext::builder(hasher.clone())
        .with_clock(1_700_000_100)
        .with_prev_height(0)
        .with_prev_block_hash(vec![0x11; 32])
        .build();
    let tx = make_transaction(100, 64);

    let mut group = c.benchmark_group("pipeline");
    group.bench_function("block_pipeline", |b| {
        b.iter(|| criterion::black_box(block_pipeline.run(&block, &ctx)));
    });
    group.bench_function("transaction_pipeline", |b| {
        b.iter(|| criterion::black_box(tx_pipeline.run(&tx, &ctx)));
    });
    group.finish();
}

fn bench_rule_execution(c: &mut Criterion) {
    let hasher: Arc<dyn HashFunction> = Arc::new(Sha256Provider);
    let block = make_block(&hasher, 50);
    let ctx = ValidationContext::builder(hasher.clone())
        .with_clock(1_700_000_100)
        .with_prev_height(0)
        .with_prev_block_hash(vec![0x11; 32])
        .build();

    let rules = default_block_rules();
    c.bench_function("individual_block_rules", |b| {
        b.iter(|| {
            let mut warnings = Vec::new();
            for rule in &rules {
                let _ = rule.validate(&block, &ctx, &mut warnings);
            }
            criterion::black_box(warnings.len())
        });
    });
}

criterion_group!(
    benches,
    bench_block_validation,
    bench_transaction_validation,
    bench_pipeline,
    bench_rule_execution
);
criterion_main!(benches);
