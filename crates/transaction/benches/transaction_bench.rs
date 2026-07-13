//! Performance benchmarks for the QSB Transaction Domain Model.
//!
//! Run with `cargo bench -p transaction`. Measures construction, hashing, builder throughput,
//! serialization, and large-transaction handling.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use cryptography::providers::Sha256Provider;
use transaction::{
    OutputType, TransactionBuilder, TransactionInput, TransactionOutput, TransactionType,
};

/// Builds a transaction with `n` inputs and `n` outputs.
fn build_tx(n: usize) -> transaction::Transaction {
    let prev = transaction::TransactionId::from_hex(&"11".repeat(32)).unwrap();
    let mut builder = TransactionBuilder::new(Sha256Provider)
        .with_timestamp(1_700_000_000)
        .with_type(TransactionType::Transfer);
    for i in 0..n {
        builder = builder
            .add_input(TransactionInput::new(prev.clone(), i as u32))
            .unwrap();
        builder = builder
            .add_output(
                TransactionOutput::new(vec![0xAB; 32], 1_000u64 + i as u64, OutputType::Standard)
                    .unwrap(),
            )
            .unwrap();
    }
    builder.finalize().unwrap()
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");
    for &size in &[1usize, 10, 100, 1_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| criterion::black_box(build_tx(size)));
        });
    }
    group.finish();
}

fn bench_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash");
    for &size in &[1usize, 100, 1_000] {
        let tx = build_tx(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &tx, |b, tx| {
            b.iter(|| criterion::black_box(tx.compute_hash(&Sha256Provider)));
        });
    }
    group.finish();
}

fn bench_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");
    for &size in &[1usize, 100, 1_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let prev = transaction::TransactionId::from_hex(&"11".repeat(32)).unwrap();
                let mut builder = TransactionBuilder::new(Sha256Provider)
                    .with_timestamp(1_700_000_000)
                    .with_type(TransactionType::Transfer);
                for i in 0..size {
                    builder = builder
                        .add_input(TransactionInput::new(prev.clone(), i as u32))
                        .unwrap();
                    builder = builder
                        .add_output(
                            TransactionOutput::new(
                                vec![0xAB; 32],
                                1_000u64 + i as u64,
                                OutputType::Standard,
                            )
                            .unwrap(),
                        )
                        .unwrap();
                }
                criterion::black_box(builder.finalize().unwrap())
            });
        });
    }
    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    for &size in &[1usize, 100, 1_000] {
        let tx = build_tx(size);
        let json = tx.to_json().unwrap();
        let binary = tx.to_binary().unwrap();
        group.bench_with_input(BenchmarkId::new("json", size), &json, |b, json| {
            b.iter(|| criterion::black_box(transaction::Transaction::from_json(json).unwrap()));
        });
        group.bench_with_input(BenchmarkId::new("binary", size), &binary, |b, binary| {
            b.iter(|| criterion::black_box(transaction::Transaction::from_binary(binary).unwrap()));
        });
    }
    group.finish();
}

fn bench_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("large");
    for &size in &[1_000usize, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| criterion::black_box(build_tx(size)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_hash,
    bench_builder,
    bench_serialization,
    bench_large
);
criterion_main!(benches);
