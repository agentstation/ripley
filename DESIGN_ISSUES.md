# Design Issues

> **Phase 6 status.** This document was captured against the iced-based
> `crates/ripley-app` (Phase 1-5) on 2026-05-22. It is the **priority-ordered
> input list for Phase 6 M27 (view migration)** — the issues drove the stack
> re-evaluation that produced [STACK_DECISION.md](STACK_DECISION.md), and the
> _priority order_ survives the iced→Tauri rewrite even though the underlying
> tech changes. Resolution path: addressed in the Tauri + React + shadcn/Base UI
> \+ Tailwind v4 rewrite, not by patching iced.
>
> Items (1)-(3) (no tray, no native menu bar, no AX tree) are dissolved by the
> stack switch itself — they were structural ceilings in iced that Tauri does
> not share. Items (4)+ (design drift, theming, severity badges, etc.) become
> concrete M27 tasks: implement once in shadcn/Tailwind tokens, ship across all
> three OSes simultaneously.

## Phase 6 disposition (M27.7)

Every numbered finding below has been routed to one of three outcomes:

| Outcome              | Meaning                                                                                                                                                                                        |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Resolved (Tauri)** | The Tauri/React migration shipped the fix as a side effect of building the corresponding component. The iced call-sites identified in the finding are retired with `crates/ripley-app` in M28. |
| **Deferred (notes)** | Specific judgment moved to `apps/desktop/DESIGN_NOTES.md` because the Tauri implementation diverges from the iced-era recommendation.                                                          |
| **Obsolete (stack)** | Finding was specific to iced widgets / theme.rs and disappears entirely with the stack switch — nothing to port.                                                                               |

### Cross-cutting

| ID                                         | Outcome          | Where it lives now                                                                                                                                                                |
| ------------------------------------------ | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CC-1 sidebar purple buttons                | Obsolete (stack) | iced `button::Style` ceiling; Tauri uses `components/ui/button.tsx` over Tailwind tokens. Sidebar nav lands with the shell in M27.8 follow-up; no equivalent regression possible. |
| CC-2 empty-state CTA purple                | Resolved (Tauri) | `Button` variants in `apps/desktop/src/components/ui/button.tsx` consume `bg-primary`/`bg-accent-hover` from `styles/theme.css`.                                                  |
| CC-3 sidebar header                        | Deferred (notes) | M27 routes ship without a sidebar shell; tray-driven navigation is the M28 model. Logo + version footer captured for the eventual shell pass.                                     |
| CC-4 sidebar badges + footer               | Deferred (notes) | Same as CC-3; not blocking M27 close-out.                                                                                                                                         |
| CC-5 top bar / view title                  | Resolved (Tauri) | Per-view `<section aria-label>` and route titles live inside each route component; no shared chrome is owed.                                                                      |
| CC-6 severity badges                       | Resolved (Tauri) | `components/ripley/SeverityBadge.tsx` is the single primitive used by Alerts, Guard log, Monitor, Deep scan.                                                                      |
| CC-7 severity background tokens            | Resolved (Tauri) | `--color-severity-*-bg` in `apps/desktop/src/styles/theme.css`.                                                                                                                   |
| CC-8 inconsistent empty-state              | Resolved (Tauri) | `components/ripley/EmptyState.tsx` is the only empty-state pattern in the Tauri routes.                                                                                           |
| CC-9 surface-hover / surface-active        | Resolved (Tauri) | `bg-surface-hover` Tailwind utility powered by `--color-surface-hover`; no `#[allow(dead_code)]` equivalent survives in CSS.                                                      |
| CC-10 monospace font                       | Resolved (Tauri) | Tailwind `font-mono` applied at every pkg@version / path / fix-command site in M27.2–M27.6 routes.                                                                                |
| CC-11 numeric counts size                  | Resolved (Tauri) | `KeyValueGrid` formats counts with DESIGN.md typography tokens.                                                                                                                   |
| CC-12 settings form                        | Resolved (Tauri) | M27.6 — full editable form over `read_settings`/`write_settings` with atomic config writes.                                                                                       |
| CC-13 sidebar background `bg` vs `SURFACE` | Obsolete (stack) | iced `theme.rs` palette mismatch dissolves with the rewrite.                                                                                                                      |

### Per-view

