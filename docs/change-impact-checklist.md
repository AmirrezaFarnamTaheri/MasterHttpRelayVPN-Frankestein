# Change Impact Checklist

Use this before implementation and again before closeout. It bridges changed
surfaces to verification profiles and parity expectations. The machine-readable
source is [`docs/change-impact-checklist.json`](change-impact-checklist.json),
guarded by `tools/check-change-impact-checklist.py`.

Profiles are still defined in
[`docs/verification-profiles.md`](verification-profiles.md). This checklist
answers the earlier question: "I touched this area; what else should I check?"

## Surface Matrix

| Surface | Typical Paths | Verification Profiles | Parity To Check |
|---|---|---|---|
| Docs / Roadmap / Changelog | `README.md`, `docs/**/*.md`, `elevation_audit_roadmap_source.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md` | `docs_governance` | docs index, changelog index, roadmap, local links, anchors |
| Config / Registry / Examples | `src/config.rs`, `docs/config-registry.json`, `config*.example.json`, Android `ConfigStore.kt` | `config_schema`, `android_ui` | Rust Config, Desktop ConfigWire, Android ConfigStore, examples, config registry, parity matrix |
| Desktop UI / Rust Runtime | `src/bin/ui.rs`, `src/**/*.rs` | `desktop_runtime` | Desktop UI, CLI, Doctor/status contracts, docs, support bundles |
| Android UI / Mobile Bridge | `android/**`, `src/android_jni.rs` | `android_ui`, `config_schema` | Android strings EN/FA, ConfigStore, JNI bridge, support snapshot, Desktop import/export |
| Backend Helpers / Relay Scripts | `assets/apps_script/**`, relay helper tools, relay docs, backend registry | `backend_helpers` | helper kind/version/protocol markers, backend registry, mode docs, examples, release checklist |
| Full Tunnel / tunnel-node | `tunnel-node/**`, `docs/udpgw.md`, `CodeFull.gs`, full-mode examples | `full_tunnel`, `backend_helpers` | CodeFull.gs, tunnel-node, Desktop full-mode guidance, Android full-mode config, docs |
| Release / Security / Trust | `.github/**`, signing/release/security/rollback docs | `docs_governance`, `release_ready` | release workflow, release checklist, security policy, signing docs, rollback docs |

## Closeout Questions

For each touched surface, answer these before calling the batch done:

- Did Desktop, Android, CLI, backend helpers, examples, docs, changelog, and
  roadmap stay aligned where affected?
- Did the selected verification profiles run, or is the skipped coverage
  explicitly documented?
- Did generated docs or indexes refresh?
- Did English/Persian user-facing copy stay paired?
- Did support bundle or copied support text avoid leaking secrets?
- Did any compatibility path remain? If yes, is it documented and tested?
- Did stale/deprecated code, docs, examples, generated files, and local caches
  get removed?
- Did the change avoid split-brain ownership, duplicate sources of truth, and
  race-prone background state?

## Cleanup Reminders

- Remove temporary `__pycache__`, syntax-check output, local build products, and
  one-off generated files before final verification.
- Do not preserve internal backward compatibility by habit. Keep compatibility
  only when it is an explicit documented surface.
- Donor code, donor binaries, and donor docs remain quarantined unless a batch
  deliberately absorbs them with licensing/security notes.
