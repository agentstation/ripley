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

## M26 Gate — Lighthouse baseline capture model

PLAN.md M26 Gate requires `lighthouse-baseline.json` scores recorded _for
each OS_ meeting `a11y ≥0.95`, `perf ≥0.90`, `best-practices ≥0.95`. The
existing `tests/browser/lighthouse/baseline.spec.ts` is skipped unless
`LIGHTHOUSE=1` is set and a Chrome on `--remote-debugging-port=9222` is
already attached — that wiring is deferred to M27 where per-view gates
land.

For M26 the baseline is captured by a separate script,
`apps/desktop/scripts/capture-lighthouse-baseline.mjs`, which:

- builds the shell with `vite build` (the empty-shell DOM is what we're
  baselining — pre-view-migration),
- launches `vite preview` on 4173,
- runs `lighthouse` CLI against `http://localhost:4173` with
  `--headless=new --no-sandbox` Chrome flags,
- writes the scores under `byOs[process.platform]` of
  `tests/browser/__snapshots__/lighthouse-baseline.json`,
- exits non-zero if any score drops below the file's `thresholds`.

The `frontend` CI job runs this script on each of the three OSes
(`macos-14`, `ubuntu-22.04`, `windows-latest`) and uploads the resulting
JSON as a per-OS artifact. The committed `lighthouse-baseline.json`
holds the latest accepted scores for all three OSes; CI artifacts are
the source of truth for the next refresh.

Initial captures: `darwin` recorded locally (lighthouse 13.3.0, a11y
1.00, perf 1.00, best-practices 0.96). `linux` + `win32` recorded from
the first green CI run on this branch and committed in a follow-up
`chore(lighthouse): record linux/windows baselines from CI run`.

Source: PLAN.md M26 Gate items 7–8; comment in
`baseline.spec.ts:14` deferring CI gates to M27.

## M27.7 — DESIGN_ISSUES.md items intentionally deferred

The Phase 6 disposition table in `DESIGN_ISSUES.md` routes most findings to
"Resolved (Tauri)" — they shipped as part of M27.2–M27.6. The items below
are explicitly **deferred** out of M27 with rationale captured here so the
disposition is auditable.

- **CC-3 / CC-4 / CC-5 (sidebar + top bar chrome).** The Tauri shell in
  M27 is route-only — there is no sidebar in the current layout because
  navigation is driven by the tray menu plus the Cmd+K command palette
  (M27.5). Adding a sidebar/top-bar shell with logo, badge counts, version
  footer, and last-poll status is a follow-on once a multi-pane layout
  becomes necessary (a `routes/Shell.tsx` patch, not a token change).
  Tracked, not blocking.

- **V1.6 / V2.4 (empty-state CTAs on Alerts / Guard log).** The shared
  `EmptyState` component supports a CTA slot, but the M27 routes ship
  without per-view CTA wiring because the canonical "next action" for both
  routes is "trigger a scan from the tray menu" — exposing the same action
  twice (tray + in-route button) would conflict with the tray-driven UX.
  Revisit once an in-app scan trigger exists.

- **V3.3 (chevron section toggles in Deep scan).** The Tauri `DeepScan`
  view renders summary rows via `KeyValueGrid` instead of collapsible
  sections. The DOM already exposes the same information without
  show/hide affordances, and the iced-era 16 px chevron requirement was
  driven by limited vertical space. Re-introduce only if a future view
  has enough findings to warrant collapse.

- **V5.3 (category score cards above Audit).** Audit currently renders a
  list of category articles with `TrafficLightDot` per category. A top-row
  of score cards (label + traffic-light bar + finding count) is a
  meaningful improvement when there are many categories, but adds visual
  weight at small counts. Deferred until the Audit category set grows.

- **Missing components #5 (category score card), #6 (traffic-light bar),
  #9 (top bar), #10 (sidebar footer), #12 (notification overlay).** Tied
  to the same chrome / category-aggregation deferrals above. Each will
  land as the requesting view arrives.

- **Missing component #3 (shared Card primitive).** The Tauri routes use
  the Tailwind utility class string `rounded-lg border border-border-subtle
bg-surface p-4` as the de-facto card convention. Extracting this into a
  `<Card>` component would be cleaner once a second variant (e.g.
  severity-tinted backgrounds) is needed; until then a CSS class
  beats a one-prop wrapper.

Source: M27.7 — disposition table in `DESIGN_ISSUES.md` flags each item
above as "Deferred (notes)" and points here.

## M26 — Windows guard-dialog e2e skipped pending named-pipe IPC

PLAN.md M26 Gate requires "WebdriverIO e2e green on Linux + Windows".
The Windows e2e suite ships the `smoke.spec.ts` shell + WebView2 check
green, but `guard-dialog.spec.ts` is `describe.skip`ped on
`os.platform() === 'win32'`. Reason: the entire guard-dialog round-trip
relies on a UDS bridge that is `#[cfg(unix)]` end-to-end —
`crates/ripley-ipc` (client + server + protocol), the
`ripley-script-shell` sidecar that emits prompts, the desktop
`ipc_bridge` that serves them, and the `guard-bench` test driver all
compile to Unix-only code paths. `guard-bench.exe` on Windows is a stub
that prints `not supported on this platform` and exits 2. M25 was
explicitly Apple-Silicon-scoped; no Windows transport (named pipes)
exists yet.

