## Product audit (implementation driver)

This document is an *implementation artifact* that captures the current state of the repo and the main newcomer failure points, so we can fix them systematically.

### Brand naming rules (must not regress)
- **User-facing product name**: **MasterHttpRelayVPN-Frankestein**
- **Short/internal name**: `mhrv-f`
  - Keep `mhrv-f` in: binary names, config directory names, paths inside commands, protocol/header identifiers.
  - Prefer the full name in: window/app titles, docs headings, About/help, “product name” strings.

### User-facing surfaces inventory (where users see the product)
- **Desktop UI (Rust / egui)**:
  - Window title: `src/bin/ui.rs` shows the full product name and version (should not include the literal `mhrv-f` in the title).
  - About/help area: needs a single clear “short name: `mhrv-f`” note (not scattered).
- **Android UI (Kotlin / Compose)**:
  - App name: `android/app/src/main/res/values/strings.xml` already uses full name.
  - Most visible copy is already in resources, but we still need to ensure *all* visible strings come from `strings.xml` (EN+FA parity).
- **Docs**:
  - `README.md` is the “full guide” and includes a quick onboarding flow, but we still need a true docs hub (`docs/index.md`) and a symptom-driven troubleshooting decision tree.
  - `SF_README.md` contains duplicated sections (English quickstart appears twice); it should become a short pointer/landing page instead of a second guide that can drift.
- **Release artifacts / launchers**:
  - Launcher scripts (`run.bat` / `run.command` / `run.sh`) and `dist/run.bat`: must keep binary names, but any user-facing title text should use full name.
  - macOS `Info.plist`: branding + icon wiring must be checked (title/icon alignment).

### Top newcomer failure points (what we must design for)
These are repeatedly referenced in the current README quickstart and are the first things users report when it “doesn’t work”.

- **(F1) Can’t reach `script.google.com` to deploy the relay**
  - Needs a clear bootstrap path: `google_only` mode + a short, step-by-step “bootstrap” section.
- **(F2) “Connect disabled” / unclear prerequisites**
  - Must show a blocker list and direct next actions in both desktop and Android UI.
- **(F3) CA / certificate trust confusion**
  - Must explain: why it’s needed, how to install, how to repair/reinstall, and how to remove it safely.
  - Must include explicit warnings not to share `ca.key`.
- **(F4) Wrong `google_ip` / wrong SNI host pairing**
  - Needs: auto-detect, SNI pool tester guidance, and a troubleshooting decision tree.
- **(F5) Apps Script quota exhaustion**
  - Must encourage multiple deployment IDs/account pools and explain what “quota exhausted” looks like.
- **(F6) “Connected but sites don’t load”**
  - Usually a combination of (F3) CA trust + (F4) google_ip issues; needs a short “verify” checklist and a “doctor/test” guided flow.

### Measurable success criteria (what “better” means)
- **Onboarding time**: a new user can reach “first successful browsing” in ≤10–15 minutes.
- **UI clarity**: both desktop and Android show (a) current state, (b) blockers, and (c) the next action without needing external docs.
- **Docs clarity**: a single hub routes users by goal, and troubleshooting is symptom-driven.
- **Trust/safety**: CA lifecycle is explained plainly and safely (install/repair/remove).

