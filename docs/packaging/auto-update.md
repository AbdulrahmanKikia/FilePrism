# Auto-update (Tauri updater)

FilePrism uses **Tauri 2** `tauri-plugin-updater` with signed update artifacts.

## Config

In `apps/desktop/src-tauri/tauri.conf.json`:

- `bundle.createUpdaterArtifacts`: `true`
- `plugins.updater.pubkey`: minisign/updater **public** key (safe to commit once generated)
- `plugins.updater.endpoints`: HTTPS JSON endpoints returning update metadata
- Capability permission: `updater:default`

Placeholder pubkey `REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY` must be replaced before shipping.

## Generate keys (human / release machine)

```bash
# Requires Tauri CLI
pnpm tauri signer generate -w ~/.tauri/fileprism.key
```

- Public key → paste into `plugins.updater.pubkey`
- Private key → GitHub secret `TAURI_SIGNING_PRIVATE_KEY` (never commit)
- Password → `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if used

## Endpoint contract

Host JSON at the configured URL (Tauri expands `{{target}}`, `{{arch}}`, `{{current_version}}`). See [Tauri updater docs](https://v2.tauri.app/plugin/updater/).

## Desktop wiring

The updater plugin is initialized in `apps/desktop/src-tauri/src/lib.rs`. In-app “Check for updates” UI can call the JS plugin (`@tauri-apps/plugin-updater`) in a follow-up; config + CI hooks are the M7 deliverable.

## Frontend optional dependency

```json
"@tauri-apps/plugin-updater": "^2"
```

Add when shipping the check-for-updates control in Settings.
