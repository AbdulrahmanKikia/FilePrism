//! Archive create/extract (M4).
//!
//! - Create: ZIP, TAR, 7z from a file or directory
//! - Extract: ZIP, TAR, 7z, RAR → directory
//! - RAR extraction runs in a **separate sidecar process** (`fileprism-unrar-sidecar`)
//!   so UnRAR/GPL code is never linked into the main binary.
//! - Password: ZIP (AES-256), 7z (AES), RAR (via sidecar). TAR rejects passwords.

mod convert;
mod formats;
#[cfg(feature = "rar")]
mod rar_ops;
mod sevenz_ops;
mod tar_ops;
mod util;
mod zip_ops;

pub use convert::ArchiveConverter;
pub use formats::{normalize_format, supports_pair, ArchiveFormat};
#[cfg(feature = "rar")]
pub use rar_ops::resolve_sidecar;

use fileprism_core::{Converter, Error, Job, Output};

impl Converter for ArchiveConverter {
    fn supports(&self, input: &str, output: &str) -> bool {
        ArchiveConverter::supports_pair(input, output)
    }

    fn convert(&self, job: &Job) -> Result<Output, Error> {
        ArchiveConverter::convert_job(job)
    }
}
