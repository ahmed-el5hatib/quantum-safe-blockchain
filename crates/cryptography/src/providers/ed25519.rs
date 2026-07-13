//! Ed25519 signature algorithm provider.
//!
//! Implements the [`SignatureAlgorithm`] trait using the `ed25519-dalek` crate from the
//! RustCrypto ecosystem.
//!
//! # Security
//!
//! Ed25519 is a widely deployed, high-performance signature scheme with strong security
//! guarantees (128-bit classical security). It is **not** post-quantum secure; Grover's
//! algorithm reduces effective security to approximately 64 bits. This provider is
//! included for backward compatibility and classical security only.
//!
//! # Key Types
//!
//! - [`Ed25519PublicKey`]: 32-byte public key
//! - [`Ed25519PrivateKey`]: 32-byte secret key
//! - [`Ed25519Signature`]: 64-byte signature

use std::fmt;

use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::core::errors::CryptoError;
use crate::core::traits::{PrivateKey, PublicKey, Signature, SignatureAlgorithm};
use crate::core::{CryptoResult, KeyPair};
use rand_core::{OsRng, RngCore};

/// An Ed25519 public key.
///
/// # Security
///
/// Public keys are not secret and do not require zeroization.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Ed25519PublicKey(VerifyingKey);

impl Ed25519PublicKey {
    /// Creates a new `Ed25519PublicKey` from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidKey` if the bytes are not a valid Ed25519 public key.
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        let array: [u8; 32] = bytes.try_into().map_err(|_| {
            CryptoError::InvalidKey(format!(
                "Ed25519 public key must be 32 bytes, got {}",
                bytes.len()
            ))
        })?;
        let verifying_key = VerifyingKey::from_bytes(&array)
            .map_err(|e| CryptoError::InvalidKey(format!("invalid Ed25519 public key: {e}")))?;
        Ok(Self(verifying_key))
    }

    /// Returns the public key as a 32-byte array.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Verifies a signature against a message using this public key.
    pub fn verify(&self, message: &[u8], signature: &Ed25519Signature) -> CryptoResult<()> {
        let bytes: [u8; 64] = signature
            .as_bytes()
            .try_into()
            .map_err(|_| CryptoError::VerificationFailed("invalid signature length".to_string()))?;
        let dalek_sig = DalekSignature::from_bytes(&bytes);
        self.0
            .verify_strict(message, &dalek_sig)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))
    }
}

impl PublicKey for Ed25519PublicKey {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    fn algorithm_name(&self) -> &str {
        "Ed25519"
    }
}

impl fmt::Display for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.as_bytes() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// An Ed25519 private key.
///
/// # Security
///
/// This type clears the secret key from memory when dropped.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Ed25519PrivateKey(SigningKey);

impl Ed25519PrivateKey {
    /// Creates a new `Ed25519PrivateKey` from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidKey` if the bytes are not a valid Ed25519 secret key.
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        let array: [u8; 32] = bytes.try_into().map_err(|_| {
            CryptoError::InvalidKey(format!(
                "Ed25519 private key must be 32 bytes, got {}",
                bytes.len()
            ))
        })?;
        let signing_key = SigningKey::from_bytes(&array);
        Ok(Self(signing_key))
    }

    /// Returns the private key as a 32-byte array.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Signs a message with this private key.
    pub fn sign(&self, message: &[u8]) -> CryptoResult<Ed25519Signature> {
        let signature = self.0.sign(message);
        Ok(Ed25519Signature(signature.to_bytes().to_vec()))
    }

    /// Returns the corresponding public key.
    pub fn public_key(&self) -> Ed25519PublicKey {
        Ed25519PublicKey(self.0.verifying_key())
    }
}

impl PrivateKey for Ed25519PrivateKey {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    fn algorithm_name(&self) -> &str {
        "Ed25519"
    }
}

impl fmt::Display for Ed25519PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.as_bytes() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// An Ed25519 digital signature.
///
/// # Security
///
/// Signatures are not secret and do not require zeroization.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Ed25519Signature(Vec<u8>);

impl Ed25519Signature {
    /// Creates a new `Ed25519Signature` from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidSignature` if the bytes are not a valid Ed25519 signature.
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != 64 {
            return Err(CryptoError::InvalidSignature(format!(
                "Ed25519 signature must be 64 bytes, got {}",
                bytes.len()
            )));
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Returns the signature as a 64-byte array.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0
            .as_slice()
            .try_into()
            .expect("Ed25519 signature is always 64 bytes")
    }
}

impl Signature for Ed25519Signature {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn algorithm_name(&self) -> &str {
        "Ed25519"
    }
}

impl fmt::Display for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Ed25519 signature algorithm provider.
///
/// This is a stateless, thread-safe implementation that can be shared across threads.
///
/// # Examples
///
/// ```
/// use cryptography::providers::Ed25519Provider;
/// use cryptography::core::traits::SignatureAlgorithm;
///
/// let provider = Ed25519Provider;
/// let keypair = provider.generate_keypair().unwrap();
/// let message = b"hello world";
/// let signature = provider.sign(keypair.private_key(), message).unwrap();
/// provider.verify(keypair.public_key(), message, &signature).unwrap();
/// ```
#[derive(Clone, Debug, Default)]
pub struct Ed25519Provider;

