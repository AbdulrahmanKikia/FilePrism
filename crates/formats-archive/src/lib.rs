//! Archive create/extract — stub until M4.
//! RAR extraction must run as a separate sidecar (GPL/UnRAR).

use fileprism_core::{Converter, Error, Job, Output};

/// Placeholder archive converter.
pub struct ArchiveConverter;

impl Converter for ArchiveConverter {
    fn supports(&self, _input: &str, _output: &str) -> bool {
        false
    }

    fn convert(&self, job: &Job) -> Result<Output, Error> {
        Err(Error::Unsupported(
            job.input_format.clone(),
            job.output_format.clone(),
        ))
    }
}
