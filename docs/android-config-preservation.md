# Android config preservation (unknown roots)

Android keeps feature parity with Desktop **without** silently dropping advanced
JSON that the mobile UI does not edit yet.

## Behavior

On **`ConfigStore.loadFromJson`**:

1. Android clones the incoming root JSON object.
2. It removes every key listed in **`ownedKeys`** inside `ConfigStore.kt` — the
   keys the Kotlin layer reads into `MhrvConfig` (plus Android-only wrappers and
   legacy import roots).
3. Whatever remains becomes **`preservedUnknownRootJson`** (may be empty).
4. On export/share, preserved JSON is merged back so Desktop-only or expert
   fields survive round-trips.

Canonical field metadata (including “preserve” rows) lives in
[`docs/config-registry.md`](config-registry.md) (generated from
[`docs/config-registry.json`](config-registry.json)).

## Drift gates

| Gate | Purpose |
|------|---------|
| **`tools/check-android-owned-keys-list.py`** | Every **`ownedKeys`** entry must be a registry root or an allowlisted Android-only / legacy key from **`tools/android_config_allowlists.py`**. |
| **`tools/check-android-config-keys.py`** | Every JSON key literal touched in **`ConfigStore.kt`** must be documented in the registry or allowlists (includes nested **`NESTED_KEYS`** for `vercel` / `account_groups[]`). |

If you add a new root field to Rust `Config`, update **`docs/config-registry.json`**,
regenerate docs, extend **`ownedKeys`** when Android should own the key, and
re-run **`python tools/run-repo-sanity.py`**.

## Limits

- Preservation is **root-level only** — unknown keys *inside* nested objects
  that Android parses (for example selective fields under **`vercel`**) follow
  Kotlin parsing rules; prefer Desktop or raw JSON edits for exotic nested
  shapes until Android exposes editors.
- Unknown blobs are **not validated** by Android — invalid JSON from Desktop
  still fails at **`JSONObject`** parse time before preservation runs.

## Related

- Donor import policy (do not treat donor rules as defaults):
  [`docs/donor-absorption-matrix.md`](donor-absorption-matrix.md)
- External JSON shape smoke vs registry:
  [`tools/report-nova-proxy-config.py`](../tools/report-nova-proxy-config.py) (`--demo` / `--path`)
