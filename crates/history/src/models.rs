//! Row types for history, presets, and settings.

use serde::{Deserialize, Serialize};

/// Job lifecycle status stored in SQLite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(JobStatus::Pending),
            "running" => Some(JobStatus::Running),
            "completed" => Some(JobStatus::Completed),
            "failed" => Some(JobStatus::Failed),
            "cancelled" => Some(JobStatus::Cancelled),
            _ => None,
        }
    }
}

/// Persisted conversion job row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub input_format: String,
    pub output_format: String,
    pub status: JobStatus,
    pub bytes_written: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Role of a file associated with a job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    Input,
    Output,
}

impl FileRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileRole::Input => "input",
            FileRole::Output => "output",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "input" => Some(FileRole::Input),
            "output" => Some(FileRole::Output),
            _ => None,
        }
    }
}

/// File row linked to a job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub job_id: String,
    pub path: String,
    pub role: FileRole,
    pub format: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Saved conversion preset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub input_format: String,
    pub output_format: String,
    /// Opaque JSON options blob (quality, resize, …).
    pub options_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Key/value setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

/// Well-known setting keys.
pub mod keys {
    /// When `"false"`, history recording should be skipped (privacy mode).
    pub const HISTORY_ENABLED: &str = "history.enabled";
}
