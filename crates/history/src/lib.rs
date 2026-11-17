//! Local SQLite history, presets, and settings.
//!
//! Schema (M1): `jobs`, `files`, `presets`, `settings` plus `schema_migrations`.
//! See `migrations/001_initial.sql` and `docs/CONTRACTS.md`.

mod error;
mod migrate;
mod models;
mod store;

pub use error::{HistoryError, Result};
pub use migrate::{current_version, migrate};
pub use models::{keys, FileRecord, FileRole, JobRecord, JobStatus, Preset, Setting};
pub use store::HistoryStore;

/// Store identity used by smoke tests.
pub fn store_name() -> &'static str {
    "fileprism-history"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_name_is_stable() {
        assert_eq!(store_name(), "fileprism-history");
    }
}
