//! 7z create / extract via pure-Rust `sevenz-rust2` (optional AES password).

use std::path::Path;

use fileprism_core::Error;
use sevenz_rust2::Password;

use crate::util::{ensure_parent_dir, total_dir_bytes};

pub fn create_7z(input: &Path, output: &Path, password: Option<&str>) -> Result<u64, Error> {
    if !input.exists() {
        return Err(Error::InputNotFound(input.display().to_string()));
    }
    ensure_parent_dir(output)?;

    match password {
        Some(pw) => sevenz_rust2::compress_to_path_encrypted(input, output, Password::from(pw))
            .map_err(|e| Error::Other(format!("7z create: {e}")))?,
        None => sevenz_rust2::compress_to_path(input, output)
            .map_err(|e| Error::Other(format!("7z create: {e}")))?,
    }

    std::fs::metadata(output)
        .map(|m| m.len())
        .map_err(|e| Error::Io(e.to_string()))
}

pub fn extract_7z(input: &Path, output: &Path, password: Option<&str>) -> Result<u64, Error> {
    if !input.is_file() {
        return Err(Error::InputNotFound(input.display().to_string()));
    }
    std::fs::create_dir_all(output).map_err(|e| {
        Error::Io(format!("create extract dir {}: {e}", output.display()))
    })?;

    match password {
        Some(pw) => {
            sevenz_rust2::decompress_file_with_password(input, output, Password::from(pw))
                .map_err(|e| Error::Other(format!("7z extract: {e}")))?
        }
        None => sevenz_rust2::decompress_file(input, output)
            .map_err(|e| Error::Other(format!("7z extract: {e}")))?,
    }

    total_dir_bytes(output)
}
