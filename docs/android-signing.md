# Android release signing policy

This document explains why **release APKs** use a **committed** keystore, what
that implies for security and continuity, and how it relates to **CI** as the
source of truth for official builds.

## What is committed

- **Keystore file:** `android/app/release.jks`
- **Gradle wiring:** `android/app/build.gradle.kts` — `signingConfigs.release`
  points at that file and uses the store/key passwords documented next to the
  config (same pattern as the inline comments there).

The signing config exists so **every official release APK shares the same
signing identity**. If the keystore changed per machine or per CI run, Android
would treat upgrades as a different app (`INSTALL_FAILED_UPDATE_INCOMPATIBLE`).

## Why this is acceptable here (and the trade-off)

**Goal:** predictable installs for users who download APKs from GitHub Releases,
without requiring every contributor to inject secrets into a local Gradle file.

**Trade-off:** anyone with this repository can build an APK that **matches the
public release key**. That is normal for open-source apps that ship a fixed
release key: the key proves **continuity of updates**, not that the binary came
from a specific maintainer machine. **Binary authenticity** for end users relies
on **trusting the release channel** (GitHub Release assets, `SHA256SUMS.txt`,
and your own verification habits), not on hiding the keystore.

## CI source of truth

Official artifacts (**universal and per-ABI APKs**, desktop archives, checksums)
are produced and published only by **`.github/workflows/release.yml`** on tag /
workflow-dispatch. Local `./gradlew assembleRelease` is for development;
**do not** treat ad-hoc local APKs as “the” release unless you are deliberately
testing.

Maintainers should align versionCode/versionName in Gradle with the tagged
release before shipping.

## Risks

- **Impersonation builds:** Same key + public repo ⇒ third parties can sign
  builds with the same identity. Mitigate by publishing hashes on the GitHub
  Release and verifying downloads.
- **Key compromise:** If `release.jks` or passwords leak in a way that matters
  for your threat model, treat it like any app signing key leak: plan rotation
  (below).
- **Passwords in repo:** The Gradle file documents passwords required for the
  committed keystore. This is intentional for reproducible CI; it is not a
  secret in the classical sense.

## Rotation (high level)

1. Generate a **new** keystore and keep it in a secure channel if you move to
   CI-only secrets, **or** commit a new `release.jks` if the project continues
   the “fixed public key” model.
2. Bump the app’s identity only if you **change the signing key** without a
   migration path — users must uninstall the old app or you must use Play App
   Signing / a controlled migration (not applicable to raw GitHub APK sideload).
3. Update `build.gradle.kts` to reference the new store file, alias, and
   passwords.
4. Document the change in release notes and tag a new version.

## Recovery

- **Lost keystore:** You cannot recover the private key. You must ship under a
  new key (new package id or instruct users to uninstall) or restore from
  backup if you kept one.
- **Wrong password in Gradle:** Release builds fail at signing; fix Gradle to
  match the committed keystore.

## Forks

Forks that publish **their own** app under the same `applicationId` with this
same keystore will produce **colliding** updates. Forks should change
`applicationId` and use **their own** keystore if they distribute to users.
