# Windows Explorer integration & installer

## Pieces

| Path | Role |
|------|------|
| `crates/explorer-win` | `IExplorerCommand` COM DLL + shared IPC schema |
| `sparse/AppxManifest.xml` | Sparse package for unpackaged (Tauri) installs |
| `register-sparse-package.ps1` | Dev build + stage + `Add-AppxPackage -Register` |
| `unregister-sparse-package.ps1` | Remove `FilePrism.Shell` |
| `nsis-hooks.nsh` | Tauri NSIS post-install / pre-uninstall (M7 register attempt) |

## Installer targets (M7)

Tauri bundler (`apps/desktop/src-tauri/tauri.conf.json`):

- **NSIS** — primary Windows installer; runs `nsis-hooks.nsh` after install
- **MSI** — WiX/MSI target for enterprise deployment

Build (release machine):

```powershell
$env:CARGO_BUILD_JOBS = "1"
pnpm tauri build
```

Artifacts land under `apps/desktop/src-tauri/target/release/bundle/{nsis,msi}/`.

Code signing and updater pubkey: see `docs/packaging/code-signing.md` and `docs/packaging/auto-update.md`.

## Dev verify (Explorer)

1. Run the desktop app so the named-pipe server is up.
2. `cargo build -j 1 -p fileprism-explorer-win`
3. From an elevated PowerShell (Developer Mode on):

   ```powershell
   .\packaging\windows\register-sparse-package.ps1 -InstallRoot <dir-with-FilePrism.exe>
   ```

4. In Explorer, right-click a file → **Open with FilePrism**.

## Production sparse package

- `AppxManifest.xml` `Identity/@Publisher` must match the signing certificate subject.
- Until a real cert is configured, the NSIS hook’s register step may no-op; testers can still use the manual script.
- Set installer env `FILEPRISM_REGISTER_SPARSE=0` to skip the hook’s PowerShell register call.
