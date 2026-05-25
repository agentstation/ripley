# Phase 6 design notes

Judgment calls made during the M24-M28 autonomous run. Each entry: what, why,
source-of-truth path.

## M24.1 — stub `ripley-desktop` crate

PLAN.md M24.1 adds `apps/desktop/src-tauri` to the root Cargo.toml `members`
array, but the directory only gets its full Tauri scaffold in M24.2. To keep
`cargo build --workspace` green at the M24.1 boundary, M24.1 lands a minimal
stub: `apps/desktop/src-tauri/Cargo.toml` with workspace-inherited metadata
and a single `fn main() {}` binary. M24.2 overwrites both files with the real
Tauri 2 scaffold.

Source: PLAN.md M24.1 verify command requires exit 0 on cargo build.

## M24.1 — `just check` skips JS steps when `apps/desktop/package.json` absent

`just check` is wired in M24.1 but the React app doesn't exist until M24.2.
The recipe runs Rust checks unconditionally and JS checks (`pnpm -F desktop
typecheck`, `lint`) only when `apps/desktop/package.json` exists. `pnpm audit`
runs against the root workspace either way. Same conditional applied to
`fmt`, `lint`, and `test` recipes.

Source: PLAN.md M24.1 verify command requires `just check` exit 0; PLAN.md
M24.6 fully wires the JS lint/format/test rigging.

## M24.4 — visual tray-only confirmation deferred to M28 release smoke

PLAN.md M24.4 Verify says `pnpm tauri dev — visually confirm tray-only
behavior on macOS dev box`. The autonomous run cannot drive a windowed
session. M24.4 ships compile-clean code that wires the documented APIs:

- `ActivationPolicy::Accessory` on macOS (no Dock icon)
- `TrayIconBuilder` with Open / Quit menu items
- `tauri-plugin-global-shortcut` registering Cmd+Shift+R → `prewarm::toggle`
- `core:tray:default` + `global-shortcut:default` capability permissions
- Window already starts with `visible: false`, `skipTaskbar: true`

Manual smoke (no Dock icon, tray menu opens, Cmd+Shift+R toggles the
window) rolls into the M28 v0.6.0 release-cut gate which already requires
hand-validation of the full app.

Source: PLAN.md M28 Phase 6 gate explicitly requires a manual smoke run
before tagging the release.

## M24.6 — ignore RUSTSEC-2025-0098 (unic-ucd-version unmaintained)

`cargo deny check` started failing on RUSTSEC-2025-0098 (`unic-ucd-version`
unmaintained, advisory date 2025). The crate is a transitive dep pulled in
by Tauri:

`tauri -> tauri-utils -> urlpattern -> unic-ucd-ident -> unic-ucd-version`

No security vulnerability — just unmaintained. No upstream fix path exists
until `urlpattern` migrates off the `unic-*` crates. This matches the
existing precedent in `deny.toml` for the gtk-rs `RUSTSEC-2024-04xx`
entries (also unmaintained-only, also transitive). Added to `deny.toml`
ignore list with a re-evaluation note for Tauri 2.12.

Source: deny.toml comment; precedent set by existing gtk-rs entries.
