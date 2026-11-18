//! Document format converters (M3).
//!
//! - PDF → PNG/JPG (text-raster via `pdf-extract` + `ab_glyph`)
//! - PDF → text (`pdf-extract`)
//! - Markdown → HTML (`pulldown-cmark`)
//! - Markdown → PDF (`printpdf` plain-text spike — see `convert` module docs)

mod convert;
mod formats;

pub use convert::DocConverter;
pub use formats::{normalize_format, DocFormat};

use fileprism_core::{Converter, Error, Job, Output};

impl Converter for DocConverter {
    fn supports(&self, input: &str, output: &str) -> bool {
        DocConverter::supports_pair(input, output)
    }

    fn convert(&self, job: &Job) -> Result<Output, Error> {
        DocConverter::convert_job(job)
    }
}
