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

### `JobOptions` (M2)

| Field | Type | Notes |
|-------|------|-------|
| `quality` | `Option<u8>` | Encode quality `1..=100` (JPEG / WebP / AVIF). Default converter policy: 85 |
| `width` | `Option<u32>` | Target width; aspect preserved if height omitted |
| `height` | `Option<u32>` | Target height; aspect preserved if width omitted |
| `strip_metadata` | `Option<bool>` | Default `true` (re-encode without EXIF) |

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

## Tauri commands (M0; unchanged)

| Command     | Args           | Returns   | Notes                          |
|-------------|----------------|-----------|--------------------------------|
| `greet`     | `name: string` | `string`  | Smoke / scaffold only          |
| `core_name` | —              | `string`  | Returns `fileprism-core` name  |

## Explorer IPC (planned M6)

Named-pipe payload: list of absolute file paths from Windows shell extension to desktop app. Schema TBD in M6.

## License key (planned M7)

Offline Ed25519-signed license blob. Validation API TBD in M7.
