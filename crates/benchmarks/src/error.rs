use thiserror::Error;

#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("benchmark failed: {0}")]
    Failed(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type BenchmarkResult<T> = Result<T, BenchmarkError>;
