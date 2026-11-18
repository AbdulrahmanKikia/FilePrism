//! [`DocConverter`] implementation.
//!
//! ## MD→PDF spike decision
//! Chosen: `printpdf` with flattened markdown text (headings/paragraphs as plain
//! lines). Avoids headless Chrome / wkhtmltopdf. Good enough for M3; richer HTML
//! layout can land later if needed.
//!
//! ## PDF→PNG/JPG decision
//! Portable text-raster fallback: extract text via `pdf-extract`, then draw onto
//! a PNG/JPEG canvas with `ab_glyph` (embedded font). Full page rasterization
//! (pdfium) is deferred until a no-bindgen binding path is available for CI.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use fileprism_core::{Error, Job, JobOptions, Output};
use image::{ImageEncoder, ImageFormat as ImgFmt, Rgb, RgbImage};
use printpdf::{Mm, PdfDocument};
use pulldown_cmark::{html, Options, Parser};

use crate::formats::{normalize_format, supports_pair, DocFormat};

/// Embedded Latin font for PDF→image text rasterization.
const FONT_BYTES: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

/// Document converter for PDF / Markdown pairs.
#[derive(Debug, Default, Clone, Copy)]
pub struct DocConverter;

impl DocConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn supports_pair(input: &str, output: &str) -> bool {
        match (normalize_format(input), normalize_format(output)) {
            (Some(i), Some(o)) => supports_pair(i, o),
            _ => false,
        }
    }

    pub fn convert_job(job: &Job) -> Result<Output, Error> {
        let input = normalize_format(&job.input_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        let output = normalize_format(&job.output_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        if !supports_pair(input, output) {
            return Err(Error::Unsupported(
                job.input_format.clone(),
                job.output_format.clone(),
            ));
        }

        let in_path = Path::new(&job.input_path);
        let out_path = Path::new(&job.output_path);
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        match (input, output) {
            (DocFormat::Markdown, DocFormat::Html) => md_to_html(in_path, out_path)?,
            (DocFormat::Markdown, DocFormat::Pdf) => md_to_pdf(in_path, out_path)?,
            (DocFormat::Pdf, DocFormat::Text) => pdf_to_text(in_path, out_path)?,
            (DocFormat::Pdf, DocFormat::Png) => {
                pdf_to_image(in_path, out_path, ImgFmt::Png, &job.options)?
            }
            (DocFormat::Pdf, DocFormat::Jpeg) => {
                pdf_to_image(in_path, out_path, ImgFmt::Jpeg, &job.options)?
            }
            _ => {
                return Err(Error::Unsupported(
                    job.input_format.clone(),
                    job.output_format.clone(),
                ));
            }
        }

        let bytes_written = std::fs::metadata(out_path)?.len();
        Ok(Output {
            job_id: job.id.clone(),
            output_path: job.output_path.clone(),
            bytes_written,
        })
    }
}

fn md_to_html(input: &Path, output: &Path) -> Result<(), Error> {
    let markdown = std::fs::read_to_string(input)?;
    let parser = Parser::new_ext(&markdown, Options::all());
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    let mut w = BufWriter::new(File::create(output)?);
    writeln!(
        w,
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>\n{html_out}</body></html>"
    )?;
    w.flush()?;
    Ok(())
}

fn md_to_pdf(input: &Path, output: &Path) -> Result<(), Error> {
    let markdown = std::fs::read_to_string(input)?;
    let lines = flatten_markdown_lines(&markdown);

    let (doc, page1, layer1) = PdfDocument::new("FilePrism", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc
        .add_builtin_font(printpdf::BuiltinFont::Helvetica)
        .map_err(|e| Error::Other(format!("pdf font: {e}")))?;
    let layer = doc.get_page(page1).get_layer(layer1);

    let mut y = 280.0;
    for line in lines.iter().take(60) {
        let truncated: String = line.chars().take(95).collect();
        layer.use_text(&truncated, 11.0, Mm(15.0), Mm(y), &font);
        y -= 6.0;
        if y < 15.0 {
            break;
        }
    }

    doc.save(&mut BufWriter::new(File::create(output)?))
        .map_err(|e| Error::Other(format!("pdf save: {e}")))?;
    Ok(())
}

fn flatten_markdown_lines(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .map(|l| {
            l.trim()
                .trim_start_matches('#')
                .trim_start_matches(['*', '-', '>', ' '])
                .replace('`', "")
                .replace('*', "")
                .replace('_', "")
        })
        .filter(|l| !l.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn pdf_to_text(input: &Path, output: &Path) -> Result<(), Error> {
    let bytes = std::fs::read(input)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| Error::Other(format!("pdf text: {e}")))?;
    std::fs::write(output, text)?;
    Ok(())
}

fn pdf_to_image(
    input: &Path,
    output: &Path,
    format: ImgFmt,
    options: &JobOptions,
) -> Result<(), Error> {
    let bytes = std::fs::read(input)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| Error::Other(format!("pdf text: {e}")))?;
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();

    let width = options.width.unwrap_or(800).clamp(200, 4000);
    let line_height = 18u32;
    let height = ((lines.len() as u32 + 2) * line_height).clamp(200, 4000);
    let mut img = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));

    let font = FontRef::try_from_slice(FONT_BYTES)
        .map_err(|e| Error::Other(format!("font load: {e}")))?;
    let scale = PxScale::from(14.0);
    let scaled = font.as_scaled(scale);

    let mut y = 20.0f32;
    for line in lines.iter().take(200) {
        let mut x = 16.0f32;
        for ch in line.chars().take(120) {
            let glyph = scaled.scaled_glyph(ch);
            let outlined = font.outline_glyph(glyph);
            if let Some(outlined) = outlined {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, v| {
                    if v < 0.3 {
                        return;
                    }
                    let px = (x + bounds.min.x + gx as f32) as i32;
                    let py = (y + bounds.min.y + gy as f32) as i32;
                    if px >= 0 && py >= 0 && (px as u32) < width && (py as u32) < height {
                        let c = (255.0 * (1.0 - v)) as u8;
                        img.put_pixel(px as u32, py as u32, Rgb([c, c, c]));
                    }
                });
                x += scaled.h_advance(scaled.glyph_id(ch));
            } else {
                x += scaled.h_advance(scaled.glyph_id(ch));
            }
        }
        y += line_height as f32;
        if y > height as f32 - 20.0 {
            break;
        }
    }

    let mut out = File::create(output)?;
    match format {
        ImgFmt::Png => img
            .write_to(&mut out, ImgFmt::Png)
            .map_err(|e| Error::Other(format!("png write: {e}")))?,
        ImgFmt::Jpeg => {
            let quality = options.quality.unwrap_or(85).clamp(1, 100);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
            encoder
                .write_image(
                    img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| Error::Other(format!("jpeg write: {e}")))?;
        }
        _ => {
            return Err(Error::Other(
                "unsupported image format for pdf render".into(),
            ))
        }
    }
    Ok(())
}
