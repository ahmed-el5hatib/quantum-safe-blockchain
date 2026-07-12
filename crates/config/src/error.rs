use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
