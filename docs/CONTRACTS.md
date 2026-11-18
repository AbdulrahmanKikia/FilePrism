# FilePrism Contracts

Shared API and IPC schemas. **Any change to contracts requires a PR that updates this file.**

## Converter trait (Rust) — M1

```rust
pub trait Converter: Send + Sync {
    fn supports(&self, input: &str, output: &str) -> bool;
    fn convert(&self, job: &Job) -> Result<Output, Error>;
}
```

Defined in `fileprism-core`. Format crates implement this trait. First registered converter whose `supports` returns true wins.

## Job / Output (M2)

| Type     | Fields |
|----------|--------|
| `Job`    | `id`, `input_path`, `output_path`, `input_format`, `output_format`, `options` (`JobOptions`, default empty) |
| `Output` | `job_id`, `output_path`, `bytes_written` |

### `JobOptions` (M2 / M4)

| Field | Type | Notes |
|-------|------|-------|
| `quality` | `Option<u8>` | Encode quality `1..=100` (JPEG / WebP / AVIF). Default converter policy: 85 |
| `width` | `Option<u32>` | Target width; aspect preserved if height omitted |
| `height` | `Option<u32>` | Target height; aspect preserved if width omitted |
| `strip_metadata` | `Option<bool>` | Default `true` (re-encode without EXIF) |
| `password` | `Option<String>` | Archive password (ZIP AES / 7z / RAR). Default: none. Missing field deserializes as `null` |

Serialization: `serde` JSON for IPC between UI and core (Tauri commands). Missing `options` deserializes as default.

## Pipeline (M1)

`Pipeline` in `fileprism-core`:

| Method | Behavior |
|--------|----------|
| `register(converter)` | Append a `Converter` implementation |
| `find_converter(input, output)` | First match via `supports` |
| `run(job, cancel, on_progress)` | detect → validate → transform → write → complete |
| `run_silent(job, cancel)` | Same without progress callback |

Stages (`PipelineStage`): `detect`, `validate`, `transform`, `write`, `complete`.

Detect accepts an existing **file or directory** input (directories needed for archive create). Write verification accepts an output **file or directory** (directories for archive extract).

### Progress

```rust
pub struct Progress {
    pub job_id: String,
    pub stage: PipelineStage,
    pub fraction: f64,          // 0.0..=1.0
    pub message: Option<String>,
}
```

### Cancellation

```rust
pub struct CancellationToken { /* Arc<AtomicBool> */ }
// cancel(), is_cancelled(), check() -> Result<(), Error::Cancelled>
```

Checked between pipeline stages. Clones share the same flag.

### Errors (`fileprism_core::Error`)

`Unsupported`, `InputNotFound`, `Validation`, `Io`, `Cancelled`, `Other`.

## Image formats (M2) — `fileprism-formats-image`

`ImageConverter` implements `Converter`.

| | Formats |
|--|---------|
| **Inputs** | `png`, `jpg`/`jpeg`, `webp`, `avif`, `heic`/`heif` |
| **Outputs** | `png`, `jpg`/`jpeg`, `webp`, `avif` |

M2 checklist pairs covered: PNG→WebP, JPG→WebP, HEIC→PNG/JPG, AVIF encode/decode (plus other raster pairs among the sets above). SVG deferred.

Implementation notes:
- Decode/encode via `image` (+ `webp` for lossy WebP quality)
- HEIC decode via pure-Rust `heic-rs`
- AVIF encode via `ravif` (image `avif` feature); decode via `avif-decode` (rav1d). CI installs `nasm` for rav1d asm.

## Document formats (M3) — `fileprism-formats-doc`

`DocConverter` implements `Converter`.

| | Formats |
|--|---------|
| **Inputs** | `pdf`, `md`/`markdown` |
| **Outputs** | `png`, `jpg`/`jpeg`, `txt`/`text`, `html`/`htm`, `pdf` |

