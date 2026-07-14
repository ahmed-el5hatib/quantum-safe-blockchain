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
│   ├── transaction/          # Immutable, crypto-agnostic transaction domain model (milestone 4.3)
│   ├── validation/            # Reusable, deterministic validation engine (milestone 4.4)
│   ├── consensus/             # Consensus algorithms (PoW, PoS, PBFT, etc.)
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

## Transaction Domain Model (Milestone 4.3)

Milestone 4.3 introduces `transaction`, an immutable, extensible, and **agnostic** transaction
domain model. It implements only the *domain objects* — no validation, state, networking, or
signing — that future components (validation engine, mempool, consensus, storage, wallet) consume.

### Design

- **Immutable after construction.** A `Transaction` is produced only by `TransactionBuilder` and
  cannot be mutated afterward.
- **Builder Pattern.** `TransactionBuilder` validates structure (duplicate inputs/outputs, zero
  amounts, missing inputs/outputs, invalid versions) *before* finalizing.
- **Crypto-agnostic hashing.** The hash is computed only through the abstract
  [`HashFunction`](crates/cryptography/src/core/traits/mod.rs) trait. SHA-256, SHA-3, BLAKE3, or any
  future provider work without changing the model.
- **Signature abstraction.** The transaction never names Ed25519. Signatures and keys are carried as
  opaque, algorithm-labelled bytes (`SignatureContainer`, `PublicKeyReference`), so ML-DSA / Falcon /
  SPHINCS+ integrate by swapping the crypto provider.
- **Signed-after-hash.** The hash covers version, timestamp, type, inputs, outputs, and metadata —
  **excluding** signatures and keys — so the hash is stable regardless of which signatures are attached.

### Core Types

| Type | Purpose |
|------|---------|
| `Transaction` | The immutable, fully-constructed transaction. |
| `TransactionId` | Unique id (equal to the transaction hash). |
| `TransactionHash` | Canonical hash of hash-relevant fields (excludes signatures). |
| `TransactionVersion` | Protocol version (forward compatible). |
| `TransactionType` | `Transfer`, `Coinbase`, or `Custom(u32)` for future families. |
| `TransactionInput` | Reference to a prior output + unlocking/script/witness placeholders. |
| `TransactionOutput` | Recipient, amount, output type, locking/contract placeholders. |
| `TransactionMetadata` | Fee, lock time, memo, and a `BTreeMap` of forward-compatible extensions. |
| `SignatureContainer` | Opaque, keyed collection of (public key, signature) entries. |
| `PublicKeyReference` | Opaque public key bytes + algorithm name. |
| `TransactionBuilder` | Validating builder that produces an immutable `Transaction`. |
| `TransactionT` | Read-only trait view used by higher-level components. |

### Serialization

Every domain type derives **Serde** and supports **JSON** (`Transaction::to_json` /
`Transaction::from_json`) and **binary** (`bincode`, via `Transaction::to_binary` /
`Transaction::from_binary`).

### Integration

- **Blockchain Core**: `Transaction` reuses the unified `CoreError` / `CoreResult` types and is the
  unit committed into block structures.
- **Merkle Tree**: transaction hashes (`TransactionHash`) are the leaves committed into the block
  Merkle root; the hash is stable so commitments never shift once signatures are added.
- **Cryptography Layer**: hashing goes only through the `HashFunction` trait, and signatures/keys are
  carried as opaque, algorithm-labelled bytes via the `SignatureAlgorithm` abstraction.
- **Future Validation Engine**: reads the `TransactionT` trait view, recomputes/verifies the hash via
  `Transaction::verify_hash`, and verifies `SignatureContainer` entries against the active
  `SignatureAlgorithm` provider — none of which require changes to this crate.

### Example

