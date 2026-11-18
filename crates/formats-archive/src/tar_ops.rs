//! TAR create / extract (no native password support).

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use fileprism_core::Error;
use tar::{Archive, Builder};
use walkdir::WalkDir;

use crate::util::{ensure_parent_dir, path_is_safe, total_dir_bytes};

pub fn create_tar(input: &Path, output: &Path, password: Option<&str>) -> Result<u64, Error> {
    if password.is_some() {
        return Err(Error::Validation(
            "tar does not support password protection".into(),
        ));
    }
    ensure_parent_dir(output)?;
    let file = File::create(output).map_err(|e| Error::Io(e.to_string()))?;
    let mut builder = Builder::new(BufWriter::new(file));

    if input.is_file() {
        let name = input
            .file_name()
            .ok_or_else(|| Error::Validation("tar create: input file has no name".into()))?;
        builder
            .append_path_with_name(input, name)
            .map_err(|e| Error::Other(format!("tar append file: {e}")))?;
    } else if input.is_dir() {
        let root = input;
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path == root {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .map_err(|e| Error::Other(format!("tar strip_prefix: {e}")))?;
            if rel.as_os_str().is_empty() {
                continue;
            }
            if path.is_dir() {
                builder
                    .append_dir(rel, path)
                    .map_err(|e| Error::Other(format!("tar append dir: {e}")))?;
            } else if path.is_file() {
                builder
                    .append_path_with_name(path, rel)
                    .map_err(|e| Error::Other(format!("tar append file: {e}")))?;
            }
        }
    } else {
        return Err(Error::InputNotFound(input.display().to_string()));
    }

    builder
        .finish()
        .map_err(|e| Error::Other(format!("tar finish: {e}")))?;
    std::fs::metadata(output)
        .map(|m| m.len())
        .map_err(|e| Error::Io(e.to_string()))
}

pub fn extract_tar(input: &Path, output: &Path, password: Option<&str>) -> Result<u64, Error> {
    if password.is_some() {
        return Err(Error::Validation(
            "tar does not support password protection".into(),
        ));
    }
    if !input.is_file() {
        return Err(Error::InputNotFound(input.display().to_string()));
    }
    std::fs::create_dir_all(output).map_err(|e| {
        Error::Io(format!("create extract dir {}: {e}", output.display()))
    })?;

    let file = File::open(input).map_err(|e| Error::Io(e.to_string()))?;
    let mut archive = Archive::new(BufReader::new(file));
    let entries = archive
        .entries()
        .map_err(|e| Error::Other(format!("tar entries: {e}")))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| Error::Other(format!("tar entry: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| Error::Other(format!("tar entry path: {e}")))?
            .into_owned();
        let out_path = output.join(&path);
        if !path_is_safe(output, &out_path) {
            return Err(Error::Validation(format!(
                "tar unsafe path: {}",
                path.display()
            )));
        }
        entry
            .unpack(&out_path)
            .map_err(|e| Error::Other(format!("tar unpack {}: {e}", path.display())))?;
    }

    total_dir_bytes(output)
}
