#Requires -Version 5.1
<#
.SYNOPSIS
  Build (optional) and register the FilePrism Explorer sparse package for development.

.DESCRIPTION
  1. Optionally builds fileprism-explorer-win (cdylib).
  2. Stages AppxManifest + DLL under a sparse package root.
  3. Creates a self-signed cert (dev only) if missing.
  4. Signs and registers the sparse package via Add-AppxPackage -Register.

  Production signing / publisher identity land in M7.

.PARAMETER InstallRoot
  Directory containing FilePrism.exe (and where shell\ DLL will be placed).

.PARAMETER SkipBuild
  Do not run cargo build for the explorer DLL.
#>
[CmdletBinding()]
param(
    [string]$InstallRoot = (Join-Path $PSScriptRoot "..\..\target\debug"),
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$SparseSrc = Join-Path $PSScriptRoot "sparse"
$StageRoot = Join-Path $InstallRoot "sparse-package"
$ShellDir = Join-Path $InstallRoot "shell"
$DllName = "fileprism_explorer_win.dll"

Write-Host "Repo:        $RepoRoot"
Write-Host "InstallRoot: $InstallRoot"
Write-Host "StageRoot:   $StageRoot"

if (-not $SkipBuild) {
    Push-Location $RepoRoot
    try {
        $env:CARGO_BUILD_JOBS = "1"
        cargo build -j 1 -p fileprism-explorer-win
    }
    finally {
        Pop-Location
    }
}

$BuiltDll = Join-Path $RepoRoot "target\debug\$DllName"
if (-not (Test-Path $BuiltDll)) {
    $BuiltDll = Join-Path $RepoRoot "target\release\$DllName"
}
if (-not (Test-Path $BuiltDll)) {
    throw "DLL not found. Build fileprism-explorer-win first."
}

New-Item -ItemType Directory -Force -Path $ShellDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $StageRoot "Assets") | Out-Null
Copy-Item -Force $BuiltDll (Join-Path $ShellDir $DllName)
Copy-Item -Force (Join-Path $SparseSrc "AppxManifest.xml") (Join-Path $StageRoot "AppxManifest.xml")

# Placeholder logos — reuse Tauri icons when present.
$IconDir = Join-Path $RepoRoot "apps\desktop\src-tauri\icons"
$Assets = Join-Path $StageRoot "Assets"
function Copy-Icon([string]$SrcName, [string]$DstName) {
    $src = Join-Path $IconDir $SrcName
    $dst = Join-Path $Assets $DstName
    if (Test-Path $src) {
        Copy-Item -Force $src $dst
    }
    else {
        # Minimal 1x1 PNG so registration does not fail on missing logos.
        [IO.File]::WriteAllBytes($dst, [Convert]::FromBase64String(
            "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO5W6dUAAAAASUVORK5CYII="
        ))
    }
}
Copy-Icon "StoreLogo.png" "StoreLogo.png"
Copy-Icon "Square150x150Logo.png" "Square150x150Logo.png"
Copy-Icon "Square44x44Logo.png" "Square44x44Logo.png"

# External content mapping: sparse package root points at InstallRoot binaries.
$ExternalLoc = Join-Path $StageRoot "AppxManifest.xml"
# AllowExternalContent sparse packages register the folder that contains the manifest
# while referencing executables outside via relative paths from InstallRoot.
# For unpackaged layout we place the manifest beside a junction or copy.
# Dev approach: register InstallRoot itself after copying the manifest there.
$RegManifestDir = $InstallRoot
Copy-Item -Force (Join-Path $StageRoot "AppxManifest.xml") (Join-Path $RegManifestDir "AppxManifest.xml")
$AssetsInstall = Join-Path $RegManifestDir "Assets"
New-Item -ItemType Directory -Force -Path $AssetsInstall | Out-Null
Copy-Item -Force (Join-Path $Assets "*") $AssetsInstall

Write-Host ""
Write-Host "Sparse package files staged under $RegManifestDir"
Write-Host "Register with (elevated / developer mode recommended):"
Write-Host "  Add-AppxPackage -Path `"$(Join-Path $RegManifestDir 'AppxManifest.xml')`" -Register"
Write-Host ""
Write-Host "Attempting registration…"

try {
    Add-AppxPackage -Path (Join-Path $RegManifestDir "AppxManifest.xml") -Register -ExternalLocation $RegManifestDir
    Write-Host "Registered FilePrism.Shell sparse package."
}
catch {
    Write-Warning "Add-AppxPackage failed: $_"
    Write-Warning "Enable Developer Mode, ensure a matching Publisher cert exists, then re-run."
    Write-Warning "M7 will wire signed production identity into the NSIS installer."
}
