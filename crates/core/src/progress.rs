//! Progress reporting for the conversion pipeline.

use serde::{Deserialize, Serialize};

/// Stages of the conversion pipeline (spec §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    Detect,
    Validate,
    Transform,
    Write,
    Complete,
}

/// Progress event emitted to the UI during a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub job_id: String,
    pub stage: PipelineStage,
    /// Overall job progress in `0.0..=1.0`.
    pub fraction: f64,
    pub message: Option<String>,
}

impl Progress {
    pub fn new(
        job_id: impl Into<String>,
        stage: PipelineStage,
        fraction: f64,
        message: impl Into<Option<String>>,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            stage,
            fraction: fraction.clamp(0.0, 1.0),
            message: message.into(),
        }
    }
}
