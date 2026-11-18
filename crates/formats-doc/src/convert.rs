//! [`DocConverter`] implementation.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use fileprism_core::{Error, Job, Output};
#[cfg(feature = "pdfium")]
use fileprism_core::JobOptions;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use pulldown_cmark::{Event, Options as MdOptions, Parser, Tag, TagEnd};

use crate::formats::{normalize_format, supports_pair, DocFormat};
#[cfg(feature = "pdfium")]
use crate::options::{effective_quality, pdf_render_hint, PdfRenderHint};
#[cfg(feature = "pdfium")]
use crate::pdfium_lib;
#[cfg(feature = "pdfium")]
use image::ImageEncoder;
#[cfg(feature = "pdfium")]
use pdfium_render::prelude::*;

/// Document converter for PDF / Markdown / HTML / text.
#[derive(Debug, Default, Clone, Copy)]
pub struct DocConverter;

impl DocConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn supports_pair(input: &str, output: &str) -> bool {
        let Some(inp) = normalize_format(input) else {
            return false;
        };
        let Some(out) = normalize_format(output) else {
            return false;
        };
        supports_pair(inp, out)
    }

    pub fn convert_job(job: &Job) -> Result<Output, Error> {
        let input_fmt = normalize_format(&job.input_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        let output_fmt = normalize_format(&job.output_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        if !supports_pair(input_fmt, output_fmt) {
            return Err(Error::Unsupported(
                job.input_format.clone(),
                job.output_format.clone(),
            ));
        }

        ensure_parent_dir(Path::new(&job.output_path))?;

        match (input_fmt, output_fmt) {
            #[cfg(feature = "pdfium")]
            (DocFormat::Pdf, DocFormat::Png) | (DocFormat::Pdf, DocFormat::Jpeg) => {
                pdf_to_image(
                    Path::new(&job.input_path),
                    Path::new(&job.output_path),
                    output_fmt,
                    &job.options,
                )?;
            }
            (DocFormat::Pdf, DocFormat::Text) => {
                pdf_to_text(Path::new(&job.input_path), Path::new(&job.output_path))?;
            }
            (DocFormat::Markdown, DocFormat::Html) => {
                md_to_html(Path::new(&job.input_path), Path::new(&job.output_path))?;
            }
            (DocFormat::Markdown, DocFormat::Pdf) => {
                md_to_pdf(Path::new(&job.input_path), Path::new(&job.output_path))?;
            }
            _ => {
                return Err(Error::Unsupported(
                    job.input_format.clone(),
                    job.output_format.clone(),
                ));
            }
        }

        let bytes_written = std::fs::metadata(&job.output_path)
            .map(|m| m.len())
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(Output {
            job_id: job.id.clone(),
            output_path: job.output_path.clone(),
            bytes_written,
        })
    }
}

fn ensure_parent_dir(path: &Path) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Io(format!("create output dir {}: {e}", parent.display()))
            })?;
        }
    }
    Ok(())
}

/// Render the first page of a PDF to PNG or JPEG.
///
/// Multi-page → multi-file output is deferred; M3 writes page 0 to `output`.
/// Gated behind the `pdfium` feature (desktop only — not available in WASM).
#[cfg(feature = "pdfium")]
fn pdf_to_image(
    input: &Path,
    output: &Path,
    format: DocFormat,
    options: &JobOptions,
) -> Result<(), Error> {
    let pdfium = pdfium_lib::pdfium()?;
    let document = pdfium
        .load_pdf_from_file(input, None)
        .map_err(|e| Error::Other(format!("pdf open: {e}")))?;

    let page = document
        .pages()
        .get(0)
        .map_err(|e| Error::Other(format!("pdf page 0: {e}")))?;

    let mut config = PdfRenderConfig::new();
    match pdf_render_hint(options) {
        PdfRenderHint::Dpi(dpi) => {
            // Pdfium's scale factor is relative to 72 DPI.
            let scale = dpi as f32 / 72.0;
            config = config.scale_page_by_factor(scale);
        }
        PdfRenderHint::MaxWidth(w) => {
            config = config.set_target_width(w as i32);
        }
    }

    let image = page
        .render_with_config(&config)
        .map_err(|e| Error::Other(format!("pdf render: {e}")))?
        .as_image()
        .map_err(|e| Error::Other(format!("pdf bitmap: {e}")))?;

    match format {
        DocFormat::Png => {
            image
                .save_with_format(output, image::ImageFormat::Png)
                .map_err(|e| Error::Other(format!("png encode: {e}")))?;
        }
        DocFormat::Jpeg => {
            let rgb = image.into_rgb8();
            let file = File::create(output).map_err(|e| Error::Io(e.to_string()))?;
            let mut writer = BufWriter::new(file);
            let quality = effective_quality(options);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
            encoder
                .write_image(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| Error::Other(format!("jpeg encode: {e}")))?;
        }
        _ => unreachable!("pdf_to_image only for png/jpeg"),
    }
    Ok(())
}

