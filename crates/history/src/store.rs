//! SQLite-backed history, presets, and settings store.

use std::path::Path;

use fileprism_core::{Job, Output};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{HistoryError, Result};
use crate::migrate;
use crate::models::{keys, FileRecord, FileRole, JobRecord, JobStatus, Preset, Setting};

/// Local SQLite store for conversion history, presets, and settings.
pub struct HistoryStore {
    conn: Connection,
}

impl HistoryStore {
    /// Open (or create) a database at `path` and run migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    /// In-memory database (tests).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::from_connection(conn)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )?;
        migrate::migrate(&conn)?;
        Ok(Self { conn })
    }

    /// Current schema migration version.
    pub fn schema_version(&self) -> Result<i64> {
        migrate::current_version(&self.conn)
    }

    /// Returns whether history recording is enabled (default: true).
    pub fn history_enabled(&self) -> Result<bool> {
        match self.get_setting(keys::HISTORY_ENABLED)? {
            Some(v) if v.eq_ignore_ascii_case("false") || v == "0" => Ok(false),
            _ => Ok(true),
        }
    }

    // --- Jobs ---

    /// Insert a job in `pending`/`running` state and link input/output file rows.
    pub fn record_job_start(&self, job: &Job, status: JobStatus) -> Result<()> {
        if !self.history_enabled()? {
            return Ok(());
        }
        let now = utc_now();
        self.conn.execute(
            "INSERT INTO jobs (
                id, input_path, output_path, input_format, output_format,
                status, bytes_written, error_message, created_at, completed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, NULL)",
            params![
                job.id,
                job.input_path,
                job.output_path,
                job.input_format,
                job.output_format,
                status.as_str(),
                now,
            ],
        )?;
        self.conn.execute(
            "INSERT INTO files (job_id, path, role, format, size_bytes)
             VALUES (?1, ?2, 'input', ?3, NULL)",
            params![job.id, job.input_path, job.input_format],
        )?;
        self.conn.execute(
            "INSERT INTO files (job_id, path, role, format, size_bytes)
             VALUES (?1, ?2, 'output', ?3, NULL)",
            params![job.id, job.output_path, job.output_format],
        )?;
        Ok(())
    }

    /// Mark a job completed and update the output file size.
    pub fn record_job_complete(&self, output: &Output) -> Result<()> {
        if !self.history_enabled()? {
            return Ok(());
        }
        let now = utc_now();
        let updated = self.conn.execute(
            "UPDATE jobs SET status = 'completed', bytes_written = ?1, completed_at = ?2
             WHERE id = ?3",
            params![output.bytes_written as i64, now, output.job_id],
        )?;
        if updated == 0 {
            return Err(HistoryError::NotFound(output.job_id.clone()));
        }
        self.conn.execute(
            "UPDATE files SET size_bytes = ?1, path = ?2
             WHERE job_id = ?3 AND role = 'output'",
            params![output.bytes_written as i64, output.output_path, output.job_id],
        )?;
        Ok(())
    }

    /// Mark a job failed or cancelled.
    pub fn record_job_finish(
        &self,
        job_id: &str,
        status: JobStatus,
        error_message: Option<&str>,
    ) -> Result<()> {
        if !self.history_enabled()? {
            return Ok(());
        }
        if !matches!(
            status,
            JobStatus::Failed | JobStatus::Cancelled | JobStatus::Completed
        ) {
            return Err(HistoryError::Other(format!(
                "invalid terminal status: {}",
                status.as_str()
            )));
        }
        let now = utc_now();
        let updated = self.conn.execute(
            "UPDATE jobs SET status = ?1, error_message = ?2, completed_at = ?3
             WHERE id = ?4",
            params![status.as_str(), error_message, now, job_id],
        )?;
        if updated == 0 {
            return Err(HistoryError::NotFound(job_id.into()));
        }
        Ok(())
    }

    pub fn get_job(&self, id: &str) -> Result<Option<JobRecord>> {
        self.conn
            .query_row(
                "SELECT id, input_path, output_path, input_format, output_format,
                        status, bytes_written, error_message, created_at, completed_at
                 FROM jobs WHERE id = ?1",
                [id],
                map_job_row,
            )
            .optional()
            .map_err(Into::into)
    }

    /// Search jobs by filename substring, format, or status. Empty query returns recent jobs.
    pub fn search_jobs(&self, query: &str, limit: usize) -> Result<Vec<JobRecord>> {
        let limit = limit.max(1) as i64;
        let q = query.trim();
        if q.is_empty() {
            let mut stmt = self.conn.prepare(
                "SELECT id, input_path, output_path, input_format, output_format,
                        status, bytes_written, error_message, created_at, completed_at
                 FROM jobs ORDER BY created_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map([limit], map_job_row)?;
            return rows.collect::<std::result::Result<Vec<_>, _>>()
                .map_err(Into::into);
        }

        let like = format!("%{q}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, input_path, output_path, input_format, output_format,
                    status, bytes_written, error_message, created_at, completed_at
             FROM jobs
             WHERE input_path LIKE ?1 ESCAPE '\\'
                OR output_path LIKE ?1 ESCAPE '\\'
                OR input_format LIKE ?1 ESCAPE '\\'
                OR output_format LIKE ?1 ESCAPE '\\'
                OR status LIKE ?1 ESCAPE '\\'
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![like, limit], map_job_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn list_files_for_job(&self, job_id: &str) -> Result<Vec<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_id, path, role, format, size_bytes
             FROM files WHERE job_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map([job_id], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                job_id: row.get(1)?,
                path: row.get(2)?,
                role: FileRole::parse(&row.get::<_, String>(3)?)
                    .unwrap_or(FileRole::Input),
                format: row.get(4)?,
                size_bytes: row
                    .get::<_, Option<i64>>(5)?
                    .map(|v| v as u64),
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    // --- Presets ---

    pub fn upsert_preset(&self, preset: &Preset) -> Result<()> {
        self.conn.execute(
            "INSERT INTO presets (id, name, input_format, output_format, options_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                input_format = excluded.input_format,
                output_format = excluded.output_format,
                options_json = excluded.options_json,
                updated_at = excluded.updated_at",
            params![
                preset.id,
                preset.name,
                preset.input_format,
                preset.output_format,
                preset.options_json,
                preset.created_at,
                preset.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_presets(&self) -> Result<Vec<Preset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, input_format, output_format, options_json, created_at, updated_at
             FROM presets ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Preset {
                id: row.get(0)?,
                name: row.get(1)?,
                input_format: row.get(2)?,
                output_format: row.get(3)?,
                options_json: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn delete_preset(&self, id: &str) -> Result<bool> {
        let n = self
            .conn
            .execute("DELETE FROM presets WHERE id = ?1", [id])?;
        Ok(n > 0)
    }

    // --- Settings ---

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = utc_now();
        self.conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_settings(&self) -> Result<Vec<Setting>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value, updated_at FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

fn map_job_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobRecord> {
    let status_raw: String = row.get(5)?;
    Ok(JobRecord {
        id: row.get(0)?,
        input_path: row.get(1)?,
        output_path: row.get(2)?,
        input_format: row.get(3)?,
        output_format: row.get(4)?,
        status: JobStatus::parse(&status_raw).unwrap_or(JobStatus::Failed),
        bytes_written: row.get::<_, Option<i64>>(6)?.map(|v| v as u64),
        error_message: row.get(7)?,
        created_at: row.get(8)?,
        completed_at: row.get(9)?,
    })
}

fn utc_now() -> String {
    // RFC 3339-ish UTC without external chrono dependency.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_unix_utc(secs)
}

fn format_unix_utc(secs: u64) -> String {
    // Simple civil date from Unix seconds (UTC). Good enough for history stamps.
    const SECS_PER_DAY: u64 = 86_400;
    let days = secs / SECS_PER_DAY;
    let time = secs % SECS_PER_DAY;
    let hour = time / 3600;
    let min = (time % 3600) / 60;
    let sec = time % 60;

    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Algorithm from Howard Hinnant's date algorithms (days since 1970-01-01 → Y-M-D).
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job(id: &str) -> Job {
        Job {
            id: id.into(),
            input_path: format!("/tmp/{id}-in.png"),
            output_path: format!("/tmp/{id}-out.webp"),
            input_format: "png".into(),
            output_format: "webp".into(),
            options: Default::default(),
        }
    }

    #[test]
    fn records_and_searches_jobs() {
        let store = HistoryStore::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), 1);

        let job = sample_job("j1");
        store
            .record_job_start(&job, JobStatus::Running)
            .unwrap();
        store
            .record_job_complete(&Output {
                job_id: "j1".into(),
                output_path: job.output_path.clone(),
                bytes_written: 42,
            })
            .unwrap();

        let got = store.get_job("j1").unwrap().unwrap();
        assert_eq!(got.status, JobStatus::Completed);
        assert_eq!(got.bytes_written, Some(42));

        let files = store.list_files_for_job("j1").unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[1].size_bytes, Some(42));

        let hits = store.search_jobs("webp", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "j1");
    }

    #[test]
    fn privacy_mode_skips_recording() {
        let store = HistoryStore::open_in_memory().unwrap();
        store.set_setting(keys::HISTORY_ENABLED, "false").unwrap();
        assert!(!store.history_enabled().unwrap());

        store
            .record_job_start(&sample_job("hidden"), JobStatus::Running)
            .unwrap();
        assert!(store.get_job("hidden").unwrap().is_none());
    }

    #[test]
    fn presets_and_settings_roundtrip() {
        let store = HistoryStore::open_in_memory().unwrap();
        let now = "2026-11-16T00:00:00Z".to_string();
        store
            .upsert_preset(&Preset {
                id: "p1".into(),
                name: "PNG to WebP".into(),
                input_format: "png".into(),
                output_format: "webp".into(),
                options_json: r#"{"quality":80}"#.into(),
                created_at: now.clone(),
                updated_at: now,
            })
            .unwrap();

        let presets = store.list_presets().unwrap();
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].name, "PNG to WebP");

        store.set_setting("theme", "dark").unwrap();
        assert_eq!(store.get_setting("theme").unwrap().as_deref(), Some("dark"));
        assert!(store.delete_preset("p1").unwrap());
        assert!(store.list_presets().unwrap().is_empty());
    }
}
