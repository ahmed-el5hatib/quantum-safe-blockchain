use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinerError {
    #[error("mining not started")]
    NotStarted,

    #[error("mining already started")]
    AlreadyStarted,

    #[error("block production failed: {0}")]
    BlockProductionFailed(String),

    #[error("no transactions available")]
    NoTransactions,
}

pub type MinerResult<T> = Result<T, MinerError>;
