use thiserror::Error;

#[derive(Error, Debug)]
pub enum MempoolError {
    #[error("pool full")]
    PoolFull,

    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("eviction failed: {0}")]
    EvictionFailed(String),
}

pub type MempoolResult<T> = Result<T, MempoolError>;
