; FilePrism NSIS installer hooks — shell extension staging (M6).
; Wired from apps/desktop/src-tauri/tauri.conf.json → bundle.windows.nsis.installerHooks.
; Full cert / sparse-package registration completes in M7 packaging.

!macro NSIS_HOOK_POSTINSTALL
  ; Stage Explorer DLL next to the installed exe when present in the payload.
  ; The sparse package (AppxManifest) is copied by the resource bundle; registration
  ; requires package identity signing which M7 adds. Until then, leave files on disk
  ; and document the manual register-sparse-package.ps1 path for testers.
  DetailPrint "FilePrism: shell extension files staged (sparse package register deferred to M7 / manual script)."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Best-effort sparse package removal when M7 registration is present.
  nsExec::ExecToLog 'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Get-AppxPackage -Name FilePrism.Shell | Remove-AppxPackage"'
!macroend
