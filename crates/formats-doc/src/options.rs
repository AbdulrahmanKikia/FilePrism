//! DPI / quality helpers for document jobs.

use fileprism_core::JobOptions;

const DEFAULT_DPI: u32 = 150;
const DEFAULT_QUALITY: u8 = 85;
/// Values at or below this are treated as DPI; above as max pixel width.
const DPI_OR_WIDTH_THRESHOLD: u32 = 600;

/// Clamp JPEG quality to `1..=100`, defaulting to 85.
pub fn effective_quality(options: &JobOptions) -> u8 {
    options.quality.unwrap_or(DEFAULT_QUALITY).clamp(1, 100)
}

/// Interpret [`JobOptions::width`] for PDF→image rendering.
///
/// - `None` → default DPI (150)
/// - `Some(w)` with `w <= 600` → DPI hint
/// - `Some(w)` with `w > 600` → max target width in pixels (DPI left at default
///   and applied via target width in the renderer)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfRenderHint {
    Dpi(u32),
    MaxWidth(u32),
}

pub fn pdf_render_hint(options: &JobOptions) -> PdfRenderHint {
    match options.width.filter(|w| *w > 0) {
        None => PdfRenderHint::Dpi(DEFAULT_DPI),
        Some(w) if w <= DPI_OR_WIDTH_THRESHOLD => PdfRenderHint::Dpi(w.clamp(36, 600)),
        Some(w) => PdfRenderHint::MaxWidth(w),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fileprism_core::JobOptions;

    #[test]
    fn width_as_dpi_when_small() {
        let opts = JobOptions {
            width: Some(200),
            ..Default::default()
        };
        assert_eq!(pdf_render_hint(&opts), PdfRenderHint::Dpi(200));
    }

    #[test]
    fn width_as_max_pixels_when_large() {
        let opts = JobOptions {
            width: Some(1200),
            ..Default::default()
        };
        assert_eq!(pdf_render_hint(&opts), PdfRenderHint::MaxWidth(1200));
    }

    #[test]
    fn quality_clamped() {
        let opts = JobOptions {
            quality: Some(0),
            ..Default::default()
        };
        assert_eq!(effective_quality(&opts), 1);
    }
}
