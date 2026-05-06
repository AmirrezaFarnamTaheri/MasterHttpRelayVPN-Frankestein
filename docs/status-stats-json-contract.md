# Status/Stats JSON Contract

This document is the shared contract for live runtime statistics JSON. It keeps
the desktop/local `/status` endpoint, support-bundle `status.json`, and Android
`Native.statsJson(handle)` from growing separate field lists.

## Owner

The source of truth is `status_api::stats_snapshot_json_value` in
`src/status_api.rs`.

Do not hand-serialize `StatsSnapshot` in Android JNI, support bundle code, UI
helpers, or tests. Add fields to the shared renderer first, then update this
document and `tools/check-status-stats-json-contract.py` in the same change.

## Envelopes

Different surfaces may wrap the same stats object differently:

| Surface | Shape | Notes |
|---|---|---|
| Local `/status` | object with `stats` field | `stats` is the shared stats object when a snapshot is available; otherwise it may be `null`. |
| Support bundle `status.json` | same as local `/status` | Support bundles call `status_api::render_status_json`, so they inherit the same shape. |
| Android `Native.statsJson(handle)` | raw stats object | Returns the shared stats object directly for lightweight polling, or an empty string when the handle has no Apps Script fronter. |
| Android Usage Today card | raw or enveloped stats object | Parses with `root.optJSONObject("stats") ?: root` so future envelope migration does not break the card. |

## Stats Object Fields

All fields below are emitted by `stats_snapshot_json_value`.

| Field | Meaning |
|---|---|
| `relay_calls` | Total relay calls counted by the active `DomainFronter`. |
| `relay_failures` | Total relay failures counted by the active `DomainFronter`. |
| `cache_hits` | Number of responses served from the relay cache. |
| `cache_misses` | Number of cache misses. |
| `cache_bytes` | Current cache byte usage. |
| `bytes_relayed` | Total bytes relayed through the Apps Script path. |
| `coalesced` | Number of coalesced requests. |
| `scripts_total` | Canonical count of configured script endpoints. |
| `scripts_blacklisted` | Canonical count of temporarily blacklisted script endpoints. |
| `total_scripts` | Android compatibility alias for `scripts_total`. |
| `blacklisted_scripts` | Android compatibility alias for `scripts_blacklisted`. |
| `today_calls` | Calls counted in the current daily quota window. |
| `today_bytes` | Bytes counted in the current daily quota window. |
| `today_reset_secs` | Seconds until the current daily quota window resets. |
| `degrade_level` | Current runtime degradation level. |
| `degrade_reason` | Human-readable degradation reason, if any. |

## Change Checklist

When adding, renaming, or removing a stats field:

1. Update `stats_snapshot_json_value` in `src/status_api.rs`.
2. Keep Android JNI calling `stats_snapshot_json_value(fronter.snapshot_stats())`.
3. Keep Android UI parsing tolerant of raw and `/status`-enveloped stats.
4. Update this document and the required-key list in
   `tools/check-status-stats-json-contract.py`.
5. Add or update Rust tests around `status_api`.
6. Run the local contract guard and repo sanity.
7. Update the detailed changelog and roadmap entry for the batch.

## Verification

```powershell
python tools\check-status-stats-json-contract.py
python tools\run-repo-sanity.py --skip-node
python tools\check-doc-links.py
```
