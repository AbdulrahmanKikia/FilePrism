//! Shared shell → desktop IPC schema (named pipe on Windows, local socket elsewhere).
//!
//! Contract: one UTF-8 JSON object per line (`\n`-terminated).

use serde::{Deserialize, Serialize};

/// Local socket / named-pipe leaf name (not a full `\\.\pipe\…` path).
pub const SHELL_IPC_NAME: &str = "fileprism-shell-v1";

/// Full Windows named-pipe path used by the Explorer DLL client.
pub const SHELL_PIPE_PATH: &str = r"\\.\pipe\fileprism-shell-v1";

/// Protocol version embedded in every message.
pub const SHELL_IPC_VERSION: u32 = 1;

/// Action requested by the shell integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShellAction {
    /// Open paths in the app (default / generic handoff).
    Open,
    /// Prefill Convert view.
    Convert,
    /// Prefill Compress view.
    Compress,
    /// Prefill Archives / extract view.
    Extract,
}

impl ShellAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Convert => "convert",
            Self::Compress => "compress",
            Self::Extract => "extract",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "open" => Some(Self::Open),
            "convert" => Some(Self::Convert),
            "compress" => Some(Self::Compress),
            "extract" | "archives" | "archive" => Some(Self::Extract),
            _ => None,
        }
    }
}

impl Default for ShellAction {
    fn default() -> Self {
        Self::Open
    }
}

/// Payload sent from Explorer / Linux desktop actions to the desktop shell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellOpenMessage {
    /// Protocol version; must be [`SHELL_IPC_VERSION`].
    pub v: u32,
    #[serde(default)]
    pub action: ShellAction,
    /// Absolute filesystem paths selected in the shell.
    pub paths: Vec<String>,
}

impl ShellOpenMessage {
    pub fn new(action: ShellAction, paths: impl IntoIterator<Item = String>) -> Self {
        Self {
            v: SHELL_IPC_VERSION,
            action,
            paths: paths.into_iter().collect(),
        }
    }

    pub fn to_line(&self) -> Result<String, serde_json::Error> {
        let mut line = serde_json::to_string(self)?;
        line.push('\n');
        Ok(line)
    }

    pub fn from_line(line: &str) -> Result<Self, String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err("empty shell IPC line".into());
        }
        let msg: Self = serde_json::from_str(trimmed).map_err(|e| e.to_string())?;
        if msg.v != SHELL_IPC_VERSION {
            return Err(format!(
                "unsupported shell IPC version {} (expected {})",
                msg.v, SHELL_IPC_VERSION
            ));
        }
        if msg.paths.is_empty() {
            return Err("shell IPC message has no paths".into());
        }
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_line() {
        let msg = ShellOpenMessage::new(
            ShellAction::Convert,
            vec![
                r"C:\Users\test\a.png".into(),
                r"C:\Users\test\b.jpg".into(),
            ],
        );
        let line = msg.to_line().unwrap();
        assert!(line.ends_with('\n'));
        let parsed = ShellOpenMessage::from_line(&line).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn rejects_bad_version() {
        let err = ShellOpenMessage::from_line(r#"{"v":99,"action":"open","paths":["x"]}"#)
            .unwrap_err();
        assert!(err.contains("version"));
    }

    #[test]
    fn action_parse() {
        assert_eq!(ShellAction::parse("EXTRACT"), Some(ShellAction::Extract));
        assert_eq!(ShellAction::parse("nope"), None);
    }
}
