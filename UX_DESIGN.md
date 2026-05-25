# UX / DX Design

First-principles redesign of Ripley's user and developer experience for the
Phase 6 Tauri rewrite. This document chooses Base UI primitives (shadcn `--base
base-ui`, style `base-vega`) for every interaction surface in [WORKFLOW.md](WORKFLOW.md),
defines the keyboard and density model, and locks the DX patterns the M27 view
migration will follow.

See:
- [WORKFLOW.md](WORKFLOW.md) — the 14 user streams this design serves
- [UI.md](UI.md) — view wireframes and per-view interaction specs
- [DESIGN.md](DESIGN.md) — token vocabulary (colors, type, spacing)
- [STACK_DECISION.md](STACK_DECISION.md) — locked stack/tooling/patterns
- [PLAN.md](PLAN.md) M24–M28 — implementation milestones
- [SETTINGS.md](SETTINGS.md) — the configuration surface the settings view binds to


---


## Design principles (re-derived)

These are the principles that govern every component choice below. They subsume
and extend the five in [UI.md](UI.md#principles).

1. **Latency is a feature.** Guard dialog cold path < 500ms (first dialog
   *after app launch*, not first ever — system reboots add Tauri launch
   cost on top); warm path < 50ms. Every other surface ≤ 100ms p99 from
   intent → painted pixel. **No skeletons** — the layout chrome (sidebar,
   header, content frame) renders immediately on launch; the content
   region shows either real data or an honest empty state, never a
   fake-data placeholder.
2. **Severity is the only hierarchy.** Accent blue is *only* for interaction
   affordance. Status, urgency, and risk all use the severity ramp.
3. **Keyboard-first, mouse-complete.** Every action reachable from `?` cheatsheet.
   `Cmd/Ctrl+Shift+R` opens the window. `Cmd/Ctrl+K` opens the command palette.
   Letter keys (`a`, `g`, `m`, `,`) jump between sidebar sections.
4. **No dead ends, toasts only for invisible side effects.** Every screen has
   a next action. Toasts are reserved for side effects the UI cannot show
   directly — clipboard copies, file exports. If the side effect is visible
   (trusted package appears in the list, alert disappears from the feed),
   that visibility *is* the confirmation. Toasts never carry the only copy
   of an error. (Precedent: Raycast HUD, macOS HIG.)
5. **Density over decoration.** 13px body, 12px monospace, 4px grid, zero
   gradients, zero illustrations. Borders for delineation, not shadow.
6. **Trust through evidence.** Show the matched line, the resolved URL, the
   exact version range. Truncation hides the thing a developer is verifying.
7. **The tray app is a server.** The desktop window is a privileged client.
   The CLI is an unprivileged client. All three render the same model.
8. **Fail safe, default block.** The 30s guard timeout defaults to Block. The
   advisory feed cache is stale-while-revalidate. Network errors never silently
   suppress findings.


---


## Component vocabulary

Ripley's UI is a small, deliberate subset of the shadcn / Base UI catalogs.
Components below are scoped to those we *will* install. Anything not on this
list requires a PR-time justification.

### Used (install in M24.3 or earlier in M27)

| shadcn slug      | Base UI primitive          | Where it lives in Ripley                                                   |
|------------------|-----------------------------|-----------------------------------------------------------------------------|
| `button`         | `useRender` + native button | All buttons. Variants: `primary`, `secondary`, `danger`, `ghost`, `icon`.   |
| `dialog`         | `Dialog`                    | Guard interception dialog (M25), confirm/destructive prompts.               |
| `alert-dialog`   | `AlertDialog`               | Destructive confirms (revoke token, contain process, uninstall).            |
| `sheet`          | `Dialog` (variant=side)     | Alert detail panel (slides in from right). Settings sub-pages on narrow.    |
| `popover`        | `Popover`                   | Row "more actions" menus, severity legend, status tooltips with actions.    |
| `tooltip`        | `Tooltip`                   | Icon-only buttons, disabled state explanations, timestamp hover.            |
| `combobox`       | `Combobox` (+ `Autocomplete` for type-ahead surfaces) | Command palette (Cmd+K, see W-Palette spec), package search, rule search, trust autocomplete (with `allowsCustomValue`). |
| `select`         | `Select`                    | Settings enums (`mode`, `harness`, `min_severity`, `poll_interval`).        |
| `switch`         | `Switch`                    | Boolean settings (`launch_at_login`, `strict`, `require_lockfile`).         |
| `checkbox`       | `Checkbox` + `CheckboxGroup`| Bulk-select rows in guard log / alerts list (M27.4 remediate-all).          |
| `radio-group`    | `Radio`                     | Guard `mode` tri-state (`strict` / `audit` / `off`).                        |
| `input`          | `Input` + `Field`           | Add-trust input, add-root input, search box header.                         |
| `number-field`   | `NumberField`               | `poll_interval_secs`, `timeout_secs`. (Replaces raw `<input type=number>`.) |
| `form`           | `Form` + `Field` + `Fieldset` | Settings view sections.                                                    |
| `label`          | `Field.Label`               | All form labels.                                                            |
| `tabs`           | `Tabs`                      | Deep scan report sections (Vulns / IOCs / Persistence / Creds / MCP).       |
| `accordion`      | `Accordion`                 | Audit-report category groups; expandable guard-log rows.                    |
| `collapsible`    | `Collapsible`               | Alert row "show evidence" reveal; settings advanced section.                |
| `separator`      | `Separator`                 | Sidebar groups, dialog title divider.                                       |
| `scroll-area`    | `ScrollArea`                | Long lists (alerts, guard log, rule registry); guarantees consistent gutter. |
| `progress`       | `Progress`                  | Scan progress in tray menu; advisory cache refresh.                         |
| `meter`          | `Meter`                     | Traffic-light bars in audit + harden views.                                 |
| `toolbar`        | `Toolbar`                   | Alerts header (Scan / Deep Scan / filter / sort).                           |
| `menubar`        | `Menubar`                   | macOS app menu only (File / Edit / View / Help). No menubar on Linux/Windows.|
| `context-menu`   | `ContextMenu`               | Right-click on alert rows ("Fix", "Copy prompt", "Dismiss", "Open path").   |
| `dropdown-menu`  | `Menu`                      | Row "⋯" actions, status-bar settings, harness picker.                       |
| `sonner` (toast) | `Toast`                     | Side-effect confirmations only ("Copied", "Trusted", "Dismissed").          |
| `badge`          | n/a (style only)            | Severity pills, sidebar counts, guard log status.                           |
| `card`           | n/a (style only)            | Alert rows, summary cards, settings sections.                               |
| `table`          | n/a (style only)            | Guard log; audit-finding rows. Use TanStack Table v8 for sort/filter.       |
| `data-table`     | TanStack Table v8           | Guard log virtualization, deep-scan tables (M27.4).                         |
| `sidebar`        | n/a (style only)            | The left nav. Custom — Base UI has no Sidebar primitive.                    |
| `kbd`            | n/a (style only)            | Inline keybinding hints in palette and tooltips.                            |

### Rejected (do not install)

| Component                            | Reason                                                                |
|--------------------------------------|------------------------------------------------------------------------|
| `skeleton`                           | DESIGN.md forbids skeletons. Render data or render nothing.            |
| `carousel`                           | No carousel surface in any workflow. Information loss in a dense UI.   |
| `aspect-ratio`                       | No images or video in Ripley.                                          |
| `avatar`                             | No people, no team UI. Author of a community rule is a text string.    |
| `breadcrumb`                         | Only two nav levels (sidebar + detail). Breadcrumbs are overkill.      |
| `chart`                              | No time-series surface in v1. Reconsider for Phase 7.                  |
| `calendar` / `date-picker`           | No date inputs anywhere. Timestamps are read-only.                     |
| `pagination`                         | Lists virtualize via TanStack Virtual — pagination would hide finds.   |
| `resizable`                          | Window is single-pane + optional slide-in sheet. No split-pane needs.  |
| `drawer` (bottom-sheet variant)      | Sheet (side) covers every reveal need; bottom drawers feel mobile.     |
| `slider`                             | All numeric settings are typed values (NumberField), not ranges.       |
| `toggle-group`                       | Use `radio-group` (semantic) for `mode` tri-state.                     |
| `hover-card` / `preview-card`        | Hover reveals are accessibility traps. Use click → popover.            |
| `input-otp`                          | No code-entry surface.                                                 |
| `navigation-menu`                    | Custom sidebar replaces it; no horizontal nav anywhere.                |

### Reserved (Phase 7+)

`chart` (forensic timeline), `command` standalone (currently composed via
`combobox`), `drawer` (mobile companion app — out of scope).


---


## Workflow → component mapping

Every stream from [WORKFLOW.md](WORKFLOW.md) maps to a concrete set of
components, a keyboard path, and the IPC events that drive it. Stream
numbering matches WORKFLOW.md.

### W1 — Setup & onboarding

Pattern: three-step inline wizard in the dashboard content area (precedent:
Linear onboarding, GitHub Desktop first-run, Tower setup). **No popovers, no
modal dialog** — dialogs feel installer-ish, popovers dismiss on outside-click
and don't compose with the native folder picker.

- **Welcome card** uses three numbered `Card`s stacked vertically. Each card
  shows its current state (pending / done) and its primary action inline.
- **Step 1 (Add project roots)**: clicking `Button[primary] "Add Project Root"`
  calls Tauri `dialog::open` directly (native folder picker). Selected root
  appears in a list of `Card[compact]` rows, each with the path (mono) and a
  trailing `Button[icon]` (✕). **Drag-and-drop**: a folder dragged onto the
  window from the OS file manager is also accepted as a project root (Tauri
  `FileDrop` event, precedent: VS Code, GitHub Desktop, iTerm). Once ≥ 1
  root exists, step 1 marks done and step 2 unlocks.
- **Step 2 (Install guard)**: presents the `ripley guard install` command in
  a `command-block` with a copy icon. The daemon polls every 5s for shim
  presence; when detected, step 2 marks done automatically without user
  intervention. No "I did it" button — passive detection only.
- **Step 3 (First scan)**: `Button[primary]` triggers `commands.scan()`. The
  button enters `aria-busy` state, disables, and the label changes to
  "Scanning…". A `Progress` row appears in the card below the button (not a
  spinner inside the button). On completion the welcome card is replaced by
  the normal alerts view.
- **Keyboard**: `1` / `2` / `3` jump focus to each step. `Enter` activates the
  focused step's primary action. `Esc` is a no-op (no overlay to close).

### W2 — Proactive scan

- **Trigger surfaces**: tray menu item ("Scan Now"); dashboard `Toolbar` button;
  command palette entry ("Scan…"); shell.
- **Header strip**: small `Progress` bar at the top of the alerts view while
  scanning; disappears on completion. No spinner anywhere — bar or nothing.
- **Triggering button feedback**: the button that initiated the scan
  (toolbar, tray menu, palette) disables, gets `aria-busy="true"`, and the
  label flips to "Scanning…". On completion it returns to "Scan Now". No
  spinner glyph — the label change + disabled state is the feedback.
- **Result delivery**: scan results push to alerts list via a Rust→React event
  (`event::emit("scan-complete", ScanSummary)`); TanStack Query
  invalidates `["alerts"]`. New rows fade in (200ms), no list jump (sorted
  insertion).
- **Severity escalation**: when an existing Low finding gets re-classified
  to High by a fresh advisory poll, the row re-sorts to its new position
  with the same 200ms fade. Treating an escalation as silent (no animation)
  would let users miss that something changed.
- **Empty result**: `Card` with `severity-clean` shield and "No findings",
  matching DESIGN.md empty-state spec.
- **Keyboard**: `s` = scan now from anywhere in the window.

### W3 — Continuous monitoring

The full tray surface (icon states, menu variants, click semantics, refresh
cadence, hotkey interactions, Quit safety) is specified separately in
**W-Tray** below. This section covers the dashboard-side monitoring view.

- **Status row** in dashboard header: a single `Badge` ("● Monitoring · 12
  projects") + `body-sm` "Last poll: 2m ago". No live ticking — the timestamp
  freezes and re-renders on the next poll event. Tickers are visual noise.
- **Heartbeat**: TanStack Query subscribes to `event::on("monitor-tick")`
  and re-renders only the status row.
- **Pause** is a `Switch` in the status bar's `DropdownMenu`; toggles
  `[monitor] enabled` via `commands.set_config_field`. The same toggle is
  the `Pause Monitoring` ↔ `Resume Monitoring` item in the tray menu (see
  W-Tray) — both surfaces flip together.

### W4 — Install interception (the critical path — M25)

- **Component**: `Dialog` (Base UI) styled per DESIGN.md `surface-elevated`,
  600px max width, no backdrop blur. **Always-on-top of every app** (Tauri
  `set_always_on_top(true)` — `NSPanel` semantics on macOS), **not modal to
  other apps** (the user may still interact with their terminal). If
  dismissed without action, the terminal that triggered the install stays
  blocked on the script-shell's UDS read until the 30s timer fires and
  defaults to Block.
- **Multi-monitor placement**: dialog opens on the monitor containing the
  active terminal that triggered the install (Tauri queries the focused
  window's display). Falls back to the focused display if the trigger
  window cannot be resolved.
- **Latency**: window pre-warmed at startup (hidden), pre-rendered with a
  skeleton-free placeholder dialog component. First show < 500ms cold,
  < 50ms warm. Measured via `performance.mark` in `commands/guard.rs`.
- **Layout** matches [UI.md § Guard Interception Dialog](UI.md#guard-interception-dialog):
  severity strip, package@version, code block with highlighted lines (`code`
  token, severity-bg on matched line), matched-rules list, action row.
- **Actions**: four `Button`s in a `Toolbar`. `Allow Once` = primary,
  `Block` = danger, `Always Trust` = secondary, `Inspect` = ghost → opens
  dashboard `Sheet` with full analysis.
- **Countdown**: a `Meter` in the title bar (not animated text — `Meter`
  has native ARIA semantics) plus the literal "Block (default in 12s)"
  label on the Block button.
- **Keyboard**: `A` / `B` / `T` / `I`; `Esc` = Block (fail-safe).
- **IPC**: `ripley-script-shell` → UDS → Rust tauri command emits
  `event::emit("guard-prompt", GuardPromptPayload)`; React component
  mounts the dialog; user choice returns via `commands.guard_respond()`.

### W5 — Alert-driven fix

- **Alert row**: `Card[hover]` with severity dot, monospace package@version,
  advisory summary, project path. Trailing area: `Button[primary] "Fix"` for
  high/critical, `Button[ghost] "Dismiss"` for low.
- **Detail**: clicking opens a right `Sheet` (Base UI `Dialog` side variant).
  Sheet contains all advisory metadata + affected-projects table + action
  row with split `DropdownMenu` for harness choice ("Fix with Claude ▾" →
  Claude / Codex / OpenCode). Closing the sheet returns focus to the row
  that opened it and preserves the `j`/`k` cursor position.
- **Bulk fix (sticky action bar pattern)**: precedent: Gmail, Linear,
  GitHub Issues. `Checkbox` per row; header `Checkbox` selects all
  visible. Once ≥ 1 row is selected, a **sticky `Toolbar` slides up from
  the bottom of the alerts viewport** (not appears-in-header, which would
  cause layout jump). The bar shows: "N selected" count, `Button[primary]
  "Fix selected ▾"` (split dropdown for harness), `Button[ghost] "Dismiss
  selected"`, trailing `Button[icon]` (✕) to clear selection. `Esc` also
  clears selection.
- **Context menu == row "⋯" menu**: right-clicking a row opens the same
  action set as clicking the row's "⋯" `DropdownMenu`. Both are bound to
  the same component (`<AlertRowActions />`) so the action lists cannot
  drift apart.
- **Keyboard**: `j` / `k` move focus down / up; `Enter` opens detail;
  `f` triggers Fix on focused row; `x` toggles row selection; `Shift+x`
  range-select; `Esc` closes sheet OR clears bulk selection (whichever is
  active); `d` dismisses focused row.

### W6 — Post-breach forensics

A forensic report is meant to be *scanned whole*. Tabs would let a user
finish reviewing without ever seeing the IOC section. Canon: Snyk,
CrowdStrike, Lighthouse, Trivy all use a linear scroll layout with
summary tiles at the top.

- **Layout**: top row of summary `Card`s (Vulns / IOCs / Persistence /
  Creds at Risk / MCP) with count + severity bg color. Below: a single
  `ScrollArea` containing one `Accordion` per category, in fixed order
  (Vulns first, MCP last). Sections containing any non-clean finding are
  **expanded by default**; all-clean sections are collapsed.
- **Per-section**: each finding is a row with severity, description, and
  a `command-block` for the remediation command. Trailing copy icon.
- **Dead Man's Switch banner**: `Card` with `severity-critical-bg`
  positioned **immediately above** the action row (`[Remediate All]
  [Export Report] [Copy Rotation Checklist]`). The banner is a hard
  pre-flight gate — the user must scroll past it to reach `[Remediate
  All]`. The rotation checklist is an ordered list (order is
  load-bearing), not bullets.
- **Export**: `Button[secondary] "Export Report"` → Tauri `dialog::save`
  picker → JSON written via `commands.export_report()`. Confirms success
  with a `Toast` (clipboard-style — saving to disk is invisible side
  effect).
- **Keyboard**: clicking a summary tile scrolls to the matching section
  and focuses it. `1`–`5` are the same: jump-scroll to section N.

### W7 — Active containment

Containment is *reversible* (Release exists), so type-back-to-confirm is the
wrong friction level — that pattern is reserved for irreversible destruction
(revoke npm token, uninstall Ripley, delete a custom rule). Precedent:
GitHub uses type-back for repo delete (irreversible) but a plain confirm
for "archive" (reversible).

- **Containment trigger**: from monitor view row, `Button[danger]
  "Contain"` → simple `AlertDialog` with explicit Cancel / Contain
  buttons (Contain = `Button[danger]`, focused by default; Esc cancels).
  No type-back.
- **Contained list**: separate `Table` in monitor view showing PID, process,
  reason, and `Button[secondary] "View Snapshot"` per row. Snapshot opens
  a `Sheet` with stdio + open-fds tree.
- **Release**: `Button[ghost] "Release"` next to each contained row →
  same simple `AlertDialog` pattern.
- **Where type-back IS used**: `[Revoke Token]` in W13, `[Uninstall
  Ripley]` in W14, `[Delete custom rule]` in the rule registry view.
  Mismatched friction in either direction is harmful — too much friction
  on Contain numbs users to the *real* destructive prompts.

### W8 — CI pipeline gate

- **Not a GUI surface** — this is `ripley scan --ci --format sarif`. The
  desktop app contributes one read-only view:
- **CI history view** (M27.6, optional): when `ripley scan --ci`
  uploads to the daemon via IPC, the dashboard adds a "CI Runs" section
  to the alerts sidebar showing last 50 CI invocations. Each row is a
  `Card` with branch, SHA, finding count, severity dot, link to SARIF.
- **No write actions** from the GUI on CI data — CI is the source of truth.

### W9 — Trust management

- **Surface**: settings view, `[guard]` section. Trust list rendered as
  `ScrollArea` of `Card[compact]` rows; each row: package pattern (mono) +
  `Button[icon]` (✕).
- **Add**: `Combobox` (Base UI) with **`allowsCustomValue` enabled** so
  free-text glob patterns (`@scope/*`, `*-cli`) are accepted alongside
  autocomplete suggestions drawn from the daemon's package index
  (matches anything in any lockfile in any project root). Without
  `allowsCustomValue`, the combobox would silently reject glob entries.
- **Remove**: clicking ✕ removes the entry immediately and shows an
  inline `Toast` "Removed @scope/pkg · **Undo**" for 5 seconds. The
  Undo link re-adds the entry. No `AlertDialog` confirm — the Undo path
  is faster than the confirm dialog and matches Gmail-style undoable
  destructive ops.
- **Promote source** (rule registry trust): same pattern in the M23 rule
  registry view — `Combobox` to add source, `Switch` to mark "trusted"
  (community rules from untrusted sources flag but cannot block).

### W10 — Environment security audit

- **Audit report view** uses the [UI.md § Audit Report View](UI.md#audit-report-view-phase-3)
  layout. Top: four `category-score-card`s using `Meter` for the bar.
- **Sections**: `Accordion` per category, expanded by default if the
  category contains any non-clean finding; collapsed if all-green.
- **Finding row**: severity dot + description + `command-block` for fix.
  Passing checks render with `severity-clean` and no command block.
- **Fix all**: `Button[primary] "Fix All with Claude ▾"` (split menu for
  harness choice).
- **Keyboard**: same `j` / `k` + `Enter` to expand row + `c` to copy command.

### W11 — PM hardening

- Same skeleton as W10 audit report. Different category cards (Pinning,
  Hardening, Provenance, Creds). Uses `harden.rs` data through the same
  IPC pattern.
- **Difference**: bottom action is `Button[primary] "Copy All Commands"` —
  hardening recommendations are commands the user copies into their shell,
  not actions Ripley executes.

### W12 — Standalone fix

- **No new view** — this is the same Alert Detail `Sheet` as W5, but
  reached via deep-link (`ripley fix <pkg>` → IPC → opens dashboard to
  alert detail). Confirms the design's reuse model: every fix path passes
  through the same component.

### W13 — Credential exposure assessment

- Rendered as a *section* inside the audit report (W10), not a separate
  view. The "Credentials at Risk" `Accordion` has heightened visual
  treatment: rows include both the exposure and the *exact rotation
  command sequence* in an ordered `command-block` group (each command on
  its own line, copy icon per command, "Copy all as checklist" toolbar
  button).

### W14 — Uninstall & upgrade

- **Upgrade**: Tauri updater fires `event::emit("update-available", ...)`.
  Dashboard shows a persistent `Card` at the top of the alerts view
  ("Update available: v0.2.0 ▸ [Install & restart]"). Never a toast —
  toasts auto-dismiss and we want the user to see this until they act.
- **Uninstall**: not a GUI flow. Documented in WORKFLOW.md as terminal
  commands. The dashboard's settings view exposes a `Button[danger]
  "Uninstall instructions…"` → `Sheet` with the per-OS command list
  (read-only `command-block`s with copy icons). The `[Uninstall Ripley]`
  button (when added to settings → Advanced) uses type-back-confirm
  (the user types `ripley` to enable the destructive button).

### W-Palette — Command palette (Cmd/Ctrl+K)

The palette is the keyboard surface. Spec follows the Raycast / Linear /
Arc / VS Code canon — not just a Combobox.

- **Component**: `Dialog` (centered, ~600×400) containing a
  `Combobox.Input` with explicit `Combobox.Group`s. `Combobox` is the
  Base UI primitive; the *palette pattern* is the Dialog wrapper + group
  composition.
- **Groups** (in this order, with separators between):
    1. **Recent** — last 5 actions invoked in this palette, persisted to
       `localStorage` under `ripley:palette:recent`. Shown only if
       non-empty.
    2. **Actions** — Scan Now, Deep Scan, Pause Monitoring, Add Project
       Root, Add Trust, Import Rule Source.
    3. **Navigate** — Alerts, Guard Log, Monitor, Settings, Audit Report.
    4. **Help** — Keyboard Shortcuts (Cmd+/), Open Docs, Report Issue,
       About.
- **Inline `<kbd>` hints**: each item with a keyboard shortcut shows the
  shortcut **right-aligned** in muted text. The kbd glyphs use the
  `<kbd>` shadcn token (mono, surface-elevated bg, 1px border).
- **Smart matching**: subsequence match (typing `sca` matches "Scan
  Now"), not just prefix. Score by match position + group priority
  (Recent > Actions > Navigate > Help). Use `match-sorter` (already
  locked in [STACK_DECISION.md](STACK_DECISION.md)).
- **Empty input**: shows all groups (Recent first if any).
- **Empty results**: shows "No commands match '<query>'" with a
  fallback `Button[ghost] "Search docs for '<query>'"` that opens the
  docs in the system browser. Never a dead-end "no results" state.
- **Activation**: `Enter` runs the focused item; `Cmd/Ctrl+Enter` runs
  it in a "background mode" if applicable (e.g., Scan Now keeps the
  palette open showing progress). `Esc` closes the palette.
- **Keyboard nav**: `↑` `↓` move between items including across groups;
  Base UI `Combobox` handles the roving tabindex.

### W-Tray — Tray surface (the spine)

The tray icon is visible 99% of the time. Dashboard, dialogs, palette are
all episodic. The tray *is* Ripley's persistent surface — it has to be
correct in every state, otherwise the rest of the design becomes
incoherent. Spec follows the canonical security-tray pattern (Little
Snitch, 1Password, Bartender, NordVPN).

**Icon states** (DESIGN.md / UI.md):

| State                | Glyph                     | When                                     |
|----------------------|---------------------------|------------------------------------------|
| Idle                 | Shield outline            | Monitoring, no unack'd findings          |
| Alert                | Shield with dot           | ≥ 1 unack'd alert, none Critical         |
| Critical             | Shield with `!`           | ≥ 1 unack'd Critical alert OR guard block |
| Paused               | Shield with pause         | `[monitor] enabled = false`              |
| Update-available     | Shield with `↑` overlay   | Tauri updater has a pending update       |
| Daemon-degraded      | Shield with `?`           | Daemon can't reach OSV feed for > 1h     |

Worst-state-wins: if both Critical and Update-available apply, the icon
shows Critical (severity beats meta). Update-available is the lowest of
the non-idle states.

**Click semantics on the icon**:

- **Single-click** = toggle cycle (Spotlight / Bartender / 1Password):
  - Hidden → show + focus dashboard
  - Shown but unfocused → focus
  - Shown + focused → hide (tray icon stays active)
- **Right-click** (or Ctrl-click on macOS) = open the tray menu

**Tray menu structure (canonical)**:

The menu opens to cached state instantly — never blocks on an IPC
roundtrip. A background `event::on("tray-data")` listener triggers
re-render if data changes while the menu is open (small flicker is
acceptable; blocking the menu-open is not). Top-of-menu contents vary by
state:

```
┌─────────────────────────────────────┐
│ {Status header — varies by state}   │  ← line 1
│ ─────────────────────────────────── │
│ {Top 5 items: alerts + guard blocks}│  ← only if any exist
│ ─────────────────────────────────── │
│ Show Dashboard       ⌘⇧R            │
│ Scan Now                            │
│ {Pause | Resume} Monitoring         │
│ ─────────────────────────────────── │
│ Settings…            ⌘,             │
│ Quit Ripley          ⌘Q             │
└─────────────────────────────────────┘
```

**Top-5 list contents**: the union of unacknowledged advisory alerts
*and* recent (last 24h) high+ guard blocks, severity-sorted then
recency-sorted. Each row: severity dot + `package@version` (mono) +
short label (`GHSA-xxxx` for an alert, `blocked` for a guard block).
Clicking an advisory row opens the dashboard to that alert's detail
sheet; clicking a guard-block row opens the Guard Log view scrolled to
that entry. Limit 5 because the menu must stay one-glance-readable.

**Status header per state**:

| State            | Header text                                         |
|------------------|------------------------------------------------------|
| Idle             | `● Monitoring 12 projects · last scan 2m ago`        |
| Alert            | `● 3 alerts — 2 high, 1 medium`                      |
| Critical         | `● 1 CRITICAL · 2 high — needs attention`            |
| Paused           | `○ Paused · click Resume to re-enable`               |
| Update-available | `● Update v0.2.0 available · in Settings → About`    |
| Daemon-degraded  | `⚠ Advisory feed unreachable · using cached data`    |
| First-run        | `○ Not monitoring · click Show Dashboard to set up`  |

**Action item variants**:

- `Scan Now`: **silent background scan** (does NOT auto-open the
  dashboard). The tray icon adopts a brief subdued "scanning" treatment
  (1px progress underline on the glyph; OS-native if available, omitted
  if not). On completion: fires a notification *only if* findings appear.
  If user wants to watch progress, they can `Show Dashboard` afterward.
  Precedent: macOS HIG — menu bar actions are quiet.
- `Pause Monitoring` ↔ `Resume Monitoring`: label flips with state.
  Toggles `[monitor] enabled` via `commands.set_config_field`. Pausing
  does NOT clear existing alerts — paused means "no new polls", not
  "forget what we know."
- `Scan Now` is **disabled** in first-run state (no project roots) with
  a tooltip "Add a project root in the dashboard first."
- `Quit Ripley`: if a guard dialog is currently showing OR a scan is in
  progress, show an `AlertDialog` confirming "Quit while a script
  decision is pending? The terminal will fall back to a text prompt." If
  the user confirms, the daemon shuts down and `ripley-script-shell`
  falls back to terminal prompts (per UI.md "IPC unavailable").

**Hotkey interactions with tray**:

- `Cmd/Ctrl+Shift+R` (global) mirrors single-click on the tray icon —
  same toggle cycle (hidden → show → focus → hide). **Exception**: if a
  guard dialog is currently showing, the hotkey is a no-op (the dialog
  is the higher-priority surface; stealing focus could let users
  inadvertently dismiss it).
- `Cmd/Ctrl+,` from anywhere opens the dashboard to the settings view
  (also a single-step from the tray menu).
- `Cmd/Ctrl+Q` is the OS-level app quit; goes through the same `Quit
  Ripley` safety confirm.

**Notification → tray → dashboard chain consistency**:

A finding fires three coordinated surfaces:

1. **OS notification** (transient, may be missed)
2. **Tray icon + menu** (persistent, surfaces in top-5 list)
3. **Dashboard alerts list** (canonical record)

All three are independent endpoints to the same model. Dismissing the
notification does not dismiss the alert. The user can reach the alert
detail from any of the three (notification body click, tray menu row
click, dashboard row click) and the destination is identical — the same
Sheet with the same focus state. This is the consistency rule: tray and
dashboard render the same model, never disagree.

**Tray menu refresh cadence**:

- Menu open = render cached state instantly. No IPC roundtrip blocks the
  open.
- On open, fire `commands.get_tray_state()` in the background; on
  response, re-render the menu items if data changed.
- Cache invalidation: any of the events that update the dashboard
  (`scan-complete`, `alert-new`, `guard-block`, `monitor-tick`) also
  push a tray-state update.
- The icon glyph updates synchronously with state events (no debounce);
  the menu re-renders only if open.

**Platform AX gaps (accepted)**:

- macOS: NSStatusItem menus have full AX support via VoiceOver.
- Windows: Shell_NotifyIcon menus are AX-supported via Narrator.
- Linux: StatusNotifierItem AX support varies by DE (GNOME / KDE / xfce
  each behave differently); Tauri exposes what the desktop allows. Not
  fixable in our layer. Documented as a known constraint.

**First-run tray menu**:

Before any project root is configured, the menu collapses to:

```
○ Not monitoring · click Show Dashboard to set up
───────────────────────────────────────
Show Dashboard       ⌘⇧R
───────────────────────────────────────
Settings…            ⌘,
Quit Ripley          ⌘Q
```

No "Scan Now", no "Pause Monitoring" (nothing to pause), no top-5 list
(none exist). The header is the only call-to-action.


---


## Keyboard model (canonical)

`Cmd` on macOS, `Ctrl` on Linux/Windows. Registered globally via Tauri
`globalShortcut` for window-open; in-window via `keydown` listener registered
in `App.tsx`.

### Global (anywhere on the OS)

| Shortcut              | Action                                                                  |
|-----------------------|--------------------------------------------------------------------------|
| `Cmd/Ctrl+Shift+R`    | Toggle dashboard window — hidden→show→focus→hide cycle. No-op if a guard dialog is showing (the dialog is higher-priority). Mirror of single-click on tray icon. See W-Tray. |

### Window (active when dashboard focused)

| Shortcut         | Action                                                       |
|------------------|---------------------------------------------------------------|
| `Cmd/Ctrl+K`     | Open command palette (Combobox)                              |
| `Cmd/Ctrl+,`     | Open settings view                                           |
| `Cmd/Ctrl+R`     | Refresh data (re-invokes the current view's queries)         |
| `Cmd/Ctrl+W`     | Hide window (tray icon stays active)                         |
| `Cmd/Ctrl+Q`     | Quit app (with confirm `AlertDialog`)                        |
| `Cmd/Ctrl+/`     | Show keyboard cheatsheet (`Dialog`)                          |
| `a`              | Jump to Alerts view                                          |
| `g`              | Jump to Guard log view                                       |
| `m`              | Jump to Monitor view                                         |
| `j` / `k`        | Move focus down / up in any list                             |
| `Enter`          | Open detail / activate primary action on focused row         |
| `Esc`            | Close sheet / popover / dialog (or hide window if top-level) |
| `f`              | Fix focused finding                                          |
| `d`              | Dismiss focused finding                                      |
| `x`              | Toggle row selection                                         |
| `Shift+x`        | Range-select to focused row                                  |
| `c`              | Copy focused command-block                                   |
| `?`              | Same as `Cmd+/`                                              |
| `1`…`9`          | Jump to nth section (scroll-to in W6, group in palette)      |

A "`?` for shortcuts" hint sits in the dashboard status bar (bottom-left,
muted, `body-sm`) so the cheatsheet is discoverable without docs.

### Guard dialog (when shown)

| Shortcut         | Action                                                       |
|------------------|---------------------------------------------------------------|
| `A`              | Allow Once                                                   |
| `B`              | Block                                                        |
| `T`              | Always Trust                                                 |
| `I`              | Inspect (opens dashboard sheet)                              |
| `Esc`            | Block (fail-safe; same as timeout)                           |

The cheatsheet (`Cmd+/`) is the source of truth — generated at build time from
a `KEYMAP` constant exported from `apps/desktop/src/lib/keymap.ts` so docs and
runtime never drift.


---


## Density and layout model

### Three densities, one chosen

Ripley is *compact* density only. No "comfortable" or "spacious" toggles. The
target user is a developer comfortable with terminal density; lower densities
would force scrolling that hides findings.

| Surface       | Row height | Vertical padding | Reasoning                          |
|---------------|------------|------------------|-------------------------------------|
| Alert row     | 56px       | `md` (12px)      | Two lines of text + dot + action    |
| Guard log row | 32px       | `sm` (8px)       | Single-line table row               |
| Sidebar item  | 36px       | `sm` (8px)       | Sparse nav, 5 items                 |
| Settings field| 48px       | `md` (12px)      | Label above + control               |
| Detail header | 64px       | `lg` (16px)      | One heading + meta line             |

### Window sizes

Defaults locked in [DESIGN.md § Spacing → Layout Constants](DESIGN.md#spacing).
Recap: default `900 × 640`, minimum `720 × 480`, dialog max `600px`.

### Responsive collapse

Below 800px the right-side `Sheet` (alert detail) replaces the alerts list
(stack mode), with a `Button[ghost] "← Alerts"` to return.

**Sidebar collapse** is **manual** via a chevron at the bottom of the
sidebar, persisted to `localStorage` under `ripley:ui:sidebar-collapsed`.
Auto-collapse only at a hard floor of <600px (smaller than the documented
minimum window, so effectively never). Auto-collapse-at-viewport
surprises users on docked or split-screen windows — precedent: VS Code,
Slack, Tower all chose manual.

### Notifications (OS-native)

- **Batching**: when ≥ 2 findings fire within a 5-second window, they
  collapse into one notification ("5 new findings — 3 high, 2 medium")
  instead of N separate banners. Precedent: Crashlytics, Sentry, GitHub
  mobile.
- **Dismissal**: dismissing an OS notification does **not** dismiss the
  alert in the dashboard. Lifecycles are independent — the notification
  is a transient signal; the alert is the persistent record.
- **Click target**: clicking the notification body opens the dashboard
  to the relevant alert detail (single finding) or the alerts list with
  the batch pre-selected (batch).


---


## DX patterns (locked for M27)

Patterns the React/TS migration MUST follow. Deviations require a PR-time
note.

### 1. Component composition

- **Primitives**: Base UI exports live in `apps/desktop/src/components/ui/`
  (copied in via `pnpm dlx shadcn@latest add <slug> --base base-ui`).
- **Compositions**: Domain components live in `apps/desktop/src/components/`
  (e.g. `AlertCard.tsx`, `GuardDialog.tsx`, `SeverityBadge.tsx`).
- **View shells**: One file per view in `apps/desktop/src/views/`
  (e.g. `AlertsView.tsx`, `GuardLogView.tsx`, `SettingsView.tsx`).
- **No deep relative imports**: `@/components/...`, `@/views/...`,
  `@/lib/...` aliases via `tsconfig.json` paths + `vite.config.ts`.

### 2. State

- **Server state** (anything from `tauri-specta` `commands.*`) lives in
  TanStack Query. Query key convention: `[viewName, resourceName, ...args]`,
  e.g. `["alerts", "list"]`, `["alerts", "detail", advisoryId]`.
- **UI state** (sidebar selection, sheet open/closed, palette open) lives
  in Zustand slices in `apps/desktop/src/store/`. One slice per concern;
  combine via `create<T & U>()(...)` only when ≥ 3 slices touch the same
  surface.
- **Never** mirror server state into Zustand. If the server is the source
  of truth, query it; if the UI is the source of truth, store it.
- **Events** from Rust (`event::emit`) feed `queryClient.invalidateQueries`
  in a single `useTauriEvents()` hook registered in `App.tsx`.

### 3. IPC

- All IPC goes through `commands.*` from generated `bindings.ts`. Direct
  `invoke()` calls fail PR review.
- Every command has a typed Rust DTO (`#[derive(specta::Type)]`) — no
  `serde_json::Value`, no `unknown` on the TS side.
- Long-running commands stream via events, not by holding the invoke
  promise open.

### 4. Theming

- Tailwind v4 `@theme` block in `apps/desktop/src/styles/theme.css` is
  the *only* place tokens are defined.
- Token names match DESIGN.md exactly: `--color-severity-critical`,
  `--color-surface`, `--font-size-body`, `--spacing-md`, etc.
- Components reference tokens via Tailwind classes (`bg-surface`,
  `text-severity-critical`); raw hex in JSX/CSS fails PR review.
- No `tailwind.config.ts`. No dark-mode toggle. No CSS-in-JS.

### 5. Forms

- Settings view binds to `config.toml` via `commands.get_config()` +
  `commands.set_config_field(field, value)`. Each field is its own
  command — atomic writes, no full-config round-trips.
- Field components use Base UI `Field` + `Field.Label` + `Field.Description`
  + `Field.Error`. Field validation runs on blur, persists on debounced
  change (300ms).
- **Per-field Undo**: after each successful save, an inline `Toast`
  appears anchored near the field: "Saved · **Undo**" (5s). Clicking
  Undo restores the prior value via the same `set_config_field` command.
  Without Undo, instant-save settings feel scary — the user lost the
  safety of an explicit Save button without gaining anything. Precedent:
  Linear settings, Arc preferences.
- "Reset to default" is per-section, surfaced via a `DropdownMenu` in the
  section header — never a global "discard changes" button (changes
  persist immediately).

### 6. Testing

- **Unit**: Vitest + `@testing-library/react` colocated as
  `Component.test.tsx`. Every component with branching logic gets a test.
- **Snapshot**: Vitest snapshots for view shells (`AlertsView.test.tsx`).
  Snapshots are reviewed, not auto-updated.
- **IPC mocks**: a single `apps/desktop/src/test/mockBindings.ts` exports
  a mock `commands` object. Tests opt-in via `vi.mock("@/lib/bindings", ...)`.
- **E2E**: WebdriverIO + `tauri-driver` (Linux + Windows). Per WORKFLOW.md,
  the macOS gap is covered by Vitest + Peekaboo screenshot review.
- **Performance**: M25's guard-dialog `<500ms cold / <50ms warm` is a CI
  gate — measured via Tauri's `performance.now()` in a headless harness.

### 7. Accessibility

- Every `Dialog`, `Sheet`, `Popover` has `aria-labelledby` pointing to its
  visible title. Base UI does this by default — don't undo it.
- Every icon-only `Button` has `aria-label`.
- Severity is never *only* color — every severity uses dot + text label
  (`CRIT`, `HIGH`, etc.) so screen readers and color-blind users get
  the same signal.
- Tab order matches reading order. `j`/`k` keyboard nav also moves DOM
  focus (using `roving-tabindex` from Base UI helpers).
- `prefers-reduced-motion` disables the alert-detail slide-in and the
  200ms fade on new rows. (We already minimize motion globally.)


---


## Open questions (M24 → M27 deferred decisions)

These are explicitly deferred. The default below is what we'll ship if no
contrary input arrives by the M27 view migration starting.

1. **Command palette grouping** — locked as `Recent / Actions / Navigate /
   Help` (see W-Palette). Reconsider after first dogfood week.
2. **Sheet width on wide screens** — default: 480px fixed; consider 50%
   split if any view needs side-by-side comparison.
3. **Toast position** — default: bottom-right with 5s timeout (matches the
   Undo dwell time). macOS HIG would prefer top-right; tradeoff is conflict
   with the OS notification surface. Defer until first user feedback.
4. **Update card dismiss** — default: persistent until installed. If users
   complain it's nagging, add "Remind me tomorrow" with a 24h snooze.
5. **Filter chips on alerts list** — deferred to Phase 7. Typical alert
   list is <50 items where sort-by-severity-then-time obviates filtering.
   Revisit if user feedback shows lists exceeding one viewport.
6. **In-palette mode prefixes** (`>action`, `#nav`) — Linear's pattern. Not
   adopted in v1 because our command set is small enough that smart
   matching + groups suffice. Revisit if total commands exceed ~30.


---


## Canonical references

What we stole from, by surface. Each pick traces back to one or more
shipping precedents — explicit so future contributors can verify the
pattern instead of re-deriving it.

| Surface                       | Precedent                                            |
|-------------------------------|------------------------------------------------------|
| Guard interception dialog     | Little Snitch (network ask), 1Password CLI prompts   |
| Tray menu w/ at-a-glance      | Little Snitch, 1Password, Bartender, NordVPN        |
| Command palette               | Raycast, Linear command bar, VS Code, Arc, Cursor   |
| Alerts list keyboard nav      | Linear issues, GitHub Issues, Sentry feed           |
| Bulk-action sticky bar        | Gmail, Linear, GitHub Issues, Notion                |
| Per-field Undo on instant-save| Linear settings, Arc preferences                    |
| Deep-scan linear scroll       | Snyk report, CrowdStrike detection, Lighthouse      |
| Audit traffic-light cards     | Lighthouse audit, Snyk advisor scorecards           |
| Type-back-to-confirm          | GitHub repo delete, Stripe destructive ops          |
| Three-step onboarding cards   | Linear onboarding, GitHub Desktop first-run, Tower  |
| Drag-and-drop folder onto win | VS Code, GitHub Desktop, iTerm                      |
| Notification batching         | Crashlytics, Sentry, GitHub mobile                  |
| Manual sidebar collapse       | VS Code, Slack, Tower                               |
| Toast-only-for-clipboard      | Raycast HUD, macOS Human Interface Guidelines       |
| Tray single-click toggle      | Bartender, 1Password, Spotlight (Cmd+Space)         |
| Silent menu-bar actions       | macOS HIG menu bar guidance                         |
| Tray at-a-glance items list   | Little Snitch (recent connections), Bartender       |
| Quit-safety confirm on pending| 1Password (locking while op pending)                |


---


## Cross-reference index

- [PLAN.md](PLAN.md) M27.1 — installs primitives in the order this doc
  uses them (Button, Dialog, Sheet, Combobox first).
- [UI.md](UI.md) — every wireframe in this doc maps to a section in UI.md;
  any new wireframes added there should reference back here for
  component choice.
- [DESIGN.md](DESIGN.md) — every token referenced here is defined there;
  any new token introduced here MUST be added to DESIGN.md's YAML header.
- [STACK_DECISION.md](STACK_DECISION.md) — locks Base UI v1.x, shadcn
  CLI v4 with `--base base-ui`, style `base-vega`, TanStack Query/Table,
  Zustand — this doc operates strictly within those locks.
- [WORKFLOW.md](WORKFLOW.md) — every "W#" section here corresponds to
  the like-numbered workflow stream.
- [SETTINGS.md](SETTINGS.md) — the form fields enumerated here bind 1:1
  to the keys defined there.
