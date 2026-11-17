//! [`ImageConverter`] implementation.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use fileprism_core::{Error, Job, JobOptions, Output};
use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder, ImageFormat as ImgFmt};

use crate::formats::{normalize_format, ImageFormat};
use crate::options::{effective_quality, effective_strip_metadata, resize_dimensions};

/// Raster image converter for PNG / JPEG / WebP / AVIF / HEIC.
#[derive(Debug, Default, Clone, Copy)]
pub struct ImageConverter;

impl ImageConverter {
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
        inp.is_input() && out.is_output()
    }

    pub fn convert_job(job: &Job) -> Result<Output, Error> {
        let input_fmt = normalize_format(&job.input_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        let output_fmt = normalize_format(&job.output_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        if !output_fmt.is_output() {
            return Err(Error::Unsupported(
                job.input_format.clone(),
                job.output_format.clone(),
            ));
        }

        let mut img = decode_image(Path::new(&job.input_path), input_fmt)?;
        img = maybe_resize(img, &job.options);
        // Re-encoding without copying EXIF satisfies strip_metadata default.
        let _strip = effective_strip_metadata(&job.options);
        let quality = effective_quality(&job.options);

        if let Some(parent) = Path::new(&job.output_path).parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    Error::Io(format!("create output dir {}: {e}", parent.display()))
                })?;
            }
        }

        encode_image(&img, Path::new(&job.output_path), output_fmt, quality)?;
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

fn maybe_resize(img: DynamicImage, options: &JobOptions) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    match resize_dimensions(w, h, options) {
        Some((tw, th)) if tw != w || th != h => img.resize_exact(tw, th, FilterType::Lanczos3),
        _ => img,
    }
}

fn decode_image(path: &Path, format: ImageFormat) -> Result<DynamicImage, Error> {
    match format {
        ImageFormat::Heic => decode_heic(path),
        ImageFormat::Avif => decode_avif(path),
        other => {
            let reader = image::ImageReader::open(path)
                .map_err(|e| Error::Io(e.to_string()))?
                .with_guessed_format()
                .map_err(|e| Error::Io(e.to_string()))?;
            // Prefer explicit format when guess fails (e.g. odd extensions).
            let decoded = match reader.format() {
                Some(_) => reader.decode(),
                None => {
                    let data = std::fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
                    image::load_from_memory_with_format(&data, to_image_crate_format(other)?)
                }
            };
            decoded.map_err(|e| Error::Other(format!("decode {}: {e}", format.as_str())))
        }
    }
}

fn decode_avif(path: &Path) -> Result<DynamicImage, Error> {
    let data = std::fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
    let decoder = avif_decode::Decoder::from_avif(&data)
        .map_err(|e| Error::Other(format!("avif open: {e}")))?;
    let image = decoder
        .to_image()
        .map_err(|e| Error::Other(format!("avif decode: {e}")))?;
    match image {
        avif_decode::Image::Rgb8(buf) => {
            let (w, h) = (buf.width() as u32, buf.height() as u32);
            let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
            for px in buf.pixels() {
                rgb.extend_from_slice(&[px.r, px.g, px.b]);
            }
            let buffer = image::RgbImage::from_raw(w, h, rgb)
                .ok_or_else(|| Error::Other("avif rgb buffer mismatch".into()))?;
            Ok(DynamicImage::ImageRgb8(buffer))
        }
        avif_decode::Image::Rgba8(buf) => {
            let (w, h) = (buf.width() as u32, buf.height() as u32);
            let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
            for px in buf.pixels() {
                rgba.extend_from_slice(&[px.r, px.g, px.b, px.a]);
            }
            let buffer = image::RgbaImage::from_raw(w, h, rgba)
                .ok_or_else(|| Error::Other("avif rgba buffer mismatch".into()))?;
            Ok(DynamicImage::ImageRgba8(buffer))
        }
        avif_decode::Image::Rgb16(buf) => {
            // Downshift 16-bit to 8-bit for the shared DynamicImage pipeline.
            let (w, h) = (buf.width() as u32, buf.height() as u32);
            let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
            for px in buf.pixels() {
                rgb.extend_from_slice(&[(px.r >> 8) as u8, (px.g >> 8) as u8, (px.b >> 8) as u8]);
            }
            let buffer = image::RgbImage::from_raw(w, h, rgb)
                .ok_or_else(|| Error::Other("avif rgb16 buffer mismatch".into()))?;
            Ok(DynamicImage::ImageRgb8(buffer))
        }
        avif_decode::Image::Rgba16(buf) => {
            let (w, h) = (buf.width() as u32, buf.height() as u32);
            let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
            for px in buf.pixels() {
                rgba.extend_from_slice(&[
                    (px.r >> 8) as u8,
                    (px.g >> 8) as u8,
                    (px.b >> 8) as u8,
                    (px.a >> 8) as u8,
                ]);
            }
            let buffer = image::RgbaImage::from_raw(w, h, rgba)
                .ok_or_else(|| Error::Other("avif rgba16 buffer mismatch".into()))?;
            Ok(DynamicImage::ImageRgba8(buffer))
        }
        avif_decode::Image::Gray8(buf) => {
            let (w, h) = (buf.width() as u32, buf.height() as u32);
            let pixels: Vec<u8> = buf.pixels().map(|p| p.value()).collect();
            let buffer = image::GrayImage::from_raw(w, h, pixels)
                .ok_or_else(|| Error::Other("avif gray buffer mismatch".into()))?;
            Ok(DynamicImage::ImageLuma8(buffer))
        }
        avif_decode::Image::Gray16(buf) => {
            let (w, h) = (buf.width() as u32, buf.height() as u32);
            let pixels: Vec<u8> = buf.pixels().map(|p| (p.value() >> 8) as u8).collect();
            let buffer = image::GrayImage::from_raw(w, h, pixels)
                .ok_or_else(|| Error::Other("avif gray16 buffer mismatch".into()))?;
            Ok(DynamicImage::ImageLuma8(buffer))
        }
    }
}

