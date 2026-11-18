//! FilePrism Windows Explorer shell extension (`IExplorerCommand`) and shared IPC.
//!
//! - Cross-platform: [`ipc`] message schema + pipe name constants.
//! - Windows: COM DLL exports (`DllGetClassObject`, …) that send selected paths to the
//!   desktop app over a named pipe (falls back to launching the exe).

pub mod ipc;

pub use ipc::{
    ShellAction, ShellOpenMessage, SHELL_IPC_NAME, SHELL_IPC_VERSION, SHELL_PIPE_PATH,
};

/// Human-readable extension name (menu title / docs).
pub fn extension_name() -> &'static str {
    "FilePrism Explorer"
}

/// Stable CLSID for the FilePrism `IExplorerCommand` COM class.
/// `{8F3A2B1C-6D4E-4F9A-9B2C-1E5D7A8F3C4E}`
pub const EXPLORER_COMMAND_CLSID: &str = "8F3A2B1C-6D4E-4F9A-9B2C-1E5D7A8F3C4E";

#[cfg(windows)]
mod win;
