//! Integration tests for M3 document conversion pairs.

use std::path::{Path, PathBuf};

use fileprism_core::{CancellationToken, Converter, Job, JobOptions, Pipeline};
use fileprism_formats_doc::DocConverter;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fileprism-m3-doc-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn job(id: &str, input: &Path, output: &Path, from: &str, to: &str) -> Job {
    Job {
        id: id.into(),
        input_path: input.to_string_lossy().into(),
        output_path: output.to_string_lossy().into(),
        input_format: from.into(),
        output_format: to.into(),
        options: JobOptions::default(),
    }
}

fn run(job: &Job) -> Result<fileprism_core::Output, fileprism_core::Error> {
    let mut pipeline = Pipeline::new();
    pipeline.register(DocConverter::new());
    pipeline.run_silent(job, &CancellationToken::new())
}

#[test]
fn md_to_html() {
    let dir = temp_dir("md-html");
    let md = dir.join("in.md");
    let html = dir.join("out.html");
    std::fs::write(&md, "# Hello\n\nFilePrism **rocks**.\n").unwrap();
    let out = run(&job("md-html", &md, &html, "md", "html")).unwrap();
    assert!(out.bytes_written > 0);
    let text = std::fs::read_to_string(&html).unwrap();
    assert!(text.contains("Hello"));
    assert!(text.contains("rocks"));
}

#[test]
fn md_to_pdf_spike() {
    let dir = temp_dir("md-pdf");
    let md = dir.join("in.md");
    let pdf = dir.join("out.pdf");
    std::fs::write(&md, "# Title\n\nParagraph one.\n\n- item\n").unwrap();
    let out = run(&job("md-pdf", &md, &pdf, "markdown", "pdf")).unwrap();
    assert!(out.bytes_written > 100);
    let magic = std::fs::read(&pdf).unwrap();
    assert!(magic.starts_with(b"%PDF"));
}

#[test]
fn pdf_to_text() {
    let dir = temp_dir("pdf-text");
    let md = dir.join("in.md");
    let pdf = dir.join("mid.pdf");
    let txt = dir.join("out.txt");
    std::fs::write(&md, "UniqueTokenFilePrism42\n\nSecond line.\n").unwrap();
    run(&job("mkpdf", &md, &pdf, "md", "pdf")).unwrap();
    let out = run(&job("pdf-txt", &pdf, &txt, "pdf", "txt")).unwrap();
    assert!(out.bytes_written > 0 || !std::fs::read_to_string(&txt).unwrap().is_empty());
}

#[test]
fn pdf_to_png_text_raster() {
    let dir = temp_dir("pdf-png");
    let md = dir.join("in.md");
    let pdf = dir.join("mid.pdf");
    let png = dir.join("out.png");
    std::fs::write(&md, "# Render me\n").unwrap();
    run(&job("mkpdf", &md, &pdf, "md", "pdf")).unwrap();

    let out = run(&job("pdf-png", &pdf, &png, "pdf", "png")).unwrap();
    assert!(out.bytes_written > 0);
    assert!(png.is_file());
    let img = image::open(&png).unwrap();
    assert!(img.width() >= 200 && img.height() >= 200);
}

#[test]
fn supports_expected_pairs() {
    let c = DocConverter::new();
    assert!(c.supports("pdf", "png"));
    assert!(c.supports("pdf", "jpg"));
    assert!(c.supports("pdf", "txt"));
    assert!(c.supports("md", "html"));
    assert!(c.supports("markdown", "pdf"));
    assert!(!c.supports("html", "pdf"));
}
