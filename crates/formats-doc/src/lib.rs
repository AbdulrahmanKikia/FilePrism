//! Document format converters (M3).
//!
//! Supported pairs:
//! - PDF → PNG / JPEG (first page via `pdfium-bundled` + `pdfium-render`)
//! - PDF → text (`pdf-extract`)
//! - Markdown → HTML (`pulldown-cmark`)
//! - Markdown → PDF (see spike note below)
//!
//! ## MD→PDF spike decision
//! Prefer a pure-Rust path over `headless_chrome` / `wkhtmltopdf` for M3.
//! Choice: parse Markdown with `pulldown-cmark`, flatten events to plain text
//! (strip structure to readable paragraphs), then lay out with `printpdf`
//! BuiltinFont Helvetica. This avoids external browsers, system fonts, and
//! HTML→PDF engines. Fidelity is intentionally simplified (no CSS / images);
//! a richer HTML→PDF path can replace this later without changing the
//! `Converter` surface.
//!
//! ## PDF→image Pdfium
//! Uses `pdfium-render` with a runtime download of matching
//! `bblanchon/pdfium-binaries` chromium/8125 (see `pdfium_lib`). Avoids
//! `pdfium-bundled` 8066↔8125 symbol skew on Windows.

mod convert;
mod formats;
mod options;
mod pdfium_lib;

pub use convert::{write_fixture_pdf, DocConverter};
pub use formats::{normalize_format, supports_pair, DocFormat};
pub use options::{effective_quality, pdf_render_hint, PdfRenderHint};

use fileprism_core::{Converter, Error, Job, Output};

impl Converter for DocConverter {
    fn supports(&self, input: &str, output: &str) -> bool {
        DocConverter::supports_pair(input, output)
    }

    fn convert(&self, job: &Job) -> Result<Output, Error> {
        DocConverter::convert_job(job)
    }
}
