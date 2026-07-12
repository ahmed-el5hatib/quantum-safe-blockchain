use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("account not found: {0}")]
    AccountNotFound(String),

    #[error("invalid password")]
    InvalidPassword,

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type WalletResult<T> = Result<T, WalletError>;
