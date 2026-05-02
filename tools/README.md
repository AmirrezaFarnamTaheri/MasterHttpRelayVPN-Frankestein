# Maintainer Tools

This directory contains helper projects and maintainer scripts that are part of
the repository contract. Tooling here should be deterministic and safe to run
without creating release artifacts unless its README says otherwise.

## Readiness Contract Generator

Rust readiness IDs and repair targets are the source of truth in
`src/readiness.rs`. Android consumes a generated Kotlin mirror at
`android/app/src/main/java/com/farnam/mhrvf/ReadinessIds.kt`.

Regenerate the Android mirror after changing readiness IDs or repair targets:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\generate-readiness-contract.ps1
```

Check that the generated file is current without modifying it:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\generate-readiness-contract.ps1 -Check
```

CI runs the check form. This generator does not run Gradle and should not create
Android build outputs.

## Example Config Contract Test

The Rust config tests load every root `config*.example.json` file through
`Config::from_json_str`, including migration and validation. Run this focused
test after changing config schema, examples, mode names, or readiness blockers:

```powershell
cargo test bundled_example_configs_load_and_validate
```

## Repository Cleanliness Check

`tools/check-repo-cleanliness.py` keeps local build output, oversized source
files, binary artifacts, local secrets, and stale-prone image references out of
the maintained source tree.

Run it from the repository root:

```powershell
python tools\check-repo-cleanliness.py
```

CI runs the same script. Local `dist/` and `releases/` folders are reported as
allowed backup/archive material, not as release sources. The official release
artifacts still come from `.github/workflows/release.yml`.

## Markdown Local Link Check

`tools/check-doc-links.py` checks local Markdown links in README, docs,
maintainer tools, Apps Script helper docs, tunnel-node docs, and release
fallback docs. It skips external URLs and pure in-page anchors, then verifies
that relative file/directory targets exist.

Run it from the repository root:

```powershell
python tools\check-doc-links.py
```

CI runs the same check.