fn pdf_to_text(input: &Path, output: &Path) -> Result<(), Error> {
    let bytes = std::fs::read(input).map_err(|e| Error::Io(e.to_string()))?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| Error::Other(format!("pdf text extract: {e}")))?;
    std::fs::write(output, text.as_bytes()).map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

fn md_to_html(input: &Path, output: &Path) -> Result<(), Error> {
    let markdown = std::fs::read_to_string(input).map_err(|e| Error::Io(e.to_string()))?;
    let mut html = String::new();
    let parser = Parser::new_ext(&markdown, MdOptions::all());
    pulldown_cmark::html::push_html(&mut html, parser);
    std::fs::write(output, html.as_bytes()).map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

/// Markdown → PDF (M3 spike): flatten markdown to plain text via pulldown-cmark
/// events, then lay out with `printpdf` BuiltinFont Helvetica.
fn md_to_pdf(input: &Path, output: &Path) -> Result<(), Error> {
    let markdown = std::fs::read_to_string(input).map_err(|e| Error::Io(e.to_string()))?;
    let plain = markdown_to_plain(&markdown);
    write_plain_pdf(&plain, output)
}

fn markdown_to_plain(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, MdOptions::all());
    let mut out = String::new();
    let mut needs_space = false;

    for event in parser {
        match event {
            Event::Text(t) | Event::Code(t) => {
                if needs_space && !out.ends_with([' ', '\n']) {
                    out.push(' ');
                }
                out.push_str(&t);
                needs_space = true;
            }
            Event::SoftBreak => {
                out.push(' ');
                needs_space = false;
            }
            Event::HardBreak => {
                out.push('\n');
                needs_space = false;
            }
            Event::Rule => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("---\n");
                needs_space = false;
            }
            Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::BlockQuote(_))
            | Event::End(TagEnd::CodeBlock) => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push('\n');
                needs_space = false;
            }
            Event::Start(Tag::Item) => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("- ");
                needs_space = false;
            }
            Event::Start(_) | Event::End(_) | Event::Html(_) | Event::InlineHtml(_) => {}
            Event::FootnoteReference(_) => {}
            Event::InlineMath(t) | Event::DisplayMath(t) => {
                out.push_str(&t);
                needs_space = true;
            }
            Event::TaskListMarker(checked) => {
                out.push_str(if checked { "[x] " } else { "[ ] " });
                needs_space = false;
            }
        }
    }

    out
}

fn write_plain_pdf(text: &str, output: &Path) -> Result<(), Error> {
    let page_w = Mm(210.0);
    let page_h = Mm(297.0);
    let margin = Mm(20.0);
    let font_size = 11.0;
    let line_height = Mm(5.5);
    let usable_width_pt = (page_w - margin - margin).0 / 25.4 * 72.0;
    // Helvetica average char width ≈ 0.5 * font_size.
    let approx_char_w = font_size * 0.5;
    let max_chars = ((usable_width_pt / approx_char_w) as usize).max(20);

    let (doc, page_idx, layer_idx) =
        PdfDocument::new("FilePrism", page_w, page_h, "Layer");
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| Error::Other(format!("pdf font: {e}")))?;

    let lines = wrap_text(text, max_chars);
    let mut page = page_idx;
    let mut layer = layer_idx;
    let mut y = page_h - margin;

    for line in lines {
        if y < margin + line_height {
            let (p, l) = doc.add_page(page_w, page_h, "Layer");
            page = p;
            layer = l;
            y = page_h - margin;
        }
        let current = doc.get_page(page).get_layer(layer);
        // printpdf rejects empty use_text in some versions — write a space.
        let draw = if line.is_empty() { " " } else { line.as_str() };
        current.use_text(draw, font_size, margin, y, &font);
        y -= line_height;
    }

    let file = File::create(output).map_err(|e| Error::Io(e.to_string()))?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| Error::Other(format!("pdf write: {e}")))?;
    Ok(())
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= max_chars {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(std::mem::take(&mut current));
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Create a tiny one-page PDF with `printpdf` (used by integration tests).
pub fn write_fixture_pdf(path: &Path, text: &str) -> Result<(), Error> {
    write_plain_pdf(text, path)
}
