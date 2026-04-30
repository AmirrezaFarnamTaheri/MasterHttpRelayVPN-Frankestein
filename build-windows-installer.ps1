[CmdletBinding()]
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root

function Get-CargoVersion {
    $line = Select-String -Path (Join-Path $Root "Cargo.toml") -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if (!$line) {
        throw "Could not read version from Cargo.toml"
    }
    return $line.Matches[0].Groups[1].Value
}

if (!$SkipBuild) {
    cargo build --release --bin mhrv-f
    cargo build --release --features ui --bin mhrv-f-ui
}

$version = Get-CargoVersion
$dist = Join-Path $Root "dist"
$packageName = "mhrv-f-windows-installer-v$version"
$packageDir = Join-Path $dist $packageName
$zipPath = Join-Path $dist "$packageName.zip"

New-Item -ItemType Directory -Force -Path $dist | Out-Null
if (Test-Path -LiteralPath $packageDir) {
    $resolvedDist = (Resolve-Path -LiteralPath $dist).Path
    $resolvedPackage = (Resolve-Path -LiteralPath $packageDir).Path
    if (!$resolvedPackage.StartsWith($resolvedDist, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove package path outside dist: $resolvedPackage"
    }
    Remove-Item -LiteralPath $packageDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $packageDir | Out-Null

$releaseDir = Join-Path $Root "target\release"
Copy-Item -LiteralPath (Join-Path $releaseDir "mhrv-f.exe") -Destination $packageDir -Force
Copy-Item -LiteralPath (Join-Path $releaseDir "mhrv-f-ui.exe") -Destination $packageDir -Force
Copy-Item -LiteralPath (Join-Path $Root "assets\launchers\run.bat") -Destination $packageDir -Force
Copy-Item -LiteralPath (Join-Path $Root "assets\windows\install-mhrv-f.ps1") -Destination $packageDir -Force
Copy-Item -LiteralPath (Join-Path $Root "assets\windows\uninstall-mhrv-f.ps1") -Destination $packageDir -Force
Copy-Item -LiteralPath (Join-Path $Root "README.md") -Destination $packageDir -Force
Copy-Item -LiteralPath (Join-Path $Root "LICENSE") -Destination $packageDir -Force
Get-ChildItem -LiteralPath $Root -File -Filter "config*.example.json" |
    Copy-Item -Destination $packageDir -Force
Copy-Item -LiteralPath (Join-Path $Root "docs") -Destination (Join-Path $packageDir "docs") -Recurse -Force
Copy-Item -LiteralPath (Join-Path $Root "tools") -Destination (Join-Path $packageDir "tools") -Recurse -Force
New-Item -ItemType Directory -Force -Path (Join-Path $packageDir "assets") | Out-Null
Copy-Item -LiteralPath (Join-Path $Root "assets\apps_script") -Destination (Join-Path $packageDir "assets\apps_script") -Recurse -Force

@"
mhrv-f Windows installer package

Quick install:
  1. Extract this folder.
  2. Right-click install-mhrv-f.ps1, choose Run with PowerShell.
  3. Launch "mhrv-f UI" from Start Menu or Desktop.

Portable run:
  Double-click run.bat from this extracted folder.

Bundled helper tools:
  The tools folder is included for no-VPS relays and edge helpers:
    tools\vercel-json-relay
    tools\netlify-json-relay
    tools\vercel-xhttp-relay
    tools\netlify-xhttp-relay
    tools\vercel-xhttp-relay-node

Uninstall:
  Run uninstall-mhrv-f.ps1 from the installed folder.
  User config is kept by default. Use -RemoveConfig only after removing the
  local MITM CA if you want a complete wipe.

Security:
  The first-run launcher may ask for Administrator/UAC only when installing
  the local MITM CA. Remove it later from the UI or with mhrv-f.exe --remove-cert.
"@ | Set-Content -LiteralPath (Join-Path $packageDir "INSTALL.txt") -Encoding UTF8

if (Test-Path -LiteralPath $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}
Compress-Archive -Path (Join-Path $packageDir "*") -DestinationPath $zipPath -Force

Write-Host "Installer folder: $packageDir"
Write-Host "Installer zip   : $zipPath"
