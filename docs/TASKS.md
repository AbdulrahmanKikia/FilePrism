# FilePrism Task Board

Tiny, checkable items. Agents check off items when done; humans review at milestone boundaries.

## M0 — Scaffold

- [x] Init Tauri 2 + Svelte 5 + TypeScript desktop app (`apps/desktop`)
- [x] Rust workspace with `fileprism-core`
- [x] Crate stubs: formats-image/doc/data/archive, history, licensing, explorer-win
- [x] Design tokens CSS (`apps/desktop/src/lib/styles/tokens.css`)
- [x] Sidebar + Home layout (Glass Pro nav)
- [x] CI stubs: `cargo check`, `cargo test`, `pnpm build`
- [x] `docs/CONTRACTS.md` and `docs/TASKS.md`

## M1 — Core engine

- [x] Finalize `Converter` + `Job` / `Output` / progress types
- [x] Pipeline with progress + cancellation
- [x] SQLite history schema + migrations
- [x] Unit tests for pipeline

## M2 — Image formats

- [x] PNG→WebP, JPG→WebP
- [x] HEIC→PNG/JPG
- [x] AVIF encode/decode
- [x] Resize + quality options
- [x] Integration tests per pair

## M3 — Document & data

- [ ] PDF→PNG/JPG, PDF→text
- [ ] MD→HTML, MD→PDF (spike renderer first)
- [ ] JSON↔CSV, JSON↔YAML, CSV↔XLSX

## M4 — Archives

- [ ] ZIP / TAR create+extract
- [ ] 7z extract
- [ ] RAR extract via sidecar process
- [ ] Password-protected archive support

## M5 — UI polish

- [ ] Convert / Compress / Archives views
- [ ] History search / filter / re-run
- [ ] Settings (theme, accent, glass, density, motion, font)
- [ ] Command palette
- [ ] Animations + reduced-motion

## M6 — Explorer integration

- [ ] Rust DLL with `IExplorerCommand`
- [ ] Sparse package + named-pipe IPC
- [ ] Installer wiring; Linux `.desktop` actions

## M7 — Licensing & packaging

- [ ] Offline signed license validation
- [ ] Merchant integration; Windows installer; code signing; auto-update

## M8 — Web / WASM

- [ ] Compile core to WASM; web UI; ads; SEO landing pages
