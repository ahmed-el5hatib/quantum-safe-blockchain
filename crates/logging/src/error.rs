use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("initialization failed: {0}")]
    InitFailed(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type LoggingResult<T> = Result<T, LoggingError>;
