//! Integration tests for M4 archive create/extract pairs.

use std::path::{Path, PathBuf};
use std::process::Command;

use fileprism_core::{CancellationToken, Job, JobOptions, Pipeline};
use fileprism_formats_archive::ArchiveConverter;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "fileprism-m4-archive-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn job(id: &str, input: &Path, output: &Path, from: &str, to: &str, password: Option<&str>) -> Job {
    Job {
        id: id.into(),
        input_path: input.to_string_lossy().into(),
        output_path: output.to_string_lossy().into(),
        input_format: from.into(),
        output_format: to.into(),
        options: JobOptions {
            password: password.map(str::to_string),
            ..Default::default()
        },
    }
}

fn run(job: &Job) -> fileprism_core::Output {
    let mut pipeline = Pipeline::new();
    pipeline.register(ArchiveConverter::new());
    pipeline
        .run_silent(job, &CancellationToken::new())
        .unwrap_or_else(|e| {
            panic!(
                "convert {}→{} failed: {e}",
                job.input_format, job.output_format
            )
        })
}

fn seed_tree(root: &Path) {
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::write(root.join("a.txt"), b"alpha").unwrap();
    std::fs::write(root.join("sub").join("b.txt"), b"bravo").unwrap();
}

fn assert_extracted(dir: &Path) {
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "alpha");
    assert_eq!(
        std::fs::read_to_string(dir.join("sub").join("b.txt")).unwrap(),
        "bravo"
    );
}

#[test]
fn zip_dir_roundtrip() {
    let dir = temp_dir("zip");
    let src = dir.join("src");
    seed_tree(&src);
    let archive = dir.join("out.zip");
    let extracted = dir.join("out");

    let out = run(&job("zip-create", &src, &archive, "dir", "zip", None));
    assert!(out.bytes_written > 0);
    assert!(archive.is_file());

    let out = run(&job("zip-extract", &archive, &extracted, "zip", "dir", None));
    assert!(out.bytes_written > 0);
    assert_extracted(&extracted);
}

#[test]
fn zip_password_roundtrip() {
    let dir = temp_dir("zip-pw");
    let src = dir.join("src");
    seed_tree(&src);
    let archive = dir.join("secret.zip");
    let extracted = dir.join("out");
    let pw = "s3cret";

    run(&job("zip-pw-create", &src, &archive, "dir", "zip", Some(pw)));
    run(&job(
        "zip-pw-extract",
        &archive,
        &extracted,
        "zip",
        "dir",
        Some(pw),
    ));
    assert_extracted(&extracted);

    let bad = dir.join("bad");
    let err = {
        let mut pipeline = Pipeline::new();
        pipeline.register(ArchiveConverter::new());
        pipeline
            .run_silent(
                &job("zip-bad-pw", &archive, &bad, "zip", "dir", Some("wrong")),
                &CancellationToken::new(),
            )
            .unwrap_err()
    };
    let msg = err.to_string().to_ascii_lowercase();
    assert!(
        msg.contains("password") || msg.contains("decrypt") || msg.contains("invalid"),
        "unexpected error: {err}"
    );
}

#[test]
fn tar_dir_roundtrip() {
    let dir = temp_dir("tar");
    let src = dir.join("src");
    seed_tree(&src);
    let archive = dir.join("out.tar");
    let extracted = dir.join("out");

    run(&job("tar-create", &src, &archive, "dir", "tar", None));
    run(&job("tar-extract", &archive, &extracted, "tar", "dir", None));
    assert_extracted(&extracted);
}

#[test]
fn tar_rejects_password() {
    let dir = temp_dir("tar-pw");
    let src = dir.join("src");
    seed_tree(&src);
    let archive = dir.join("out.tar");
    let mut pipeline = Pipeline::new();
    pipeline.register(ArchiveConverter::new());
    let err = pipeline
        .run_silent(
            &job("tar-pw", &src, &archive, "dir", "tar", Some("x")),
            &CancellationToken::new(),
        )
        .unwrap_err();
    assert!(err.to_string().contains("password"));
}

#[test]
fn sevenz_dir_roundtrip() {
    let dir = temp_dir("7z");
    let src = dir.join("src");
    seed_tree(&src);
    let archive = dir.join("out.7z");
    let extracted = dir.join("out");

    run(&job("7z-create", &src, &archive, "dir", "7z", None));
    run(&job("7z-extract", &archive, &extracted, "7z", "dir", None));
    assert_extracted(&extracted);
}

#[test]
fn sevenz_password_roundtrip() {
    let dir = temp_dir("7z-pw");
    let src = dir.join("src");
    seed_tree(&src);
    let archive = dir.join("secret.7z");
    let extracted = dir.join("out");
    let pw = "hunter2";

    run(&job("7z-pw-create", &src, &archive, "dir", "7z", Some(pw)));
    run(&job(
        "7z-pw-extract",
        &archive,
        &extracted,
        "7z",
        "dir",
        Some(pw),
    ));
    assert_extracted(&extracted);
}

#[test]
fn file_to_zip() {
    let dir = temp_dir("file-zip");
    let file = dir.join("solo.txt");
    std::fs::write(&file, b"solo").unwrap();
    let archive = dir.join("solo.zip");
    let extracted = dir.join("out");

    run(&job("file-zip", &file, &archive, "file", "zip", None));
    run(&job("unzip", &archive, &extracted, "zip", "dir", None));
    assert_eq!(
        std::fs::read_to_string(extracted.join("solo.txt")).unwrap(),
        "solo"
    );
}

#[test]
fn supports_rar_pair() {
    assert!(ArchiveConverter::supports_pair("rar", "dir"));
    assert!(!ArchiveConverter::supports_pair("dir", "rar"));
}

#[test]
fn rar_extract_via_sidecar() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.rar");
    assert!(
        fixture.is_file(),
        "missing fixture {}",
        fixture.display()
    );

    // Build sidecar into target/debug (cooler: -j 1).
    let status = Command::new("cargo")
        .args([
            "build",
            "-j",
            "1",
            "-p",
            "fileprism-unrar-sidecar",
            "--quiet",
        ])
        .status()
        .expect("spawn cargo build sidecar");
    assert!(status.success(), "sidecar build failed");

    let sidecar = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/debug")
        .join(if cfg!(windows) {
            "fileprism-unrar-sidecar.exe"
        } else {
            "fileprism-unrar-sidecar"
        });
    let sidecar = sidecar
        .canonicalize()
        .unwrap_or_else(|_| sidecar.clone());
    assert!(sidecar.is_file(), "sidecar missing at {}", sidecar.display());

    let dir = temp_dir("rar");
    let extracted = dir.join("out");
    std::env::set_var("FILEPRISM_UNRAR_SIDECAR", &sidecar);

    let out = run(&job("rar-extract", &fixture, &extracted, "rar", "dir", None));
    assert!(out.bytes_written > 0);
    // Fixture from unrar crate `version.rar`.
    let version = extracted.join("VERSION");
    assert!(
        version.is_file(),
        "expected VERSION under {}",
        extracted.display()
    );

    let crypted = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/crypted.rar");
    let extracted_pw = dir.join("out-pw");
    let out = run(&job(
        "rar-pw",
        &crypted,
        &extracted_pw,
        "rar",
        "dir",
        Some("unrar"),
    ));
    assert!(out.bytes_written > 0);
    assert!(extracted_pw.join(".gitignore").is_file());
}
