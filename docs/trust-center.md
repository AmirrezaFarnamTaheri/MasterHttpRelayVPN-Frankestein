# Trust Center (certificates, trust, and signing)

**Trust Center** is the umbrella for **everything that affects “who you trust”** when
using `mhrv-f`: local CA lifecycle, OS vs browser trust stores, Android
limitations, APK signing expectations, diagnostics, and support bundles.

This document is the **canonical hub**. Desktop includes a dedicated read-only
**Trust** tab plus a brief Help snapshot, Android includes a compact **Trust
Center** card on the main screen, and the CLI exposes the same snapshot through
`doctor`, `trust-center`, and `support-bundle`.

---

## Scope

| In scope | Out of scope (document elsewhere) |
|----------|-----------------------------------|
| Local MITM CA (`ca/`), install/remove/repair | Relay quota math → [`relay-modes.md`](relay-modes.md), [`advanced-options.md`](advanced-options.md) |
| OS trust store vs Firefox NSS | XHTTP/V2Ray clients → relay docs, [`field-notes.md`](field-notes.md) |
| Android user CA + per-app trust behavior | Full-tunnel VPS hardening → [`tunnel-node/README.md`](../tunnel-node/README.md) |
| Release APK signing policy (committed keystore) | GitHub Release process → [`release-checklist.md`](release-checklist.md) |
| Diagnostics + **redaction** expectations | Raw feature design → donor matrix ([`donor-absorption-matrix.md`](donor-absorption-matrix.md)) |

---

## Surface matrix (today)

| Capability | Desktop (`mhrv-f-ui`) | Android | CLI (`mhrv-f`) | Docs |
|------------|----------------------|---------|----------------|------|
| Install / remove local CA | Trust tab action row + [`safety-security.md`](safety-security.md) | Trust Center card opens the CA install/repair flow | `--install-cert` / `--remove-cert` | This page + safety |
| CA / trust **readiness** rows | Dashboard + repair targets | Readiness card + Trust Center card | `doctor` | [`readiness-matrix.md`](readiness-matrix.md) |
| Firefox / NSS cleanup/probe | Removal path attempts profile discovery (see safety); Trust snapshot has read-only profile/tool counts plus redacted per-profile details | N/A (different store) | Same as desktop removal; `support-bundle` exports `trust.json` | Below |
| Support / diagnostics bundle | Trust tab shows bundle manifest counts; Monitor has logs + Doctor | Trust Center explains share discipline; Live logs/config export remain manual | `support-bundle`, `doctor` | [`doctor.md`](doctor.md); includes `trust.json` |
| Signing **explanation** | Trust tab / Help links | Trust Center card + release policy | N/A | [`android-signing.md`](android-signing.md) |

Gaps (intentional roadmap targets): **Android bundle preview/share UI** and
richer Desktop Trust Center repair/probe flows beyond the current read-only
tab. A shared, non-mutating trust snapshot exists in Rust
(`src/trust_center.rs`), is exported as `trust.json` in support bundles,
printed by `mhrv-f trust-center`, and rendered by Desktop. The Android card is
a mobile projection of the same trust vocabulary, but it does not yet call the
Rust snapshot directly because Android certificate state lives in
`AndroidCAStore` and is checked through the platform wrapper.

---

## Local CA lifecycle (MITM modes)

Applicable when `mode` is `apps_script`, `vercel_edge`, or `direct` with HTTPS
through the proxy — see [`relay-modes.md`](relay-modes.md).

| Phase | What happens | User actions |
|-------|----------------|--------------|
| **Create** | Engine generates `ca/ca.crt` + `ca/ca.key` under the config dir | First Start or explicit install |
| **Install** | Public cert added to OS trust (platform-specific) | Desktop **Install CA** or CLI |
| **Verify** | Browser must trust CA for HTTPS MITM | Doctor + manual site load |
| **Repair** | Re-install if cert rotated or corrupted | Install again; readiness IDs may flag `ca.trust` |
| **Remove** | Strip OS trust, attempt NSS nickname cleanup, delete `ca/` when safe | **Remove CA** or CLI; may need elevated shell |

Details and caveats: [**`docs/safety-security.md`**](safety-security.md).

---

## OS trust vs Firefox (NSS)

Many desktops use:

1. **OS trust store** — Chrome / Edge usually pick this up after install.
2. **Firefox** — separate **NSS** database; the app’s remove path attempts to
   delete the matching nickname in discovered profiles (see safety doc).

**Trust Center principle (future UI):** always say **which store** is relevant
when a site still shows `SEC_ERROR_UNKNOWN_ISSUER` after “OS CA installed.”

Current `trust.json` includes a **read-only** browser probe:

- whether `certutil` is available;
- how many Firefox profiles with NSS cert databases were discovered;
- how many Firefox NSS databases contain the mhrv-f CA nickname when `certutil`
  is available;
- how many profiles contain the app-managed `enterprise_roots` marker;
- how many profiles contain a user-owned `enterprise_roots` preference;
- redacted per-profile details in `browser.firefox_profiles`, using only the
  Firefox profile directory name and not the parent path/home directory;
- whether the Linux Chrome/Chromium shared NSS database exists;
- whether the Linux Chrome/Chromium shared NSS database contains the mhrv-f CA
  nickname when `certutil` is available.

