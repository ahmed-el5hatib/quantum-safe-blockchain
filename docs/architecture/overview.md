# Architecture Overview

## Design Principles

QSB is built on four foundational architectural patterns:

1. **Layered Architecture**: Separation of concerns across presentation, application, domain, and infrastructure layers.
2. **Hexagonal Architecture (Ports and Adapters)**: Core business logic isolated from external concerns.
3. **Dependency Injection**: Dependencies are injected via constructors and traits, enabling testability.
4. **Trait-based Interfaces**: Every implementation is replaceable through trait abstractions.

## Workspace Structure

```
quantum-safe-blockchain/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── blockchain-core/    # Core types, traits, abstractions
│   ├── cryptography/       # Crypto primitives and traits
│   ├── wallet/             # Key management and signing
│   ├── transaction/        # Transaction types
│   ├── consensus/          # Consensus algorithms
│   ├── networking/         # libp2p networking
│   ├── node/               # Node orchestration
│   ├── storage/            # Storage backends
│   ├── mempool/            # Transaction pool
│   ├── miner/              # Mining logic
│   ├── rpc/                # JSON-RPC server
│   ├── cli/                # CLI interface
│   ├── sdk/                # Developer SDK
│   ├── benchmarks/         # Performance benchmarks
│   ├── explorer/           # Block explorer
│   ├── config/             # Configuration
│   ├── telemetry/          # Metrics
│   ├── logging/            # Structured logging
│   ├── serialization/      # Serialization
│   ├── testing/            # Test utilities
│   ├── examples/           # Examples
│   └── integration-tests/  # Integration tests
└── docs/                   # Documentation
```

## Dependency Rules

```
Layer 0 (Foundation):
  blockchain-core, cryptography, serialization, config, logging

Layer 1 (Domain):
  wallet, transaction, consensus, storage, mempool

Layer 2 (Infrastructure):
  networking, miner, rpc, cli, explorer, telemetry

Layer 3 (Orchestration):
  node, sdk

Layer 4 (Observability & Testing):
  benchmarks, testing, examples, integration-tests
```

**Rule**: A crate may only depend on crates in its own layer or lower layers. Never depend on higher layers.

## Crypto Agility

All cryptographic operations are defined through traits:

- `SignatureScheme`: `sign()`, `verify()`, `keypair()`
- `HashFunction`: `hash()`, `hash_len()`
- `KemScheme`: `encapsulate()`, `decapsulate()`
- `Cipher`: `encrypt()`, `decrypt()`

New algorithms are added by implementing these traits. No existing code needs to change.

## Consensus Agility

Consensus algorithms are isolated crates implementing the `ConsensusEngine` trait:

```rust
pub trait ConsensusEngine: Send + Sync {
    type Block: BlockT;
    type Error: std::error::Error;
    
    async fn validate_block(&self, block: Self::Block) -> Result<(), Self::Error>;
    async fn propose_block(&self) -> Result<Self::Block, Self::Error>;
    async fn finalize_block(&self, block: Self::Block) -> Result<(), Self::Error>;
}
```

## Storage Agility

Storage backends implement the `StorageBackend` trait:

```rust
pub trait StorageBackend: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    fn delete(&self, key: &[u8]) -> Result<(), StorageError>;
    fn contains(&self, key: &[u8]) -> Result<bool, StorageError>;
}
```

## Networking

Networking uses libp2p with modular protocols:

- **Identify**: Peer identity and metadata
- **Ping**: Liveness detection
- **MDNS**: Local peer discovery
- **Gossipsub**: Block and transaction propagation
- **Request-Response**: Chain sync protocol
- **Autonat**: NAT traversal
- **Relay**: Circuit relay for unreachable peers

## Error Handling

- Libraries use `thiserror` for typed errors
- Applications use `anyhow` for context-rich errors
- All errors are `Send + Sync + 'static`
- No panics in library code

## Security

- `zeroize` for sensitive data
- Constant-time comparisons with `subtle`
- Input validation at all boundaries
- No secrets in logs
- All unsafe blocks documented