M3 checklist pairs: PDF→PNG/JPG, PDF→text, MD→HTML, MD→PDF (plain-text spike via `printpdf`).

Implementation notes:
- Markdown→HTML via `pulldown-cmark`
- Markdown→PDF spike: flatten markdown lines into Helvetica text with `printpdf` (no CSS layout)
- PDF→text via `pdf-extract`
- PDF→PNG/JPG: `pdfium-render` page rasterization; shared library for Pdfium 8125 is downloaded/cached at runtime (`PDFIUM_LIB_PATH` / `FILEPRISM_PDFIUM_CACHE` overrides)

## Data formats (M3) — `fileprism-formats-data`

`DataConverter` implements `Converter`.

| | Formats |
|--|---------|
| **Inputs** | `json`, `csv`, `yaml`/`yml`, `xlsx` |
| **Outputs** | `json`, `csv`, `yaml`/`yml`, `xlsx` |

M3 checklist pairs: JSON↔CSV, JSON↔YAML, CSV↔XLSX.

Implementation notes:
- JSON/CSV via `serde_json` + `csv`
- YAML via `serde_yaml`
- XLSX read via `calamine`; write via `rust_xlsxwriter`
- JSON→CSV requires a JSON array of objects (or equivalent tabular shape)

## Archive formats (M4) — `fileprism-formats-archive`

`ArchiveConverter` implements `Converter`.

| | Formats |
|--|---------|
| **Create inputs** | `file`, `dir` (`directory` / `folder`) |
| **Create outputs** | `zip`, `tar`, `7z` (`sevenz` / `7zip`) |
| **Extract inputs** | `zip`, `tar`, `7z`, `rar` |
| **Extract output** | `dir` (`directory` / `folder` / `extract`) |

M4 checklist: ZIP/TAR create+extract, 7z create+extract, RAR extract via sidecar, password-protected ZIP/7z/RAR.

Implementation notes:
- ZIP via `zip` crate (AES-256 when `JobOptions.password` set)
- TAR via `tar` (passwords rejected — format has no native encryption)
- 7z via pure-Rust `sevenz-rust2` (create + extract; AES when password set)
- RAR extract **only** via separate binary `fileprism-unrar-sidecar` (`crates/unrar-sidecar`, links `unrar` / UnRAR). Main archive crate never links UnRAR. Path: `FILEPRISM_UNRAR_SIDECAR`, else next to current exe, else `PATH`
- Create does not support RAR (licensing restriction)

## History schema (M1) — `fileprism-history`

SQLite via `rusqlite` (bundled). Migrations in `crates/history/migrations/`; version tracked in `schema_migrations`.

### Tables

**`jobs`**

| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT PK | Job id |
| `input_path` | TEXT NOT NULL | |
| `output_path` | TEXT NOT NULL | |
| `input_format` | TEXT NOT NULL | |
| `output_format` | TEXT NOT NULL | |
| `status` | TEXT NOT NULL | `pending` \| `running` \| `completed` \| `failed` \| `cancelled` |
| `bytes_written` | INTEGER | nullable until complete |
| `error_message` | TEXT | nullable |
| `created_at` | TEXT NOT NULL | UTC `YYYY-MM-DDTHH:MM:SSZ` |
| `completed_at` | TEXT | nullable |

**`files`**

| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | autoincrement |
| `job_id` | TEXT NOT NULL | FK → `jobs(id)` ON DELETE CASCADE |
| `path` | TEXT NOT NULL | |
| `role` | TEXT NOT NULL | `input` \| `output` |
| `format` | TEXT | |
| `size_bytes` | INTEGER | |
| UNIQUE(`job_id`, `role`, `path`) | | |

**`presets`**

| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT PK | |
| `name` | TEXT NOT NULL UNIQUE | |
| `input_format` | TEXT NOT NULL | |
| `output_format` | TEXT NOT NULL | |
| `options_json` | TEXT NOT NULL | default `{}` |
| `created_at` | TEXT NOT NULL | |
| `updated_at` | TEXT NOT NULL | |

