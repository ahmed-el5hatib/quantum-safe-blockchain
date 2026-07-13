//! Core cryptographic traits.
//!
//! These traits define the abstract interfaces that all cryptographic providers must implement.
//! The blockchain and all higher-level code depend only on these traits, never on concrete
//! cryptographic implementations. This enables crypto-agility: new algorithms can be added
//! without changing any blockchain logic.

use std::io::Read;

use crate::core::errors::CryptoResult;

// =============================================================================
// Core Types
// =============================================================================

/// A hash digest produced by a [`HashFunction`].
///
/// # Security
///
/// Unlike key material, hash digests are not secret and do not require zeroization.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct HashDigest(Vec<u8>);

impl HashDigest {
    /// Creates a new `HashDigest` from raw bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the digest as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the length of the digest in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the digest is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consumes the `HashDigest` and returns the inner byte vector.
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl std::fmt::Display for HashDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// A key pair containing a public and private key.
///
/// The generic parameters allow this to work with any algorithm that implements
/// the corresponding key traits.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyPair<Public, Private>
where
    Public: PublicKey,
    Private: PrivateKey,
{
    pub public_key: Public,
    pub private_key: Private,
}

impl<Public, Private> KeyPair<Public, Private>
where
    Public: PublicKey,
    Private: PrivateKey,
{
    /// Creates a new `KeyPair`.
    pub fn new(public_key: Public, private_key: Private) -> Self {
        Self {
            public_key,
            private_key,
        }
    }

    /// Returns a reference to the public key.
    pub fn public_key(&self) -> &Public {
        &self.public_key
    }

    /// Returns a reference to the private key.
    pub fn private_key(&self) -> &Private {
        &self.private_key
    }

    /// Consumes the `KeyPair` and returns the inner components.
    pub fn into_inner(self) -> (Public, Private) {
        (self.public_key, self.private_key)
    }
}

// =============================================================================
// Key Traits
// =============================================================================

/// A public key.
///
/// # Security
///
/// Public keys are not secret and do not require zeroization.
pub trait PublicKey: Send + Sync + std::fmt::Debug {
    /// Returns the public key bytes.
    fn as_bytes(&self) -> &[u8];

    /// Returns the algorithm name (e.g., `"Ed25519"`, `"ECDSA"`).
    fn algorithm_name(&self) -> &str;
}

/// A private key.
///
/// # Security
///
/// Private keys are highly sensitive. Implementations MUST ensure key material is
/// zeroized when no longer needed. Implementations should derive `ZeroizeOnDrop`
/// or manually implement `Drop` to clear memory.
pub trait PrivateKey: Send + Sync + std::fmt::Debug {
    /// Returns the private key bytes.
    fn as_bytes(&self) -> &[u8];

    /// Returns the algorithm name (e.g., `"Ed25519"`, `"ECDSA"`).
    fn algorithm_name(&self) -> &str;
}

/// A digital signature.
///
/// # Security
///
/// Signatures are not secret and do not require zeroization.
pub trait Signature: Send + Sync + std::fmt::Debug {
    /// Returns the signature bytes.
    fn as_bytes(&self) -> &[u8];

    /// Returns the algorithm name (e.g., `"Ed25519"`, `"ECDSA"`).
    fn algorithm_name(&self) -> &str;
}

// =============================================================================
// Algorithm Traits
// =============================================================================

/// A hash function.
///
/// Implementations must be deterministic: the same input always produces the same output.
pub trait HashFunction: Send + Sync + std::fmt::Debug {
    /// Computes the hash of the given data.
    fn hash(&self, data: &[u8]) -> HashDigest;

    /// Computes the hash and returns the raw bytes.
    fn hash_bytes(&self, data: &[u8]) -> Vec<u8> {
        self.hash(data).into_inner()
    }

    /// Computes the hash by reading from a stream.
    ///
    /// This is useful for hashing large files without loading them entirely into memory.
    fn hash_stream(&self, reader: &mut dyn Read) -> CryptoResult<HashDigest>;

    /// Returns the size of the hash output in bytes.
    fn digest_size(&self) -> usize;

