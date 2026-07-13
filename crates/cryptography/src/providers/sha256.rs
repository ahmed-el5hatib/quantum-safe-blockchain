//! SHA-256 hash function provider.
//!
//! Implements the [`HashFunction`] trait using the `sha2` crate from the RustCrypto ecosystem.
//!
//! # Security
//!
//! SHA-256 is a well-studied hash function suitable for general-purpose use. It is not
//! collision-resistant against quantum computers (Grover's algorithm reduces effective security
//! to 128 bits), but it remains secure for classical pre-image resistance.
//!
//! For quantum-resistant applications, migrate to SHA3-256 or BLAKE3 once those providers
//! are available.

use std::io::Read;

use sha2::{Digest, Sha256};

use crate::core::traits::HashFunction;
use crate::core::HashDigest;
use crate::core::{CryptoError, CryptoResult};

/// SHA-256 hash function provider.
///
/// This is a stateless, thread-safe implementation that can be shared across threads.
///
/// # Examples
///
/// ```
/// use cryptography::providers::Sha256Provider;
/// use cryptography::core::traits::HashFunction;
///
/// let hasher = Sha256Provider;
/// let digest = hasher.hash(b"hello world");
/// assert_eq!(digest.len(), 32);
/// ```
#[derive(Clone, Debug, Default)]
pub struct Sha256Provider;

impl Sha256Provider {
    /// Creates a new `Sha256Provider`.
    pub fn new() -> Self {
        Self
    }
}

impl HashFunction for Sha256Provider {
    fn hash(&self, data: &[u8]) -> HashDigest {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        HashDigest::new(result.to_vec())
    }

    fn hash_stream(&self, reader: &mut dyn Read) -> CryptoResult<HashDigest> {
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let n = reader.read(&mut buffer).map_err(|e| {
                CryptoError::InternalCryptoError(format!("failed to read stream: {e}"))
            })?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(HashDigest::new(hasher.finalize().to_vec()))
    }

    fn digest_size(&self) -> usize {
        32
    }

    fn algorithm_name(&self) -> &str {
        "SHA-256"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::HashFunction;

    #[test]
    fn test_sha256_known_vector() {
        let hasher = Sha256Provider;
        let digest = hasher.hash(b"abc");
        let expected =
            hex::decode("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
                .unwrap();
        assert_eq!(digest.as_bytes(), expected.as_slice());
    }

    #[test]
    fn test_sha256_deterministic() {
        let hasher = Sha256Provider;
        let d1 = hasher.hash(b"test");
        let d2 = hasher.hash(b"test");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_sha256_different_inputs() {
        let hasher = Sha256Provider;
        let d1 = hasher.hash(b"test");
        let d2 = hasher.hash(b"Test");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_sha256_empty_input() {
        let hasher = Sha256Provider;
        let digest = hasher.hash(b"");
        let expected =
            hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
                .unwrap();
        assert_eq!(digest.as_bytes(), expected.as_slice());
    }

    #[test]
    fn test_sha256_digest_size() {
        let hasher = Sha256Provider;
        assert_eq!(hasher.digest_size(), 32);
    }

    #[test]
    fn test_sha256_algorithm_name() {
        let hasher = Sha256Provider;
        assert_eq!(hasher.algorithm_name(), "SHA-256");
    }

    #[test]
    fn test_sha256_large_input() {
        let hasher = Sha256Provider;
        let data = vec![0xAB; 10000];
        let digest = hasher.hash(&data);
        assert_eq!(digest.len(), 32);
    }

    #[test]
    fn test_sha256_stream() {
        let hasher = Sha256Provider;
        let data = b"hello world";
        let direct = hasher.hash(data);
        let mut reader = data.as_slice();
        let stream = hasher.hash_stream(&mut reader).unwrap();
        assert_eq!(direct, stream);
    }
}
