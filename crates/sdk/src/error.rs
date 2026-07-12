use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("rpc error: {0}")]
    Rpc(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type SdkResult<T> = Result<T, SdkError>;
