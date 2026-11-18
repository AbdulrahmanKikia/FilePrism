//! Document format identifiers.

/// Normalized document format labels used by [`crate::DocConverter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFormat {
    Pdf,
    Png,
    Jpeg,
    Markdown,
    Html,
    Text,
}

impl DocFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            DocFormat::Pdf => "pdf",
            DocFormat::Png => "png",
            DocFormat::Jpeg => "jpg",
            DocFormat::Markdown => "md",
            DocFormat::Html => "html",
            DocFormat::Text => "txt",
        }
    }
}

/// Parse a format string (`"jpeg"` / `"markdown"` / `"TXT"` …) into [`DocFormat`].
pub fn normalize_format(raw: &str) -> Option<DocFormat> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "pdf" => Some(DocFormat::Pdf),
        "png" => Some(DocFormat::Png),
        "jpg" | "jpeg" => Some(DocFormat::Jpeg),
        "md" | "markdown" => Some(DocFormat::Markdown),
        "html" | "htm" => Some(DocFormat::Html),
        "txt" | "text" => Some(DocFormat::Text),
        _ => None,
    }
}

/// Returns `true` when `input` → `output` is a supported M3 document pair.
pub fn supports_pair(input: DocFormat, output: DocFormat) -> bool {
    matches!(
        (input, output),
        (DocFormat::Pdf, DocFormat::Png)
            | (DocFormat::Pdf, DocFormat::Jpeg)
            | (DocFormat::Pdf, DocFormat::Text)
            | (DocFormat::Markdown, DocFormat::Html)
            | (DocFormat::Markdown, DocFormat::Pdf)
    )
}
