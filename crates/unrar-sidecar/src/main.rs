//! FilePrism UnRAR sidecar — extract only.
//!
//! Linked against UnRAR separately from the main FilePrism binary to keep
//! GPL/UnRAR licensing isolated (see product spec §4.5).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use unrar::Archive;

#[derive(Parser, Debug)]
#[command(
    name = "fileprism-unrar-sidecar",
    about = "Extract RAR archives for FilePrism (sidecar process)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract archive contents into an output directory.
    Extract {
        /// Path to the `.rar` archive.
        archive: PathBuf,
        /// Destination directory (created if missing).
        output: PathBuf,
        /// Optional archive password.
        #[arg(long, short = 'p')]
        password: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Extract {
            archive,
            output,
            password,
        } => extract(archive, output, password),
    }
}

fn extract(archive: PathBuf, output: PathBuf, password: Option<String>) -> Result<(), String> {
    if !archive.is_file() {
        return Err(format!("archive not found: {}", archive.display()));
    }
    std::fs::create_dir_all(&output)
        .map_err(|e| format!("create output {}: {e}", output.display()))?;

    let mut open = match password {
        Some(pw) => Archive::with_password(&archive, &pw)
            .open_for_processing()
            .map_err(|e| format!("open rar: {e}"))?,
        None => Archive::new(&archive)
            .open_for_processing()
            .map_err(|e| format!("open rar: {e}"))?,
    };

    while let Some(header) = open.read_header().map_err(|e| format!("read header: {e}"))? {
        let entry = header.entry();
        let name = entry.filename.clone();
        open = if entry.is_file() {
            header
                .extract_with_base(&output)
                .map_err(|e| format!("extract {}: {e}", name.display()))?
        } else {
            header.skip().map_err(|e| format!("skip {}: {e}", name.display()))?
        };
    }

    Ok(())
}
