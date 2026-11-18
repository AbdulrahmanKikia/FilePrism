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
 * Lane A: `@fileprism/wasm` (default). Set `VITE_WASM_MODULE` to override
 * (e.g. `/wasm/fileprism_wasm.js`). On load failure, falls back to canvas stub.
 */
let override: ConvertEngine | null = null;
let cached: ConvertEngine | null = null;

/** Test / Lane A hook: inject an engine explicitly. */
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
  if (typeof window === "undefined") return null;

  const spec =
    (import.meta.env.VITE_WASM_MODULE as string | undefined)?.trim() || "@fileprism/wasm";

  try {
    const mod =
      spec === "@fileprism/wasm"
        ? await import("@fileprism/wasm")
        : await import(/* @vite-ignore */ spec);

    const init = mod.default ?? mod.init;
    if (typeof init === "function") {
      await init();
    }

    const convert = mod.convert_image ?? mod.convertImage ?? mod.convert;
    if (typeof convert !== "function") {
      console.warn("[fileprism] WASM module missing convert_image; using canvas stub.");
      return null;
    }

    return {
      id: "wasm",
      ready: true,
      async convert(input, inputFormat, outputFormat, options) {
        const raw = convert(input, inputFormat, outputFormat, options ?? {});
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
          bytesWritten: Number(raw.bytes_written ?? bytes.byteLength),
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
