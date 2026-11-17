//! Integration tests for M2 image conversion pairs.

use std::path::{Path, PathBuf};

use fileprism_core::{CancellationToken, Converter, Job, JobOptions, Pipeline};
use fileprism_formats_image::ImageConverter;
use image::{ImageBuffer, Rgb};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fileprism-m2-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_rgb_png(path: &Path, w: u32, h: u32) {
    let img: ImageBuffer<Rgb<u8>, _> =
        ImageBuffer::from_fn(w, h, |x, y| Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
    img.save(path).unwrap();
}

fn write_rgb_jpeg(path: &Path, w: u32, h: u32) {
    let img: ImageBuffer<Rgb<u8>, _> =
        ImageBuffer::from_fn(w, h, |x, y| Rgb([(x % 256) as u8, 64, (y % 256) as u8]));
    img.save(path).unwrap();
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
    pipeline.register(ImageConverter::new());
    pipeline
        .run_silent(job, &CancellationToken::new())
        .unwrap_or_else(|e| panic!("convert {}→{} failed: {e}", job.input_format, job.output_format))
}

fn assert_image_readable(path: &Path) {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "avif" {
        // image::open lacks portable AVIF decode; use our converter path.
        let data = std::fs::read(path).unwrap();
        let img = avif_decode::Decoder::from_avif(&data)
            .unwrap()
            .to_image()
            .unwrap();
        let (w, h) = match &img {
            avif_decode::Image::Rgb8(b) => (b.width(), b.height()),
            avif_decode::Image::Rgba8(b) => (b.width(), b.height()),
            avif_decode::Image::Rgb16(b) => (b.width(), b.height()),
            avif_decode::Image::Rgba16(b) => (b.width(), b.height()),
            avif_decode::Image::Gray8(b) => (b.width(), b.height()),
            avif_decode::Image::Gray16(b) => (b.width(), b.height()),
        };
        assert!(w > 0 && h > 0);
        return;
    }
    let img = image::open(path).unwrap_or_else(|e| panic!("open {}: {e}", path.display()));
    assert!(img.width() > 0 && img.height() > 0);
}

#[test]
fn png_to_webp() {
    let dir = temp_dir("png-webp");
    let input = dir.join("in.png");
    let output = dir.join("out.webp");
    write_rgb_png(&input, 32, 24);
    let out = run(&job("png-webp", &input, &output, "png", "webp", JobOptions {
        quality: Some(80),
        ..Default::default()
    }));
    assert!(out.bytes_written > 0);
    assert_image_readable(&output);
}

#[test]
fn jpg_to_webp() {
    let dir = temp_dir("jpg-webp");
    let input = dir.join("in.jpg");
    let output = dir.join("out.webp");
    write_rgb_jpeg(&input, 40, 30);
    let out = run(&job("jpg-webp", &input, &output, "jpg", "webp", JobOptions {
        quality: Some(75),
        ..Default::default()
    }));
    assert!(out.bytes_written > 0);
    assert_image_readable(&output);
}

#[test]
fn avif_encode_and_decode_roundtrip() {
    let dir = temp_dir("avif-rt");
    let png = dir.join("in.png");
    let avif = dir.join("mid.avif");
    let back = dir.join("out.png");
    write_rgb_png(&png, 24, 24);

    run(&job("to-avif", &png, &avif, "png", "avif", JobOptions {
        quality: Some(60),
        ..Default::default()
    }));
    assert_image_readable(&avif);

    run(&job("from-avif", &avif, &back, "avif", "png", JobOptions::default()));
    assert_image_readable(&back);
}

#[test]
fn resize_and_quality_options() {
    let dir = temp_dir("resize");
    let input = dir.join("in.png");
    let output = dir.join("out.jpg");
    write_rgb_png(&input, 100, 80);

    let out = run(&job(
        "resize",
        &input,
        &output,
        "png",
        "jpg",
        JobOptions {
            quality: Some(50),
            width: Some(50),
            height: None,
            strip_metadata: Some(true),
        },
    ));
    assert!(out.bytes_written > 0);
    let img = image::open(&output).unwrap();
    assert_eq!(img.width(), 50);
    assert_eq!(img.height(), 40);
}

#[test]
fn heic_to_png_and_jpg() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.heic");
    assert!(
        fixture.is_file(),
        "missing HEIC fixture at {}",
        fixture.display()
    );

    let dir = temp_dir("heic");
    let png = dir.join("out.png");
    let jpg = dir.join("out.jpg");

    run(&job("heic-png", &fixture, &png, "heic", "png", JobOptions::default()));
    assert_image_readable(&png);

    run(&job(
        "heic-jpg",
        &fixture,
        &jpg,
        "heic",
        "jpg",
        JobOptions {
            quality: Some(85),
            width: Some(64),
            ..Default::default()
        },
    ));
    let img = image::open(&jpg).unwrap();
    assert_eq!(img.width(), 64);
}

#[test]
fn supports_expected_pairs() {
    let c = ImageConverter::new();
    assert!(c.supports("png", "webp"));
    assert!(c.supports("jpg", "webp"));
    assert!(c.supports("jpeg", "avif"));
    assert!(c.supports("heic", "png"));
    assert!(c.supports("heif", "jpg"));
    assert!(c.supports("avif", "png"));
    assert!(c.supports("webp", "jpg"));
    assert!(!c.supports("png", "heic"));
    assert!(!c.supports("svg", "png"));
}
