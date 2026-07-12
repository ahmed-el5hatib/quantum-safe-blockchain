use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("consensus not running")]
    NotRunning,

    #[error("consensus already running")]
    AlreadyRunning,

    #[error("finalization failed: {0}")]
    FinalizationFailed(String),

    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type ConsensusResult<T> = Result<T, ConsensusError>;
