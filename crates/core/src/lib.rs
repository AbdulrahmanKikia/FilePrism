//! FilePrism core conversion engine.
//!
//! Provides the [`Converter`] trait, [`Job`] / [`Output`] types, a
//! [`Pipeline`] with progress callbacks and cooperative cancellation, and
//! shared error types used by format crates and the history layer.

mod cancel;
mod converter;
mod pipeline;
mod progress;
mod types;

pub use cancel::CancellationToken;
pub use converter::Converter;
pub use pipeline::Pipeline;
pub use progress::{PipelineStage, Progress};
pub use types::{Error, Job, JobOptions, Output};

/// Workspace identity check used by smoke tests / Tauri commands.
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

    #[test]
    fn job_roundtrips_json() {
        let job = Job {
            id: "j1".into(),
            input_path: "/in.png".into(),
            output_path: "/out.webp".into(),
            input_format: "png".into(),
            output_format: "webp".into(),
            options: JobOptions::default(),
        };
        let json = serde_json::to_string(&job).unwrap();
        let back: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(job, back);
    }
}
