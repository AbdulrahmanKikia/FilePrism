//! Named-pipe client + exe launch fallback for shell handoff.

use std::ffi::OsStr;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GENERIC_WRITE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Pipes::WaitNamedPipeW;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::ipc::{ShellOpenMessage, SHELL_PIPE_PATH};

fn to_wide(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

/// Send a JSON line to the desktop named pipe. Returns `Ok(())` on success.
pub fn send_message(msg: &ShellOpenMessage) -> Result<(), String> {
    let line = msg.to_line().map_err(|e| e.to_string())?;
    let pipe_wide = to_wide(SHELL_PIPE_PATH);

    // Wait briefly for the desktop app to accept a connection.
    unsafe {
        let _ = WaitNamedPipeW(PCWSTR(pipe_wide.as_ptr()), 800);
    }

    let handle = unsafe {
        CreateFileW(
            PCWSTR(pipe_wide.as_ptr()),
            GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
    .map_err(|e| format!("open pipe: {e}"))?;

    if handle == INVALID_HANDLE_VALUE {
        return Err("open pipe: invalid handle".into());
    }

    let result = (|| {
        let bytes = line.as_bytes();
        let mut written = 0u32;
        unsafe {
            WriteFile(handle, Some(bytes), Some(&mut written), None)
                .map_err(|e| format!("write pipe: {e}"))?;
        }
        if written as usize != bytes.len() {
            return Err(format!(
                "short pipe write: {written} of {} bytes",
                bytes.len()
            ));
        }
        Ok(())
    })();

    unsafe {
        let _ = CloseHandle(handle);
    }
    result
}

/// Resolve `FilePrism.exe` next to the DLL, via `FILEPRISM_EXE`, or parent install dir.
pub fn resolve_exe_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("FILEPRISM_EXE") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }

    let mut module = vec![0u16; 260];
    let len = unsafe { GetModuleFileNameW(None, &mut module) };
    if len == 0 {
        return None;
    }
    module.truncate(len as usize);
    let dll_path = PathBuf::from(std::ffi::OsString::from_wide(&module));
    let dir = dll_path.parent()?;

    // Install layout: <root>/FilePrism.exe and <root>/shell/fileprism_explorer_win.dll
    let candidates = [
        dir.join("FilePrism.exe"),
        dir.join("fileprism.exe"),
        dir.parent()
            .map(|p| p.join("FilePrism.exe"))
            .unwrap_or_default(),
        dir.join("fileprism-desktop.exe"),
    ];
    candidates.into_iter().find(|c| c.is_file())
}

fn quote_arg(arg: &str) -> String {
    if !arg.chars().any(|c| c == ' ' || c == '"' || c == '\t') {
        return arg.to_string();
    }
    let mut out = String::from('"');
    let mut backslashes = 0usize;
    for ch in arg.chars() {
        match ch {
            '\\' => backslashes += 1,
            '"' => {
                out.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                out.push('"');
                backslashes = 0;
            }
            _ => {
                out.extend(std::iter::repeat_n('\\', backslashes));
                backslashes = 0;
                out.push(ch);
            }
        }
    }
    out.extend(std::iter::repeat_n('\\', backslashes * 2));
    out.push('"');
    out
}

/// Launch the desktop app with `--shell-open` and the given paths (when pipe is down).
pub fn launch_exe(msg: &ShellOpenMessage) -> Result<(), String> {
    let exe = resolve_exe_path().ok_or_else(|| "FilePrism.exe not found".to_string())?;
    let mut args = String::from("--shell-open ");
    args.push_str(msg.action.as_str());
    for path in &msg.paths {
        args.push(' ');
        args.push_str(&quote_arg(path));
    }

    let exe_wide = to_wide(exe.as_os_str());
    let args_wide = to_wide(OsStr::new(&args));
    let verb = to_wide("open");
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR(args_wide.as_ptr()),
            None,
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW returns > 32 on success (HINSTANCE cast).
    if result.0 as usize <= 32 {
        return Err(format!("ShellExecuteW failed: {:?}", result));
    }
    Ok(())
}

/// Prefer named-pipe IPC; fall back to launching the exe.
pub fn handoff(msg: &ShellOpenMessage) {
    if send_message(msg).is_ok() {
        return;
    }
    let _ = launch_exe(msg);
}