    /// Returns the algorithm name (e.g., `"SHA-256"`, `"BLAKE3"`).
    fn algorithm_name(&self) -> &str;
}

/// A signature algorithm.
///
/// Implementations must be constant-time where possible, especially for verification.
pub trait SignatureAlgorithm: Send + Sync + std::fmt::Debug {
    /// The public key type for this algorithm.
    type PublicKey: PublicKey;

    /// The private key type for this algorithm.
    type PrivateKey: PrivateKey;

    /// The signature type for this algorithm.
    type Signature: Signature;

    /// Generates a new random key pair.
    fn generate_keypair(&self) -> CryptoResult<KeyPair<Self::PublicKey, Self::PrivateKey>>;

    /// Signs a message with the given private key.
    fn sign(&self, private_key: &Self::PrivateKey, message: &[u8])
        -> CryptoResult<Self::Signature>;

    /// Verifies a signature against a message and public key.
    fn verify(
        &self,
        public_key: &Self::PublicKey,
        message: &[u8],
        signature: &Self::Signature,
    ) -> CryptoResult<()>;

    /// Returns the algorithm name (e.g., `"Ed25519"`, `"ECDSA"`).
    fn algorithm_name(&self) -> &str;
}

/// A cryptographically secure random number generator.
///
/// Implementations must use OS-level randomness and never be predictable.
pub trait RandomGenerator: Send + Sync + std::fmt::Debug {
    /// Fills the destination buffer with random bytes.
    ///
    /// # Panics
    ///
    /// This method must not panic. If filling fails, it should return an error.
    fn fill(&self, dest: &mut [u8]) -> CryptoResult<()>;

    /// Attempts to fill the destination buffer with random bytes.
    ///
    /// Unlike `fill`, this method may fail without panicking if the randomness source
    /// is temporarily unavailable.
    fn try_fill(&self, dest: &mut [u8]) -> CryptoResult<()>;

    /// Returns the algorithm name (e.g., `"OSRng"`, `"ChaCha20"`).
    fn algorithm_name(&self) -> &str;
}

// =============================================================================
// Encoding Traits
// =============================================================================

/// An encoder for cryptographic material.
pub trait Encoder: Send + Sync + std::fmt::Debug {
    /// Encodes raw bytes to a string.
    fn encode(&self, data: &[u8]) -> CryptoResult<String>;

    /// Encodes a public key to a string.
    fn encode_public_key(&self, public_key: &dyn PublicKey) -> CryptoResult<String> {
        self.encode(public_key.as_bytes())
    }

    /// Encodes a private key to a string.
    fn encode_private_key(&self, private_key: &dyn PrivateKey) -> CryptoResult<String> {
        self.encode(private_key.as_bytes())
    }

    /// Encodes a signature to a string.
    fn encode_signature(&self, signature: &dyn Signature) -> CryptoResult<String> {
        self.encode(signature.as_bytes())
    }

    /// Encodes a hash digest to a string.
    fn encode_hash(&self, hash: &HashDigest) -> CryptoResult<String> {
        self.encode(hash.as_bytes())
    }
}

/// A decoder for cryptographic material.
pub trait Decoder: Send + Sync + std::fmt::Debug {
    /// Decodes a string to raw bytes.
    fn decode(&self, data: &str) -> CryptoResult<Vec<u8>>;

    /// Decodes a string to a hash digest.
    fn decode_hash(&self, data: &str) -> CryptoResult<HashDigest> {
        Ok(HashDigest::new(self.decode(data)?))
    }

    /// Decodes a string to a public key.
    ///
    /// The returned key is a generic wrapper. Callers should use algorithm-specific
    /// decoders when the key type is known in advance.
    fn decode_public_key(&self, data: &str) -> CryptoResult<Box<dyn PublicKey>>;

    /// Decodes a string to a private key.
    ///
    /// The returned key is a generic wrapper. Callers should use algorithm-specific
    /// decoders when the key type is known in advance.
    fn decode_private_key(&self, data: &str) -> CryptoResult<Box<dyn PrivateKey>>;

    /// Decodes a string to a signature.
    ///
    /// The returned signature is a generic wrapper. Callers should use algorithm-specific
    /// decoders when the signature type is known in advance.
    fn decode_signature(&self, data: &str) -> CryptoResult<Box<dyn Signature>>;
}
