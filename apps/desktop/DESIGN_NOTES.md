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

## M25.6 — guard dialog WCAG AA contrast adjustments

Two token-level adjustments to land the guard-dialog axe pass (no
serious/critical violations under wcag2a / wcag2aa / wcag21a / wcag21aa):

1. `severity-high` foreground brightened from `#db6d28` to `#f0883e`.
   The old hue measured 4.09:1 against `#1c2128` (dialog surface), under
   the 4.5:1 AA threshold for normal text. `#f0883e` lifts it to ~4.93:1
   while staying inside the warm-orange band. `severity-high-bg` is
   unchanged (`#db6d2820`) because backgrounds aren't constrained by the
   text-contrast rule. Updated in `apps/desktop/src/styles/theme.css` and
   in DESIGN.md (severity table + token list).

2. Guard-dialog section headers ("Matched rules", "Script") switched from
   `text-text-muted` to `text-text-secondary`. DESIGN.md scopes
   `text-muted` to "Disabled text, placeholders" only — section labels
   are not disabled UI. `text-text-muted` (`#484f58`) measures 1.95:1 on
   the dialog surface, far below AA; `text-text-secondary` (`#8b949e`)
   measures ~6.4:1 and matches the intended semantic role.

3. Primary button background moved from `accent` (`#58a6ff`) to a new
   `accent-strong` token (`#1f6feb`). White on `#58a6ff` only reaches
   2.52:1; white on `#1f6feb` lifts to ~4.57:1 (just above the AA
   normal-text bar). `--color-primary` in theme.css points at the new
   shade; `--color-accent` is untouched so the brand blue still drives
   links, sidebar active item, and accent-muted highlights. DESIGN.md
   token list and Button spec updated to reference `accent-strong`. The
   shared `text-on-accent` token stays at `#ffffff` because it is also
   used as the toggle-knob color, which is a visual control rather than
   text.

4. Destructive button background moved from `severity-critical`
   (`#f85149`) to a new `severity-critical-strong` token (`#cf222e`).
   White on `#f85149` only reaches 3.35:1; white on `#cf222e` lifts to
   ~5.5:1. `--color-destructive` in theme.css points at the new shade;
   `--color-severity-critical` is untouched so the badge foreground and
   "critical" severity affordances still render in the brighter red on
   dark tinted backgrounds where contrast is already adequate.

Source: axe-core/playwright report against
`apps/desktop/tests/browser/a11y/guard-dialog.spec.ts`; manual WCAG
contrast checks per the algorithm in WCAG 2.1 SC 1.4.3.
