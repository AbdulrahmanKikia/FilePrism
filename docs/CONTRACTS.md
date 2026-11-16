# FilePrism Contracts

Shared API and IPC schemas. **Any change to contracts requires a PR that updates this file.**

## Converter trait (Rust)

```rust
pub trait Converter {
    fn supports(&self, input: &str, output: &str) -> bool;
    fn convert(&self, job: &Job) -> Result<Output, Error>;
}
```

Defined in `fileprism-core`. Format crates implement this trait.

## Job / Output (M0 scaffold)

| Type     | Fields                                                                 |
|----------|------------------------------------------------------------------------|
| `Job`    | `id`, `input_path`, `output_path`, `input_format`, `output_format`     |
| `Output` | `job_id`, `output_path`, `bytes_written`                               |

Serialization: `serde` JSON for IPC between UI and core (Tauri commands).

## Tauri commands (M0)

| Command     | Args           | Returns   | Notes                          |
|-------------|----------------|-----------|--------------------------------|
| `greet`     | `name: string` | `string`  | Smoke / scaffold only          |
| `core_name` | —              | `string`  | Returns `fileprism-core` name  |

## History schema (planned M1)

Tables: `jobs`, `files`, `presets`, `settings` — exact columns TBD in M1 PR.

## Explorer IPC (planned M6)

Named-pipe payload: list of absolute file paths from Windows shell extension to desktop app. Schema TBD in M6.

## License key (planned M7)

Offline Ed25519-signed license blob. Validation API TBD in M7.
