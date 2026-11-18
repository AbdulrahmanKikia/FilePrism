//! Shell integration listener: named pipe / local socket + CLI `--shell-open` handoff.

use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use fileprism_explorer_win::{ShellAction, ShellOpenMessage, SHELL_IPC_NAME};
use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Stream};
use tauri::{AppHandle, Emitter, Manager};

static STARTED: AtomicBool = AtomicBool::new(false);

/// Event name emitted to the frontend when shell paths arrive.
pub const SHELL_FILES_EVENT: &str = "shell-files";

/// Start the shell IPC server (idempotent). Uses a Windows named pipe /
/// Unix domain socket named [`SHELL_IPC_NAME`].
pub fn start_listener(app: AppHandle) {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let handle = app.clone();
    thread::spawn(move || server_loop(handle));
}

/// Parse `argv` for `--shell-open <action> <paths…>` (used when pipe was unavailable).
pub fn consume_cli_args(app: &AppHandle, args: &[String]) {
    let mut iter = args.iter().peekable();
    // Skip exe path.
    let _ = iter.next();
    while let Some(arg) = iter.next() {
        if arg == "--shell-open" {
            let action = iter
                .peek()
                .and_then(|s| ShellAction::parse(s))
                .unwrap_or(ShellAction::Open);
            if ShellAction::parse(iter.peek().map(|s| s.as_str()).unwrap_or("")).is_some() {
                let _ = iter.next();
            }
            let mut paths = Vec::new();
            for path in iter.by_ref() {
                if path.starts_with("--") {
                    break;
                }
                paths.push(path.clone());
            }
            if !paths.is_empty() {
                emit_message(app, &ShellOpenMessage::new(action, paths));
            }
            return;
        }
    }
}

fn emit_message(app: &AppHandle, msg: &ShellOpenMessage) {
    if let Err(err) = app.emit(SHELL_FILES_EVENT, msg) {
        eprintln!("FilePrism: failed to emit {SHELL_FILES_EVENT}: {err}");
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn server_loop(app: AppHandle) {
    loop {
        match bind_listener() {
            Ok(listener) => {
                eprintln!("FilePrism: shell IPC listening on {SHELL_IPC_NAME}");
                accept_loop(&app, listener);
            }
            Err(err) => {
                eprintln!("FilePrism: shell IPC bind failed ({err}); retrying…");
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

fn bind_listener() -> std::io::Result<interprocess::local_socket::Listener> {
    let name = SHELL_IPC_NAME
        .to_ns_name::<GenericNamespaced>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    ListenerOptions::new().name(name).create_sync()
}

fn accept_loop(app: &AppHandle, listener: interprocess::local_socket::Listener) {
    loop {
        match listener.accept() {
            Ok(stream) => handle_client(app, stream),
            Err(err) => {
                eprintln!("FilePrism: shell IPC accept error: {err}");
                thread::sleep(Duration::from_millis(200));
                // Re-bind on fatal accept failures.
                if err.kind() == std::io::ErrorKind::ConnectionRefused
                    || err.kind() == std::io::ErrorKind::NotConnected
                {
                    return;
                }
            }
        }
    }
}

fn handle_client(app: &AppHandle, stream: Stream) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => {}
        Ok(_) => match ShellOpenMessage::from_line(&line) {
            Ok(msg) => emit_message(app, &msg),
            Err(err) => eprintln!("FilePrism: bad shell IPC payload: {err}"),
        },
        Err(err) => eprintln!("FilePrism: shell IPC read error: {err}"),
    }
}
