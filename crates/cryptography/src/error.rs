use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("verification failed: {0}")]
    VerificationFailed(String),

    #[error("key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("hash error: {0}")]
    HashError(String),

    #[error("kem error: {0}")]
    KemError(String),

    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("random generation failed: {0}")]
    RandomGenerationFailed(String),
}

pub type CryptoResult<T> = Result<T, CryptoError>;
