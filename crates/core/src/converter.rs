//! Converter trait implemented by every format module.

use crate::types::{Error, Job, Output};

/// Trait implemented by every format module.
///
/// Format crates (`formats-image`, `formats-doc`, …) implement this trait.
/// The pipeline selects a converter via [`Converter::supports`] then calls
/// [`Converter::convert`].
pub trait Converter: Send + Sync {
    /// Returns `true` when this converter can handle `input` → `output`
    /// format identifiers (typically lowercase extensions, e.g. `"png"`).
    fn supports(&self, input: &str, output: &str) -> bool;

    /// Run the conversion for `job`. Implementations should write the output
    /// file and return byte counts; they may ignore cancellation (the pipeline
    /// checks between stages).
    fn convert(&self, job: &Job) -> Result<Output, Error>;
}
