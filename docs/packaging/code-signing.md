# Code signing (Windows)

Actual certificates are **never** committed. Configure via CI secrets / local secure store.

## What to sign

| Artifact | Mechanism |
|----------|-----------|
| `FilePrism.exe` | Authenticode (Tauri `bundle.windows.certificateThumbprint`) |
| NSIS / MSI installer | Same cert (Tauri signs when thumbprint + cert available) |
| Sparse package / MSIX identity | Publisher CN must match cert subject (`AppxManifest.xml`) |
| Updater artifacts (`.sig`) | Separate Tauri updater keypair (see [auto-update.md](./auto-update.md)) |

## Tauri config hooks

`apps/desktop/src-tauri/tauri.conf.json`:

```json
"windows": {
  "certificateThumbprint": null,
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
}
```

Set `certificateThumbprint` at release time (or inject via CI) to the SHA1 thumbprint of the code-signing cert in the Windows certificate store / cloud HSM.

## Required human secrets

| Secret | Used for |
|--------|----------|
| `WINDOWS_CERT_THUMBPRINT` | Authenticode thumbprint |
| `WINDOWS_CERT_PFX` / `WINDOWS_CERT_PASSWORD` | Optional PFX import in CI |
| `FILEPRISM_LICENSE_SIGNING_SEED` | Minting customer license keys (not Authenticode) |
| `TAURI_SIGNING_PRIVATE_KEY` (+ password) | Updater artifact signatures |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | If the updater key is encrypted |

## CI

`.github/workflows/release.yml` expects the secrets above. Until they are present, the workflow documents the steps and can build unsigned artifacts for smoke tests.

## Sparse package publisher

Update `packaging/windows/sparse/AppxManifest.xml`:

```xml
<Identity Publisher="CN=Your Certificate Subject" ... />
```

Dev builds may keep `CN=FilePrism Dev` and register via Developer Mode + `register-sparse-package.ps1`.
