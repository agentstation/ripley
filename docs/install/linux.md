# Ripley on Linux

Ripley ships an `.AppImage`, `.deb`, and `.rpm` for x86_64 Linux. The desktop
app is a Tauri 2 shell over WebKitGTK; the `ripley-guard` CLI is a single
static binary that runs without GTK.

## System requirements

- **Distribution:** Ubuntu 22.04+, Fedora 38+, Debian 12+, or equivalent
- **Display server:** X11 or Wayland. Hyprland and other wlroots compositors
  are supported with caveats — see [Hyprland](#hyprland-and-wlroots) below
- **Architecture:** x86_64 (aarch64 builds are tracked in
  [#TODO-aarch64-linux] but not yet shipped)

## Runtime libraries

The bundled apps dynamically link against:

| Library | Purpose | Provided by |
|---|---|---|
| `libwebkit2gtk-4.1` | Web view runtime | `libwebkit2gtk-4.1-0` (Ubuntu 22.04+) |
| `libgtk-3` | Window chrome | `libgtk-3-0` |
| `libayatana-appindicator3` | Tray icon (SNI) | `libayatana-appindicator3-1` |
| `librsvg2` | Icon rendering | `librsvg2-2` |
| `libsoup-3.0` | WebKit networking | `libsoup-3.0-0` |
| `libjavascriptcoregtk-4.1` | WebKit JS engine | `libjavascriptcoregtk-4.1-0` |
| `libxdo` | Global shortcuts | `libxdo3` |

The `.deb` / `.rpm` packages declare these as dependencies. The `.AppImage`
bundles WebKit but still expects host GTK + AppIndicator libraries.

## Building from source

Install build deps (Ubuntu 22.04 / Debian 12):

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  libxdo-dev
```

Fedora 38+:

```bash
sudo dnf install webkit2gtk4.1-devel gtk3-devel libayatana-appindicator3-devel \
  librsvg2-devel libsoup3-devel javascriptcoregtk4.1-devel xdotool-devel
```

Then:

```bash
cargo build --workspace --release
pnpm install --frozen-lockfile
pnpm -F desktop tauri build --target x86_64-unknown-linux-gnu
```

Bundles land in `apps/desktop/src-tauri/target/release/bundle/`.

## Tray icon

Ripley uses the StatusNotifierItem (SNI) protocol via
`libayatana-appindicator3`. Out-of-the-box tray support varies by desktop
environment.

### KDE Plasma 6 — works as shipped

SNI is the native tray protocol on KDE. No extra configuration needed.

### GNOME 45+ (stock) — needs an extension

Stock GNOME removed legacy tray icons in 3.26 and only renders SNI via an
extension. Install one of:

- [AppIndicator and KStatusNotifierItem Support](https://extensions.gnome.org/extension/615/appindicator-support/)
  (recommended)
- [Tray Icons: Reloaded](https://extensions.gnome.org/extension/2890/tray-icons-reloaded/)

Without an extension, Ripley's process runs but no tray icon appears. The
CLI (`ripley`) still works.

### Hyprland and wlroots

wlroots compositors do not implement SNI directly. Install a tray host such
as [`waybar`](https://github.com/Alexays/Waybar) configured with the `tray`
module, or [`gBar`](https://github.com/scorpion-26/gBar). Ripley's tray icon
will appear in the host's tray area.

If no SNI host is running, Ripley logs a warning at startup but otherwise
functions normally — the dialog still opens on guard events and the global
shortcut (Cmd/Super+Shift+R) still toggles the dashboard.

### Sway

Same as Hyprland: install a tray host (waybar) configured for SNI.

## Troubleshooting

**Tray icon doesn't appear.** Confirm SNI is available:

```bash
gdbus introspect --session --dest org.kde.StatusNotifierWatcher \
  --object-path /StatusNotifierWatcher | head
```

If the call fails with `Error: ServiceUnknown`, no SNI host is registered.
Install one per the desktop-environment section above.

**`libwebkit2gtk-4.1` not found on Ubuntu 22.04.** Some early 22.04 minimal
images ship without the package in their default sources. Enable `universe`
and re-run apt-get:

```bash
sudo add-apt-repository universe
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-0
```

**Global shortcut doesn't fire.** Wayland compositors require explicit
permission for global shortcuts. On GNOME, grant via Settings → Keyboard →
Customize. On KDE, allow via System Settings → Shortcuts. Hyprland users
can bind through `hyprctl` directly.
