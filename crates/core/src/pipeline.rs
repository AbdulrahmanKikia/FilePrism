//! Conversion pipeline: detect → validate → transform → write.

use std::path::Path;

use crate::cancel::CancellationToken;
use crate::converter::Converter;
use crate::progress::{PipelineStage, Progress};
use crate::types::{Error, Job, Output};

/// Runs conversion jobs through registered [`Converter`]s with progress and
/// cooperative cancellation.
#[derive(Default)]
pub struct Pipeline {
    converters: Vec<Box<dyn Converter>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a format converter. First matching converter wins.
    pub fn register(&mut self, converter: impl Converter + 'static) {
        self.converters.push(Box::new(converter));
    }

    /// Number of registered converters.
    pub fn converter_count(&self) -> usize {
        self.converters.len()
    }

    /// Find the first converter that supports `input` → `output`.
    pub fn find_converter(&self, input: &str, output: &str) -> Option<&dyn Converter> {
        self.converters
            .iter()
            .map(|c| c.as_ref() as &dyn Converter)
            .find(|c| c.supports(input, output))
    }

    /// Execute `job` through detect → validate → transform → write.
    ///
    /// `on_progress` is invoked at each stage boundary (and may be called more
    /// than once per stage). Cancellation is checked between stages.
    pub fn run<F>(
        &self,
        job: &Job,
        cancel: &CancellationToken,
        mut on_progress: F,
    ) -> Result<Output, Error>
    where
        F: FnMut(Progress),
    {
        let emit = |stage: PipelineStage, fraction: f64, message: &str, on_progress: &mut F| {
            on_progress(Progress::new(
                job.id.clone(),
                stage,
                fraction,
                Some(message.to_string()),
            ));
        };

        // --- Detect ---
        cancel.check()?;
        emit(
            PipelineStage::Detect,
            0.05,
            "detecting input",
            &mut on_progress,
        );
        detect_input(job)?;

        // --- Validate ---
        cancel.check()?;
        emit(
            PipelineStage::Validate,
            0.2,
            "validating conversion",
            &mut on_progress,
        );
        let converter = self
            .find_converter(&job.input_format, &job.output_format)
            .ok_or_else(|| {
                Error::Unsupported(job.input_format.clone(), job.output_format.clone())
            })?;
        validate_paths(job)?;

        // --- Transform ---
        cancel.check()?;
        emit(
            PipelineStage::Transform,
            0.4,
            "transforming",
            &mut on_progress,
        );
        let output = converter.convert(job)?;

        // --- Write (verify converter produced output) ---
        cancel.check()?;
        emit(
            PipelineStage::Write,
            0.85,
            "verifying output",
            &mut on_progress,
        );
        verify_output(&output)?;

        emit(
            PipelineStage::Complete,
            1.0,
            "complete",
            &mut on_progress,
        );
        Ok(output)
    }

    /// Convenience: run without a progress callback.
    pub fn run_silent(
        &self,
        job: &Job,
        cancel: &CancellationToken,
    ) -> Result<Output, Error> {
        self.run(job, cancel, |_| {})
    }
}

fn detect_input(job: &Job) -> Result<(), Error> {
    let path = Path::new(&job.input_path);
    if !path.exists() {
        return Err(Error::InputNotFound(job.input_path.clone()));
    }
    if !path.is_file() {
        return Err(Error::Validation(format!(
            "input is not a file: {}",
            job.input_path
        )));
    }
    if job.input_format.trim().is_empty() || job.output_format.trim().is_empty() {
        return Err(Error::Validation(
            "input_format and output_format must be non-empty".into(),
        ));
    }
    Ok(())
}

