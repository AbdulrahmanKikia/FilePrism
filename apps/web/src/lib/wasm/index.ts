import { canvasStubEngine } from "./canvas-stub";
import type { ConvertEngine, ConvertOptions, ConvertResult, ImageFormat } from "./types";

export type {
  ConvertEngine,
  ConvertOptions,
  ConvertResult,
  ImageFormat,
} from "./types";
export {
  EXT_BY_FORMAT,
  MIME_BY_FORMAT,
  WEB_IMAGE_FORMATS,
  normalizeImageFormat,
} from "./types";

/**
 * Convert engine resolver.
 *
 * Lane A swap path:
 * 1. Publish/build wasm-bindgen output under `static/wasm/` (or `@fileprism/wasm`).
 * 2. Set `VITE_WASM_MODULE=/wasm/fileprism_core.js` (or package name).
 * 3. Expose `init` + `convert_image(bytes, inputFmt, outputFmt, options)`.
 *
 * Until then, the canvas stub fulfills PNG/JPG/WebP Convert MVP.
 */
let override: ConvertEngine | null = null;
let cached: ConvertEngine | null = null;

/** Test / Lane A hook: inject the real engine once bindings exist. */
export function setConvertEngine(engine: ConvertEngine | null): void {
  override = engine;
  cached = null;
}

export async function getConvertEngine(): Promise<ConvertEngine> {
  if (override) return override;
  if (cached) return cached;

  const wasm = await tryLoadWasmEngine();
  cached = wasm ?? canvasStubEngine;
  return cached;
}

export async function convertImage(
  input: Uint8Array,
  inputFormat: ImageFormat,
  outputFormat: ImageFormat,
  options?: ConvertOptions,
): Promise<ConvertResult> {
  const engine = await getConvertEngine();
  return engine.convert(input, inputFormat, outputFormat, options);
}

async function tryLoadWasmEngine(): Promise<ConvertEngine | null> {
  const spec = (import.meta.env.VITE_WASM_MODULE as string | undefined)?.trim();
  if (!spec) return null;

  try {
    const mod = await import(/* @vite-ignore */ spec);
    if (typeof mod.default === "function") await mod.default();
    else if (typeof mod.init === "function") await mod.init();

    const convert = mod.convert_image ?? mod.convertImage ?? mod.convert;
    if (typeof convert !== "function") return null;

    return {
      id: "wasm",
      ready: true,
      async convert(input, inputFormat, outputFormat, options) {
        const raw = await convert(input, inputFormat, outputFormat, options ?? {});
        if (raw instanceof Uint8Array) {
          return {
            bytes: raw,
            mimeType: mimeFor(outputFormat),
            outputFormat,
            bytesWritten: raw.byteLength,
            engine: "wasm",
          };
        }
        const bytes: Uint8Array = raw.bytes ?? raw.data;
        return {
          bytes,
          mimeType: raw.mimeType ?? raw.mime_type ?? mimeFor(outputFormat),
          outputFormat,
          bytesWritten: bytes.byteLength,
          engine: "wasm",
        };
      },
    };
  } catch (err) {
    console.warn("[fileprism] WASM module failed to load; using canvas stub.", err);
    return null;
  }
}

function mimeFor(format: ImageFormat): string {
  if (format === "jpg") return "image/jpeg";
  if (format === "webp") return "image/webp";
  return "image/png";
}
