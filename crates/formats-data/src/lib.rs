//! Data format converters (M3).
//!
//! Supports JSON↔CSV, JSON↔YAML, and CSV↔XLSX. `JobOptions` fields
//! (quality / width / height / strip_metadata) are ignored.

mod convert;
mod formats;

pub use convert::DataConverter;
pub use formats::{normalize_format, supports_pair, DataFormat};

use fileprism_core::{Converter, Error, Job, Output};

impl Converter for DataConverter {
    fn supports(&self, input: &str, output: &str) -> bool {
        DataConverter::supports_pair(input, output)
    }

    fn convert(&self, job: &Job) -> Result<Output, Error> {
        DataConverter::convert_job(job)
    }
}
