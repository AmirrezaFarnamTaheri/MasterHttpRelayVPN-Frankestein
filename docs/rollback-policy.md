# Rollback Policy

Rollback means getting users back to a known-good state without creating a
second source of truth. The GitHub Release and CI-built artifacts remain
authoritative.

## Bad Desktop / CLI Release

1. Mark the GitHub Release as problematic with a clear note.
2. Publish a fixed release rather than replacing artifacts in-place.
3. Keep `SHA256SUMS.txt` tied to the exact artifacts that shipped.
4. Add a `docs/changelog/v<fixed-version>.md` note explaining the fix and user
   action.

## Bad Android Release

1. Do not rotate signing material unless the key itself is compromised.
2. Publish a fixed APK/AAB through the normal CI release path.
3. Document install-over compatibility, signing continuity, and any required
   uninstall/reinstall step in the release notes.
4. Update `docs/android-signing.md` only if the signing policy changes.

## Bad Config Migration

1. Keep the canonical config shape clean.
2. Add a narrow importer/repair path only when users can realistically have the
   broken shape on disk.
3. Add Doctor/readiness messaging that identifies the bad shape and the repair.
4. Remove the temporary compatibility path once the project deliberately stops
   supporting that broken shape, and document the cleanup.

## Bad Backend Helper

1. Restore or publish a corrected helper file under `assets/apps_script/`,
   Cloudflare, Vercel/Netlify, or tunnel-node as appropriate.
2. Bump helper compatibility markers when the deployed helper shape changed.
3. Update deploy docs with exact redeploy steps.
4. Add release notes that say whether existing deployment IDs can be reused.

## Bad tunnel-node Release

1. Publish a fixed binary/container through CI.
2. Document whether server config, auth key, ports, or client config must
   change.
3. Keep Doctor/full-mode docs aligned with the fixed tunnel-node behavior.

## Communication

- GitHub Release is canonical.
- Telegram is an optional mirror only.
- `docs/RELEASE_NOTES.md`, `docs/changelog/v*.md`, and maintainer batch logs
  must agree on user action.
