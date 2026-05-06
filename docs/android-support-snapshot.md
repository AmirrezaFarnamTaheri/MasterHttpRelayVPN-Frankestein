# Android Support Snapshot

The Android Trust Center can copy a small redacted text snapshot for support.
It is intentionally lighter than Desktop/CLI `support-bundle`: it is safe to
paste into an issue or chat after a quick review, while the full bundle remains
the richer diagnostic artifact.

## Current Schema

Current marker:

```text
schema: android-support-snapshot/v2
```

`v2` includes the original Android routing/trust/config-preservation fields plus
a redacted Doctor summary. There is no compatibility promise for old internal
snapshot shapes; if the copied text gains or loses fields, the schema marker,
tests, docs, and changelog must move together.

## Field Groups

Identity and routing:

- mode
- connection mode
- split mode
- split app count
- listen host and ports
- LAN sharing state

Trust:

- whether the selected mode needs a user CA
- whether Android currently reports the CA as installed
- whether release-signing continuity policy applies through the Trust Center
  docs

Backend/config shape:

- Apps Script deployment count
- masked Apps Script deployment IDs
- whether auth keys/tokens are configured
- Serverless relay fields configured or absent
- SNI host count
- advanced root preservation status

Doctor summary, when the in-app Doctor has been run for the current config:

- `doctor_available`
- `doctor_ok`
- `doctor_items_total`
- `doctor_items_ok`
- `doctor_items_warn`
- `doctor_items_fail`
- `doctor_problem_ids`

## Redaction Contract

The snapshot must not include:

- raw `auth_key`
- serverless `AUTH_KEY`
- LAN token
- upstream SOCKS5 credentials
- raw unknown preserved JSON
- full Apps Script deployment IDs
- raw Doctor JSON
- Doctor titles, details, fixes, endpoint URLs, or host-specific diagnostic text

Doctor output is summarized by item ID only. The full Doctor JSON remains a
Desktop/CLI support-bundle artifact where users can preview the bundle before
sharing.

## Ownership

Android snapshot generation is owned by
`android/app/src/main/java/com/farnam/mhrvf/SupportRedaction.kt`.
`HomeScreen.kt` is only allowed to call that utility and pass current state.

Guard rails:

- `android/app/src/test/java/com/farnam/mhrvf/SupportRedactionTest.kt`
- `tools/check-android-support-redaction.py`
- `tools/run-repo-sanity.py --skip-node`

After changing snapshot fields, update this document, the static guard, the JVM
test source, `docs/android.md`, `docs/android.fa.md`, `docs/trust-center.md`,
the roadmap, and the batch changelog in the same step.
