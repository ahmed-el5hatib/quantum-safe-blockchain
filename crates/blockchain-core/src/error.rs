use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("invalid header: {0}")]
    InvalidHeader(String),

    #[error("chain validation failed: {0}")]
    ChainValidation(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
