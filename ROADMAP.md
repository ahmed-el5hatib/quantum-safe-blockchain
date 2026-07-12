# Roadmap

## Vision

QSB aims to become the leading open-source research platform for quantum-safe blockchain technology, enabling researchers worldwide to experiment with next-generation cryptographic primitives, consensus algorithms, and distributed systems.

## Milestones

### v0.1.0 - Foundation (Current)
- [x] Initialize Cargo workspace
- [x] Create core crate structure
- [x] Configure CI/CD pipeline
- [ ] Implement core type system
- [ ] Implement cryptography traits
- [ ] Implement Ed25519 signature support
- [ ] Implement X25519 KEM support
- [ ] Implement SHA-256 and BLAKE3 hashing

### v0.2.0 - Blockchain Core
- [ ] Genesis block generation
- [ ] Block and transaction types
- [ ] Merkle tree implementation
- [ ] Chain validation logic
- [ ] Difficulty adjustment
- [ ] Fork handling

### v0.3.0 - Networking
- [ ] libp2p integration
- [ ] Peer discovery
- [ ] Secure channels (Noise)
- [ ] Message propagation
- [ ] Chain synchronization protocol

### v0.4.0 - Consensus
- [ ] Proof of Work implementation
- [ ] Proof of Stake implementation
- [ ] PBFT implementation
- [ ] Consensus trait abstraction
- [ ] Consensus switching mechanism

### v0.5.0 - Storage and State
- [ ] Storage trait abstraction
- [ ] RocksDB backend
- [ ] Sled backend
- [ ] State pruning
- [ ] UTXO set management

### v0.6.0 - Wallet and Transactions
- [ ] Address generation
- [ ] Key generation and management
- [ ] Transaction creation and signing
- [ ] Transaction validation
- [ ] Mempool implementation

### v0.7.0 - Node and CLI
- [ ] Node orchestration
- [ ] CLI interface (clap)
- [ ] Configuration management (TOML, env vars)
- [ ] Logging and telemetry
- [ ] RPC server

### v0.8.0 - Performance and Testing
- [ ] Benchmark suite (TPS, latency, CPU, memory)
- [ ] Property tests
- [ ] Integration tests
- [ ] Performance regression suite

### v0.9.0 - Advanced Features
- [ ] Post-quantum signatures (ML-DSA, Falcon)
- [ ] Post-quantum KEM (ML-KEM, hybrid)
- [ ] Smart contract engine (research)
- [ ] Zero knowledge proofs (research)
- [ ] Light client support

### v1.0.0 - Stable Release
- [ ] Production-ready PoW or PoS consensus
- [ ] Full test coverage
- [ ] Complete documentation
- [ ] Security audit
- [ ] Performance optimization

## Research Directions

- Quantum attack simulators (Harvest Now, Decrypt Later)
- 51% attack simulator
- Sybil attack simulator
- Eclipse attack simulator
- Cross-chain communication protocols
- Hardware wallet integration
- MPC threshold signatures

## Long-term Vision

- Become the standard research framework for quantum-safe blockchain technology
- Support NIST post-quantum algorithms as they are standardized
- Enable reproducible blockchain research
- Foster academic and industrial collaboration
- Maintain highest security and code quality standards
