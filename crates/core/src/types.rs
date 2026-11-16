//! Shared job / output types and core errors.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A conversion job request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub input_format: String,
    pub output_format: String,
}

/// Result of a successful conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub job_id: String,
    pub output_path: String,
    pub bytes_written: u64,
}

/// Core conversion and pipeline errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("unsupported conversion: {0} → {1}")]
    Unsupported(String, String),
    #[error("input not found: {0}")]
    InputNotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.to_string())
    }
}
