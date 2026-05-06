# Batch 6 - README Persian Guide Links Gate

Date: 2026-05-05

## Summary

Added a static guard for the README Persian setup resources. The README already
absorbed the useful upstream guide links in the compact two-item form; this
batch makes that onboarding surface deliberate and regression-resistant.

## Changes

- Added `tools/check-readme-persian-guides.py`.
- Wired the gate into `tools/run-repo-sanity.py`.
- Added it to the CI/local parity guard.
- Documented it in `tools/README.md`.

## Guarded Contract

- The English/Persian README language switcher must stay near the top.
- The Persian YouTube setup video link must remain near the top.
- Kian Irani's Persian text guide and credit link must remain near the top.
- External guide links must keep `target="_blank"` and
  `rel="noopener noreferrer"`.
- The README must not reintroduce a large YouTube thumbnail embed.

## Parity / Split-Brain Notes

- This guards README onboarding only; product docs remain canonical for detailed
  setup instructions.
- The compact external guide block complements, but does not replace,
  `docs/android.fa.md`, `docs/index.fa.md`, or the in-repo Persian docs.

## Verification

- `python tools/check-readme-persian-guides.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py`
- `python tools/check-repo-cleanliness.py`

All checks passed on 2026-05-05. The full repo sanity route confirmed the new
README Persian guide gate is included in local/CI sanity.

## Cleanup

- Removed generated `.github/scripts/__pycache__` and `tools/__pycache__`
  directories after verification.
- No Gradle command was run and no Android build output was generated.
- Process inspection found no active `gradle`, `java`, or `kotlinc` processes
  during closeout.
