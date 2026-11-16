//! Image format converters — stub until M2.

use fileprism_core::{Converter, Error, Job, Output};

/// Placeholder image converter.
pub struct ImageConverter;

impl Converter for ImageConverter {
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
