# Desktop Installer

Windows local packaging is handled by `build-windows-installer.ps1`.

## Build

From the repository root on Windows:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-windows-installer.ps1
```

The script builds:

- `target\release\mhrv-f.exe`
- `target\release\mhrv-f-ui.exe`

Then it creates:

- `dist\mhrv-f-windows-installer-v<version>\`
- `dist\mhrv-f-windows-installer-v<version>.zip`

## Install

Extract the ZIP and run:

```powershell
powershell -ExecutionPolicy Bypass -File .\install-mhrv-f.ps1
```

Default install location:

```text
%LOCALAPPDATA%\Programs\mhrv-f
```

The installer copies the CLI, desktop UI, launcher, docs, Apps Script assets,
bundled helper tools, and uninstall script. The helper tools include the
Vercel/Netlify JSON relays, Vercel/Netlify XHTTP helpers, and Cloudflare Worker
JSON relay so the installed folder is useful offline after extraction. It also
creates Start Menu and Desktop shortcuts unless `-NoShortcuts` is passed.

## Portable Use

The extracted package is also portable. Double-click `run.bat` from the
extracted folder to initialize the CA and launch the UI.

## Uninstall

Run:

```powershell
powershell -ExecutionPolicy Bypass -File .\uninstall-mhrv-f.ps1
```

The uninstall script removes application files and shortcuts. It keeps user
config by default and does not silently remove the MITM CA. Use the UI
**Remove CA** button, or run `mhrv-f.exe --remove-cert`, before running
`uninstall-mhrv-f.ps1 -RemoveConfig` if you want a complete local wipe.