| ID                        | Outcome          | Notes                                                                                                                                                                                                                                          |
| ------------------------- | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V1.1–V1.5 alerts row      | Resolved (Tauri) | M27.2 — `routes/Alerts.tsx` ships severity badge + monospace pkg@version + advisory id + project_path.                                                                                                                                         |
| V1.6 alerts empty CTA     | Deferred (notes) | Empty state is shared (CC-8); per-view CTA copy/CTA wiring tracked in DESIGN_NOTES.md M27.7 entry.                                                                                                                                             |
| V2.1–V2.3 guard log table | Resolved (Tauri) | M27.3 — `routes/GuardLog.tsx` is a columnar `DataTable` with severity badge + monospace script.                                                                                                                                                |
| V2.4 guard log empty CTA  | Deferred (notes) | Same shared-empty-state caveat as V1.6.                                                                                                                                                                                                        |
| V3.1–V3.8 deep scan       | Resolved (Tauri) | M27.4 — `routes/DeepScan.tsx` uses `KeyValueGrid` + `SeverityBadge`; dead-man-switch warning surfaces via a labeled row. Section chevrons (V3.3) intentionally not implemented — the Tauri DOM does not need them; tracked in DESIGN_NOTES.md. |
| V4.1–V4.4 monitor         | Resolved (Tauri) | M27.4 — `routes/Monitor.tsx` splits severity badge + monospace process + secondary timestamp via `TimestampCell`.                                                                                                                              |
| V5.1–V5.5 audit           | Resolved (Tauri) | M27.4 — `routes/Audit.tsx` standardised empty state, `TrafficLightDot` for category status, monospace fix-command blocks. Category score cards (V5.3) consciously omitted; tracked in DESIGN_NOTES.md.                                         |
| V6.1–V6.5 posture         | Resolved (Tauri) | M27.4 — `routes/Posture.tsx` mirrors Audit pattern; detected PMs render as monospace pill row.                                                                                                                                                 |
| V7.1–V7.4 settings        | Resolved (Tauri) | M27.6 — `routes/Settings.tsx` ships section-grouped form (General / Monitor / Guard / Posture) with radio group + checkboxes + numeric inputs.                                                                                                 |

### Missing-component checklist

| #   | Component                 | Outcome                                                                                                                                             |
| --- | ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Severity badge            | Resolved — `SeverityBadge`.                                                                                                                         |
| 2   | Severity dot              | Resolved — `TrafficLightDot`.                                                                                                                       |
| 3   | Card                      | Resolved — Tailwind `rounded-lg border border-border-subtle bg-surface p-4` is the convention; not abstracted further on purpose (DESIGN_NOTES.md). |
| 4   | Summary card              | Resolved — `KeyValueGrid` covers the numeric-summary case in Deep scan; severity-tinted variants are deferred.                                      |
| 5   | Category score card       | Deferred (notes) — Audit/Posture render category headers with `TrafficLightDot` instead.                                                            |
| 6   | Traffic-light bar         | Deferred (notes) — `TrafficLightDot` is the only traffic-light primitive shipped; bar variant tracked.                                              |
| 7   | Code / command block      | Resolved — `<code>` with `bg-bg px-2 py-0.5 font-mono` is the convention across routes.                                                             |
| 8   | Input / dropdown / toggle | Resolved (Tauri) — M27.6 settings form uses native form elements styled via DESIGN tokens; Base UI dropdown lands when a route needs it.            |
| 9   | Top bar                   | Deferred (notes) — see CC-5.                                                                                                                        |
| 10  | Sidebar footer            | Deferred (notes) — see CC-3/CC-4.                                                                                                                   |
| 11  | Alert row                 | Resolved — `routes/Alerts.tsx`.                                                                                                                     |
| 12  | Notification overlay      | Deferred (notes) — tray + OS notifications are the M28 path; in-app overlay is not on the Phase 6 critical path.                                    |

The deferred items above are individually captured in
`apps/desktop/DESIGN_NOTES.md` (M27.7 entry) with the rationale for why the
Tauri shell intentionally diverges or postpones each one. The iced
findings below are retained verbatim as the source-of-truth record of
what drove the stack switch and the M27 component set; they are not
re-validated against the Tauri code.

Audit of the Ripley dashboard (`crates/ripley-app`) against [DESIGN.md](DESIGN.md)
and [UI.md](UI.md), captured 2026-05-22 by driving the running app with Peekaboo
and inspecting source.

