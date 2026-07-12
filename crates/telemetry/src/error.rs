use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("metrics error: {0}")]
    Metrics(String),

    #[error("export error: {0}")]
    Export(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type TelemetryResult<T> = Result<T, TelemetryError>;
