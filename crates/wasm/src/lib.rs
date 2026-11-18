//! Browser WASM bindings for FilePrism image conversion.
//!
//! Built with `wasm-bindgen` / `wasm-pack` and published as `@fileprism/wasm`
//! under `packages/wasm`. Lane B loads via `VITE_WASM_MODULE=@fileprism/wasm`
//! (or a copied `/wasm/…` URL).

use fileprism_core::JobOptions;
use fileprism_formats_image::{normalize_format, supported_pairs, ImageConverter, ImageFormat};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

/// Optional convert knobs (matches desktop `JobOptions` / Lane B `ConvertOptions`).
#[derive(Debug, Default, Deserialize)]
pub struct WasmConvertOptions {
    pub quality: Option<u8>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub strip_metadata: Option<bool>,
}

impl From<WasmConvertOptions> for JobOptions {
    fn from(value: WasmConvertOptions) -> Self {
        JobOptions {
            quality: value.quality,
            width: value.width,
            height: value.height,
            strip_metadata: value.strip_metadata,
            password: None,
        }
    }
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

fn to_js_error(err: fileprism_core::Error) -> JsValue {
    JsValue::from_str(&err.to_string())
}

/// Initialize the WASM module (panic hook). Safe to call multiple times.
#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
}

/// Explicit init for bundlers that call `await init()` / `await default()`.
#[wasm_bindgen]
pub fn init_sync() {
    set_panic_hook();
}

/// Convert image bytes in-memory.
///
/// `options` is a plain object: `{ quality?, width?, height?, strip_metadata? }`.
/// Returns `{ bytes: Uint8Array, mime_type, output_format, bytes_written }`.
#[wasm_bindgen]
pub fn convert_image(
    input: &[u8],
    input_format: &str,
    output_format: &str,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let opts: WasmConvertOptions = if options.is_undefined() || options.is_null() {
        WasmConvertOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| JsValue::from_str(&format!("invalid options: {e}")))?
    };

    let bytes = ImageConverter::convert_bytes(
        input,
        input_format,
        output_format,
        &JobOptions::from(opts),
    )
    .map_err(to_js_error)?;

    let out_fmt = normalize_format(output_format).unwrap_or(ImageFormat::Png);
    let result = js_sys::Object::new();
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("bytes"),
        &js_sys::Uint8Array::from(bytes.as_slice()).into(),
    )?;
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("mime_type"),
        &JsValue::from_str(out_fmt.mime_type()),
    )?;
    // CamelCase alias for Lane B (`raw.mimeType ?? raw.mime_type`).
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("mimeType"),
        &JsValue::from_str(out_fmt.mime_type()),
    )?;
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("output_format"),
        &JsValue::from_str(out_fmt.as_str()),
    )?;
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("bytes_written"),
        &JsValue::from_f64(bytes.len() as f64),
    )?;
    Ok(result.into())
}

/// List supported `input→output` pairs for the WASM feature set (PNG/JPG/WebP).
#[wasm_bindgen]
pub fn list_image_pairs() -> JsValue {
    let pairs: Vec<String> = supported_pairs()
        .into_iter()
        .map(|(a, b)| format!("{}→{}", a.as_str(), b.as_str()))
        .collect();
    serde_wasm_bindgen::to_value(&pairs).unwrap_or(JsValue::NULL)
}

/// Formats that require the desktop app (native sidecars / Explorer / pdfium).
#[wasm_bindgen]
pub fn is_desktop_only(format: &str) -> bool {
    matches!(
        format.trim().to_ascii_lowercase().as_str(),
        "rar"
            | "heic"
            | "heif"
            | "avif"
            | "pdf"
            | "explorer"
            | "7z"
            | "sevenz"
            | "7zip"
            | "zip"
            | "tar"
            | "xlsx"
            | "xls"
    )
}

/// Crate identity for smoke checks.
#[wasm_bindgen]
pub fn crate_name() -> String {
    "fileprism-wasm".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fileprism_core::JobOptions;
    use image::{ImageBuffer, Rgb};

    #[test]
    fn png_to_webp_bytes() {
        let img: ImageBuffer<Rgb<u8>, _> =
            ImageBuffer::from_fn(8, 8, |x, y| Rgb([(x * 20) as u8, (y * 20) as u8, 128]));
        let mut png = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png),
            image::ImageFormat::Png,
        )
        .expect("encode fixture png");

        let out = ImageConverter::convert_bytes(&png, "png", "webp", &JobOptions::default())
            .expect("convert");
        assert!(!out.is_empty());
        assert!(ImageConverter::supports_pair("png", "webp"));
        // HEIC is feature-gated off in this crate's dependency set.
        assert!(!ImageConverter::supports_pair("heic", "png"));
    }
}
