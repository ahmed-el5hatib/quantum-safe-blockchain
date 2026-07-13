# cryptography

Cryptographic primitives, traits, and abstractions for the Quantum Safe Blockchain.

## Architecture

This crate follows a strict **Provider Architecture** that separates interfaces from implementations. Nothing outside the `providers/` directory depends on any specific cryptographic algorithm.

```text
cryptography/
├── src/
│   ├── lib.rs                    # Crate root, public API
│   ├── core/
│   │   ├── errors/               # CryptoError, CryptoResult
│   │   ├── traits/               # HashFunction, SignatureAlgorithm, PublicKey, etc.
│   │   ├── key/                  # HashDigest, KeyPair
│   │   ├── encoding/             # HexEncoder, HexDecoder, Base64Encoder, Base64Decoder
│   │   ├── random/               # StdRngGenerator (OS-level RNG)
│   │   ├── hash/                 # HashFunction trait re-export
│   │   └── signature/            # SignatureAlgorithm trait re-export
│   └── providers/
│       ├── sha256.rs             # SHA-256 hash provider
│       └── ed25519.rs            # Ed25519 signature provider
├── benches/
│   └── crypto_bench.rs           # Criterion benchmarks
└── examples/
    ├── generate_keypair.rs
    ├── hash_message.rs
    ├── sign_message.rs
    └── export_import_key.rs
```

## Provider Architecture

The blockchain never knows that Ed25519 or SHA-256 exist. It only communicates through traits:

- [`HashFunction`]
- [`SignatureAlgorithm`]
- [`PublicKey`]
- [`PrivateKey`]
- [`Signature`]
- [`RandomGenerator`]
- [`Encoder`]
- [`Decoder`]

When post-quantum algorithms are added (ML-DSA, ML-KEM, Falcon, SPHINCS+), only the provider implementation changes. No blockchain code needs modification.

## Implemented Features

### Hash Providers
- **SHA-256**: Deterministic hashing with `hash()`, `hash_bytes()`, `hash_stream()`, `digest_size()`, and `algorithm_name()`.

### Signature Providers
- **Ed25519**: Key generation, signing, verification, serialization/deserialization, and byte export/import.

### Key Types
- Strongly typed wrappers: `PublicKey`, `PrivateKey`, `Signature`, `HashDigest`, `KeyPair`
- `Display`, `Debug`, `Clone`, `Eq`, `Serde` where appropriate
- Private keys implement memory zeroization on drop

### Encoding
- Hexadecimal encoding/decoding
- Base64 encoding/decoding

### Randomness
- OS-level cryptographically secure RNG via `rand_core::OsRng`

### Error Handling
- Comprehensive `CryptoError` enum with `thiserror`
- All operations return `CryptoResult<T>`

## Security Considerations

- All randomness comes from OS-level entropy (`rand_core::OsRng`).
- Private keys clear memory on drop.
- No panics in library code; all operations return `CryptoResult`.
- No secrets are logged.
- Ed25519 verification uses `verify_strict` to prevent malleability attacks.

## Future PQC Roadmap

- [ ] ML-DSA (Dilithium) - NIST Level 2/3/5
- [ ] ML-KEM (Kyber) - NIST Level 1/3/5
- [ ] Falcon - NIST Level 1/5
- [ ] SPHINCS+ - NIST Level 1/3/5
- [ ] Hybrid KEM (X25519 + ML-KEM)
- [ ] SHA3 and BLAKE3 hash providers
- [ ] ECDSA P-256 provider

## Running Examples

```bash
cargo run --example generate_keypair
cargo run --example hash_message
cargo run --example sign_message
cargo run --example export_import_key
```

## Running Benchmarks

```bash
cargo bench -p cryptography
```
