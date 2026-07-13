use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cryptography::core::traits::{HashFunction, SignatureAlgorithm};
use cryptography::providers::{Ed25519Provider, Sha256Provider};

fn bench_sha256(c: &mut Criterion) {
    let hasher = Sha256Provider;
    let data = black_box(b"hello world");
    c.bench_function("sha256_hash", |b| b.iter(|| hasher.hash(data)));
}

fn bench_sha256_large(c: &mut Criterion) {
    let hasher = Sha256Provider;
    let data = black_box(vec![0xAB; 1024 * 1024]);
    c.bench_function("sha256_hash_large", |b| b.iter(|| hasher.hash(&data)));
}

fn bench_ed25519_keygen(c: &mut Criterion) {
    let provider = Ed25519Provider;
    c.bench_function("ed25519_keygen", |b| b.iter(|| provider.generate_keypair()));
}

fn bench_ed25519_sign(c: &mut Criterion) {
    let provider = Ed25519Provider;
    let keypair = provider.generate_keypair().unwrap();
    let message = black_box(b"hello world");
    c.bench_function("ed25519_sign", |b| {
        b.iter(|| provider.sign(keypair.private_key(), message))
    });
}

fn bench_ed25519_verify(c: &mut Criterion) {
    let provider = Ed25519Provider;
    let keypair = provider.generate_keypair().unwrap();
    let message = black_box(b"hello world");
    let signature = provider.sign(keypair.private_key(), message).unwrap();
    c.bench_function("ed25519_verify", |b| {
        b.iter(|| provider.verify(keypair.public_key(), message, &signature))
    });
}

criterion_group!(
    benches,
    bench_sha256,
    bench_sha256_large,
    bench_ed25519_keygen,
    bench_ed25519_sign,
    bench_ed25519_verify
);
criterion_main!(benches);
