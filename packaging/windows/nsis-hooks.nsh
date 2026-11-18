; FilePrism NSIS installer hooks — Explorer sparse-package registration (M6/M7).
; Wired from apps/desktop/src-tauri/tauri.conf.json → bundle.windows.nsis.installerHooks.
;
; Production identity: Publisher CN in AppxManifest must match the Authenticode /
; package-signing certificate (see docs/packaging/code-signing.md).

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "FilePrism: staging Explorer shell extension / sparse package files."
  ; Best-effort register. Signed publisher identity comes from release certs (CI secrets).
  ; Script is bundled as a Tauri resource next to AppxManifest.
  nsExec::ExecToLog 'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$$script = Get-ChildItem -Path ''$INSTDIR'' -Recurse -Filter register-sparse-package.ps1 -ErrorAction SilentlyContinue | Select-Object -First 1; if ($$script) { & $$script.FullName -InstallRoot ''$INSTDIR'' } else { Write-Host ''register-sparse-package.ps1 not found; skip'' }"'
  DetailPrint "FilePrism: if Explorer menu is missing, run packaging/windows/register-sparse-package.ps1 manually."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "FilePrism: removing sparse package FilePrism.Shell (best-effort)."
  nsExec::ExecToLog 'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Get-AppxPackage -Name FilePrism.Shell | Remove-AppxPackage"'
!macroend