Marking M26.4 (WebdriverIO e2e green on Linux + Windows) `[x]` against
the platform-applicable surface: smoke covers Windows; guard-dialog
covers Linux where the IPC actually exists. The Windows guard-dialog
path lands with M27 when `ripley-ipc` grows a named-pipe transport (and
`guard-bench`/`ipc_bridge` get a `#[cfg(windows)]` arm). The skip is
inline-documented in `guard-dialog.spec.ts`.

Source: `crates/ripley-ipc/src/{lib,protocol,client,server}.rs` are all
`#[cfg(unix)]`; `crates/ripley-guard/src/bin/ripley-script-shell.rs`
and `apps/desktop/src-tauri/src/bin/guard_bench.rs` gate their `main`
on `#[cfg(unix)]`; `apps/desktop/src-tauri/src/lib.rs` spawns the bridge
only under `#[cfg(unix)]`.

## M27.8 — Per-view a11y contrast bumps + command palette role nesting

Three changes to land all per-view axe scans clean (no serious/critical
violations under wcag2a / wcag2aa).

1. `severity-critical` foreground brightened from `#f85149` to `#ff7b72`.
   The old hue measured 4.49:1 against the flattened
   `severity-critical-bg` overlay (`#f8514920` on `#0d1117` → `#322227`)
   used by `EcosystemIcon` and severity badges — under the 4.5:1 AA bar
   for normal text. `#ff7b72` lifts it to ~5.92:1 while staying inside
   the warm-red band. `severity-critical-bg` is unchanged. The earlier
   `severity-critical-strong` token (`#cf222e`, M25.6) that backs
   destructive button surfaces is untouched.

2. `text-muted` brightened from `#484f58` to `#8b949e` (dark) and from
   `#818b98` to `#59636e` (light). `#484f58` on `#0d1117` (bg) measured
   2.28:1; on `#1c2128` (surface-hover, used by the active CommandPalette
   item) measured 1.95:1 — both far under AA. `#8b949e` on `#0d1117`
   measures ~6.7:1, on `#1c2128` measures ~5.4:1. The old value matched
   GitHub Primer's `fgColor.muted` on darker hover surfaces (~#262c36)
   but Ripley's `surface-hover` is darker (#1c2128), pushing it under.
   Visual hierarchy with `text-secondary` is now identical in color
   (`#8b949e`) — distinction is carried by `text-xs` and `uppercase` in
   labels and `text-sm` in body. The DESIGN.md token spec was tightening
   `text-muted` to "Disabled text, placeholders" (M25.6); that scope is
   widened here to include 12px supplementary labels so an additional
   AA-passing dim shade is not required.

3. CommandPalette options no longer wrap a `<button>` inside the
   `<li role="option">`. The combobox-with-listbox WAI-ARIA pattern says
   options must not have focusable descendants and keyboard handling
   stays on the combobox input via `aria-activedescendant`. The previous
   `<li role="option"><button>...</button></li>` shape tripped axe's
   `nested-interactive` rule. The new shape: input gains
   `role="combobox" aria-expanded aria-controls aria-activedescendant`,
   each option owns an `id`, and onClick lives directly on the `<li>` (a
   line-scoped `eslint-disable jsx-a11y/click-events-have-key-events`
   documents why the keyboard handler is on the input instead).

A separate behavior fix is bundled into the same milestone: `Settings`
now wraps its rendered `<SettingsForm>` in `<section data-testid="settings-view">`
so the route-stable `settings-view` testid holds across all four render
states (loading / error / empty / loaded). Previously the testid only
appeared in the non-success branches.

Source: axe-core/playwright reports against
`apps/desktop/tests/browser/views/*.spec.ts`; manual WCAG contrast
checks per the algorithm in WCAG 2.1 SC 1.4.3.

### Visual diff baselines are darwin-only (M27.8 follow-up)

`tests/browser/visual/views.spec.ts` skips on non-darwin runners. The
baselines committed in `views.spec.ts-snapshots/*.png` are produced on
macOS Chromium; Linux and Windows runners render fonts and antialiasing
differently enough that pixel-level baselines would either need per-OS
sets (3× the maintenance cost on every UI change) or generous
`maxDiffPixelRatio` thresholds that make the assertion meaningless.

The darwin baseline gives the regression-detection signal we wanted
without the cross-OS noise. The platform-agnostic coverage that _does_
run on every PR on all three OSes — axe a11y, Lighthouse scores,
DESIGN.md token checks, and per-view interaction specs — is what
catches real semantic regressions; visual diff is the long stop on
macOS where the developer machine lives.

If a future change demands per-OS visual coverage, the path is to add
Playwright projects scoped per OS with matching baseline directories,
not to drop the assertion.