**`settings`**

| Column | Type | Notes |
|--------|------|-------|
| `key` | TEXT PK | |
| `value` | TEXT NOT NULL | |
| `updated_at` | TEXT NOT NULL | |

Well-known key: `history.enabled` — when `"false"` / `"0"`, `HistoryStore` skips job recording (privacy mode).

### `HistoryStore` API (M1)

| Method | Notes |
|--------|-------|
| `open(path)` / `open_in_memory()` | Runs migrations |
| `record_job_start(job, status)` | Inserts job + input/output file rows |
| `record_job_complete(output)` | Sets completed + bytes |
| `record_job_finish(id, status, error?)` | Terminal failed/cancelled/completed |
| `get_job` / `search_jobs` / `list_files_for_job` | Read path |
| `upsert_preset` / `list_presets` / `delete_preset` | Presets |
| `set_setting` / `get_setting` / `list_settings` | Settings |
| `history_enabled` | Privacy mode check |

History recording is **not** inside `Pipeline` (core must not depend on history). Callers (desktop/Tauri) invoke the store around `Pipeline::run`.

## Tauri commands (M5)

Desktop shell (`apps/desktop/src-tauri`) registers converters from image / doc / data / archive crates into a `Pipeline`, and opens SQLite history under the app data dir (`history.db`).

| Command | Args | Returns | Notes |
|---------|------|---------|-------|
| `greet` | `name: string` | `string` | Smoke / scaffold |
| `core_name` | — | `string` | Returns `fileprism-core` name |
| `run_job` | `args: { inputPath, outputPath, inputFormat, outputFormat, options?, id? }` | `Output` | Runs pipeline; records history when `history.enabled` |
| `search_history` | `query: string`, `limit?: number` | `JobRecord[]` | Empty query → recent jobs |
| `get_history_job` | `id: string` | `JobRecord?` | |
| `rerun_job` | `id: string` | `Output` | New job id; same paths/formats; default options |
| `list_settings` | — | `Setting[]` | |
| `get_setting` | `key: string` | `string?` | |
| `set_setting` | `key: string`, `value: string` | — | |

### UI settings keys (M5)

Persisted via `settings` table; applied as CSS `data-*` on the shell:

| Key | Values |
|-----|--------|
| `ui.theme` | `system` \| `light` \| `dark` |
| `ui.accent` | `blue` \| `violet` \| `teal` \| `orange` \| `pink` \| `green` |
| `ui.glass` | `off` \| `subtle` \| `full` |
| `ui.density` | `comfortable` \| `compact` |
| `ui.motion` | `full` \| `reduced` |
| `ui.font` | `system` \| `small` \| `large` |

Also: `history.enabled` — `"false"` / `"0"` skips job recording (privacy mode).

Frontend invoke wrappers live in `apps/desktop/src/lib/api.ts`. Command palette: Ctrl/⌘K.

CI note: `cargo test --workspace --exclude fileprism-desktop` — the desktop `cdylib` link pulls rav1d/dav1d without `-fPIC` on Linux (rust-lld `R_X86_64_PC32`). Desktop remains covered by `cargo check` + `pnpm build`.

## Explorer / shell IPC (M6)

### Transport

| Platform | Transport | Name |
|----------|-----------|------|
| Windows | Named pipe | `\\.\pipe\fileprism-shell-v1` (leaf `fileprism-shell-v1`) |
| Linux / macOS | Local socket (same leaf name via `interprocess`) | `fileprism-shell-v1` |

Also accepted at process start: CLI `--shell-open <action> <paths…>` (used when the pipe is down, and by Linux `.desktop` actions).

### Message (one UTF-8 JSON line, `\n`-terminated)

```json
{ "v": 1, "action": "open", "paths": ["C:\\abs\\file.png", "D:\\other.jpg"] }
```

