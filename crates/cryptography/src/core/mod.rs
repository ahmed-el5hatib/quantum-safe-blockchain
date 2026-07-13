//! Core cryptographic interfaces and types.
//!
//! This module contains all abstract traits, error types, and strongly typed wrappers
//! that form the foundation of the cryptography layer. Nothing in this module depends
//! on any specific cryptographic algorithm or provider.

pub mod encoding;
pub mod errors;
pub mod hash;
pub mod random;
pub mod signature;
pub mod traits;

pub use errors::{CryptoError, CryptoResult};
pub use traits::{HashDigest, KeyPair};
