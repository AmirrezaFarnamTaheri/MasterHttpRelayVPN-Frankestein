[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\mhrv-f",
    [switch]$NoShortcuts
)

$ErrorActionPreference = "Stop"
$SourceDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function New-MhrvShortcut {
    param(
        [string]$Path,
        [string]$Target,
        [string]$WorkingDirectory,
        [string]$Description
    )
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($Path)
    $shortcut.TargetPath = $Target
    $shortcut.WorkingDirectory = $WorkingDirectory
    $shortcut.Description = $Description
    $shortcut.Save()
}

$cli = Join-Path $SourceDir "mhrv-f.exe"
$ui = Join-Path $SourceDir "mhrv-f-ui.exe"
if (!(Test-Path -LiteralPath $cli)) {
    throw "mhrv-f.exe was not found next to install-mhrv-f.ps1"
}
if (!(Test-Path -LiteralPath $ui)) {
    throw "mhrv-f-ui.exe was not found next to install-mhrv-f.ps1"
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$items = @(
    "mhrv-f.exe",
    "mhrv-f-ui.exe",
    "run.bat",
    "README.md",
    "LICENSE",
    "INSTALL.txt",
    "uninstall-mhrv-f.ps1"
)

foreach ($item in $items) {
    $src = Join-Path $SourceDir $item
    if (Test-Path -LiteralPath $src) {
        Copy-Item -LiteralPath $src -Destination (Join-Path $InstallDir $item) -Force
    }
}

foreach ($dir in @("docs", "assets", "tools")) {
    $src = Join-Path $SourceDir $dir
    if (Test-Path -LiteralPath $src) {
        Copy-Item -LiteralPath $src -Destination (Join-Path $InstallDir $dir) -Recurse -Force
    }
}

if (!$NoShortcuts) {
    $startMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\mhrv-f"
    New-Item -ItemType Directory -Force -Path $startMenu | Out-Null
    $target = Join-Path $InstallDir "mhrv-f-ui.exe"
    New-MhrvShortcut `
        -Path (Join-Path $startMenu "mhrv-f UI.lnk") `
        -Target $target `
        -WorkingDirectory $InstallDir `
        -Description "MasterHttpRelayVPN-Frankestein desktop UI"
    New-MhrvShortcut `
        -Path (Join-Path ([Environment]::GetFolderPath("Desktop")) "mhrv-f UI.lnk") `
        -Target $target `
        -WorkingDirectory $InstallDir `
        -Description "MasterHttpRelayVPN-Frankestein desktop UI"
}

Write-Host "Installed mhrv-f to $InstallDir"
Write-Host "Run mhrv-f-ui.exe from that folder, or use the Start Menu shortcut."
