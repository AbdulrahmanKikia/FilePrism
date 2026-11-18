//! Archive format identifiers and supported pairs.

/// Logical archive / path formats handled by [`crate::ArchiveConverter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// Single file source for create.
    File,
    /// Directory source for create, or extract destination.
    Dir,
    Zip,
    Tar,
    SevenZ,
    Rar,
}

/// Normalize a format string (`7z`/`sevenz`, `folder`/`extract` → `dir`, …).
pub fn normalize_format(raw: &str) -> Option<ArchiveFormat> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "file" | "path" => Some(ArchiveFormat::File),
        "dir" | "directory" | "folder" | "extract" => Some(ArchiveFormat::Dir),
        "zip" => Some(ArchiveFormat::Zip),
        "tar" => Some(ArchiveFormat::Tar),
        "7z" | "sevenz" | "7zip" => Some(ArchiveFormat::SevenZ),
        "rar" => Some(ArchiveFormat::Rar),
        _ => None,
    }
}

/// Create: `file`/`dir` → `zip`/`tar`/`7z`. Extract: archive → `dir`.
pub fn supports_pair(input: ArchiveFormat, output: ArchiveFormat) -> bool {
    match (input, output) {
        (ArchiveFormat::File | ArchiveFormat::Dir, ArchiveFormat::Zip) => true,
        (ArchiveFormat::File | ArchiveFormat::Dir, ArchiveFormat::Tar) => true,
        (ArchiveFormat::File | ArchiveFormat::Dir, ArchiveFormat::SevenZ) => true,
        (ArchiveFormat::Zip, ArchiveFormat::Dir) => true,
        (ArchiveFormat::Tar, ArchiveFormat::Dir) => true,
        (ArchiveFormat::SevenZ, ArchiveFormat::Dir) => true,
        #[cfg(feature = "rar")]
        (ArchiveFormat::Rar, ArchiveFormat::Dir) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_aliases() {
        assert_eq!(normalize_format("ZIP"), Some(ArchiveFormat::Zip));
        assert_eq!(normalize_format("7zip"), Some(ArchiveFormat::SevenZ));
        assert_eq!(normalize_format("folder"), Some(ArchiveFormat::Dir));
        assert_eq!(normalize_format("extract"), Some(ArchiveFormat::Dir));
        assert!(normalize_format("docx").is_none());
    }

    #[test]
    fn pairs() {
        assert!(supports_pair(ArchiveFormat::Dir, ArchiveFormat::Zip));
        #[cfg(feature = "rar")]
        assert!(supports_pair(ArchiveFormat::Rar, ArchiveFormat::Dir));
        assert!(!supports_pair(ArchiveFormat::Rar, ArchiveFormat::Zip));
        assert!(!supports_pair(ArchiveFormat::Dir, ArchiveFormat::Rar));
    }
}
