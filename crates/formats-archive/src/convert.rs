//! [`ArchiveConverter`] implementation.

use std::path::Path;

use fileprism_core::{Error, Job, Output};

use crate::formats::{normalize_format, supports_pair, ArchiveFormat};
#[cfg(feature = "rar")]
use crate::rar_ops;
use crate::{sevenz_ops, tar_ops, zip_ops};

/// Archive create / extract converter (ZIP, TAR, 7z; RAR extract via sidecar).
#[derive(Debug, Default, Clone, Copy)]
pub struct ArchiveConverter;

impl ArchiveConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn supports_pair(input: &str, output: &str) -> bool {
        let Some(inp) = normalize_format(input) else {
            return false;
        };
        let Some(out) = normalize_format(output) else {
            return false;
        };
        supports_pair(inp, out)
    }

    pub fn convert_job(job: &Job) -> Result<Output, Error> {
        let input_fmt = normalize_format(&job.input_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        let output_fmt = normalize_format(&job.output_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        if !supports_pair(input_fmt, output_fmt) {
            return Err(Error::Unsupported(
                job.input_format.clone(),
                job.output_format.clone(),
            ));
        }

        let input_path = Path::new(&job.input_path);
        let output_path = Path::new(&job.output_path);
        let password = job.options.password.as_deref();

        validate_paths(input_fmt, output_fmt, input_path)?;

        let bytes_written = match (input_fmt, output_fmt) {
            (ArchiveFormat::File | ArchiveFormat::Dir, ArchiveFormat::Zip) => {
                zip_ops::create_zip(input_path, output_path, password)?
            }
            (ArchiveFormat::File | ArchiveFormat::Dir, ArchiveFormat::Tar) => {
                tar_ops::create_tar(input_path, output_path, password)?
            }
            (ArchiveFormat::File | ArchiveFormat::Dir, ArchiveFormat::SevenZ) => {
                sevenz_ops::create_7z(input_path, output_path, password)?
            }
            (ArchiveFormat::Zip, ArchiveFormat::Dir) => {
                zip_ops::extract_zip(input_path, output_path, password)?
            }
            (ArchiveFormat::Tar, ArchiveFormat::Dir) => {
                tar_ops::extract_tar(input_path, output_path, password)?
            }
            (ArchiveFormat::SevenZ, ArchiveFormat::Dir) => {
                sevenz_ops::extract_7z(input_path, output_path, password)?
            }
            #[cfg(feature = "rar")]
            (ArchiveFormat::Rar, ArchiveFormat::Dir) => {
                rar_ops::extract_rar(input_path, output_path, password)?
            }
            _ => {
                return Err(Error::Unsupported(
                    job.input_format.clone(),
                    job.output_format.clone(),
                ));
            }
        };

        Ok(Output {
            job_id: job.id.clone(),
            output_path: job.output_path.clone(),
            bytes_written,
        })
    }
}

fn validate_paths(
    input_fmt: ArchiveFormat,
    _output_fmt: ArchiveFormat,
    input_path: &Path,
) -> Result<(), Error> {
    match input_fmt {
        ArchiveFormat::File => {
            if !input_path.is_file() {
                return Err(Error::InputNotFound(input_path.display().to_string()));
            }
        }
        ArchiveFormat::Dir => {
            if !input_path.is_dir() {
                return Err(Error::Validation(format!(
                    "expected directory input: {}",
                    input_path.display()
                )));
            }
        }
        ArchiveFormat::Zip | ArchiveFormat::Tar | ArchiveFormat::SevenZ | ArchiveFormat::Rar => {
            if !input_path.is_file() {
                return Err(Error::InputNotFound(input_path.display().to_string()));
            }
        }
    }
    Ok(())
}