impl Ed25519Provider {
    /// Creates a new `Ed25519Provider`.
    pub fn new() -> Self {
        Self
    }
}

impl SignatureAlgorithm for Ed25519Provider {
    type PublicKey = Ed25519PublicKey;
    type PrivateKey = Ed25519PrivateKey;
    type Signature = Ed25519Signature;

    fn generate_keypair(&self) -> CryptoResult<KeyPair<Self::PublicKey, Self::PrivateKey>> {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();
        Ok(KeyPair::new(
            Ed25519PublicKey(verifying_key),
            Ed25519PrivateKey(signing_key),
        ))
    }

    fn sign(
        &self,
        private_key: &Self::PrivateKey,
        message: &[u8],
    ) -> CryptoResult<Self::Signature> {
        let signature = private_key.0.sign(message);
        Ok(Ed25519Signature(signature.to_bytes().to_vec()))
    }

    fn verify(
        &self,
        public_key: &Self::PublicKey,
        message: &[u8],
        signature: &Self::Signature,
    ) -> CryptoResult<()> {
        let bytes: [u8; 64] = signature
            .as_bytes()
            .try_into()
            .map_err(|_| CryptoError::VerificationFailed("invalid signature length".to_string()))?;
        let dalek_sig = DalekSignature::from_bytes(&bytes);
        public_key
            .0
            .verify_strict(message, &dalek_sig)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))?;
        Ok(())
    }

    fn algorithm_name(&self) -> &str {
        "Ed25519"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::SignatureAlgorithm;

    #[test]
    fn test_keypair_generation() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        assert_eq!(keypair.public_key().algorithm_name(), "Ed25519");
        assert_eq!(keypair.private_key().algorithm_name(), "Ed25519");
    }

    #[test]
    fn test_sign_verify() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let message = b"hello world";
        let signature = provider.sign(keypair.private_key(), message).unwrap();
        provider
            .verify(keypair.public_key(), message, &signature)
            .unwrap();
    }

    #[test]
    fn test_verify_wrong_message() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let signature = provider.sign(keypair.private_key(), b"hello").unwrap();
        let result = provider.verify(keypair.public_key(), b"world", &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_key() {
        let provider = Ed25519Provider;
        let kp1 = provider.generate_keypair().unwrap();
        let kp2 = provider.generate_keypair().unwrap();
        let signature = provider.sign(kp1.private_key(), b"hello").unwrap();
        let result = provider.verify(kp2.public_key(), b"hello", &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_deserialize_public_key() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let bytes = bincode::serialize(keypair.public_key()).unwrap();
        let restored: Ed25519PublicKey = bincode::deserialize(&bytes).unwrap();
        assert_eq!(keypair.public_key().as_bytes(), restored.as_bytes());
    }

    #[test]
    fn test_serialize_deserialize_private_key() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let bytes = bincode::serialize(keypair.private_key()).unwrap();
        let restored: Ed25519PrivateKey = bincode::deserialize(&bytes).unwrap();
        assert_eq!(keypair.private_key().as_bytes(), restored.as_bytes());
    }

    #[test]
    fn test_serialize_deserialize_signature() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let message = b"test message";
        let signature = provider.sign(keypair.private_key(), message).unwrap();
        let bytes = bincode::serialize(&signature).unwrap();
        let restored: Ed25519Signature = bincode::deserialize(&bytes).unwrap();
        assert_eq!(signature.as_bytes(), restored.as_bytes());
    }

    #[test]
    fn test_empty_message() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let signature = provider.sign(keypair.private_key(), b"").unwrap();
        provider
            .verify(keypair.public_key(), b"", &signature)
            .unwrap();
    }

    #[test]
    fn test_large_message() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let message = vec![0xAB; 10000];
        let signature = provider.sign(keypair.private_key(), &message).unwrap();
        provider
            .verify(keypair.public_key(), &message, &signature)
            .unwrap();
    }

    #[test]
    fn test_repeated_signing_same_signature() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let message = b"deterministic test";
        let sig1 = provider.sign(keypair.private_key(), message).unwrap();
        let sig2 = provider.sign(keypair.private_key(), message).unwrap();
        assert_eq!(sig1.as_bytes(), sig2.as_bytes());
    }

    #[test]
    fn test_from_bytes_roundtrip() {
        let provider = Ed25519Provider;
        let keypair = provider.generate_keypair().unwrap();
        let pk_bytes = keypair.public_key().as_bytes().to_vec();
        let sk_bytes = keypair.private_key().as_bytes().to_vec();

        let pk = Ed25519PublicKey::from_bytes(&pk_bytes).unwrap();
        let sk = Ed25519PrivateKey::from_bytes(&sk_bytes).unwrap();

        let message = b"roundtrip";
        let sig = provider.sign(&sk, message).unwrap();
        provider.verify(&pk, message, &sig).unwrap();
    }

    #[test]
    fn test_invalid_public_key_length() {
        let result = Ed25519PublicKey::from_bytes(b"short");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_private_key_length() {
        let result = Ed25519PrivateKey::from_bytes(b"short");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature_length() {
        let result = Ed25519Signature::from_bytes(b"short");
        assert!(result.is_err());
    }
}
