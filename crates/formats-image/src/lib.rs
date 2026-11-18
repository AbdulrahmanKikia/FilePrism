//! Image format converters (M2).
//!
//! Supports raster conversions among PNG, JPEG, WebP, and (with features) AVIF,
//! plus HEIC/HEIF decode. Options: quality, resize, strip metadata.
//!
//! Feature flags:
//! - `heic` / `avif` / `webp-lossy` / `parallel` — on by default for desktop
//! - WASM builds use `default-features = false` (PNG/JPEG/WebP only)

mod convert;
mod formats;
mod options;

pub use convert::ImageConverter;
pub use formats::{
    normalize_format, supported_pairs, web_mvp_formats, ImageFormat,
};
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
