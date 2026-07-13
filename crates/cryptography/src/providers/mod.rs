//! Cryptographic providers.
//!
//! This module re-exports all concrete cryptographic algorithm implementations.
//! Downstream code should depend on the abstract traits in [`crate::core`], not on
//! these concrete types directly.

pub mod ed25519;
pub mod sha256;

pub use ed25519::{Ed25519PrivateKey, Ed25519Provider, Ed25519PublicKey, Ed25519Signature};
pub use sha256::Sha256Provider;
