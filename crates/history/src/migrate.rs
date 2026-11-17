//! Versioned SQLite migrations.

use rusqlite::Connection;

use crate::error::{HistoryError, Result};

/// Embedded migration scripts. Index `i` is version `i + 1`.
const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_initial.sql")];

/// Apply all pending migrations. Safe to call repeatedly.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for (idx, sql) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i64;
        if version <= current {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql).map_err(|e| {
            HistoryError::Migration(format!("migration {version} failed: {e}"))
        })?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            [version],
        )?;
        tx.commit()?;
    }

    Ok(())
}

/// Returns the highest applied migration version (0 if none).
pub fn current_version(conn: &Connection) -> Result<i64> {
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
        [],
        |row| row.get(0),
    )?;
    if !exists {
        return Ok(0);
    }
    let version = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        assert_eq!(current_version(&conn).unwrap(), 1);

        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
            )
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.iter().any(|t| t == "jobs"));
        assert!(tables.iter().any(|t| t == "files"));
        assert!(tables.iter().any(|t| t == "presets"));
        assert!(tables.iter().any(|t| t == "settings"));
        assert!(tables.iter().any(|t| t == "schema_migrations"));
    }
}
