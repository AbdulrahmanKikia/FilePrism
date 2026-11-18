# @fileprism/web

Client-side FilePrism web app (SvelteKit + TypeScript). Convert images in the browser; files never upload.

## Routes (Lane B)

- `/` — home
- `/convert` — file picker → convert → download (PNG/JPG/WebP)
- `/history` — localStorage job list
- `/settings` — Glass Pro theme tokens (aligned with desktop)

Lane C owns SEO/landing routes under `apps/web` (pair landings / ad slots). Prefer adding new route folders without rewriting Convert/History/Settings.

## WASM

Until Lane A ships bindings, Convert uses a **canvas stub** (`src/lib/wasm/canvas-stub.ts`).

Swap path:

1. Drop wasm-bindgen output (or package) and set `VITE_WASM_MODULE` (e.g. `/wasm/fileprism_core.js`).
2. Expose `init` + `convert_image(bytes, inputFmt, outputFmt, options)`.
3. Or call `setConvertEngine(...)` from `src/lib/wasm/index.ts`.

## Develop

```bash
pnpm install
pnpm --filter @fileprism/web dev
pnpm --filter @fileprism/web build
```