```rust
use cryptography::providers::Sha256Provider;
use transaction::{TransactionBuilder, TransactionInput, TransactionOutput, OutputType};

let tx = TransactionBuilder::new(Sha256Provider)
    .with_timestamp(1_700_000_000)
    .add_input(TransactionInput::new(
        transaction::TransactionId::from_hex(&"11".repeat(32)).unwrap(),
        0,
    ))
    .unwrap()
    .add_output(TransactionOutput::new(vec![0xAA; 32], 100, OutputType::Standard).unwrap())
    .unwrap()
    .finalize()
    .unwrap();

assert!(tx.verify_hash(&Sha256Provider));
```

## Validation Engine (Milestone 4.4)

Milestone 4.4 introduces `validation`, a reusable, deterministic, and **trait-based** validation
engine. It validates **blocks** and **transactions** independently of networking, storage,
consensus, and execution, so every future QSB blockchain implementation reuses it unchanged.

### Design

- **Rule → Pipeline → Engine.** Validation is composed from atomic [`ValidationRule`](crates/validation/src/rule.rs)
  checks collected into a [`ValidationPipeline`](crates/validation/src/pipeline.rs), orchestrated by
  the [`ValidationEngine`](crates/validation/src/engine.rs).
- **Stateless & panic-free.** Rules read only their target and the
  [`ValidationContext`](crates/validation/src/context.rs), and every validator returns
  `Result<T, ValidationError>`.
- **Crypto-agile.** The engine depends only on the abstract
  [`HashFunction`](crates/cryptography/src/core/traits/mod.rs) trait. The concrete algorithm (SHA-256,
  SHA-3, BLAKE3, or a future post-quantum hash) is injected through the context — the engine never
  names a specific algorithm.
- **Configurable rules.** Each rule can be enabled/disabled through
  [`ValidationConfig`](crates/validation/src/context.rs), so operators tune policy without code changes.
- **Rich reports.** Validation yields a [`ValidationReport`](crates/validation/src/report.rs) with
  success/failure, every executed rule (and its timing), error details, and optional warnings.

### Block Rules

`header_format`, `hash_consistency`, `merkle_root`, `timestamp`, `height`, `previous_hash`,
`genesis_constraints`, `metadata`.

### Transaction Rules

`structure`, `version`, `inputs_exist`, `outputs_exist`, `duplicate_inputs`, `duplicate_outputs`,
`amount`, `hash_integrity`, `signature_placeholder`. Transaction validation is structural/semantic
only (no UTXO set, no signature verification — those belong to later milestones).

### Integration with the Future Blockchain Engine

The Blockchain Engine (milestone 4.5+) will:

1. Build blocks whose hashes and Merkle roots follow the engine's canonical commitments
   (`compute_block_hash`, `compute_merkle_root`), reusing the same `HashFunction` provider.
2. Call `engine.check_block(block, ctx)` / `engine.check_transaction(tx, ctx)` during block
   reception, mining, and mempool admission.
3. Inject chain facts (clock, previous hash/height, genesis flag) through `ValidationContext`,
   leaving the engine itself stateless and reusable.

### Example

```rust
use std::sync::Arc;
use cryptography::providers::Sha256Provider;
use validation::{ValidationContext, ValidationEngine};

let engine = ValidationEngine::with_defaults();
let ctx = ValidationContext::new(Arc::new(Sha256Provider));
// let report = engine.validate_block(&block, &ctx);
// engine.check_block(&block, &ctx)?; // Result<(), ValidationError>
```

See [`docs/architecture/validation-engine.md`](docs/architecture/validation-engine.md) for the full
architecture, rule pipeline, validation lifecycle, extensibility, and performance analysis.

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
- **Transaction Domain Model (4.3)**: Immutable, crypto-agnostic transaction domain objects
  (`Transaction`, `TransactionBuilder`, inputs/outputs, signatures, metadata) with builder-based
  validation, canonical hashing, and JSON/binary serialization
- **Validation Engine (4.4)**: Reusable, deterministic, trait-based validation framework
  (`ValidationEngine`, `ValidationPipeline`, `ValidationRule`, `ValidationContext`,
  `ValidationReport`) with modular block/transaction rules, configurable enable/disable, rich
  reporting, comprehensive tests, and benchmarks

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