Captured screenshots: `/tmp/ripley-view-1-alerts.png` … `/tmp/ripley-view-7-settings.png`.

Verification harness: `peekaboo hotkey --keys cmd,<N> --app ripley-app`
switches views 1–7; `peekaboo image --window-id <wid>` snapshots. The Cmd+1..7
shortcuts were added in `crates/ripley-app/src/app.rs` for design iteration. In
the Tauri rewrite the Peekaboo loop continues to work against the DOM
accessibility tree exposed by WKWebView — same workflow, richer signal.

## Executive summary

The dashboard renders, but visually it is roughly 15 % of what DESIGN.md
specifies. The colors in `theme.rs` match the spec, but most of them are
dead code; `button(…)` widgets inherit iced's default purple palette
(visible on every sidebar nav item and every empty-state CTA), `Settings`
is a `println!`-style dump of `config.toml`, severity badges/dots/pills
don't exist, summary cards are present in Deep Scan but use the wrong
backgrounds, and the empty-state pattern is inconsistent across views.

There is also a structural gap: the implemented widgets don't reuse a
shared theming layer, so every view re-declares colors/sizes inline. Any
fix that touches the look-and-feel needs to touch ~7 files unless we add a
`components` module first.

Severity legend used below:

- **P0** — Breaks the design identity (e.g. wrong brand color across the app).
- **P1** — Visible inconsistency or missing core component from DESIGN.md.
- **P2** — Polish, empty-state coherence, hierarchy.
- **P3** — Cleanup / nice-to-have.

## Root cause

`theme.rs` is a flat list of color constants. It does **not** export:

- Spacing constants (DESIGN.md spec `xs=4`…`3xl=48`)
- Typography size constants (DESIGN.md `display=28`, `heading=15`, `body=13`, etc.)
- Radius constants (`sm=4`, `md=6`, `lg=8`, `full=9999`)
- Severity background variants (`severity-*-bg`, 20 % alpha)
- Accent variants (`accent-muted`, `accent-active`)
- Surface variants used in interactive states (`surface-hover`, `surface-active`)
  are present but marked `#[allow(dead_code)]`
- Style factories — there is no `nav_button_style()`, `primary_button_style()`,
  `card_style()`, `severity_badge()`, etc.

Result: every call site uses raw `button(text(label))` which inherits iced's
default `Theme::Dark` palette (purple). The only widgets that pick up the
real palette are those built from `container(...)` with an explicit
`.style(...)` closure.

This is fixable with one PR that introduces `theme/components.rs`
(or `theme/buttons.rs` + `theme/cards.rs` + `theme/badges.rs`) and migrates
the existing call sites.

## Cross-cutting issues

### CC-1 (P0) — Sidebar buttons render in iced default purple

**Where:** `crates/ripley-app/src/app.rs:289-300` (`nav_button`).

**Problem:** `button(text(label).color(...))` only styles the label text.
The button background, border, hover, and pressed states fall back to
`iced::widget::button::Style::default()` for `Theme::Dark`, which is the
violet `#7C8AC7`. All seven sidebar items render as filled purple
rectangles in every screenshot.

**DESIGN.md spec (Sidebar, components.sidebar-item):**

- Default: transparent background, `text-secondary` text, `body` font
  (13 px), padding `sm` vertical + `md` horizontal, radius `md`.
- Active: `accent-muted` background, `accent` text, 2 px left border in `accent`.
- Hover: `surface-hover` background.

**Fix:** Apply a `button::Style` closure that returns:

```rust
button::Style {
    background: if active { Some(ACCENT_MUTED.into()) } else { None },
    text_color: if active { ACCENT } else { TEXT_SECONDARY },
    border: Border { /* 2px left in accent when active */ },
    ..
}
```

Requires `ACCENT_MUTED` to be added to `theme.rs` first (see RC).

### CC-2 (P0) — Empty-state CTAs render in iced default purple

**Where:**

- `crates/ripley-app/src/views/deep_scan.rs:74` (“Run Deep Scan”)
- `crates/ripley-app/src/views/monitor.rs:17` (“Enable Monitor”)
- `crates/ripley-app/src/views/deep_scan.rs:330,335` (“Export Report”, “Copy Rotation Checklist”)

**Problem:** Same as CC-1 — `button(text(...).color(Colors::ACCENT))`
only colors the label; the bg is iced violet.

