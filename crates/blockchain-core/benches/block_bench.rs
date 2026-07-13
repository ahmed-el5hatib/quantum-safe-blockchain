use blockchain_core::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cryptography::providers::Sha256Provider;

fn bench_block_construction(c: &mut Criterion) {
    c.bench_function("block_construction", |b| {
        b.iter(|| {
            let header = BlockHeader::new(
                Version::new(1),
                Height::new(black_box(1000)),
                Timestamp::new(black_box(1234567890)),
                PreviousHash::zero(),
                MerkleRoot::new(vec![0u8; 32]),
                Difficulty::new(black_box(1)),
                Nonce::new(black_box(0)),
            );
            let transactions: Vec<Transaction> = vec![];
            let metadata = BlockMetadata::new(0, transactions.len());
            let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));
            Block::new(header, transactions, metadata, block_hash)
        })
    });
}

fn bench_header_hashing(c: &mut Criterion) {
    let header = BlockHeader::new(
        Version::new(1),
        Height::new(0),
        Timestamp::new(1000),
        PreviousHash::zero(),
        MerkleRoot::new(vec![0u8; 32]),
        Difficulty::new(1),
        Nonce::new(0),
    );
    let transactions: Vec<Transaction> = vec![];
    let metadata = BlockMetadata::new(0, transactions.len());
    let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));
    let block = Block::new(header, transactions, metadata, block_hash);
    let hasher = Sha256Provider;

    c.bench_function("header_hashing", |b| b.iter(|| block.hash_with(&hasher)));
}

fn bench_header_serialization(c: &mut Criterion) {
    let header = BlockHeader::new(
        Version::new(1),
        Height::new(0),
        Timestamp::new(1000),
        PreviousHash::zero(),
        MerkleRoot::new(vec![0u8; 32]),
        Difficulty::new(1),
        Nonce::new(0),
    );

    c.bench_function("header_serialization", |b| {
        b.iter(|| bincode::serialize(&header).unwrap())
    });
}

fn bench_block_serialization(c: &mut Criterion) {
    let header = BlockHeader::new(
        Version::new(1),
        Height::new(0),
        Timestamp::new(1000),
        PreviousHash::zero(),
        MerkleRoot::new(vec![0u8; 32]),
        Difficulty::new(1),
        Nonce::new(0),
    );
    let transactions: Vec<Transaction> = vec![];
    let metadata = BlockMetadata::new(0, transactions.len());
    let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));
    let block = Block::new(header, transactions, metadata, block_hash);

    c.bench_function("block_serialization", |b| {
        b.iter(|| bincode::serialize(&block).unwrap())
    });
}

criterion_group!(
    benches,
    bench_block_construction,
    bench_header_hashing,
    bench_header_serialization,
    bench_block_serialization
);
criterion_main!(benches);
