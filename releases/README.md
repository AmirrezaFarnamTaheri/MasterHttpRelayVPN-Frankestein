# Offline / Blocked-Releases Distribution

This folder is reserved for release artifacts that may be mirrored in the
repository for users who cannot open the GitHub Releases page.

## When GitHub Releases Is Blocked

Use one of these fallback paths:

1. Download the repository as a ZIP from the green **Code** button.
2. Extract the ZIP.
3. Open the extracted `releases/` folder.
4. Pick the archive for your platform.

If `git clone` works better than browser download:

```sh
git clone https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein.git
cd mhrv-f/releases
```

## Expected Artifact Names

- Windows: `mhrv-f-windows-amd64.zip`
- macOS Intel: `mhrv-f-macos-amd64.tar.gz` or app ZIP
- macOS Apple Silicon: `mhrv-f-macos-arm64.tar.gz` or app ZIP
- Linux x86_64: `mhrv-f-linux-amd64.tar.gz`
- Linux arm64: `mhrv-f-linux-arm64.tar.gz`
- OpenWRT / Alpine static builds: `mhrv-f-linux-musl-*.tar.gz`
- Android: `mhrv-f-android-universal-*.apk`

The exact list can vary by release. If this folder only contains
`.gitattributes`, the maintainer has not mirrored binaries in the repository
for the current version; use the normal Releases page or a trusted mirror.

## After Download

Windows:

1. Extract the ZIP.
2. Run `run.bat`.
3. Accept the UAC prompt if you choose to install the MITM CA.

macOS / Linux:

```sh
tar xzf mhrv-f-linux-amd64.tar.gz
cd mhrv-f-linux-amd64
./run.sh
```

Android:

1. Copy the APK to the phone.
2. Open it from Files.
3. Allow "Install unknown apps" for that app.
4. Follow the Android guide in `docs/android.md`.

## Integrity

Prefer artifacts from the official release tag. If hashes are published next
to the release, verify them before running:

```sh
sha256sum mhrv-f-linux-amd64.tar.gz
```

On Windows PowerShell:

```powershell
Get-FileHash .\mhrv-f-windows-amd64.zip -Algorithm SHA256
```

Compare the printed SHA-256 with the maintainer-published value. If the
values differ, do not run the binary.