fn decode_heic(path: &Path) -> Result<DynamicImage, Error> {
    let data = std::fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
    let options = heic_rs::DecodeOptions {
        layout: heic_rs::PixelLayout::Rgba8,
        ..heic_rs::DecodeOptions::default()
    };
    let decoded = heic_rs::decode(&data, &options)
        .map_err(|e| Error::Other(format!("heic decode: {e}")))?;

    let width = decoded.width;
    let height = decoded.height;
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| Error::Other("heic dimensions overflow".into()))?;
    if decoded.data.len() < expected {
        return Err(Error::Other(format!(
            "heic rgba length {} < {}x{}x4",
            decoded.data.len(),
            width,
            height
        )));
    }
    // Drop row padding if present.
    let mut rgba = Vec::with_capacity(expected);
    let row_bytes = decoded.row_bytes();
    for y in 0..height as usize {
        let start = y * row_bytes;
        rgba.extend_from_slice(&decoded.data[start..start + width as usize * 4]);
    }
    let buffer = image::RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| Error::Other("heic rgba buffer mismatch".into()))?;
    Ok(DynamicImage::ImageRgba8(buffer))
}

fn to_image_crate_format(format: ImageFormat) -> Result<ImgFmt, Error> {
    match format {
        ImageFormat::Png => Ok(ImgFmt::Png),
        ImageFormat::Jpeg => Ok(ImgFmt::Jpeg),
        ImageFormat::Webp => Ok(ImgFmt::WebP),
        ImageFormat::Avif => Ok(ImgFmt::Avif),
        ImageFormat::Heic => Err(Error::Other(
            "heic is not handled by the image crate".into(),
        )),
    }
}

fn encode_image(
    img: &DynamicImage,
    path: &Path,
    format: ImageFormat,
    quality: u8,
) -> Result<(), Error> {
    let file = File::create(path).map_err(|e| Error::Io(e.to_string()))?;
    let mut writer = BufWriter::new(file);

    match format {
        ImageFormat::Png => {
            img.write_to(&mut writer, ImgFmt::Png)
                .map_err(|e| Error::Other(format!("png encode: {e}")))?;
        }
        ImageFormat::Jpeg => {
            let rgb = img.to_rgb8();
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
        ImageFormat::Webp => {
            let rgba = img.to_rgba8();
            if quality >= 100 {
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut writer);
                encoder
                    .encode(
                        rgba.as_raw(),
                        rgba.width(),
                        rgba.height(),
                        image::ExtendedColorType::Rgba8,
                    )
                    .map_err(|e| Error::Other(format!("webp encode: {e}")))?;
            } else {
                let encoded = webp::Encoder::from_rgba(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                )
                .encode(f32::from(quality))
                .to_vec();
                use std::io::Write;
                writer
                    .write_all(&encoded)
                    .map_err(|e| Error::Io(e.to_string()))?;
            }
        }
        ImageFormat::Avif => {
            let rgba = img.to_rgba8();
            let encoder = image::codecs::avif::AvifEncoder::new_with_speed_quality(
                &mut writer,
                6,
                quality,
            );
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| Error::Other(format!("avif encode: {e}")))?;
        }
        ImageFormat::Heic => {
            return Err(Error::Unsupported("any".into(), "heic".into()));
        }
    }
    Ok(())
}
