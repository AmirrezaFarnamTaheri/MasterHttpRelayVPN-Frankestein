[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\mhrv-f",
    [switch]$KeepConfig,
    [switch]$RemoveConfig
)

$ErrorActionPreference = "Stop"

$desktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "mhrv-f UI.lnk"
$startMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\mhrv-f"

if (Test-Path -LiteralPath $desktopShortcut) {
    Remove-Item -LiteralPath $desktopShortcut -Force
}
if (Test-Path -LiteralPath $startMenuDir) {
    Remove-Item -LiteralPath $startMenuDir -Recurse -Force
}
if (Test-Path -LiteralPath $InstallDir) {
    Remove-Item -LiteralPath $InstallDir -Recurse -Force
}

$configDir = Join-Path $env:APPDATA "mhrv-f"
if ($RemoveConfig -and !$KeepConfig) {
    if (Test-Path -LiteralPath $configDir) {
        Remove-Item -LiteralPath $configDir -Recurse -Force
        Write-Host "Removed user config from $configDir"
    }
} elseif (Test-Path -LiteralPath $configDir) {
    Write-Host "User config remains at $configDir"
    Write-Host "Run this script with -RemoveConfig only after removing the MITM CA if you want a complete wipe."
}

Write-Host "Uninstalled mhrv-f application files."
