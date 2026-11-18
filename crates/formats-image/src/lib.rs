//! Image format converters (M2).
//!
//! Supports raster conversions among PNG, JPEG, WebP, and AVIF, plus HEIC/HEIF
//! decode → PNG/JPEG/WebP/AVIF. Options: quality, resize, strip metadata.

mod convert;
mod formats;
mod options;

pub use convert::ImageConverter;
pub use formats::{normalize_format, ImageFormat};
pub use options::{effective_quality, effective_strip_metadata, resize_dimensions};

use fileprism_core::{Converter, Error, Job, Output};

impl Converter for ImageConverter {
    fn supports(&self, input: &str, output: &str) -> bool {
        ImageConverter::supports_pair(input, output)
    }

    fn convert(&self, job: &Job) -> Result<Output, Error> {
        ImageConverter::convert_job(job)
    }
}
