//! FilePrism core conversion engine.
//!
//! Format crates implement [`Converter`]. Pipeline, progress, and cancellation
//! land in M1.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A conversion job request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub input_format: String,
    pub output_format: String,
}

/// Result of a successful conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub job_id: String,
    pub output_path: String,
    pub bytes_written: u64,
}

/// Core conversion errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported conversion: {0} → {1}")]
    Unsupported(String, String),
    #[error("io error: {0}")]
    Io(String),
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

/// Trait implemented by every format module.
pub trait Converter {
    fn supports(&self, input: &str, output: &str) -> bool;
    fn convert(&self, job: &Job) -> Result<Output, Error>;
}

/// Workspace identity check used by M0 smoke tests.
pub fn crate_name() -> &'static str {
    "fileprism-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_is_stable() {
        assert_eq!(crate_name(), "fileprism-core");
    }
}