**DESIGN.md spec (button-primary):** background `accent` (`#58a6ff`),
text `text-on-accent` (white), radius `md`, padding `sm`×`lg`, weight 600.

**Fix:** Introduce `theme::primary_button_style()` and
`theme::secondary_button_style()` and replace all `button(text(...))`
constructions.

### CC-3 (P1) — Sidebar header has no logo / no rule / no spacing

**Where:** `crates/ripley-app/src/app.rs:237`.

**Problem:** `text("Ripley").size(15)` followed by `Rule::horizontal(1)`
sits at the top of the sidebar with no padding distinct from the items,
and no logo glyph beside it. UI.md’s wireframe shows a `LOGO` block.

**Fix:** wrap header in a `row![logo, text("Ripley")…]` with deliberate
top/bottom padding, drop the iced `Rule` and use a `container` with
bottom-border-only styling so the separator color matches `BORDER`.

### CC-4 (P1) — Sidebar missing badge counts and bottom status

**Where:** `crates/ripley-app/src/app.rs:235-265`.

**DESIGN.md / UI.md spec:**

- Each nav item can show a right-aligned `badge` count (e.g. `Alerts (8)`
  in `severity-critical`).
- Bottom of sidebar: version (`v0.1.0`), connection dot (green when
  daemon connected, muted otherwise).

**Currently rendered:** plain text labels, no badges, no bottom block.

**Fix:** add a `nav_item(label, badge: Option<(usize, Color)>, active)`
factory and a `sidebar_footer(version, daemon_state)` element.

### CC-5 (P1) — No top bar / no current-view title outside the body

**Where:** `crates/ripley-app/src/app.rs:200-224` (`view`).

**Problem:** Main pane has no header row. UI.md wireframe shows:

```
Ripley           ● Monitoring · 12 projects
                 Last poll: 2m ago
```

Each view title is currently rendered _inside_ the body of that view
(audit.rs:11, posture.rs:12, settings.rs:10, deep_scan.rs:21), which
gives inconsistent vertical offsets and no place for the status strip.

**Fix:** Hoist the view title into the shell. `app::view` should render
`column![top_bar(current_view, status), content]` and individual view
files drop their leading `text("…").size(18)`.

### CC-6 (P1) — Severity badges don't exist as a component

**Where:** rendered ad-hoc in `audit.rs:43`, `posture.rs:64`,
`deep_scan.rs:199,261`, `monitor.rs:60-66`.

**Problem:** Each view inlines its own `format!("[{}]", ...)` or
`format!("{}", severity)` rendered as plain `text(...)`. DESIGN.md
specifies pill-shaped badges with rounded-full background fill at
20 % opacity, uppercase, +0.5 px tracking, weight 600.

**Fix:** Add `severity_badge(level: Severity) -> Element` that returns a
`container(text("CRIT").size(11)…)` with rounded `Border::full` and the
20 % bg color. Use it everywhere severity is shown.

### CC-7 (P1) — Severity background tokens are missing entirely

**Where:** `crates/ripley-app/src/theme.rs`.

**Problem:** DESIGN.md defines five `severity-*-bg` tokens (20 % alpha
variants) used for badge fills, summary card backgrounds, and alert
row highlights. None exist in `theme.rs`.

**Fix:** add `SEVERITY_CRITICAL_BG`, `SEVERITY_HIGH_BG`, … as
`Color::from_rgba(r, g, b, 0.125)` matching `#…20`.

### CC-8 (P2) — Inconsistent empty-state pattern

| View      | Empty state                                                                        |
| --------- | ---------------------------------------------------------------------------------- |
| Alerts    | centered "No findings / Your dependencies look clean." (no CTA)                    |
| Guard log | centered "No guard log entries yet." (no CTA)                                      |
| Deep Scan | centered title + body + **CTA button** (Run Deep Scan)                             |
| Monitor   | centered title + body + **CTA button** (Enable Monitor)                            |
| Audit     | top-left aligned title + "Run `ripley audit` to generate…" (no centering, no CTA)  |
| Posture   | top-left aligned title + "Run `ripley harden` to generate…" (no centering, no CTA) |
| Settings  | no empty state — it always shows                                                   |

