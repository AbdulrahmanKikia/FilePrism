//! Image format identifiers.

/// Normalized image format labels used by [`crate::ImageConverter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
    Avif,
    Heic,
}

impl ImageFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Webp => "webp",
            ImageFormat::Avif => "avif",
            ImageFormat::Heic => "heic",
        }
    }

    pub fn is_output(self) -> bool {
        !matches!(self, ImageFormat::Heic)
    }

    pub fn is_input(self) -> bool {
        true
    }
}

/// Parse a format string (`"jpeg"` / `"JPG"` / `"heif"` …) into [`ImageFormat`].
pub fn normalize_format(raw: &str) -> Option<ImageFormat> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "webp" => Some(ImageFormat::Webp),
        "avif" => Some(ImageFormat::Avif),
        "heic" | "heif" => Some(ImageFormat::Heic),
        _ => None,
    }
}
