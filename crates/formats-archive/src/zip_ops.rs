//! ZIP create / extract (optional AES-256 password).

use std::fs::File;
use std::io::{copy, BufReader, BufWriter, Write};
use std::path::Path;

use fileprism_core::Error;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{AesMode, CompressionMethod, ZipArchive, ZipWriter};

use crate::util::{ensure_parent_dir, path_is_safe, total_dir_bytes};

pub fn create_zip(input: &Path, output: &Path, password: Option<&str>) -> Result<u64, Error> {
    ensure_parent_dir(output)?;
    let file = File::create(output).map_err(|e| Error::Io(e.to_string()))?;
    let mut zip = ZipWriter::new(BufWriter::new(file));

    let options = match password {
        Some(pw) => SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .with_aes_encryption(AesMode::Aes256, pw),
        None => SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
    };

    if input.is_file() {
        let name = input
            .file_name()
            .ok_or_else(|| Error::Validation("zip create: input file has no name".into()))?
            .to_string_lossy();
        zip.start_file(name.as_ref(), options)
            .map_err(|e| Error::Other(format!("zip start_file: {e}")))?;
        let mut src = File::open(input).map_err(|e| Error::Io(e.to_string()))?;
        copy(&mut src, &mut zip).map_err(|e| Error::Io(e.to_string()))?;
    } else if input.is_dir() {
        let root = input;
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .map_err(|e| Error::Other(format!("zip strip_prefix: {e}")))?;
            let name = rel.to_string_lossy().replace('\\', "/");
            if name.is_empty() {
                continue;
            }
            zip.start_file(&name, options)
                .map_err(|e| Error::Other(format!("zip start_file {name}: {e}")))?;
            let mut src = File::open(path).map_err(|e| Error::Io(e.to_string()))?;
            copy(&mut src, &mut zip).map_err(|e| Error::Io(e.to_string()))?;
        }
    } else {
        return Err(Error::InputNotFound(input.display().to_string()));
    }

    zip.finish()
        .map_err(|e| Error::Other(format!("zip finish: {e}")))?;
    std::fs::metadata(output)
        .map(|m| m.len())
        .map_err(|e| Error::Io(e.to_string()))
}

pub fn extract_zip(input: &Path, output: &Path, password: Option<&str>) -> Result<u64, Error> {
    if !input.is_file() {
        return Err(Error::InputNotFound(input.display().to_string()));
    }
    std::fs::create_dir_all(output).map_err(|e| {
        Error::Io(format!("create extract dir {}: {e}", output.display()))
    })?;

    let file = File::open(input).map_err(|e| Error::Io(e.to_string()))?;
    let mut archive =
        ZipArchive::new(BufReader::new(file)).map_err(|e| Error::Other(format!("zip open: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = match password {
            Some(pw) => archive.by_index_decrypt(i, pw.as_bytes()).map_err(|e| {
                let msg = e.to_string().to_ascii_lowercase();
                if msg.contains("password") || msg.contains("invalid") {
                    Error::Validation("zip: invalid or missing password".into())
                } else {
                    Error::Other(format!("zip decrypt entry {i}: {e}"))
                }
            })?,
            None => archive
                .by_index(i)
                .map_err(|e| Error::Other(format!("zip entry {i}: {e}")))?,
        };

        let name = entry.name().replace('\\', "/");
        if name.ends_with('/') {
            let dir = output.join(&name);
            if !path_is_safe(output, &dir) {
                return Err(Error::Validation(format!("zip unsafe path: {name}")));
            }
            std::fs::create_dir_all(&dir).map_err(|e| Error::Io(e.to_string()))?;
            continue;
        }

        let out_path = output.join(&name);
        if !path_is_safe(output, &out_path) {
            return Err(Error::Validation(format!("zip unsafe path: {name}")));
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
        }
        let mut out = File::create(&out_path).map_err(|e| Error::Io(e.to_string()))?;
        copy(&mut entry, &mut out).map_err(|e| Error::Io(e.to_string()))?;
        out.flush().map_err(|e| Error::Io(e.to_string()))?;
    }

    total_dir_bytes(output)
}
