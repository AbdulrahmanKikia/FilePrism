//! Integration tests for M3 document conversion pairs.

use std::path::{Path, PathBuf};

use fileprism_core::{CancellationToken, Converter, Job, JobOptions, Pipeline};
use fileprism_formats_doc::{write_fixture_pdf, DocConverter};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fileprism-m3-doc-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn job(id: &str, input: &Path, output: &Path, from: &str, to: &str, options: JobOptions) -> Job {
    Job {
        id: id.into(),
        input_path: input.to_string_lossy().into(),
        output_path: output.to_string_lossy().into(),
        input_format: from.into(),
        output_format: to.into(),
        options,
    }
}

fn run(job: &Job) -> fileprism_core::Output {
    let mut pipeline = Pipeline::new();
    pipeline.register(DocConverter::new());
    pipeline
        .run_silent(job, &CancellationToken::new())
        .unwrap_or_else(|e| {
            panic!(
                "convert {}→{} failed: {e}",
                job.input_format, job.output_format
            )
        })
}

#[test]
fn supports_expected_pairs() {
    let c = DocConverter::new();
    assert!(c.supports("pdf", "png"));
    assert!(c.supports("pdf", "jpg"));
    assert!(c.supports("pdf", "jpeg"));
    assert!(c.supports("pdf", "txt"));
    assert!(c.supports("pdf", "text"));
    assert!(c.supports("md", "html"));
    assert!(c.supports("markdown", "html"));
    assert!(c.supports("md", "pdf"));
    assert!(c.supports("markdown", "pdf"));
    assert!(!c.supports("html", "pdf"));
    assert!(!c.supports("png", "pdf"));
    assert!(!c.supports("docx", "pdf"));
}

#[test]
fn md_to_html() {
    let dir = temp_dir("md-html");
    let input = dir.join("in.md");
    let output = dir.join("out.html");
    std::fs::write(
        &input,
        "# Hello\n\nThis is **bold** and a [link](https://example.com).\n",
    )
    .unwrap();

    let out = run(&job(
        "md-html",
        &input,
        &output,
        "md",
        "html",
        JobOptions::default(),
    ));
    assert!(out.bytes_written > 0);
    let html = std::fs::read_to_string(&output).unwrap();
    assert!(html.contains("<h1>") || html.contains("<h1 "));
    assert!(html.contains("Hello"));
    assert!(html.contains("<strong>bold</strong>") || html.contains("<strong>"));
}

#[test]
fn md_to_pdf() {
    let dir = temp_dir("md-pdf");
    let input = dir.join("in.md");
    let output = dir.join("out.pdf");
    std::fs::write(
        &input,
        "# Title\n\nParagraph one.\n\n- item a\n- item b\n",
    )
    .unwrap();

    let out = run(&job(
        "md-pdf",
        &input,
        &output,
        "markdown",
        "pdf",
        JobOptions::default(),
    ));
    assert!(out.bytes_written > 100);
    let magic = std::fs::read(&output).unwrap();
    assert!(magic.starts_with(b"%PDF"));
}

#[test]
fn pdf_to_text() {
    let dir = temp_dir("pdf-txt");
    let input = dir.join("in.pdf");
    let output = dir.join("out.txt");
    write_fixture_pdf(&input, "FilePrism extract me please").unwrap();

    let out = run(&job(
        "pdf-txt",
        &input,
        &output,
        "pdf",
        "txt",
        JobOptions::default(),
    ));
    assert!(out.bytes_written > 0);
    let text = std::fs::read_to_string(&output).unwrap();
    assert!(
        text.contains("FilePrism") || text.contains("extract"),
        "unexpected extract: {text:?}"
    );
}

#[test]
fn pdf_to_png_and_jpg() {
    let dir = temp_dir("pdf-img");
    let input = dir.join("in.pdf");
    write_fixture_pdf(&input, "Render this page").unwrap();

    let png = dir.join("out.png");
    let out = run(&job(
        "pdf-png",
        &input,
        &png,
        "pdf",
        "png",
        JobOptions {
            width: Some(96), // DPI hint
            ..Default::default()
        },
    ));
    assert!(out.bytes_written > 0);
    let img = image::open(&png).unwrap_or_else(|e| panic!("open png: {e}"));
    assert!(img.width() > 0 && img.height() > 0);

    let jpg = dir.join("out.jpg");
    let out = run(&job(
        "pdf-jpg",
        &input,
        &jpg,
        "pdf",
        "jpeg",
        JobOptions {
            width: Some(800), // max width pixels
            quality: Some(70),
            ..Default::default()
        },
    ));
    assert!(out.bytes_written > 0);
    let img = image::open(&jpg).unwrap_or_else(|e| panic!("open jpg: {e}"));
    assert!(img.width() > 0 && img.height() > 0);
    assert!(img.width() <= 800);
}
