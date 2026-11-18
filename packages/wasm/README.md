# `@fileprism/wasm`

Browser bindings for FilePrism image conversion (PNG / JPG / WebP). Built from
`crates/wasm` via `wasm-bindgen`.

## Lane B import

From the monorepo (pnpm workspace includes `packages/*`):

```ts
import init, {
  convert_image,
  list_image_pairs,
  is_desktop_only,
} from "@fileprism/wasm";

await init(); // loads fileprism_wasm_bg.wasm next to the JS glue

const result = convert_image(inputBytes, "png", "webp", {
  quality: 85,
  width: undefined,
  height: undefined,
});
// result.bytes: Uint8Array
// result.mimeType / result.mime_type: "image/webp"
```

### Vite / SvelteKit (`apps/web`)

1. Depend on the workspace package (already listed under `packages/`):

   ```bash
   pnpm --filter @fileprism/web add @fileprism/wasm@workspace:*
   ```

2. Set the engine module (Lane B resolver):

   ```env
   VITE_WASM_MODULE=@fileprism/wasm
   ```

   Or copy `packages/wasm/pkg/*` into `apps/web/static/wasm/` and set
   `VITE_WASM_MODULE=/wasm/fileprism_wasm.js`.

3. Fallback: leave `VITE_WASM_MODULE` unset to keep the canvas stub.

### Rebuild (Windows cooler: `-j 1`)

```bash
# from repo root
pnpm --filter @fileprism/wasm build:release
# or
node packages/wasm/scripts/build.mjs --release
```

Requires: `rustup target add wasm32-unknown-unknown`, `cargo install wasm-bindgen-cli`.

## Desktop-only formats

`is_desktop_only(fmt)` is `true` for RAR, HEIC/AVIF, PDF→image (pdfium), archives,
Explorer, etc. Web UI should show “desktop only” rather than attempting convert.

## API

| Export | Notes |
|--------|-------|
| `default` / `init` | wasm-bindgen loader (async) |
| `convert_image(bytes, in, out, options?)` | Returns `{ bytes, mimeType, mime_type, output_format, bytes_written }` |
| `list_image_pairs()` | `string[]` like `"png→webp"` |
| `is_desktop_only(format)` | boolean |
| `crate_name()` | `"fileprism-wasm"` |
