//! Download + bind a Pdfium shared library matching `pdfium-render`'s
//! `pdfium_latest` (chromium/8125). Avoids `pdfium-bundled`'s 8066 skew.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use fileprism_core::Error;
use pdfium_render::prelude::*;

/// Must stay in sync with `pdfium-render` feature `pdfium_latest`.
const PDFIUM_VERSION: &str = "8125";

static PDFIUM: OnceLock<Result<Pdfium, String>> = OnceLock::new();

/// Bind Pdfium once per process (download + cache on first call).
pub fn pdfium() -> Result<&'static Pdfium, Error> {
    let result = PDFIUM.get_or_init(|| bind_pdfium().map_err(|e| e.to_string()));
    match result {
        Ok(p) => Ok(p),
        Err(msg) => Err(Error::Other(format!("pdfium bind: {msg}"))),
    }
}

fn bind_pdfium() -> Result<Pdfium, Error> {
    if let Ok(path) = std::env::var("PDFIUM_LIB_PATH") {
        return bind_at_path(Path::new(&path));
    }

    let lib_path = ensure_cached_library()?;
    let dir = lib_path
        .parent()
        .ok_or_else(|| Error::Other("pdfium cache path has no parent".into()))?;
    bind_at_path(dir)
}

fn bind_at_path(dir: &Path) -> Result<Pdfium, Error> {
    let name = Pdfium::pdfium_platform_library_name_at_path(dir);
    let bindings = Pdfium::bind_to_library(name)
        .map_err(|e| Error::Other(format!("pdfium LoadLibrary {}: {e}", dir.display())))?;
    Ok(Pdfium::new(bindings))
}

fn ensure_cached_library() -> Result<PathBuf, Error> {
    let cache_dir = cache_root()?.join(format!("pdfium-{PDFIUM_VERSION}"));
    let lib_path = cache_dir.join(platform_lib_filename());
    if lib_path.is_file() {
        return Ok(lib_path);
    }

    fs::create_dir_all(&cache_dir)
        .map_err(|e| Error::Io(format!("pdfium cache dir: {e}")))?;

    let url = download_url()?;
    let archive_path = cache_dir.join("pdfium.tgz");
    download_file(&url, &archive_path)?;
    extract_library(&archive_path, &cache_dir)?;
    let _ = fs::remove_file(&archive_path);

    if !lib_path.is_file() {
        return Err(Error::Other(format!(
            "pdfium extract missing {}",
            lib_path.display()
        )));
    }
    Ok(lib_path)
}

fn cache_root() -> Result<PathBuf, Error> {
    if let Ok(dir) = std::env::var("FILEPRISM_PDFIUM_CACHE") {
        return Ok(PathBuf::from(dir));
    }
    let base = dirs_next_cache().ok_or_else(|| {
        Error::Other("cannot resolve cache directory for pdfium".into())
    })?;
    Ok(base.join("fileprism").join("pdfium"))
}

fn dirs_next_cache() -> Option<PathBuf> {
    // Prefer LOCALAPPDATA / XDG without pulling another dep.
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        return Some(PathBuf::from(local));
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Some(PathBuf::from(xdg));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".cache"))
}

fn platform_lib_filename() -> &'static str {
    if cfg!(windows) {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    }
}

fn download_url() -> Result<String, Error> {
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(all(windows, target_arch = "x86")) {
        "x86"
    } else {
        return Err(Error::Other(format!(
            "unsupported arch for pdfium: {}",
            std::env::consts::ARCH
        )));
    };

    let os = if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err(Error::Other(format!(
            "unsupported OS for pdfium: {}",
            std::env::consts::OS
        )));
    };

    Ok(format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F{PDFIUM_VERSION}/pdfium-{os}-{arch}.tgz"
    ))
}

fn download_file(url: &str, dest: &Path) -> Result<(), Error> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| Error::Other(format!("pdfium download {url}: {e}")))?;
    let mut reader = response.into_body().into_reader();
    let mut file = File::create(dest).map_err(|e| Error::Io(e.to_string()))?;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| Error::Io(format!("pdfium download read: {e}")))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| Error::Io(format!("pdfium download write: {e}")))?;
    }
    Ok(())
}

fn extract_library(archive: &Path, dest_dir: &Path) -> Result<(), Error> {
    let file = File::open(archive).map_err(|e| Error::Io(e.to_string()))?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    let want = platform_lib_filename();

    for entry in archive
        .entries()
        .map_err(|e| Error::Other(format!("pdfium tar: {e}")))?
    {
        let mut entry = entry.map_err(|e| Error::Other(format!("pdfium tar entry: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| Error::Other(format!("pdfium tar path: {e}")))?
            .to_path_buf();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name == want {
            let out = dest_dir.join(want);
            entry
                .unpack(&out)
                .map_err(|e| Error::Other(format!("pdfium unpack: {e}")))?;
            return Ok(());
        }
    }

    Err(Error::Other(format!(
        "pdfium archive missing {want}"
    )))
}
