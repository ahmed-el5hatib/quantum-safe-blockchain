use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("method not found: {0}")]
    MethodNotFound(String),

    #[error("invalid params: {0}")]
    InvalidParams(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("not connected")]
    NotConnected,
}

pub type RpcResult<T> = Result<T, RpcError>;
