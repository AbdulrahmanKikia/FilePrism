#Requires -Version 5.1
<#
.SYNOPSIS
  Remove the FilePrism.Shell sparse package registration.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$pkg = Get-AppxPackage -Name "FilePrism.Shell" -ErrorAction SilentlyContinue
if (-not $pkg) {
    Write-Host "FilePrism.Shell is not registered."
    return
}
Remove-AppxPackage -Package $pkg.PackageFullName
Write-Host "Removed $($pkg.PackageFullName)"
