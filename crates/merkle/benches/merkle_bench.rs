//! Performance benchmarks for the QSB Merkle Tree engine.
//!
//! Run with `cargo bench -p merkle`. Covers tree construction, root generation, proof generation,
//! and proof verification across small, medium, and large trees (1K, 10K, 100K leaves).

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use cryptography::providers::Sha256Provider;
use merkle::MerkleTree;

/// Generates `n` distinct leaf payloads.
fn make_leaves(n: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|i| format!("bench-leaf-{i}").into_bytes())
        .collect()
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");
    for &size in &[1usize, 2, 3, 100, 1_000, 10_000, 100_000] {
        let leaves = make_leaves(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &leaves, |b, leaves| {
            b.iter(|| {
                let tree = MerkleTree::build(leaves.clone(), Sha256Provider).unwrap();
                criterion::black_box(tree.root_hash_bytes())
            });
        });
    }
    group.finish();
}

fn bench_root(c: &mut Criterion) {
    let mut group = c.benchmark_group("root");
    for &size in &[1_000usize, 10_000, 100_000] {
        let tree = MerkleTree::build(make_leaves(size), Sha256Provider).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(size), &tree, |b, tree| {
            b.iter(|| criterion::black_box(tree.root_hash_bytes()));
        });
    }
    group.finish();
}

fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generation");
    for &size in &[1_000usize, 10_000, 100_000] {
        let tree = MerkleTree::build(make_leaves(size), Sha256Provider).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(size), &tree, |b, tree| {
            b.iter(|| {
                let proof = tree.proof(0).unwrap();
                criterion::black_box(proof)
            });
        });
    }
    group.finish();
}

fn bench_proof_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_verification");
    for &size in &[1_000usize, 10_000, 100_000] {
        let tree = MerkleTree::build(make_leaves(size), Sha256Provider).unwrap();
        let proof = tree.proof(0).unwrap();
        let verifier = merkle::ProofVerifier::new(Sha256Provider);
        let root = tree.root_hash_bytes();
        group.bench_with_input(BenchmarkId::from_parameter(size), &tree, |b, _tree| {
            b.iter(|| {
                criterion::black_box(verifier.verify_against(&proof, &root));
            });
        });
    }
    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    for &size in &[1_000usize, 10_000, 100_000] {
        let tree = MerkleTree::build(make_leaves(size), Sha256Provider).unwrap();
        let json = tree.to_json();
        let binary = tree.to_binary().unwrap();
        group.bench_with_input(BenchmarkId::new("json", size), &json, |b, json| {
            b.iter(|| {
                let restored = MerkleTree::from_json(json, Sha256Provider).unwrap();
                criterion::black_box(restored.root_hash_bytes())
            });
        });
        group.bench_with_input(BenchmarkId::new("binary", size), &binary, |b, binary| {
            b.iter(|| {
                let restored = MerkleTree::from_binary(binary, Sha256Provider).unwrap();
                criterion::black_box(restored.root_hash_bytes())
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_root,
    bench_proof_generation,
    bench_proof_verification,
    bench_serialization
);
criterion_main!(benches);
