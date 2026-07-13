# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-07-13

### Added

- Production-ready cryptography framework with strict Provider Architecture
- Core cryptographic traits fully decoupled from implementations:
  - `HashFunction`, `SignatureAlgorithm`, `PublicKey`, `PrivateKey`, `Signature`
  - `RandomGenerator`, `Encoder`, `Decoder`
- SHA-256 hash provider (`Sha256Provider`) with deterministic hashing, streaming support, and algorithm metadata
- Ed25519 signature provider (`Ed25519Provider`) with key generation, signing, verification, serialization, and byte-level import/export
- Strongly typed key wrappers: `HashDigest`, `KeyPair` with `Display`, `Debug`, `Clone`, `Eq`, `Serde`
- HEX and Base64 encoding/decoding implementations
- OS-level cryptographically secure RNG (`StdRngGenerator`) via `rand_core::OsRng`
- Comprehensive `CryptoError` enum covering all cryptographic failure modes
- Criterion benchmark suite for SHA-256 and Ed25519 operations
- Four runnable examples: `generate_keypair`, `hash_message`, `sign_message`, `export_import_key`

### Changed

- Reorganized `cryptography` crate into `core/` (traits, errors, types) and `providers/` (concrete algorithms)
- Established crypto-agility guarantee: blockchain logic depends only on traits, never on concrete algorithms
- Updated root README with Cryptography Layer overview, badges, and Project Status section
- Updated CHANGELOG with structured release notes

### Documentation

- Added per-crate READMEs with architecture diagrams and future roadmaps
- Documented security considerations, algorithm limitations, and PQC migration strategy
- Added intra-doc links and rustdoc comments to all public APIs
- Created architecture documentation with Mermaid diagrams in `docs/architecture/overview.md`

### Tests

- 25 unit tests covering determinism, edge cases, serialization, encoding/decoding, and invalid inputs
- 9 doctests validating all public examples in documentation
- All tests pass on stable Rust 1.75.0

### Benchmarks

- Criterion benchmarks for SHA-256 hashing (small and large inputs)
- Criterion benchmarks for Ed25519 key generation, signing, and verification
- Benchmark reports generated with `cargo bench -p cryptography`

### Known Limitations

- Full workspace build is blocked on Windows due to missing `libclang` (required by `bindgen` for native dependencies such as RocksDB). Pure-Rust crates (`cryptography`, `blockchain-core`) build and test successfully.
- Only classical cryptography is implemented in this release. Post-quantum algorithms (ML-DSA, ML-KEM, Falcon, SPHINCS+) are planned for future milestones.
- No blockchain logic, networking, consensus, or wallet functionality is implemented yet.

### Future Work

- Add SHA3 and BLAKE3 hash providers
- Add ECDSA P-256 provider
- Integrate ML-DSA, ML-KEM, Falcon, and SPHINCS+ in a future PQC milestone
- Implement blockchain-core domain logic (blocks, transactions, chain validation)
- Implement networking, consensus, wallet, and RPC layers

## [0.0.1] - 2026-07-13

### Added

- Repository initialization
- Project foundation and tooling
- CI/CD pipeline with GitHub Actions
- Documentation foundation (README, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, ROADMAP)
- rustfmt and clippy configuration
- Dependabot configuration
- cargo-deny configuration
