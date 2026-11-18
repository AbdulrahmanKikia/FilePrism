/** Shared convert contract for WASM bindings (Lane A) and the canvas stub. */

export type ImageFormat = "png" | "jpg" | "webp";

export type ConvertOptions = {
  quality?: number;
  width?: number;
  height?: number;
};

export type ConvertResult = {
  bytes: Uint8Array;
  mimeType: string;
  outputFormat: ImageFormat;
  bytesWritten: number;
  engine: "wasm" | "canvas-stub";
};

export type ConvertEngine = {
  id: "wasm" | "canvas-stub";
  ready: boolean;
  convert(
    input: Uint8Array,
    inputFormat: ImageFormat,
    outputFormat: ImageFormat,
    options?: ConvertOptions,
  ): Promise<ConvertResult>;
};

export const WEB_IMAGE_FORMATS: ImageFormat[] = ["png", "jpg", "webp"];

export const MIME_BY_FORMAT: Record<ImageFormat, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  webp: "image/webp",
};

export const EXT_BY_FORMAT: Record<ImageFormat, string> = {
  png: "png",
  jpg: "jpg",
  webp: "webp",
};

export function normalizeImageFormat(value: string): ImageFormat | null {
  const v = value.toLowerCase().replace(/^\./, "");
  if (v === "jpeg") return "jpg";
  if (v === "png" || v === "jpg" || v === "webp") return v;
  return null;
}
