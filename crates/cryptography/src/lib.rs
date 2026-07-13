//! # Quantum Safe Blockchain Cryptography
//!
//! Production-grade cryptographic framework for QSB.
//!
//! This crate provides trait-based interfaces for all cryptographic operations. Algorithms
//! are pluggable and can be replaced without changing downstream code. This enables
//! **crypto-agility**: future post-quantum algorithms (ML-DSA, ML-KEM, Falcon, SPHINCS+)
//! can be added without modifying any blockchain logic.
//!
//! ## Architecture
//!
//! ```text
//! cryptography/
//! ├── core/
//! │   ├── errors/      # CryptoError, CryptoResult
//! │   ├── traits/      # HashFunction, SignatureAlgorithm, PublicKey, etc.
//! │   ├── key/         # HashDigest, KeyPair
//! │   ├── encoding/    # HexEncoder, HexDecoder, Base64Encoder, Base64Decoder
//! │   ├── random/      # StdRngGenerator
//! │   ├── hash/        # HashFunction trait re-export
//! │   └── signature/   # SignatureAlgorithm trait re-export
//! └── providers/
//!     ├── sha256/      # SHA-256 implementation
//!     └── ed25519/     # Ed25519 implementation
//! ```
//!
//! ## Usage
//!
//! ### Hashing
//!
//! ```
//! use cryptography::providers::Sha256Provider;
//! use cryptography::core::traits::HashFunction;
//!
//! let hasher = Sha256Provider;
//! let digest = hasher.hash(b"hello world");
//! println!("SHA-256: {}", digest);
//! ```
//!
//! ### Signing
//!
//! ```
//! use cryptography::providers::{Ed25519Provider, Ed25519PublicKey};
//! use cryptography::core::traits::SignatureAlgorithm;
//!
//! let provider = Ed25519Provider;
//! let keypair = provider.generate_keypair().unwrap();
//! let message = b"hello world";
//! let signature = provider.sign(keypair.private_key(), message).unwrap();
//! provider.verify(keypair.public_key(), message, &signature).unwrap();
//! ```
//!
//! ## Crypto Agility
//!
//! The blockchain never knows that Ed25519 exists. It only communicates through traits:
//!
//! - [`HashFunction`]
//! - [`SignatureAlgorithm`]
//!
//! When post-quantum algorithms are added, only the provider implementation changes.
//! No blockchain code needs modification.
//!
//! ## Security Considerations
//!
//! - All randomness comes from OS-level entropy (`rand_core::OsRng`).
//! - Private keys implement [`ZeroizeOnDrop`] to clear memory on drop.
//! - No panics in library code; all operations return [`CryptoResult`].
//! - No secrets are logged.
//! - Ed25519 verification uses `verify_strict` to prevent malleability attacks.
//!
//! ## Future PQC Roadmap
//!
//! - [ ] ML-DSA (Dilithium) - NIST Level 2/3/5
//! - [ ] ML-KEM (Kyber) - NIST Level 1/3/5
//! - [ ] Falcon - NIST Level 1/5
//! - [ ] SPHINCS+ - NIST Level 1/3/5
//! - [ ] Hybrid KEM (X25519 + ML-KEM)
//! - [ ] SHA3 and BLAKE3 hash providers

pub mod core;
pub mod providers;

pub use core::traits::{
    Decoder, Encoder, HashFunction, PrivateKey, PublicKey, RandomGenerator, Signature,
    SignatureAlgorithm,
};
pub use core::{CryptoError, CryptoResult, HashDigest, KeyPair};
pub use providers::{
    Ed25519PrivateKey, Ed25519Provider, Ed25519PublicKey, Ed25519Signature, Sha256Provider,
};
