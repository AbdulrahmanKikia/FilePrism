//! RAR extract via separate `fileprism-unrar-sidecar` process (GPL isolation).

use std::path::{Path, PathBuf};
use std::process::Command;

use fileprism_core::Error;

use crate::util::total_dir_bytes;

const SIDECAR_ENV: &str = "FILEPRISM_UNRAR_SIDECAR";
const SIDECAR_NAME: &str = if cfg!(windows) {
    "fileprism-unrar-sidecar.exe"
} else {
    "fileprism-unrar-sidecar"
};

/// Resolve sidecar binary: `FILEPRISM_UNRAR_SIDECAR`, then next to current exe, then PATH.
pub fn resolve_sidecar() -> Result<PathBuf, Error> {
    if let Ok(path) = std::env::var(SIDECAR_ENV) {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(Error::Other(format!(
            "RAR sidecar not found at {SIDECAR_ENV}={}",
            p.display()
        )));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(SIDECAR_NAME);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    // Fall back to PATH lookup via Command (no path validation until spawn).
    Ok(PathBuf::from(SIDECAR_NAME.trim_end_matches(".exe")))
}

pub fn extract_rar(input: &Path, output: &Path, password: Option<&str>) -> Result<u64, Error> {
    if !input.is_file() {
        return Err(Error::InputNotFound(input.display().to_string()));
    }
    std::fs::create_dir_all(output).map_err(|e| {
        Error::Io(format!("create extract dir {}: {e}", output.display()))
    })?;

    let sidecar = resolve_sidecar()?;
    let mut cmd = Command::new(&sidecar);
    cmd.arg("extract")
        .arg(input)
        .arg(output)
        .env("RUST_BACKTRACE", "0");

    if let Some(pw) = password {
        cmd.arg("--password").arg(pw);
    }

    let output_proc = cmd.output().map_err(|e| {
        Error::Other(format!(
            "failed to spawn RAR sidecar '{}': {e}. Set {SIDECAR_ENV} to the fileprism-unrar-sidecar binary.",
            sidecar.display()
        ))
    })?;

    if !output_proc.status.success() {
        let stderr = String::from_utf8_lossy(&output_proc.stderr);
        let stdout = String::from_utf8_lossy(&output_proc.stdout);
        let detail = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        return Err(Error::Other(format!(
            "RAR sidecar failed (exit {:?}): {detail}",
            output_proc.status.code()
        )));
    }

    total_dir_bytes(output)
}
