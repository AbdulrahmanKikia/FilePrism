//! Document format identifiers.

/// Normalized document format labels used by [`crate::DocConverter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFormat {
    Pdf,
    Markdown,
    Html,
    Text,
    Png,
    Jpeg,
}

impl DocFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            DocFormat::Pdf => "pdf",
            DocFormat::Markdown => "md",
            DocFormat::Html => "html",
            DocFormat::Text => "txt",
            DocFormat::Png => "png",
            DocFormat::Jpeg => "jpg",
        }
    }
}

/// Parse a format string (`"PDF"` / `"markdown"` / `"jpeg"` …) into [`DocFormat`].
pub fn normalize_format(raw: &str) -> Option<DocFormat> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "pdf" => Some(DocFormat::Pdf),
        "md" | "markdown" => Some(DocFormat::Markdown),
        "html" | "htm" => Some(DocFormat::Html),
        "txt" | "text" => Some(DocFormat::Text),
        "png" => Some(DocFormat::Png),
        "jpg" | "jpeg" => Some(DocFormat::Jpeg),
        _ => None,
    }
}

/// Supported conversion pairs for M3 docs.
pub fn supports_pair(input: DocFormat, output: DocFormat) -> bool {
    matches!(
        (input, output),
        (DocFormat::Markdown, DocFormat::Html)
            | (DocFormat::Markdown, DocFormat::Pdf)
            | (DocFormat::Pdf, DocFormat::Text)
            | (DocFormat::Pdf, DocFormat::Png)
            | (DocFormat::Pdf, DocFormat::Jpeg)
    )
}
