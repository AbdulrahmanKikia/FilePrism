# FilePrism

Local-first desktop file conversion and compression for Windows, Linux, and macOS.

Files never leave the device by default. Shared Rust core (`fileprism-core`) powers the Tauri desktop app today and a future WASM web client.

## Stack

- **Shell:** Tauri 2
- **UI:** Svelte 5 + TypeScript (Glass Pro)
- **Core:** Rust workspace

## Layout

```
crates/          # fileprism-core + format / history / licensing / explorer stubs
apps/desktop/    # Tauri + SvelteKit desktop app
docs/            # CONTRACTS.md, TASKS.md
```

## Develop

Prerequisites: Rust stable, Node 20+, pnpm 9+, platform Tauri deps.

```bash
pnpm install
cargo check --workspace
cargo test --workspace --exclude fileprism-desktop
pnpm build
pnpm tauri:dev
```

## Spec

Product & engineering baseline: M0–M2 (scaffold, core engine, image formats). See `docs/TASKS.md`.
