# Workspace cleanup (local artifacts)

After builds, tests, or installer work, the tree can accumulate **ignored**
outputs (`target/`, Android `.gradle/`, `dist/`, Python `__pycache__/`, etc.).
They are not meant to be committed; this page is the **garbage-collection**
procedure to return to a clean working copy.

## What is safe to delete

Typical regenerable paths (also listed in `.gitignore`):

| Path | Origin |
|------|--------|
| `target/` | Rust (root) |
| `tunnel-node/target/` | Rust (tunnel-node) |
| `android/.gradle/`, `android/**/build/` | Gradle |
| `tools/**/__pycache__/` | Python |
| `dist/`, `releases/` | Local packaging backups (optional; see `releases/README.md`) |
| `*.pyc` | Python |

Do **not** delete committed source, `android/app/release.jks` (signing policy:
[`docs/android-signing.md`](android-signing.md)), or user config outside the repo.

## Windows (PowerShell)

Use **Windows PowerShell 5.1** or **PowerShell 7 (`pwsh`)** — both work for cleanup commands below. Readiness contract checks accept either shell (see [`docs/release-checklist.md`](release-checklist.md)).

From the repository root:

```powershell
# Rust
if (Test-Path .\target) { Remove-Item -Recurse -Force .\target }
if (Test-Path .\tunnel-node\target) { Remove-Item -Recurse -Force .\tunnel-node\target }

# Android / Gradle caches under the repo
if (Test-Path .\android\.gradle) { Remove-Item -Recurse -Force .\android\.gradle }
Get-ChildItem -Path .\android -Recurse -Directory -Filter build -ErrorAction SilentlyContinue |
  Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

# Python caches under tools/
Get-ChildItem -Path .\tools -Recurse -Directory -Filter __pycache__ -ErrorAction SilentlyContinue |
  Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
```

Optional — only if you **choose** to drop local archive trees:

```powershell
# Optional: local release backups (re-download from CI if needed)
# Remove-Item -Recurse -Force .\dist, .\releases
```

## Linux / macOS

From the repository root:

```bash
rm -rf target tunnel-node/target
rm -rf android/.gradle
find android -type d -name build -prune -exec rm -rf {} +
find tools -type d -name __pycache__ -prune -exec rm -rf {} +
```

## Verify after cleanup

```bash
python3 tools/check-repo-cleanliness.py
python3 tools/run-repo-sanity.py
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cd tunnel-node && cargo clippy --all-targets -- -D warnings && cd ..
```

(`run-repo-sanity` includes the donor **`--demo`** config report, Markdown gates, and Android drift checks — mirror of CI.)

See also: [`tools/README.md`](../tools/README.md), [`docs/release-checklist.md`](release-checklist.md).
