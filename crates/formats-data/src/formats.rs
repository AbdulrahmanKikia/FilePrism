//! Data format identifiers.

/// Normalized data format labels used by [`crate::DataConverter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    Json,
    Csv,
    Yaml,
    Xlsx,
}

impl DataFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            DataFormat::Json => "json",
            DataFormat::Csv => "csv",
            DataFormat::Yaml => "yaml",
            DataFormat::Xlsx => "xlsx",
        }
    }
}

/// Parse a format string (`"JSON"` / `"yml"` / `"xlsx"` …) into [`DataFormat`].
pub fn normalize_format(raw: &str) -> Option<DataFormat> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "json" => Some(DataFormat::Json),
        "csv" => Some(DataFormat::Csv),
        "yaml" | "yml" => Some(DataFormat::Yaml),
        "xlsx" => Some(DataFormat::Xlsx),
        _ => None,
    }
}

/// Supported conversion pairs: JSON↔CSV, JSON↔YAML, CSV↔XLSX.
pub fn supports_pair(input: DataFormat, output: DataFormat) -> bool {
    matches!(
        (input, output),
        (DataFormat::Json, DataFormat::Csv)
            | (DataFormat::Csv, DataFormat::Json)
            | (DataFormat::Json, DataFormat::Yaml)
            | (DataFormat::Yaml, DataFormat::Json)
            | (DataFormat::Csv, DataFormat::Xlsx)
            | (DataFormat::Xlsx, DataFormat::Csv)
    )
}
