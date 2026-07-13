//! Encoding implementations for cryptographic material.
//!
//! Provides `Encoder` and `Decoder` implementations for hexadecimal and base64 encoding.
//! These encodings are used for displaying keys, signatures, and hashes in human-readable
//! form and for serialization to text-based formats.

use base64::engine::general_purpose;
use base64::Engine as _;
use hex::decode as hex_decode;
use hex::encode as hex_encode;
use zeroize::Zeroize;

use crate::core::errors::CryptoError;
use crate::core::traits::{Decoder, Encoder, PrivateKey, PublicKey, Signature};
use crate::core::CryptoResult;

/// Encodes data as hexadecimal strings.
///
/// # Examples
///
/// ```
/// use cryptography::core::encoding::HexEncoder;
/// use cryptography::core::traits::Encoder;
///
/// let encoder = HexEncoder;
/// let encoded = encoder.encode(b"hello").unwrap();
/// assert_eq!(encoded, "68656c6c6f");
/// ```
#[derive(Clone, Debug, Default)]
pub struct HexEncoder;

impl HexEncoder {
    /// Creates a new `HexEncoder`.
    pub fn new() -> Self {
        Self
    }
}

impl Encoder for HexEncoder {
    fn encode(&self, data: &[u8]) -> CryptoResult<String> {
        Ok(hex_encode(data))
    }
}

/// Decodes hexadecimal strings to bytes.
///
/// # Examples
///
/// ```
/// use cryptography::core::encoding::HexDecoder;
/// use cryptography::core::traits::Decoder;
///
/// let decoder = HexDecoder;
/// let decoded = decoder.decode("68656c6c6f").unwrap();
/// assert_eq!(decoded, b"hello");
/// ```
#[derive(Clone, Debug, Default)]
pub struct HexDecoder;

impl HexDecoder {
    /// Creates a new `HexDecoder`.
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for HexDecoder {
    fn decode(&self, data: &str) -> CryptoResult<Vec<u8>> {
        hex_decode(data).map_err(|e| CryptoError::DecodingFailed(e.to_string()))
    }

    fn decode_public_key(&self, data: &str) -> CryptoResult<Box<dyn PublicKey>> {
        let bytes = self.decode(data)?;
        Ok(Box::new(GenericPublicKey::new(bytes)))
    }

    fn decode_private_key(&self, data: &str) -> CryptoResult<Box<dyn PrivateKey>> {
        let bytes = self.decode(data)?;
        Ok(Box::new(GenericPrivateKey::new(bytes)))
    }

    fn decode_signature(&self, data: &str) -> CryptoResult<Box<dyn Signature>> {
        let bytes = self.decode(data)?;
        Ok(Box::new(GenericSignature::new(bytes)))
    }
}

/// Encodes data as base64 strings.
///
/// # Examples
///
/// ```
/// use cryptography::core::encoding::Base64Encoder;
/// use cryptography::core::traits::Encoder;
///
/// let encoder = Base64Encoder;
/// let encoded = encoder.encode(b"hello").unwrap();
/// assert_eq!(encoded, "aGVsbG8=");
/// ```
#[derive(Clone, Debug, Default)]
pub struct Base64Encoder;

impl Base64Encoder {
    /// Creates a new `Base64Encoder`.
    pub fn new() -> Self {
        Self
    }
}

impl Encoder for Base64Encoder {
    fn encode(&self, data: &[u8]) -> CryptoResult<String> {
        Ok(general_purpose::STANDARD.encode(data))
    }
}

/// Decodes base64 strings to bytes.
///
/// # Examples
///
/// ```
/// use cryptography::core::encoding::Base64Decoder;
/// use cryptography::core::traits::Decoder;
///
/// let decoder = Base64Decoder;
/// let decoded = decoder.decode("aGVsbG8=").unwrap();
/// assert_eq!(decoded, b"hello");
/// ```
#[derive(Clone, Debug, Default)]
pub struct Base64Decoder;

impl Base64Decoder {
    /// Creates a new `Base64Decoder`.
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for Base64Decoder {
    fn decode(&self, data: &str) -> CryptoResult<Vec<u8>> {
        general_purpose::STANDARD
            .decode(data)
            .map_err(|e| CryptoError::DecodingFailed(e.to_string()))
    }

    fn decode_public_key(&self, data: &str) -> CryptoResult<Box<dyn PublicKey>> {
        let bytes = self.decode(data)?;
        Ok(Box::new(GenericPublicKey::new(bytes)))
    }

    fn decode_private_key(&self, data: &str) -> CryptoResult<Box<dyn PrivateKey>> {
        let bytes = self.decode(data)?;
        Ok(Box::new(GenericPrivateKey::new(bytes)))
    }

    fn decode_signature(&self, data: &str) -> CryptoResult<Box<dyn Signature>> {
        let bytes = self.decode(data)?;
        Ok(Box::new(GenericSignature::new(bytes)))
    }
}

// =============================================================================
// Generic wrappers for decoded material
// =============================================================================

/// A generic public key wrapper for decoded bytes.
///
/// This is returned by generic decoders when the algorithm is not known in advance.
/// Callers should prefer algorithm-specific decoders when possible.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct GenericPublicKey {
    algorithm: String,
    bytes: Vec<u8>,
}

impl GenericPublicKey {
    /// Creates a new generic public key.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            algorithm: "unknown".to_string(),
            bytes,
        }
    }

    /// Creates a new generic public key with an algorithm hint.
    pub fn with_algorithm(algorithm: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            algorithm: algorithm.into(),
            bytes,
        }
    }

    /// Returns the algorithm name.
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    /// Returns the key bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PublicKey for GenericPublicKey {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn algorithm_name(&self) -> &str {
        &self.algorithm
    }
}

/// A generic private key wrapper for decoded bytes.
///
/// # Security
///
/// This type clears key material from memory when dropped.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GenericPrivateKey {
    algorithm: String,
    bytes: Vec<u8>,
}

impl GenericPrivateKey {
    /// Creates a new generic private key.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            algorithm: "unknown".to_string(),
            bytes,
        }
    }

    /// Creates a new generic private key with an algorithm hint.
    pub fn with_algorithm(algorithm: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            algorithm: algorithm.into(),
            bytes,
        }
    }

    /// Returns the algorithm name.
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    /// Returns the key bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PrivateKey for GenericPrivateKey {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn algorithm_name(&self) -> &str {
        &self.algorithm
    }
}

impl Drop for GenericPrivateKey {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// A generic signature wrapper for decoded bytes.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct GenericSignature {
    algorithm: String,
    bytes: Vec<u8>,
}

impl GenericSignature {
    /// Creates a new generic signature.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            algorithm: "unknown".to_string(),
            bytes,
        }
    }

    /// Creates a new generic signature with an algorithm hint.
    pub fn with_algorithm(algorithm: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            algorithm: algorithm.into(),
            bytes,
        }
    }

    /// Returns the algorithm name.
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    /// Returns the signature bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Signature for GenericSignature {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn algorithm_name(&self) -> &str {
        &self.algorithm
    }
}