| Field | Type | Notes |
|-------|------|-------|
| `v` | `u32` | Must be `1` (`SHELL_IPC_VERSION`) |
| `action` | string | `open` \| `convert` \| `compress` \| `extract` (default `open`) |
| `paths` | `string[]` | Absolute filesystem paths; must be non-empty |

Defined in `fileprism-explorer-win::ipc` (`ShellOpenMessage`, `ShellAction`). Desktop listens in `apps/desktop/src-tauri/src/shell_ipc.rs` and emits the Tauri event `shell-files` with the same payload. Frontend store: `apps/desktop/src/lib/shell.svelte.ts`.

### Windows Explorer DLL

- Crate: `fileprism-explorer-win` (`cdylib` + `rlib`)
- COM: `IExplorerCommand` + `IClassFactory`
- CLSID: `{8F3A2B1C-6D4E-4F9A-9B2C-1E5D7A8F3C4E}`
- On Invoke: collect `SIGDN_FILESYSPATH` paths → named-pipe handoff → fall back to `ShellExecuteW` of `FilePrism.exe --shell-open …`
- Sparse package manifest: `packaging/windows/sparse/AppxManifest.xml`
- Dev register/unregister: `packaging/windows/register-sparse-package.ps1` / `unregister-sparse-package.ps1`
- NSIS hooks: `packaging/windows/nsis-hooks.nsh` (post-install best-effort sparse register; pre-uninstall remove)
- Installer targets: NSIS + MSI (`tauri.conf.json` `bundle.targets`)

### Linux desktop actions

- File: `packaging/linux/fileprism.desktop`
- Actions: Convert / Compress / Extract → `fileprism --shell-open <action> %F`
- Install notes: `packaging/linux/README.md`

## License & entitlement (M7) — `fileprism-licensing`

### Key format

```
FP1.<base64url(payload_json)>.<base64url(ed25519_sig)>
```

Signature covers raw UTF-8 payload bytes. Payload JSON:

| Field | Type | Notes |
|-------|------|-------|
| `v` | `u32` | Must be `1` (`PAYLOAD_VERSION`) |
| `lid` | string | License id |
| `email` | string? | Optional |
| `issued` | `i64` | Unix seconds UTC |
| `updates_until` | `i64` | Unix seconds; typically issue + 1 year |

Verifying key: embedded `EMBEDDED_PUBLIC_KEY_HEX`, override `FILEPRISM_LICENSE_PUBKEY`. Signing seed: secret `FILEPRISM_LICENSE_SIGNING_SEED` only (feature `mint`).

### Trial

| Constant | Value |
|----------|-------|
| `TRIAL_DAYS` | `14` (full-feature; spec §7 allows 7–14) |

Settings keys: `license.trial_started_at` (unix secs string), `license.key` (full `FP1.…` string).

### Entitlement (`resolve_entitlement`)

| `kind.type` | `fullFeature` | Notes |
|-------------|---------------|-------|
| `licensed` | `true` | Fields: `lid`, `email?`, `updatesActive`, `updatesUntil` |
| `trial` | `true` | `daysRemaining`, `endsAt` |
| `trialExpired` | `false` | |
| `notStarted` | `false` | Host should stamp trial start |

### Tauri commands (M7)

| Command | Args | Returns |
|---------|------|---------|
| `license_status` | — | `{ entitlement, hasStoredKey, checkoutUrl, trialDays, maskedKey }` |
| `activate_license` | `key: string` | same as status (after verify + persist) |
| `deactivate_license` | — | status with key cleared |
| `purchase_url` | — | `string` (`FILEPRISM_CHECKOUT_URL` or Paddle placeholder) |

Merchant: **Paddle** primary, Lemon Squeezy alternate — stubs in `docs/licensing/merchant.md` (no live secrets in repo).

### Packaging / updater (M7)

- Code signing docs: `docs/packaging/code-signing.md`
- Auto-update: `tauri-plugin-updater` + `docs/packaging/auto-update.md`
- Release workflow hooks: `.github/workflows/release.yml` (secrets for certs / updater key)
