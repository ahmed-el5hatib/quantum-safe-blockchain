# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Cargo workspace structure
- Core crate skeletons
- CI/CD pipeline with GitHub Actions
- Documentation foundation (README, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, ROADMAP)
- rustfmt and clippy configuration
- Dependabot configuration
- cargo-deny configuration
- Cryptography framework with Provider Architecture
- Core cryptographic traits: `HashFunction`, `SignatureAlgorithm`, `PublicKey`, `PrivateKey`, `Signature`, `RandomGenerator`, `Encoder`, `Decoder`
- SHA-256 hash provider (`Sha256Provider`)
- Ed25519 signature provider (`Ed25519Provider`)
- Strongly typed key wrappers with `Display`, `Debug`, `Clone`, `Eq`, `Serde`
- HEX and Base64 encoding/decoding
- OS-level secure random number generator (`StdRngGenerator`)
- Comprehensive `CryptoError` type with `thiserror`
- Extensive unit tests and doctests
- Criterion benchmarks for SHA-256 and Ed25519 operations
- Runnable examples: `generate_keypair`, `hash_message`, `sign_message`, `export_import_key`

## [0.0.1] - 2026-07-13

### Added
- Repository initialization
- Project foundation and tooling
