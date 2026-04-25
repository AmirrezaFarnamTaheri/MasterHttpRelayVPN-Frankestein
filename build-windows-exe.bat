@echo off
REM Local Windows-only: build mhrv-f.exe + mhrv-f-ui.exe (default host triple, MSVC or GNU per rustup).
REM Release APK and all other platforms are built by .github/workflows/release.yml — not here.
setlocal
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
  echo error: cargo not on PATH. Install Rust from https://rustup.rs
  exit /b 1
)

echo Building mhrv-f (CLI)...
cargo build --release --bin mhrv-f
if errorlevel 1 exit /b 1

echo Building mhrv-f-ui (desktop UI)...
cargo build --release --features ui --bin mhrv-f-ui
if errorlevel 1 exit /b 1

echo.
echo OK: target\release\mhrv-f.exe
echo     target\release\mhrv-f-ui.exe
echo Copy assets\launchers\run.bat next to those if you want the same launcher as release zips.
exit /b 0
