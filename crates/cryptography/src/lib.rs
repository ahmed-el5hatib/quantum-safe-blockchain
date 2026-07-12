//! Cryptographic primitives, traits, and abstractions.
//!
//! This crate provides trait-based interfaces for all cryptographic operations.
//! Algorithms are pluggable and can be replaced without changing downstream code.
pub mod error;
pub mod hash;
pub mod kem;
pub mod signatures;
pub mod traits;
pub use error::CryptoResult;
