//! Data format converters — stub until M3.

use fileprism_core::{Converter, Error, Job, Output};

/// Placeholder data converter.
pub struct DataConverter;

impl Converter for DataConverter {
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