**DESIGN.md (Do's):** "Every view has a next action. No dead-end screens."

**Fix:** Standardize via `views::empty_state(icon, title, body, cta)`
helper. Alerts/Guard/Audit/Posture should all get a primary CTA
(Scan Now / View Recent Activity / Run Audit / Run Harden).

### CC-9 (P2) — No use of `surface-hover` / `surface-active`

**Where:** `theme.rs:16-26` — both constants are `#[allow(dead_code)]`.

**Problem:** Hover and pressed states are invisible across the entire
app. Sidebar buttons, list rows, and CTAs all lack the subtle layer
shift DESIGN.md specifies as the only depth mechanism.

**Fix:** When migrating to component styles (CC-1, CC-2), wire up hover
states via `button::Status::Hovered` / `Pressed` branches.

### CC-10 (P2) — Monospace font isn't used anywhere

**DESIGN.md rule:** package names, versions, file paths, script content
must be `code` (system monospace).

**Where this is violated:**

- `views/alerts.rs:30-31` — package@version rendered with default sans
- `views/guard_log.rs:25` — package + action rendered as sans
- `views/deep_scan.rs:208,268` — `finding.path.display()` rendered as sans
- `views/audit.rs:55`, `posture.rs:84` — fix commands rendered as sans

**Fix:** Define `MONO: Font = Font::MONOSPACE;` in `theme.rs` (or use
`iced::Font::with_name("SF Mono")` with a fallback) and apply via
`.font(theme::MONO)` on the relevant `text(...)` builders.

### CC-11 (P2) — Numeric counts don't use the `display` (28 px) size

**Where:** `views/deep_scan.rs:106` — `text(count.to_string()).size(18)`.

**DESIGN.md (Summary Cards, typography.display):** numbers should be
28 px bold.

**Fix:** size 28, weight bold (iced `Font { weight: Bold, .. }`).

### CC-12 (P3) — `Settings` view has no form controls at all

**Where:** `crates/ripley-app/src/views/settings.rs:7-71`.

**Problem:** The view dumps the config as 9 `text(format!(...))` lines.
UI.md specifies dropdowns, toggles, multi-value lists with `[✕]` rows,
"+ Add root" buttons, and radio groups for guard mode. None of this is
implemented.

**Fix:** This is a substantial body of work and should likely be its
own milestone. For now: at minimum group with section headers and add
a `[● Live · edit config.toml]` reference so users know it's read-only.

### CC-13 (P3) — Sidebar uses `SURFACE` background, DESIGN.md says `bg`

**Where:** `crates/ripley-app/src/app.rs:276-284`.

**Problem:** Sidebar background is `Colors::SURFACE` (`#161b22`). DESIGN.md
(components.sidebar) specifies `bg` (`#0d1117`) with a `border-right`.
The current treatment makes the sidebar feel _lifted_ rather than
recessed.

**Fix:** Swap to `BG` and rely on the right border for separation.

## Per-view findings

### V1 — Alerts (default view) `views/alerts.rs`

| ID   | P   | Finding                                                                                                                                                                 |
| ---- | --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V1.1 | P0  | List rows lack severity dot, severity badge, and `[Fix]` action — the row is a single `text(format!("[{}] {}@{}", sev, pkg, ver))`. Compare UI.md wireframe (line 138). |
| V1.2 | P1  | Package@version is rendered in default sans, must be monospace (CC-10).                                                                                                 |
| V1.3 | P1  | No advisory ID line (UI.md spec line 152).                                                                                                                              |
| V1.4 | P1  | No project path / timestamp / action button on each row.                                                                                                                |
| V1.5 | P2  | Row containers have `padding(12)` but no border-bottom and no hover state (CC-9).                                                                                       |
| V1.6 | P2  | Empty state has no CTA (CC-8).                                                                                                                                          |

### V2 — Guard log `views/guard_log.rs`

| ID   | P   | Finding                                                                                                                            |
| ---- | --- | ---------------------------------------------------------------------------------------------------------------------------------- |
| V2.1 | P1  | Renders as freeform text instead of the columnar table in UI.md lines 192-200. No TIME / PACKAGE / SCRIPT / RISK / ACTION columns. |
| V2.2 | P1  | Risk level is rendered as `format!("Risk: {} · Script: {}", …)` — should be a severity badge + monospace script name.              |
| V2.3 | P2  | No expandable row interaction to show matched rules / script excerpt (DESIGN.md Tables, "Expandable rows").                        |
| V2.4 | P2  | Empty state has no CTA (CC-8).                                                                                                     |

### V3 — Deep Scan `views/deep_scan.rs`

| ID   | P   | Finding                                                                                                                                                                                                       |
| ---- | --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V3.1 | P1  | Summary cards render the count with `size(18)` not the spec's `28` (CC-11).                                                                                                                                   |
| V3.2 | P1  | Summary card backgrounds are all `Colors::SURFACE` — DESIGN.md says the bg should be the severity background color matching the metric (clean green or critical red, never neutral).                          |
| V3.3 | P1  | Section header arrows are `">"` / `"v"` text — should be 16 px chevron glyphs (DESIGN.md iconography).                                                                                                        |
| V3.4 | P1  | "Run Deep Scan" button is iced purple (CC-2).                                                                                                                                                                 |
| V3.5 | P1  | Dead-man-switch panel uses `SURFACE` background but DESIGN.md treatment for critical containers would call for `severity-critical-bg` (the translucent red) so it visually screams.                           |
| V3.6 | P2  | Severity label rendered via `format!("{}", finding.severity)` instead of badge component (CC-6).                                                                                                              |
| V3.7 | P2  | Action buttons row at the bottom (`Export Report`, `Copy Rotation Checklist`) is left-aligned but has no `[secondary]` styling; both should be `button-secondary` not `button-primary` per UI.md conventions. |
| V3.8 | P3  | "DEAD MAN SWITCH DETECTED" is `text(...)` in `SEVERITY_CRITICAL`; should also be a badge/banner with `severity-critical-bg` fill.                                                                             |

### V4 — Monitor `views/monitor.rs`

| ID   | P   | Finding                                                                                                                                                        |
| ---- | --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V4.1 | P1  | "Enable Monitor" CTA in iced purple (CC-2).                                                                                                                    |
| V4.2 | P1  | Alert rows show `[12:04:05Z] [HIGH] process (PID 123)` as a single text run. Should split into severity badge + monospace process name + secondary timestamp.  |
| V4.3 | P2  | `format_timestamp` strips date — fine for live view, but it should reuse a shared `format_short_time(u64)` so the same format appears in Alerts/Guard/Monitor. |
| V4.4 | P2  | Empty state lacks an icon glyph.                                                                                                                               |

### V5 — Audit `views/audit.rs`

| ID   | P   | Finding                                                                                                                                                |
| ---- | --- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| V5.1 | P1  | Empty state is top-left aligned plain text — break from the centered pattern of Deep Scan/Monitor (CC-8).                                              |
| V5.2 | P1  | Category traffic-light indicator is the unicode glyph `●◐○` followed by text — DESIGN.md (Traffic-Light Bars) specifies an 8 px filled bar with track. |
| V5.3 | P1  | No category score cards on top — UI/DESIGN spec calls for a row of `category-score-card`s before the per-category list.                                |
| V5.4 | P2  | Fix command line uses arrow `→` then plain sans — should use the `command-block` component (monospace + subtle bg + copy icon).                        |
| V5.5 | P2  | Summary line at bottom (`"3 green, 1 yellow, 0 red"`) renders without color — should use `severity-*` colors inline.                                   |

### V6 — Posture `views/posture.rs`

| ID   | P   | Finding                                                                                                                                                                                  |
| ---- | --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V6.1 | P1  | Same empty-state inconsistency as V5.1.                                                                                                                                                  |
| V6.2 | P1  | Same traffic-light glyph treatment as V5.2.                                                                                                                                              |
| V6.3 | P2  | "Detected: {pms_text}" line shows package managers as comma-joined plain text — should be a row of monospace pills (e.g. `npm v10.2.1` · `brew v4.2`).                                   |
| V6.4 | P2  | Same fix-command treatment as V5.4.                                                                                                                                                      |
| V6.5 | P2  | The traffic-light dot color in the category header (line 60) is a bare `text("●")` not a real dot graphic — fine for now, but DESIGN.md calls for the 8 px filled circle as a primitive. |

### V7 — Settings `views/settings.rs`

| ID   | P   | Finding                                                                                                                                            |
| ---- | --- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| V7.1 | P0  | The entire view is read-only `text(format!(...))` output. UI.md specifies a full editable form with dropdowns, toggles, list editors. (See CC-12.) |
| V7.2 | P1  | "Monitor" subheading shares the same 14 px sans as the rest — DESIGN.md headings are 15 px **semibold** with bottom-rule.                          |
| V7.3 | P1  | No section grouping or visual separation between General / Monitoring / Guard / Monitor sections.                                                  |
| V7.4 | P2  | Boolean values printed as `yes` / `no` — should be toggle widgets per DESIGN.md (Toggles).                                                         |

## Missing components (not yet implemented at all)

Per DESIGN.md `components` section, these are referenced by views but
have no implementation:

1. **Severity badge** (pill, uppercase, full radius) — CC-6.
2. **Severity dot** (8 px filled circle) — partially faked with `●` glyph.
3. **Card** (surface bg, border, lg radius, lg padding) — `deep_scan.rs:113-121`
   inlines a partial version; not shared.
4. **Summary card** (display number + secondary label, severity bg).
5. **Category score card** (label + traffic-light bar + finding count).
6. **Traffic-light bar** (8 px filled bar with track).
7. **Code block / command block** (mono font, bg surface, optional copy icon).
8. **Input field**, **dropdown**, **toggle** (all needed for Settings).
9. **Top bar** (current view + monitoring status + last-poll timestamp).
10. **Sidebar footer** (version + daemon status dot).
11. **Alert row** (severity dot + badge + pkg@ver mono + advisory + path + time + action).
12. **Notification overlay** (in-app variant of the OS notification).

## Recommended remediation order

Each block is roughly one PR. Bullets in priority order.

### Block A — Theme foundation (unblocks everything else)

1. Add missing color tokens to `theme.rs`: `ACCENT_MUTED`, `ACCENT_ACTIVE`,
   `TEXT_ON_ACCENT`, `SEVERITY_*_BG` (5 entries), `SURFACE_ELEVATED`.
   Remove the `#[allow(dead_code)]` markers from `SURFACE_HOVER` and
   `SURFACE_ACTIVE`.
2. Add `theme::spacing` and `theme::radius` constants matching DESIGN.md.
3. Add `theme::font::{display, heading, body, body_sm, badge, code, code_sm}`
   helpers that return sized `text` builders.
4. Add `theme::buttons::{primary, secondary, danger, nav}` style closures.

### Block B — Shell shape (CC-3, CC-4, CC-5)

1. Add top bar with current view title + monitoring status + last poll.
2. Rebuild sidebar with logo header, badge support, footer (version + status dot).
3. Move view titles out of view bodies.

### Block C — Severity primitives (CC-6, CC-7, V1.1, V3.6, V5.2)

1. Implement `severity_badge`, `severity_dot`, `traffic_light_bar` as
   reusable elements.
2. Replace every inline `format!("[{}]", sev)` with the badge.
3. Replace every `text("●")` / `text("◐")` with the dot primitive.

### Block D — Empty-state unification (CC-8)

1. Single `empty_state(title, body, cta)` helper.
2. Convert Alerts, Guard, Audit, Posture to use it with appropriate CTAs.

### Block E — Alerts + Guard row layouts (V1.1–1.5, V2.1–2.3)

Significant work — closest to the UI.md wireframes; should be its own PR.

### Block F — Settings form (V7.x, CC-12)

Largest single piece of work; depends on `iced` toggle + radio implementations.
Tracking-only for now.

## Verification workflow

While iterating:

```bash
# rebuild + relaunch
pkill -x ripley-app; cargo build -p ripley-app && open ./target/debug/ripley-app

# find current window id (changes per launch)
swift -e 'import CoreGraphics
let w = CGWindowListCopyWindowInfo([.optionAll], 0) as! [[String: Any]]
for x in w {
  let n = x["kCGWindowOwnerName" as String] as? String ?? ""
  let b = x["kCGWindowBounds" as String] as? [String: Any] ?? [:]
  if n.contains("ripley") && (b["Width"] as? Double ?? 0) > 500 {
    print(x["kCGWindowNumber" as String] as? Int ?? 0)
  }
}'

# capture each view
for i in 1 2 3 4 5 6 7; do
  peekaboo hotkey --keys "cmd,$i" --app ripley-app
  sleep 0.4
  peekaboo image --window-id <WID> --path /tmp/ripley-view-$i.png
done
```

Cmd+1..7 keyboard shortcuts that enable this loop live in
`crates/ripley-app/src/app.rs` in the `subscription` method.
