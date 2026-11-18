import {
  MIME_BY_FORMAT,
  type ConvertEngine,
  type ConvertOptions,
  type ConvertResult,
  type ImageFormat,
} from "./types";

/**
 * Temporary browser convert path until Lane A ships wasm-bindgen.
 * Handles PNG / JPG / WebP via Canvas — swap out when real WASM is ready.
 */
export const canvasStubEngine: ConvertEngine = {
  id: "canvas-stub",
  ready: true,
  async convert(input, _inputFormat, outputFormat, options = {}) {
    return convertWithCanvas(input, outputFormat, options);
  },
};

async function convertWithCanvas(
  input: Uint8Array,
  outputFormat: ImageFormat,
  options: ConvertOptions,
): Promise<ConvertResult> {
  const blob = new Blob([input.slice()], { type: "application/octet-stream" });
  const bitmap = await createImageBitmap(blob);

  try {
    const srcW = bitmap.width;
    const srcH = bitmap.height;
    let width = options.width && options.width > 0 ? Math.floor(options.width) : srcW;
    let height = options.height && options.height > 0 ? Math.floor(options.height) : srcH;

    if (options.width && options.width > 0 && !(options.height && options.height > 0)) {
      height = Math.max(1, Math.round((srcH / srcW) * width));
    } else if (options.height && options.height > 0 && !(options.width && options.width > 0)) {
      width = Math.max(1, Math.round((srcW / srcH) * height));
    }

    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Canvas 2D context unavailable in this browser.");
    }
    ctx.drawImage(bitmap, 0, 0, width, height);

    const mimeType = MIME_BY_FORMAT[outputFormat];
    const quality =
      outputFormat === "png"
        ? undefined
        : clampQuality(options.quality);

    const outBlob = await new Promise<Blob>((resolve, reject) => {
      canvas.toBlob(
        (b) => (b ? resolve(b) : reject(new Error(`Browser could not encode ${outputFormat}.`))),
        mimeType,
        quality,
      );
    });

    const buffer = new Uint8Array(await outBlob.arrayBuffer());
    return {
      bytes: buffer,
      mimeType,
      outputFormat,
      bytesWritten: buffer.byteLength,
      engine: "canvas-stub",
    };
  } finally {
    bitmap.close();
  }
}

function clampQuality(quality?: number): number {
  if (quality == null || Number.isNaN(quality)) return 0.92;
  if (quality > 1) return Math.min(1, Math.max(0.01, quality / 100));
  return Math.min(1, Math.max(0.01, quality));
}
