# FilePrism

Local-first desktop file conversion and compression for Windows, Linux, and macOS.

Files never leave the device by default. Shared Rust core (`fileprism-core`) powers the Tauri desktop app today and a future WASM web client.

## Stack

- **Shell:** Tauri 2
- **UI:** Svelte 5 + TypeScript (Glass Pro)
- **Core:** Rust workspace

## Layout

```
crates/            # core, formats, history, licensing, explorer-win (shell DLL)
apps/desktop/      # Tauri + SvelteKit desktop app
apps/web/          # SvelteKit web client (browser convert via WASM / canvas stub)
packaging/         # Windows sparse package + Linux .desktop actions
docs/              # CONTRACTS.md, TASKS.md
```

## Develop

Prerequisites: Rust stable, Node 20+, pnpm 9+, platform Tauri deps.

```bash
pnpm install
cargo check --workspace
cargo test --workspace --exclude fileprism-desktop --exclude fileprism-wasm
cargo test -p fileprism-wasm   # isolated: no AVIF/dav1d feature-unify
pnpm build
pnpm tauri:dev
pnpm dev:web
```

Linux CI excludes `fileprism-desktop` and `fileprism-wasm` from workspace `cargo test` (host `cdylib` + rav1d/dav1d `-fPIC` / `R_X86_64_PC32`). WASM unit tests run via `cargo test -p fileprism-wasm`. Browser artifacts: `pnpm --filter @fileprism/wasm build:release`.

## Shell integration (M6)

- **Windows:** build `fileprism-explorer-win`, then run `packaging/windows/register-sparse-package.ps1` (dev). Context menu sends paths over `\\.\pipe\fileprism-shell-v1`.
- **Linux:** install `packaging/linux/fileprism.desktop` (see `packaging/linux/README.md`).

## Spec

Product & engineering baseline through M6 (Explorer / shell integration). See `docs/TASKS.md` and `docs/CONTRACTS.md`.
