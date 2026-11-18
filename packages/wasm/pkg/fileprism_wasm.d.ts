/* tslint:disable */
/* eslint-disable */

/**
 * Convert image bytes in-memory.
 *
 * `options` is a plain object: `{ quality?, width?, height?, strip_metadata? }`.
 * Returns `{ bytes: Uint8Array, mime_type, output_format, bytes_written }`.
 */
export function convert_image(input: Uint8Array | number[], input_format: string, output_format: string, options: any): any;

/**
 * Crate identity for smoke checks.
 */
export function crate_name(): string;

/**
 * Explicit init for bundlers that call `await init()` / `await default()`.
 */
export function init_sync(): void;

/**
 * Formats that require the desktop app (native sidecars / Explorer / pdfium).
 */
export function is_desktop_only(format: string): boolean;

/**
 * List supported `input→output` pairs for the WASM feature set (PNG/JPG/WebP).
 */
export function list_image_pairs(): any;

/**
 * Initialize the WASM module (panic hook). Safe to call multiple times.
 */
export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly convert_image: (a: number, b: number, c: number, d: number, e: number, f: number, g: any) => [number, number, number];
    readonly crate_name: () => [number, number];
    readonly init_sync: () => void;
    readonly is_desktop_only: (a: number, b: number) => number;
    readonly list_image_pairs: () => any;
    readonly start: () => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
