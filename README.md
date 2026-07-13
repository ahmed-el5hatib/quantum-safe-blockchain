# Quantum Safe Blockchain (QSB)

![Rust](https://img.shields.io/badge/rust-stable-red)
![License](https://img.shields.io/badge/License-MIT%2FApache%202.0-blue)
![Status](https://img.shields.io/badge/status-active-brightgreen)

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
- **Classical**: Ed25519, ECDSA (P-256)
- **Post-Quantum (Future)**: ML-DSA (Dilithium), Falcon, SPHINCS+

### KEM
- **Classical**: X25519
- **Post-Quantum (Future)**: ML-KEM (Kyber), Hybrid KEM

### Hash Functions
- SHA-256, SHA-3, BLAKE3

### Consensus (Future)
- Proof of Work, Proof of Stake, PBFT, Raft, HotStuff

## Quick Start

```bash
git clone https://github.com/quantum-safe-blockchain/quantum-safe-blockchain.git
cd quantum-safe-blockchain
cargo build --workspace
cargo test --workspace
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.

## Security

See [SECURITY.md](SECURITY.md) for our security policy and how to report vulnerabilities.

## License

Dual-licensed under MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

## Roadmap

See [ROADMAP.md](ROADMAP.md) for project milestones and future plans.
