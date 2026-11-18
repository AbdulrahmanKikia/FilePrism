//! [`ImageConverter`] implementation.

use std::io::Cursor;
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
        if !input_fmt.is_input() || !output_fmt.is_output() {
            return Err(Error::Unsupported(
                job.input_format.clone(),
                job.output_format.clone(),
            ));
        }

        let data = std::fs::read(&job.input_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::InputNotFound(job.input_path.clone())
            } else {
                Error::Io(e.to_string())
            }
        })?;
        let encoded = convert_bytes(&data, input_fmt, output_fmt, &job.options)?;

        if let Some(parent) = Path::new(&job.output_path).parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    Error::Io(format!("create output dir {}: {e}", parent.display()))
                })?;
            }
        }
        std::fs::write(&job.output_path, &encoded).map_err(|e| Error::Io(e.to_string()))?;

        Ok(Output {
            job_id: job.id.clone(),
            output_path: job.output_path.clone(),
            bytes_written: encoded.len() as u64,
        })
    }

    /// In-memory convert for WASM / browser (no filesystem).
    pub fn convert_bytes(
        input: &[u8],
        input_format: &str,
        output_format: &str,
        options: &JobOptions,
    ) -> Result<Vec<u8>, Error> {
        let input_fmt = normalize_format(input_format)
            .ok_or_else(|| Error::Unsupported(input_format.into(), output_format.into()))?;
        let output_fmt = normalize_format(output_format)
            .ok_or_else(|| Error::Unsupported(input_format.into(), output_format.into()))?;
        if !input_fmt.is_input() || !output_fmt.is_output() {
            return Err(Error::Unsupported(input_format.into(), output_format.into()));
        }
        convert_bytes(input, input_fmt, output_fmt, options)
    }
}

fn convert_bytes(
    input: &[u8],
    input_fmt: ImageFormat,
    output_fmt: ImageFormat,
    options: &JobOptions,
) -> Result<Vec<u8>, Error> {
    if input.is_empty() {
        return Err(Error::Validation("input bytes are empty".into()));
    }
    let mut img = decode_image_bytes(input, input_fmt)?;
    img = maybe_resize(img, options);
    let _strip = effective_strip_metadata(options);
    let quality = effective_quality(options);
    encode_image_bytes(&img, output_fmt, quality)
}

fn maybe_resize(img: DynamicImage, options: &JobOptions) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    match resize_dimensions(w, h, options) {
        Some((tw, th)) if tw != w || th != h => img.resize_exact(tw, th, FilterType::Lanczos3),
        _ => img,
    }
}

fn decode_image_bytes(data: &[u8], format: ImageFormat) -> Result<DynamicImage, Error> {
    match format {
        #[cfg(feature = "heic")]
        ImageFormat::Heic => decode_heic(data),
        #[cfg(not(feature = "heic"))]
        ImageFormat::Heic => Err(Error::Unsupported("heic".into(), "any".into())),
        #[cfg(feature = "avif")]
        ImageFormat::Avif => decode_avif(data),
        #[cfg(not(feature = "avif"))]
        ImageFormat::Avif => Err(Error::Unsupported("avif".into(), "any".into())),
        other => {
            let decoded = image::load_from_memory_with_format(data, to_image_crate_format(other)?)
                .or_else(|_| image::load_from_memory(data));
            decoded.map_err(|e| Error::Other(format!("decode {}: {e}", format.as_str())))
        }
    }
}

#[cfg(feature = "avif")]
fn decode_avif(data: &[u8]) -> Result<DynamicImage, Error> {
    let decoder = avif_decode::Decoder::from_avif(data)
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

#[cfg(feature = "heic")]
fn decode_heic(data: &[u8]) -> Result<DynamicImage, Error> {
    let options = heic_rs::DecodeOptions {
        layout: heic_rs::PixelLayout::Rgba8,
        ..heic_rs::DecodeOptions::default()
    };
    let decoded =
        heic_rs::decode(data, &options).map_err(|e| Error::Other(format!("heic decode: {e}")))?;

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

fn encode_image_bytes(
    img: &DynamicImage,
    format: ImageFormat,
    quality: u8,
) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    let mut writer = Cursor::new(&mut buf);

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
            encode_webp(img, &mut writer, quality)?;
        }
        #[cfg(feature = "avif")]
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
        #[cfg(not(feature = "avif"))]
        ImageFormat::Avif => {
            return Err(Error::Unsupported("any".into(), "avif".into()));
        }
        ImageFormat::Heic => {
            return Err(Error::Unsupported("any".into(), "heic".into()));
        }
    }
    Ok(buf)
}

fn encode_webp(
    img: &DynamicImage,
    writer: &mut Cursor<&mut Vec<u8>>,
    quality: u8,
) -> Result<(), Error> {
    let rgba = img.to_rgba8();
    #[cfg(feature = "webp-lossy")]
    {
        if quality >= 100 {
            let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut *writer);
            encoder
                .encode(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| Error::Other(format!("webp encode: {e}")))?;
        } else {
            let encoded =
                webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height())
                    .encode(f32::from(quality))
                    .to_vec();
            use std::io::Write;
            writer
                .write_all(&encoded)
                .map_err(|e| Error::Io(e.to_string()))?;
        }
        return Ok(());
    }
    #[cfg(not(feature = "webp-lossy"))]
    {
        // Pure-Rust path for WASM: image crate WebP encoder is lossless-only.
        let _ = quality;
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut *writer);
        encoder
            .encode(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| Error::Other(format!("webp encode: {e}")))?;
        Ok(())
    }
}