It does **not** edit profiles, create NSS databases, or install/remove
certificates; mutation remains limited to explicit install/remove flows.

CLI access:

```bash
./mhrv-f trust-center
./mhrv-f trust-center --json
```

The text form is for humans; `--json` emits the exact shared snapshot shape used
by support bundles.

---

## Android trust limitations

On Android 7+, apps may **opt out** of user CAs. Browsers typically opt in;
banking and chat often do not. VPN vs proxy-only and split routing change
exposure — see **`docs/safety-security.md`** and **`docs/android.md`**.

**Trust Center principle:** Android cannot fully mirror Desktop “install CA →
everything MITM’d” expectations; readiness and Help copy must say so plainly.
The Android main screen now has a **Trust Center** card that:

- shows whether the selected mode needs a local user CA;
- shows whether the exported CA fingerprint is present in `AndroidCAStore`;
- explains that Android apps can opt out of user CAs;
- links trust to the documented committed-keystore / CI release policy;
- reminds users to review exported configs and copied logs before sharing.
- can copy a redacted Android support snapshot with mode, routing, trust,
  deployment-count, and advanced-preservation state while omitting secrets.

---

## Release signing (APK)

Public policy: **`docs/android-signing.md`** (committed keystore, CI authority,
rotation, forks, risks).

Trust Center treats signing as **continuity-of-updates** identity, not proof a
binary was built on a specific maintainer laptop — users verify **GitHub
Release hashes**.

---

## Diagnostics and redaction

| Artifact | Secrets / risk | Guidance |
|----------|----------------|----------|
| `doctor` output | May include hostnames, endpoints; scrub before public post | [`doctor.md`](doctor.md) |
| `support-bundle` | Designed to aggregate logs/config; **review** before sharing | `mhrv-f support-bundle --preview`; includes `manifest.json`, `config.redacted.json`, `doctor.json`, `status.json`, `trust.json`, and `recent-logs.txt` |
| Exported JSON config | Contains `auth_key`, LAN tokens, deployment IDs | Use Advanced export discipline; Android preserves unknown roots ([`android-config-preservation.md`](android-config-preservation.md)) |

CLI preview exists now via `mhrv-f support-bundle --preview`; Desktop shows the
same manifest counts in the Trust tab. Android can copy a smaller redacted
support snapshot from the Trust Center card. Android still needs a future
bundle preview/export UI before it can mirror the desktop manifest table.

Rust-side support redaction is centralized in `src/redaction.rs` for deployment
ID masking, config-secret text scrubbing, LAN-token scrubbing, serverless
auth-key scrubbing, and URL credential removal. Support-bundle config/log
redaction and Doctor tunnel-node URL display use that shared policy so future
diagnostic surfaces do not grow separate masking rules.

Android's smaller copied support snapshot is intentionally separate until a
future JNI/shared bundle exporter exists, but its policy is also no longer
embedded in UI code: `SupportRedaction.kt` owns Android deployment-ID masking
and snapshot generation, with `SupportRedactionTest.kt` guarding the contract.
When Android has a current in-app Doctor result, the snapshot includes only
Doctor availability, overall ok/fail state, item counts, and warning/failing
item IDs. It deliberately omits Doctor titles, details, fixes, endpoint URLs,
and raw JSON so mobile sharing does not become a second unredacted Doctor
export path. The copied-text schema is documented in
[`android-support-snapshot.md`](android-support-snapshot.md).
`tools/check-android-support-redaction.py` is the no-Gradle local/CI drift gate:
it keeps `HomeScreen.kt` as a caller only and requires static assertions for
auth keys, serverless auth keys, LAN tokens, upstream SOCKS5 credentials, raw
unknown JSON, and full deployment IDs.

---

## UX contracts (implementation targets)

These align roadmap Batch **3** engineering:

1. **Serialized repair** — CA install, trust probes, and signing-related flows
   should not overlap ambiguously (one owning modal/job at a time).
2. **Stale results** — If config changes mid-check, UI must not show the old
   probe as current truth without invalidation.
3. **One vocabulary** — Readiness IDs + Doctor should reuse the same labels as
   Trust Center copy ([`readiness-matrix.md`](readiness-matrix.md)).
4. **Shared snapshot** — Trust state that appears in CLI/support/UI surfaces
   should come from the Rust Trust Center snapshot instead of duplicated ad hoc
   checks.

---

## Related readiness IDs (examples)

Exact set evolves in Rust → generated matrix:

- CA / MITM warnings: rows mentioning `ca.` in [`readiness-matrix.md`](readiness-matrix.md).
- LAN exposure: `lan.*` IDs — tied to trust boundaries.

---

## See also

- [`safety-security.md`](safety-security.md) — plain-language MITM + CA file warnings  
- [`android-signing.md`](android-signing.md) — APK signing policy  
- [`doctor.md`](doctor.md) — structured diagnostics  
- [`backend-registry.md`](backend-registry.md) — relay backends vs MITM trust surfaces  
- [`sharing-and-per-app-routing.md`](sharing-and-per-app-routing.md) — LAN token exposure  

Last reviewed: 2026-05-03
