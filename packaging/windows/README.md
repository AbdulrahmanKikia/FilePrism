# Windows Explorer integration

## Pieces

| Path | Role |
|------|------|
| `crates/explorer-win` | `IExplorerCommand` COM DLL + shared IPC schema |
| `sparse/AppxManifest.xml` | Sparse package for unpackaged (Tauri) installs |
| `register-sparse-package.ps1` | Dev build + stage + `Add-AppxPackage -Register` |
| `unregister-sparse-package.ps1` | Remove `FilePrism.Shell` |
| `nsis-hooks.nsh` | Tauri NSIS post-install / pre-uninstall stubs |

## Dev verify

1. Run the desktop app (`pnpm tauri:dev` or a built `FilePrism.exe`) so the named-pipe server is up.
2. `cargo build -j 1 -p fileprism-explorer-win`
3. From an elevated PowerShell (Developer Mode on):

   ```powershell
   .\packaging\windows\register-sparse-package.ps1 -InstallRoot <dir-with-FilePrism.exe>
   ```

4. In Explorer, right-click a file → **Open with FilePrism**. The app should focus and open Convert with the path filled.
5. Quit the app and invoke again — the DLL should launch the exe with `--shell-open`.

Signed publisher identity and automatic sparse-package registration in the NSIS installer are deferred to **M7**.
