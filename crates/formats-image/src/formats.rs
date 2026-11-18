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
        match self {
            ImageFormat::Heic => false,
            ImageFormat::Avif => cfg!(feature = "avif"),
            ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Webp => true,
        }
    }

    pub fn is_input(self) -> bool {
        match self {
            ImageFormat::Heic => cfg!(feature = "heic"),
            ImageFormat::Avif => cfg!(feature = "avif"),
            ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Webp => true,
        }
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Webp => "image/webp",
            ImageFormat::Avif => "image/avif",
            ImageFormat::Heic => "image/heic",
        }
    }
}

/// Parse a format string (`"jpeg"` / `"JPG"` / `"heif"` …) into [`ImageFormat`].
///
/// Returns `None` for unknown labels **or** formats disabled by Cargo features
/// (e.g. HEIC/AVIF when building the WASM slice).
pub fn normalize_format(raw: &str) -> Option<ImageFormat> {
    let fmt = match raw.trim().to_ascii_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::Webp,
        "avif" => ImageFormat::Avif,
        "heic" | "heif" => ImageFormat::Heic,
        _ => return None,
    };
    if fmt.is_input() || fmt.is_output() {
        Some(fmt)
    } else {
        None
    }
}

/// Web MVP pairs: PNG / JPEG / WebP (any direction among the set).
pub fn web_mvp_formats() -> &'static [ImageFormat] {
    &[ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Webp]
}

/// List `input→output` pairs supported by the current feature set.
pub fn supported_pairs() -> Vec<(ImageFormat, ImageFormat)> {
    let inputs = [
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Webp,
        ImageFormat::Avif,
        ImageFormat::Heic,
    ];
    let outputs = [
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Webp,
        ImageFormat::Avif,
    ];
    let mut out = Vec::new();
    for inp in inputs {
        if !inp.is_input() {
            continue;
        }
        for outp in outputs {
            if outp.is_output() {
                out.push((inp, outp));
            }
        }
    }
    out
}
