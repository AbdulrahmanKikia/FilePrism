//! Shared path helpers for archive ops.

use std::path::{Component, Path};

use fileprism_core::Error;
use walkdir::WalkDir;

pub fn ensure_parent_dir(path: &Path) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Io(format!("create output dir {}: {e}", parent.display()))
            })?;
        }
    }
    Ok(())
}

/// Reject zip-slip style paths that escape `root`.
pub fn path_is_safe(root: &Path, candidate: &Path) -> bool {
    for c in candidate.components() {
        if matches!(c, Component::ParentDir) {
            // Still allow if canonicalization keeps it under root; cheap reject first.
            // Fall through to prefix check after normalize attempt below.
        }
    }
    let Ok(root_canon) = root.canonicalize().or_else(|_| Ok::<_, ()>(root.to_path_buf())) else {
        return false;
    };
    // candidate may not exist yet — canonicalize parent + join file name.
    let check = if candidate.exists() {
        candidate.canonicalize().unwrap_or_else(|_| candidate.to_path_buf())
    } else if let Some(parent) = candidate.parent() {
        let parent_canon = if parent.as_os_str().is_empty() {
            root_canon.clone()
        } else if parent.exists() {
            parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf())
        } else {
            // Ensure parent under root by string-ish containment after joining.
            parent.to_path_buf()
        };
        match candidate.file_name() {
            Some(name) => parent_canon.join(name),
            None => parent_canon,
        }
    } else {
        candidate.to_path_buf()
    };

    // Normalize `..` components without requiring the path to exist.
    let mut normalized = std::path::PathBuf::new();
    for c in check.components() {
        match c {
            Component::ParentDir => {
                if !normalized.pop() {
                    return false;
                }
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    let root_norm = {
        let mut n = std::path::PathBuf::new();
        for c in root_canon.components() {
            match c {
                Component::ParentDir => {
                    let _ = n.pop();
                }
                Component::CurDir => {}
                other => n.push(other.as_os_str()),
            }
        }
        n
    };

    normalized.starts_with(&root_norm) || check.starts_with(root)
}

pub fn total_dir_bytes(dir: &Path) -> Result<u64, Error> {
    let mut total = 0u64;
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    Ok(total)
}
