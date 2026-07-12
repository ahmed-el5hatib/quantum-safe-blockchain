use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("key not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("backend error: {0}")]
    Backend(String),

    #[error("corruption: {0}")]
    Corruption(String),
}

pub type StorageResult<T> = Result<T, StorageError>;
