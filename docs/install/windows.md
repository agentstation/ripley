# Ripley on Windows

Ripley ships an MSI installer and a standalone `.exe` for x64 Windows. The
desktop app is a Tauri 2 shell over WebView2; the `ripley-guard` CLI is a
single binary that runs without WebView2.

## System requirements

- **OS:** Windows 10 21H2+ or Windows 11. Windows 11 is the primary test
  target; Windows 10 works but is not in CI past M26
- **Architecture:** x64. ARM64 builds are tracked in [#TODO-arm64-windows]
  but not shipped yet
- **WebView2 Runtime:** Pre-installed on Windows 11; ships with Edge on
  Windows 10. See [WebView2 setup](#webview2-runtime) below

## Install via MSI

1. Download `Ripley-<version>-x64.msi` from the release artifacts
2. Run the installer. SmartScreen will warn on **unsigned** builds — see
   [Code signing](#code-signing) below
3. Ripley launches automatically and shows the tray icon in the system
   notification area
4. The `ripley` CLI is added to `%PATH%` and available in new shells

Uninstall via Settings → Apps → Installed apps → Ripley → Uninstall.

## Code signing

Through Phase 6 / M26, Windows builds are **unsigned**. SmartScreen will
flag the MSI as "Unknown publisher" and require an extra "Run anyway"
click. Signed builds land in M28.2 via Azure Trusted Signing.

If you want to silence SmartScreen before signing ships, right-click the
MSI → Properties → check "Unblock" → Apply. This is per-user and per-file.

## WebView2 Runtime

Ripley uses Microsoft Edge WebView2 to render the UI. Windows 11 ships it.
On Windows 10, it arrived with Edge Chromium (Jan 2020); any current
install has it.

Verify:

```powershell
Get-ItemProperty `
  -Path 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' `
  -Name 'pv' -ErrorAction SilentlyContinue
```

If the key is missing, install the
[Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/?form=MA13LH#download).
The MSI installer also chains the bootstrapper for first-run safety.

## Tray icon

Ripley uses the native Shell_NotifyIcon API via Tauri's tray-icon backend.
No additional configuration is needed on either Windows 10 or 11.

By default Windows 11 hides infrequent tray icons under the chevron
overflow. Pin Ripley by:

1. Click the chevron (^) in the system tray
2. Drag the Ripley icon onto the visible tray area

Or via Settings → Personalization → Taskbar → Other system tray icons →
Ripley → On.

## Global shortcut

`Ctrl+Shift+R` opens the dashboard. The shortcut registers via
`RegisterHotKey` (user32). If another app already owns the binding,
Ripley logs a warning and silently skips registration — the tray icon
still works.

## Building from source

Prerequisites:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools `
  --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install Rustlang.Rustup
winget install pnpm.pnpm
winget install Microsoft.EdgeWebView2Runtime
```

Then:

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --workspace --release
pnpm install --frozen-lockfile
pnpm -F desktop tauri build --target x86_64-pc-windows-msvc
```

MSI lands in `apps\desktop\src-tauri\target\release\bundle\msi\`.

## Troubleshooting

**Tray icon missing after upgrade.** Windows caches icon visibility per
executable hash. After upgrading, run `explorer.exe /restart` from an
elevated prompt or sign out and back in.

**WebView2 fails to launch (`HRESULT 0x80004005`).** WebView2 stores user
data under `%LOCALAPPDATA%\Ripley\EBWebView`. Delete that directory and
relaunch.

**`ripley` CLI not on PATH after install.** Open a fresh terminal. The MSI
adds the install dir to `HKLM\SYSTEM\CurrentControlSet\Control\Session
Manager\Environment\Path`, which only propagates to new shells.

**Antivirus quarantines the MSI.** Pre-signing, some AV vendors heuristically
flag unsigned installers. Add an exclusion or wait for the M28.2 signed
build.
