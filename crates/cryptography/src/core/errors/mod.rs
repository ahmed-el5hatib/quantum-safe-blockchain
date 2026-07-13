//! Cryptography core error types.
//!
//! All cryptographic operations return `CryptoResult<T>`, which is a `Result<T, CryptoError>`.
//! This ensures that every failure mode is explicitly handled and no panics occur in library code.

use thiserror::Error;

/// Result type for all cryptographic operations.
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Comprehensive error type for cryptographic operations.
///
/// Every variant represents a distinct failure mode that callers may need to handle differently.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// The provided key is malformed, wrong size, or invalid for the algorithm.
    #[error("invalid key: {0}")]
    InvalidKey(String),

    /// The signature format or value is invalid.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// Signature verification failed.
    #[error("verification failed: {0}")]
    VerificationFailed(String),

    /// Signing operation failed.
    #[error("signing failed: {0}")]
    SigningFailed(String),

    /// Serialization of a cryptographic object failed.
    #[error("serialization failed: {0}")]
    SerializationFailed(String),

    /// Deserialization of a cryptographic object failed.
    #[error("deserialization failed: {0}")]
    DeserializationFailed(String),

    /// Encoding (e.g., hex, base64) failed.
    #[error("encoding failed: {0}")]
    EncodingFailed(String),

    /// Decoding (e.g., hex, base64) failed.
    #[error("decoding failed: {0}")]
    DecodingFailed(String),

    /// Secure random number generation failed.
    #[error("random generation failed: {0}")]
    RandomGenerationFailed(String),

    /// The requested algorithm is not supported by the current provider.
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    /// An internal cryptographic operation failed.
    #[error("internal crypto error: {0}")]
    InternalCryptoError(String),
}
