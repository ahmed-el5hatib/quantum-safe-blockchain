# Quantum Safe Blockchain (QSB)

[![Rust](https://img.shields.io/badge/rust-stable-red)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT%2FApache%202.0-blue)](LICENSE-MIT)
[![Status](https://img.shields.io/badge/status-active-brightgreen)](https://github.com/ahmed-el5hatib/quantum-safe-blockchain)
[![Build Status](https://img.shields.io/badge/build-passing-green)](https://github.com/ahmed-el5hatib/quantum-safe-blockchain/actions)
[![Documentation](https://img.shields.io/badge/docs-latest-blue)](https://docs.rs/cryptography/latest/cryptography/)

**QSB v0.3.0 — Cryptography Foundation**

A production-grade modular blockchain framework designed for research and experimentation with post-quantum cryptography, distributed systems, and consensus algorithms.

## Features

- **Post-Quantum Cryptography**: Crypto-agile architecture supporting classical and post-quantum signature schemes, KEMs, and hash functions.
- **Modular Consensus**: Pluggable consensus mechanisms (PoW, PoS, PBFT, Raft, HotStuff, and research algorithms).
- **Secure Networking**: Built on libp2p with peer discovery, secure channels, and message propagation.
- **Multiple Storage Backends**: Abstracted storage layer supporting RocksDB, Sled, and future backends.
- **Performance Engineering**: Comprehensive benchmarking suite measuring TPS, latency, CPU, memory, and disk usage.
- **Security First**: Constant-time cryptography, zero hardcoded secrets, validated inputs, and timing-leak prevention.

## Cryptography Layer

The cryptography layer follows a strict **Provider Architecture** that separates interfaces from implementations.

- **Core Traits**: `HashFunction`, `SignatureAlgorithm`, `PublicKey`, `PrivateKey`, `Signature`, `RandomGenerator`, `Encoder`, `Decoder`
- **Providers**: SHA-256 (hash), Ed25519 (signatures)
- **Crypto Agility**: The blockchain never knows that Ed25519 or SHA-256 exist. Future post-quantum algorithms (ML-DSA, ML-KEM, Falcon, SPHINCS+) require only new provider implementations.
- **Security**: OS-level randomness, zeroized private keys, no panics in library code, constant-time verification.

## Project Structure

```text
quantum-safe-blockchain/
├── crates/
│   ├── blockchain-core/      # Core types, traits, and abstractions
│   ├── merkle/               # Generic Merkle tree engine (milestone 4.2)
│   ├── cryptography/         # Signature, hash, KEM traits and implementations
│   │   ├── core/             # Implementation-independent traits and types
│   │   └── providers/        # Concrete algorithm implementations
│   ├── wallet/               # Key generation, signing, verification
│   ├── transaction/          # Transaction types and validation
│   ├── consensus/            # Consensus algorithms (PoW, PoS, PBFT, etc.)
│   ├── networking/           # libp2p networking layer
│   ├── node/                 # Node orchestration
│   ├── storage/              # Storage backend abstractions
│   ├── mempool/              # Transaction mempool
│   ├── miner/                # Mining logic
│   ├── rpc/                  # JSON-RPC server
│   ├── cli/                  # Command-line interface
│   ├── sdk/                  # Developer SDK
│   ├── benchmarks/           # Performance benchmarks
│   ├── explorer/             # Block explorer
│   ├── config/               # Configuration management
│   ├── telemetry/            # Metrics and telemetry
│   ├── logging/              # Structured logging
│   ├── serialization/        # Serialization utilities
│   ├── testing/              # Testing utilities
│   ├── examples/             # Example applications
│   └── integration-tests/    # Integration test suite
├── docs/                     # Documentation
└── .github/                  # CI/CD workflows
```

## Supported Algorithms

### Signatures
- **Implemented**: Ed25519
- **Future**: ECDSA (P-256), ML-DSA (Dilithium), Falcon, SPHINCS+

### KEM
- **Future**: X25519, ML-KEM (Kyber), Hybrid KEM

### Hash Functions
- **Implemented**: SHA-256
- **Future**: SHA-3, BLAKE3

### Consensus
- **Future**: Proof of Work, Proof of Stake, PBFT, Raft, HotStuff

## Merkle Tree Engine

Milestone 4.2 introduces `merkle`, a production-quality, deterministic, and **generic** Merkle Tree
library. It is the single, reusable engine that every future QSB module uses for commitments and
inclusion proofs.

### Design

- **Generic over the hash provider.** The engine depends only on the abstract
  [`HashFunction`](crates/cryptography/src/core/traits/mod.rs) trait. SHA-256, SHA-3, BLAKE3, or any
  future post-quantum hash works without changing a single line of tree logic.
- **No direct hashing.** The engine never calls SHA-256 (or any algorithm) directly; all hashing
  goes through the provider.
- **Deterministic.** The same leaves and the same provider always produce the same root.
- **Duplicate-last-leaf strategy.** When a level has an odd number of nodes, the last node is
  duplicated and hashed with itself (Bitcoin-style). This is enabled by default and can be disabled.

### Public API

| Type | Purpose |
|------|---------|
| `MerkleNode` | A single tree node (leaf or internal) in the node arena. |
| `MerkleTree` | The tree itself; build, root, height, leaf count, contains, proof, verify. |
| `MerkleRoot` | The root commitment; converts to/from `blockchain_core::MerkleRoot`. |
| `MerkleProof` | An `O(log n)` inclusion proof; serializable to JSON and binary. |
| `ProofStep` | One sibling hash plus its side (left/right). |
| `TreeBuilder` | Fluent builder for configuring and constructing trees. |
| `ProofVerifier` | Stateless verifier that checks proofs against a trusted root. |

### Operations

- `MerkleTree::build` — build from arbitrary data (leaves are hashed).
- `MerkleTree::build_from_hashes` — build directly from pre-computed hashes (e.g. transaction hashes).
- `root` / `height` / `depth` / `leaf_count` / `contains` — tree queries.
- `proof(index)` — generate a proof for any leaf.
- `verify` / `ProofVerifier` — verify a proof statelessly against a trusted root.

### Serialization

`MerkleTree`, `MerkleProof`, `MerkleRoot`, and `ProofStep` all support **Serde**, **JSON**, and
**binary** (`bincode`). The hash provider is not serialized; it is supplied again when reconstructing
a tree via `MerkleTree::from_json` / `MerkleTree::from_binary`.

### Integration with the Blockchain Data Model

- `merkle::MerkleRoot` converts losslessly to/from `blockchain_core::MerkleRoot`, which is stored in a
  block header.
- `MerkleTree` implements `blockchain_core::traits::MerkleTreeT`, so it can drop directly into the
  existing blockchain abstractions.
- Errors are unified with the blockchain error model via `blockchain_core::CoreError` (new variants:
  `EmptyTree`, `InvalidProof`, `InvalidLeaf`, `InvalidTree`).

### Example

```rust
use cryptography::providers::Sha256Provider;
use merkle::{MerkleTree, ProofVerifier};

let hasher = Sha256Provider;
let tree = MerkleTree::build(
    vec![b"tx-1".as_ref(), b"tx-2".as_ref(), b"tx-3".as_ref()],
    hasher.clone(),
).unwrap();
let root = tree.root();

let proof = tree.proof(1).unwrap();
let verifier = ProofVerifier::new(hasher);
assert!(verifier.verify(&proof));
assert_eq!(proof.root_hash, root.as_bytes());
```

## Quick Start

```bash
git clone https://github.com/quantum-safe-blockchain/quantum-safe-blockchain.git
cd quantum-safe-blockchain
cargo build --workspace
cargo test --workspace
```

## Project Status

### Completed

- **Repository Foundation**: CI/CD pipeline, Dependabot, cargo-deny, rustfmt, clippy
- **Architecture**: Modular hexagonal architecture, strict dependency layers, provider-based design
- **Cryptography Framework**: Production-ready crypto-agile layer with SHA-256 and Ed25519 providers
- **Merkle Tree Engine (4.2)**: Generic, deterministic, reusable Merkle tree with proof generation,
  verification, JSON/binary serialization, and blockchain-core integration

### In Progress / Future Milestones

- Blockchain Core (blocks, transactions, chain state)
- Networking (libp2p, peer discovery, sync)
- Consensus (PoW, PoS, PBFT)
- Wallet & SDK
- Explorer & GUI

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.

## Security

See [SECURITY.md](SECURITY.md) for our security policy and how to report vulnerabilities.

## License

Dual-licensed under MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

## Roadmap

See [ROADMAP.md](ROADMAP.md) for project milestones and future plans.
