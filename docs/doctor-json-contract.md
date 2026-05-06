# Doctor JSON Contract

This document is the shared contract for structured Doctor diagnostics. It keeps
CLI diagnostics, support-bundle `doctor.json`, Desktop diagnostic cards, and
Android JNI diagnostics from maintaining separate Doctor field lists.

## Owner

The source of truth is `doctor::doctor_report_json_value` in `src/doctor.rs`.
Individual items are rendered through `doctor::doctor_item_json_value`.

Do not hand-build `doctor.json` in support-bundle code or future UI bridges.
If a field is added to `DoctorItem`, update the shared renderer, this document,
tests, and `tools/check-doctor-json-contract.py` in the same change.

## Shape

```json
{
  "ok": true,
  "items": [
    {
      "id": "mode",
      "level": "ok",
      "title": "Mode",
      "detail": "mode = apps_script",
      "fix": null
    }
  ]
}
```

## Fields

Top-level fields:

| Field | Meaning |
|---|---|
| `ok` | `true` when no item has `level = "fail"`. |
| `items` | Ordered list of Doctor findings. The order is the user-facing diagnostic order. |

Item fields:

| Field | Meaning |
|---|---|
| `id` | Stable machine-readable readiness/diagnostic ID. |
| `level` | One of `ok`, `warn`, or `fail`. |
| `title` | Short user-facing diagnostic label. |
| `detail` | Human-readable explanation. |
| `fix` | Suggested repair text, or `null` when there is no action. |

## Consumers

| Consumer | Contract |
|---|---|
| CLI `mhrv-f doctor` | May keep pretty text output, but should use the same `DoctorReport` data and level meanings. |
| Support bundle `doctor.json` | Must call `doctor::doctor_report_json_value`. |
| Desktop Monitor Doctor summary | Uses the typed `DoctorReport` directly and is guarded by `tools/check-desktop-doctor-summary.py`. |
| Android `Native.doctorJson(configJson)` | Must run Rust Doctor and return `doctor::doctor_report_json_value(&report)`. Guarded by `tools/check-android-doctor-jni-bridge.py`. |
| Android Doctor summary card | Must consume `Native.doctorJson(configJson)`, parse the same shape, ignore stale results after config changes, and keep visible copy localized. Guarded by `tools/check-android-doctor-summary-ui.py`. |

## Change Checklist

1. Add the new data to `DoctorItem` / `DoctorReport`.
2. Update `doctor_item_json_value` or `doctor_report_json_value`.
3. Update this document and `tools/check-doctor-json-contract.py`.
4. Add or update Rust tests for the renderer.
5. Ensure support bundles still call the shared renderer.
6. Update Desktop/Android/docs parity notes if the field is user visible.
7. Update the detailed changelog and roadmap.

## Verification

```powershell
python tools\check-doctor-json-contract.py
cargo test doctor_report_json_renderer_keeps_support_bundle_shape --lib
python tools\run-repo-sanity.py --skip-node
```
