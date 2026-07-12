use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("transport error: {0}")]
    Transport(String),

    #[error("peer error: {0}")]
    PeerError(String),

    #[error("discovery error: {0}")]
    DiscoveryError(String),

    #[error("codec error: {0}")]
    CodecError(String),

    #[error("sync error: {0}")]
    SyncError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type NetworkResult<T> = Result<T, NetworkError>;
