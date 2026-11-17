//! Resize / quality helpers for image jobs.

use fileprism_core::JobOptions;

const DEFAULT_QUALITY: u8 = 85;

/// Clamp quality to `1..=100`, defaulting to 85.
pub fn effective_quality(options: &JobOptions) -> u8 {
    options
        .quality
        .unwrap_or(DEFAULT_QUALITY)
        .clamp(1, 100)
}

/// Strip metadata unless explicitly disabled.
pub fn effective_strip_metadata(options: &JobOptions) -> bool {
    options.strip_metadata.unwrap_or(true)
}

/// Compute target `(width, height)` preserving aspect when only one side is set.
/// Returns `None` when neither dimension is requested.
pub fn resize_dimensions(
    src_w: u32,
    src_h: u32,
    options: &JobOptions,
) -> Option<(u32, u32)> {
    let tw = options.width.filter(|w| *w > 0);
    let th = options.height.filter(|h| *h > 0);
    match (tw, th) {
        (None, None) => None,
        (Some(w), Some(h)) => Some((w, h)),
        (Some(w), None) => {
            let h = ((src_h as u64 * w as u64) / src_w.max(1) as u64).max(1) as u32;
            Some((w, h))
        }
        (None, Some(h)) => {
            let w = ((src_w as u64 * h as u64) / src_h.max(1) as u64).max(1) as u32;
            Some((w, h))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fileprism_core::JobOptions;

    #[test]
    fn resize_preserves_aspect_from_width() {
        let opts = JobOptions {
            width: Some(50),
            ..Default::default()
        };
        assert_eq!(resize_dimensions(100, 80, &opts), Some((50, 40)));
    }

    #[test]
    fn quality_clamped() {
        let opts = JobOptions {
            quality: Some(0),
            ..Default::default()
        };
        assert_eq!(effective_quality(&opts), 1);
        let opts = JobOptions {
            quality: Some(200),
            ..Default::default()
        };
        assert_eq!(effective_quality(&opts), 100);
    }
}
