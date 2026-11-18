# @fileprism/web

Client-side FilePrism web app (SvelteKit + TypeScript). Convert images in the browser; files never upload.

## Routes (Lane B)

- `/` — home
- `/convert` — file picker → convert → download (PNG/JPG/WebP)
- `/history` — localStorage job list
- `/settings` — Glass Pro theme tokens (aligned with desktop)

Lane C owns SEO/landing routes under `apps/web` (pair landings / ad slots). Prefer adding new route folders without rewriting Convert/History/Settings.

## WASM (Lane A)

Depends on workspace package `@fileprism/wasm`.

```env
VITE_WASM_MODULE=@fileprism/wasm
```

`src/lib/wasm/index.ts` calls `await init()` then `convert_image(...)`. On load failure it falls back to the canvas stub.

Rebuild bindings (Windows cooler `-j 1`):

```bash
pnpm --filter @fileprism/wasm build:release
```

## Develop

```bash
pnpm install
pnpm --filter @fileprism/web dev
pnpm --filter @fileprism/web build
```