fn validate_paths(job: &Job) -> Result<(), Error> {
    if job.output_path.trim().is_empty() {
        return Err(Error::Validation("output_path must be non-empty".into()));
    }
    if let Some(parent) = Path::new(&job.output_path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Io(format!(
                    "failed to create output directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn verify_output(output: &Output) -> Result<(), Error> {
    let path = Path::new(&output.output_path);
    if !path.is_file() {
        return Err(Error::Io(format!(
            "converter did not write output file: {}",
            output.output_path
        )));
    }
    let meta = std::fs::metadata(path)?;
    if meta.len() == 0 && output.bytes_written > 0 {
        return Err(Error::Io(format!(
            "output file is empty but converter reported {} bytes",
            output.bytes_written
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    /// Test converter that copies the input file to the output path.
    struct CopyConverter {
        from: String,
        to: String,
        delay: Option<Duration>,
    }

    impl Converter for CopyConverter {
        fn supports(&self, input: &str, output: &str) -> bool {
            input == self.from && output == self.to
        }

        fn convert(&self, job: &Job) -> Result<Output, Error> {
            if let Some(delay) = self.delay {
                thread::sleep(delay);
            }
            std::fs::copy(&job.input_path, &job.output_path)?;
            let bytes = std::fs::metadata(&job.output_path)?.len();
            Ok(Output {
                job_id: job.id.clone(),
                output_path: job.output_path.clone(),
                bytes_written: bytes,
            })
        }
    }

    /// Converter that always fails.
    struct FailingConverter;

    impl Converter for FailingConverter {
        fn supports(&self, input: &str, output: &str) -> bool {
            input == "bad" && output == "worse"
        }

        fn convert(&self, _job: &Job) -> Result<Output, Error> {
            Err(Error::Other("boom".into()))
        }
    }

    fn temp_paths(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("fileprism-pipeline-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("in.txt");
        let output = dir.join("out.txt");
        (input, output)
    }

    #[test]
    fn pipeline_runs_happy_path_with_progress() {
        let (input, output) = temp_paths("happy");
        std::fs::write(&input, b"hello fileprism").unwrap();

        let mut pipeline = Pipeline::new();
        pipeline.register(CopyConverter {
            from: "txt".into(),
            to: "txt".into(),
            delay: None,
        });

        let job = Job {
            id: "job-1".into(),
            input_path: input.to_string_lossy().into(),
            output_path: output.to_string_lossy().into(),
            input_format: "txt".into(),
            output_format: "txt".into(),
        };

        let events: Arc<Mutex<Vec<Progress>>> = Arc::new(Mutex::new(Vec::new()));
        let events_cb = Arc::clone(&events);
        let cancel = CancellationToken::new();

        let result = pipeline
            .run(&job, &cancel, |p| {
                events_cb.lock().unwrap().push(p);
            })
            .unwrap();

        assert_eq!(result.job_id, "job-1");
        assert_eq!(result.bytes_written, 15);
        assert_eq!(std::fs::read_to_string(&output).unwrap(), "hello fileprism");

        let stages: Vec<_> = events
            .lock()
            .unwrap()
            .iter()
            .map(|p| p.stage)
            .collect();
        assert!(stages.contains(&PipelineStage::Detect));
        assert!(stages.contains(&PipelineStage::Validate));
        assert!(stages.contains(&PipelineStage::Transform));
        assert!(stages.contains(&PipelineStage::Write));
        assert_eq!(*stages.last().unwrap(), PipelineStage::Complete);
        assert!((events.lock().unwrap().last().unwrap().fraction - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pipeline_rejects_missing_input() {
        let mut pipeline = Pipeline::new();
        pipeline.register(CopyConverter {
            from: "txt".into(),
            to: "txt".into(),
            delay: None,
        });

        let job = Job {
            id: "missing".into(),
            input_path: "/nonexistent/fileprism-missing-input.txt".into(),
            output_path: "/tmp/out.txt".into(),
            input_format: "txt".into(),
            output_format: "txt".into(),
        };

        let err = pipeline
            .run_silent(&job, &CancellationToken::new())
            .unwrap_err();
        assert!(matches!(err, Error::InputNotFound(_)));
    }

    #[test]
    fn pipeline_rejects_unsupported_pair() {
        let (input, output) = temp_paths("unsupported");
        std::fs::write(&input, b"x").unwrap();

        let pipeline = Pipeline::new();
        let job = Job {
            id: "no-conv".into(),
            input_path: input.to_string_lossy().into(),
            output_path: output.to_string_lossy().into(),
            input_format: "png".into(),
            output_format: "webp".into(),
        };

        let err = pipeline
            .run_silent(&job, &CancellationToken::new())
            .unwrap_err();
        assert_eq!(err, Error::Unsupported("png".into(), "webp".into()));
    }

    #[test]
    fn pipeline_honours_pre_transform_cancellation() {
        let (input, output) = temp_paths("cancel");
        std::fs::write(&input, b"data").unwrap();

        let mut pipeline = Pipeline::new();
        pipeline.register(CopyConverter {
            from: "txt".into(),
            to: "txt".into(),
            delay: None,
        });

        let job = Job {
            id: "cancel-me".into(),
            input_path: input.to_string_lossy().into(),
            output_path: output.to_string_lossy().into(),
            input_format: "txt".into(),
            output_format: "txt".into(),
        };

        let cancel = CancellationToken::new();
        // Cancel as soon as we see Validate so Transform never runs.
        let result = pipeline.run(&job, &cancel, |p| {
            if p.stage == PipelineStage::Validate {
                cancel.cancel();
            }
        });

        assert_eq!(result.unwrap_err(), Error::Cancelled);
        assert!(!output.exists());
    }

    #[test]
    fn pipeline_surfaces_converter_errors() {
        let (input, output) = temp_paths("fail");
        std::fs::write(&input, b"x").unwrap();

        let mut pipeline = Pipeline::new();
        pipeline.register(FailingConverter);

        let job = Job {
            id: "fail".into(),
            input_path: input.to_string_lossy().into(),
            output_path: output.to_string_lossy().into(),
            input_format: "bad".into(),
            output_format: "worse".into(),
        };

        let err = pipeline
            .run_silent(&job, &CancellationToken::new())
            .unwrap_err();
        assert_eq!(err, Error::Other("boom".into()));
    }

    #[test]
    fn cancellation_token_is_shared_across_clones() {
        let token = CancellationToken::new();
        let clone = token.clone();
        assert!(!token.is_cancelled());
        clone.cancel();
        assert!(token.is_cancelled());
        assert_eq!(token.check().unwrap_err(), Error::Cancelled);
    }
}
