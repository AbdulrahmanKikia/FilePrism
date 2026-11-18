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
cargo test --workspace --exclude fileprism-desktop
pnpm build
pnpm tauri:dev
pnpm dev:web
```

## Shell integration (M6)

- **Windows:** build `fileprism-explorer-win`, then run `packaging/windows/register-sparse-package.ps1` (dev). Context menu sends paths over `\\.\pipe\fileprism-shell-v1`.
- **Linux:** install `packaging/linux/fileprism.desktop` (see `packaging/linux/README.md`).

## Spec

Product & engineering baseline through M6 (Explorer / shell integration). See `docs/TASKS.md` and `docs/CONTRACTS.md`.
