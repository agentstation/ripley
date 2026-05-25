---
version: alpha
name: Ripley
description: >
  Ripley is a supply chain defense tool for developers. The visual identity
  is dark, dense, and terminal-adjacent — a security operations interface
  that earns trust through transparency, not polish. Developers spend all
  day in dark IDEs and terminal windows. Ripley sits in that environment
  without standing out, until something demands attention. When it does,
  severity is the only hierarchy that matters. The palette is lifted from
  the GitHub dark theme family — familiar enough to feel native, distinct
  enough to read as a security tool. Monospace code, colored severity dots,
  and information density signal that this is a tool built by developers
  for developers, not a dashboard built by designers for executives.

colors:
  # Surfaces
  bg: "#0d1117"
  surface: "#161b22"
  surface-hover: "#1c2128"
  surface-active: "#282e36"
  surface-elevated: "#1c2128"
  border: "#30363d"
  border-subtle: "#21262d"

  # Text
  text-primary: "#e6edf3"
  text-secondary: "#8b949e"
  text-muted: "#484f58"
  text-on-accent: "#ffffff"
  text-on-severity: "#ffffff"

  # Accent
  accent: "#58a6ff"
  accent-hover: "#79c0ff"
  accent-active: "#388bfd"
  accent-muted: "#58a6ff30"

  # Severity
  severity-critical: "#f85149"
  severity-critical-bg: "#f8514920"
  severity-high: "#db6d28"
  severity-high-bg: "#db6d2820"
  severity-medium: "#d29922"
  severity-medium-bg: "#d2992220"
  severity-low: "#58a6ff"
  severity-low-bg: "#58a6ff20"
  severity-clean: "#3fb950"
  severity-clean-bg: "#3fb95020"

  # Semantic
  success: "#3fb950"
  warning: "#d29922"
  error: "#f85149"
  info: "#58a6ff"

typography:
  display:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 28px
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: -0.5px
  heading:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 15px
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: 0px
  body:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: 0px
  body-sm:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: 0px
  badge:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 11px
    fontWeight: 600
    lineHeight: 1.0
    letterSpacing: 0.5px
  code:
    fontFamily: "ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, monospace"
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: 0px
  code-sm:
    fontFamily: "ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, monospace"
    fontSize: 11px
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: 0px

rounded:
  none: 0px
  sm: 4px
  md: 6px
  lg: 8px
  full: 9999px

spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  2xl: 32px
  3xl: 48px

components:
  button-primary:
    background: "{colors.accent}"
    color: "{colors.text-on-accent}"
    fontSize: "{typography.body.fontSize}"
    fontWeight: 600
    paddingX: "{spacing.lg}"
    paddingY: "{spacing.sm}"
    borderRadius: "{rounded.md}"
  button-primary-hover:
    background: "{colors.accent-hover}"
  button-primary-active:
    background: "{colors.accent-active}"
  button-secondary:
    background: transparent
    color: "{colors.text-primary}"
    border: "1px solid {colors.border}"
    fontSize: "{typography.body.fontSize}"
    fontWeight: 500
    paddingX: "{spacing.lg}"
    paddingY: "{spacing.sm}"
    borderRadius: "{rounded.md}"
  button-secondary-hover:
    background: "{colors.surface-hover}"
  button-danger:
    background: "{colors.severity-critical}"
    color: "{colors.text-on-severity}"
    fontSize: "{typography.body.fontSize}"
    fontWeight: 600
    paddingX: "{spacing.lg}"
    paddingY: "{spacing.sm}"
    borderRadius: "{rounded.md}"
  severity-badge:
    fontSize: "{typography.badge.fontSize}"
    fontWeight: "{typography.badge.fontWeight}"
    letterSpacing: "{typography.badge.letterSpacing}"
    paddingX: "{spacing.sm}"
    paddingY: 2px
    borderRadius: "{rounded.full}"
    textTransform: uppercase
  card:
    background: "{colors.surface}"
    border: "1px solid {colors.border}"
    borderRadius: "{rounded.lg}"
    padding: "{spacing.lg}"
  card-hover:
    background: "{colors.surface-hover}"
  sidebar:
    width: 200px
    background: "{colors.bg}"
    borderRight: "1px solid {colors.border}"
    padding: "{spacing.md}"
  sidebar-item:
    fontSize: "{typography.body.fontSize}"
    paddingX: "{spacing.md}"
    paddingY: "{spacing.sm}"
    borderRadius: "{rounded.md}"
    color: "{colors.text-secondary}"
  sidebar-item-active:
    color: "{colors.accent}"
    background: "{colors.accent-muted}"
    borderLeft: "2px solid {colors.accent}"
  input:
    background: "{colors.surface}"
    color: "{colors.text-primary}"
    border: "1px solid {colors.border}"
    fontSize: "{typography.body.fontSize}"
    paddingX: "{spacing.md}"
    paddingY: "{spacing.sm}"
    borderRadius: "{rounded.md}"
  input-focus:
    border: "1px solid {colors.accent}"
  dropdown:
    background: "{colors.surface}"
    border: "1px solid {colors.border}"
    borderRadius: "{rounded.md}"
  dropdown-item:
    paddingX: "{spacing.md}"
    paddingY: "{spacing.sm}"
  dropdown-item-hover:
    background: "{colors.surface-hover}"
  toggle:
    width: 36px
    height: 20px
    borderRadius: "{rounded.full}"
    background-off: "{colors.border}"
    background-on: "{colors.accent}"
    knob: "{colors.text-on-accent}"
  code-block:
    background: "{colors.bg}"
    border: "1px solid {colors.border}"
    borderRadius: "{rounded.md}"
    padding: "{spacing.md}"
    fontFamily: "{typography.code.fontFamily}"
    fontSize: "{typography.code.fontSize}"
    lineHeight: "{typography.code.lineHeight}"
  alert-row:
    paddingX: "{spacing.lg}"
    paddingY: "{spacing.md}"
    borderBottom: "1px solid {colors.border-subtle}"
  alert-row-hover:
    background: "{colors.surface-hover}"
  summary-card:
    padding: "{spacing.lg}"
    borderRadius: "{rounded.lg}"
    minWidth: 120px
    textAlign: center
  notification:
    maxWidth: 380px
  table-header:
    fontSize: "{typography.body-sm.fontSize}"
    fontWeight: 600
    color: "{colors.text-secondary}"
    textTransform: uppercase
    letterSpacing: 0.5px
    paddingY: "{spacing.sm}"
    borderBottom: "1px solid {colors.border}"
  table-row:
    fontSize: "{typography.body.fontSize}"
    paddingY: "{spacing.sm}"
    borderBottom: "1px solid {colors.border-subtle}"
  table-row-hover:
    background: "{colors.surface-hover}"
  traffic-light-bar:
    height: 8px
    borderRadius: "{rounded.full}"
    background-track: "{colors.border}"
  category-score-card:
    padding: "{spacing.lg}"
    borderRadius: "{rounded.lg}"
    minWidth: 120px
    textAlign: center
  command-block:
    background: "{colors.bg}"
    fontFamily: "{typography.code.fontFamily}"
    fontSize: "{typography.code.fontSize}"
    paddingX: "{spacing.sm}"
    paddingY: 2px
    borderRadius: "{rounded.md}"
  countdown-timer:
    fontSize: "{typography.body.fontSize}"
    color: "{colors.text-secondary}"
    color-urgent: "{colors.text-primary}"
---


# Design

Design system for Ripley. This document is a token dictionary that defines
the visual language --- colors, typography, spacing, and component styles.
For view wireframes and interaction specs, see [UI.md](UI.md). For
component picks per workflow, keyboard model, and DX patterns, see
[UX_DESIGN.md](UX_DESIGN.md). For system architecture, see
[ARCHITECTURE.md](ARCHITECTURE.md). For user workflows, see
[WORKFLOW.md](WORKFLOW.md). For settings, see [SETTINGS.md](SETTINGS.md).
For technical decisions, see [DECISIONS.md](DECISIONS.md).


---


## Overview

Ripley lives in the developer's peripheral vision. It runs as a system
tray icon on macOS, invisible until a supply chain threat needs attention.
The visual language is built for that context: dark, dense, and
monospace-forward. It looks like it belongs next to a terminal, not
next to Slack.

Severity is the only visual hierarchy that matters. When nothing is wrong,
the UI is a quiet list of green checks. When something is wrong, the
affected item turns red and every surrounding element recedes. There is no
branding color that competes with critical. The accent blue is reserved for
interactive affordances --- links, active nav, selected states --- never
for status.

Every surface answers "what do I do now?" Alerts have [Fix]. Guard
findings have [Block] or [Allow]. Scan results have [Remediate]. Dead
ends don't exist.


---


## Colors

### Surfaces

Dark canvas with subtle layering. Background → surface → elevated surface.
Each step is a slight increase in lightness, never a hue shift.

| Token             | Hex       | Usage                                     |
|-------------------|-----------|-------------------------------------------|
| `bg`              | `#0d1117` | Window background, deepest layer          |
| `surface`         | `#161b22` | Cards, panels, list items, inputs         |
| `surface-hover`   | `#1c2128` | Interactive surface on hover              |
| `surface-active`  | `#282e36` | Pressed / selected surface                |
| `surface-elevated`| `#1c2128` | Popovers, dropdowns, dialogs              |
| `border`          | `#30363d` | Primary dividers, card outlines           |
| `border-subtle`   | `#21262d` | Row separators, secondary dividers        |

### Text

Three tiers. Primary for content the user is reading. Secondary for
labels and metadata. Muted for disabled states and placeholders.

| Token             | Hex       | Usage                                     |
|-------------------|-----------|-------------------------------------------|
| `text-primary`    | `#e6edf3` | Body text, headings, interactive labels   |
| `text-secondary`  | `#8b949e` | Timestamps, descriptions, secondary info  |
| `text-muted`      | `#484f58` | Disabled text, placeholders               |
| `text-on-accent`  | `#ffffff` | Text on accent-colored backgrounds        |
| `text-on-severity`| `#ffffff` | Text on severity-colored backgrounds      |

### Accent

Blue for interactive elements. Never used for status or severity.

| Token             | Hex       | Usage                                     |
|-------------------|-----------|-------------------------------------------|
| `accent`          | `#58a6ff` | Links, primary buttons, active nav item   |
| `accent-hover`    | `#79c0ff` | Accent on hover                           |
| `accent-active`   | `#388bfd` | Accent on press                           |
| `accent-muted`    | `#58a6ff30`| Active sidebar item background           |

### Severity

The most important palette. Used consistently across every surface:
alerts, guard findings, scan results, notification badges, summary cards,
CLI output. Each level has a foreground (text/icon) and a translucent
background (badge fill, row highlight).

| Level    | Foreground | Background   | When to use                         |
|----------|------------|--------------|-------------------------------------|
| Critical | `#f85149`  | `#f8514920`  | Active exploits, known malware, dead man switches |
| High     | `#db6d28`  | `#db6d2820`  | Network calls, eval, binary download, credential theft |
| Medium   | `#d29922`  | `#d2992220`  | Env harvesting, scope escape, posture warnings |
| Low      | `#58a6ff`  | `#58a6ff20`  | Informational signals, minor findings |
| Clean    | `#3fb950`  | `#3fb95020`  | No findings, healthy status, passed checks |

### Semantic

Convenience aliases that map to severity where appropriate.

| Token     | Hex       | Maps to           |
|-----------|-----------|-------------------|
| `success` | `#3fb950` | `severity-clean`  |
| `warning` | `#d29922` | `severity-medium` |
| `error`   | `#f85149` | `severity-critical` |
| `info`    | `#58a6ff` | `severity-low` / `accent` |


---


## Typography

System fonts only. No bundled typefaces, no web font loads. CSS uses
`font-family: system-ui` (sans) and `ui-monospace` (mono) — the WebView
resolves to the platform default automatically. macOS gets SF Pro (sans) and
SF Mono (mono). Linux gets the system default. Windows gets Segoe UI and
Cascadia Mono. Tailwind v4 exposes these via `font-sans` / `font-mono`.

### Scale

| Token      | Family      | Size  | Weight   | Line Height | Tracking  | Usage |
|------------|-------------|-------|----------|-------------|-----------|-------|
| `display`  | System sans | 28px  | Bold     | 1.2         | -0.5px    | Summary card numbers (3 Vulns, 2 IOCs) |
| `heading`  | System sans | 15px  | Semibold | 1.4         | 0         | Section headers, panel titles |
| `body`     | System sans | 13px  | Regular  | 1.5         | 0         | Primary body text, labels, buttons |
| `body-sm`  | System sans | 12px  | Regular  | 1.5         | 0         | Timestamps, secondary descriptions |
| `badge`    | System sans | 11px  | Semibold | 1.0         | +0.5px    | Severity labels (CRIT, HIGH, MED, LOW) |
| `code`     | System mono | 12px  | Regular  | 1.5         | 0         | Package names, file paths, version numbers, script content |
| `code-sm`  | System mono | 11px  | Regular  | 1.5         | 0         | Line numbers, secondary code references |

### Font Stack

```
Sans:  system-ui, -apple-system, BlinkMacSystemFont, sans-serif
Mono:  ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, monospace
```

### Rules

- Package names and versions are always `code` (monospace).
- File paths are always `code`.
- Script content in the guard dialog is always `code`.
- Advisory IDs (GHSA-xxxx, CVE-2026-xxxxx) are `body`, not `code`.
- Severity badges use `badge` with uppercase and wide tracking.
- Numbers in summary cards use `display` for visual weight.


---


## Spacing

4px base grid. All spacing values are multiples of 4.

| Token | Value | Usage |
|-------|-------|-------|
| `xs`  | 4px   | Tight gaps: icon-to-label, badge padding vertical |
| `sm`  | 8px   | Inner padding: buttons, input fields, list items |
| `md`  | 12px  | Card gaps, sidebar item padding, section spacing |
| `lg`  | 16px  | Card inner padding, content area padding |
| `xl`  | 24px  | Major section spacing, panel margins |
| `2xl` | 32px  | Top-level layout spacing |
| `3xl` | 48px  | Page-level padding (rarely used) |

### Layout Constants

| Element            | Value  |
|--------------------|--------|
| Sidebar width      | 200px  |
| Card gap           | 12px   |
| Card padding       | 16px   |
| Window default     | 900 x 640 |
| Window minimum     | 720 x 480 |
| Dialog max width   | 600px  |
| Notification width | 380px  |


---


## Elevation & Depth

No drop shadows. Depth is conveyed through background color steps and
borders. This keeps rendering cheap and avoids visual noise in a dense
interface.

| Level     | Background          | Border             | Usage |
|-----------|---------------------|--------------------|-------|
| Base      | `bg` (`#0d1117`)    | none               | Window background |
| Surface   | `surface` (`#161b22`) | `border` (`#30363d`) | Cards, panels, list areas |
| Elevated  | `surface-elevated` (`#1c2128`) | `border` (`#30363d`) | Dropdowns, popovers, guard dialog |
| Overlay   | `#0d1117cc` (bg at 80% opacity) | none | Modal backdrop |

No blur, no gradients, no transparency effects beyond the overlay
backdrop. Flat surfaces with border delineation.


---


## Shapes

### Border Radius

| Token  | Value  | Usage |
|--------|--------|-------|
| `none` | 0px    | Table cells, inline elements |
| `sm`   | 4px    | Small inline elements, tight corners |
| `md`   | 6px    | Buttons, inputs, sidebar items, code blocks, dropdowns |
| `lg`   | 8px    | Cards, summary cards, dialog window |
| `full` | 9999px | Severity badge pills, toggle switches |

### Iconography

- Tray icon: monochrome shield glyph, 22x22 template image on macOS.
- Severity dots: 8px filled circles using severity foreground color.
- Nav icons: 16px monochrome line icons, `text-secondary` default,
  `accent` when active.
- No emoji. No illustrated graphics. No gradients.


---


## Components

Detailed styling for each reusable component. All values reference design
tokens from the sections above.

### Buttons

Three variants. Primary for the main action. Secondary for alternatives.
Danger for destructive/containment actions.

| Variant   | Background     | Text Color        | Border               | Radius   |
|-----------|----------------|-------------------|-----------------------|----------|
| Primary   | `accent`       | `text-on-accent`  | none                  | `md`     |
| Secondary | transparent    | `text-primary`    | 1px solid `border`    | `md`     |
| Danger    | `severity-critical` | `text-on-severity` | none            | `md`     |

Padding: `sm` vertical, `lg` horizontal. Font: `body` at weight 600.

Hover states: Primary → `accent-hover`. Secondary → `surface-hover`
background. Danger → 10% lighter red.

### Severity Badges

Pill-shaped inline labels. Used in alert rows, guard log, scan results.

- Font: `badge` (11px, semibold, +0.5px tracking, uppercase).
- Padding: 2px vertical, `sm` horizontal.
- Border radius: `full`.
- Background: severity background color (20% opacity).
- Text: severity foreground color.
- Content: `CRIT`, `HIGH`, `MED`, `LOW`, `CLEAN`.

### Cards

Container for grouped content. Used for alert rows, settings sections,
summary cards.

- Background: `surface`.
- Border: 1px solid `border`.
- Border radius: `lg`.
- Padding: `lg` (16px).
- Hover: background shifts to `surface-hover` (for interactive cards).

### Summary Cards (Deep Scan Report)

Compact metric display. Shows a large number and a label.

- Min width: 120px.
- Text align: center.
- Number: `display` (28px bold).
- Label: `body-sm` in `text-secondary`.
- Background: severity background color matching the category.
- Border radius: `lg`.
- Padding: `lg`.

### Alert Rows

List items in the alerts view.

- Padding: `lg` horizontal, `md` vertical.
- Bottom border: 1px solid `border-subtle`.
- Hover: `surface-hover` background.
- Structure: severity dot + badge | package@version (code) | advisory summary (body-sm, text-secondary) | project path (code-sm, text-secondary) | time (body-sm, text-muted) | action button.
- Active row (detail panel open): `surface-active` background.

### Code Blocks

Used in the guard interception dialog for script content.

- Background: `bg`.
- Border: 1px solid `border`.
- Border radius: `md`.
- Padding: `md`.
- Font: `code`.
- Line numbers: `code-sm` in `text-muted`, right-aligned, `md` gap to content.
- Highlighted lines: severity background color on the full line, severity indicator at the end.

### Inputs

Text fields, dropdowns, and toggles in the settings view.

- Background: `surface`.
- Border: 1px solid `border`. On focus: 1px solid `accent`.
- Border radius: `md`.
- Padding: `sm` vertical, `md` horizontal.
- Font: `body`.
- Placeholder: `text-muted`.

### Toggles

On/off switches for boolean settings.

- Size: 36px wide, 20px tall.
- Border radius: `full`.
- Off: `border` background.
- On: `accent` background.
- Knob: `text-on-accent`, 16px circle.

### Traffic-Light Bars

Compact visual summary of a category's health. Used in the audit report
and harden views.

- Width: min 100px, flexible.
- Height: 8px.
- Border radius: `full`.
- Background: `border` (track).
- Fill: severity foreground color matching the worst finding in the
  category. Green = all checks pass. Yellow = medium findings. Red =
  critical/high findings.
- Label below: `body-sm` in `text-secondary` — e.g., "1R 1Y" for
  one red and one yellow finding.

Example: `████░░░░` where filled portion uses `severity-critical`
and empty portion uses `border`.

### Category Score Cards

Used at the top of the audit report and harden views. Similar to
summary cards but with a traffic-light bar instead of a large number.

- Min width: 120px.
- Text align: center.
- Label: `heading` (15px semibold) — category name.
- Traffic-light bar: centered below the label.
- Finding count: `body-sm` in `text-secondary` — e.g., "1R 1Y" or
  "all clear".
- Background: severity background color matching the worst finding.
- Border radius: `lg`.
- Padding: `lg`.

### Command Blocks

Copyable command output. Used in audit findings and harden recommendations.
Extends the code block component with a copy affordance.

- Same as `code-block` styling (bg, border, radius, font).
- Single-line variant (inline): `code` font, `bg` background, `sm`
  horizontal padding, `rounded.md`. No line numbers.
- Copy icon: 16px icon at right edge, `text-muted` default,
  `accent` on hover. Clicking copies the command text to clipboard.
- On copy: icon changes to a check mark for 2 seconds, then reverts.
- Used for: `→ sudo fdesetup enable`, `→ npm config set save-exact true`.

### Countdown Timer

Shown in the guard interception dialog during the timeout period.

- Position: right side of dialog title bar.
- Font: `body` in `text-secondary`. Uses `text-primary` when < 10 seconds.
- Format: "⏱ 12s" — clock emoji + seconds remaining.
- The Block button label includes the countdown: "Block (default in 12s)".
- At 0 seconds: dialog auto-dismisses, action = Block.
- No animation — the number decrements every second. Simplicity over
  visual flair.

### Tables

Used in the guard log view.

- Header: `body-sm`, semibold, `text-secondary`, uppercase, +0.5px tracking. Bottom border: 1px solid `border`.
- Rows: `body` font. Bottom border: 1px solid `border-subtle`. Hover: `surface-hover`.
- Expandable rows: clicking reveals a child area with `bg` background, `code` font for script excerpts.

### Sidebar

Fixed left navigation panel.

- Width: 200px.
- Background: `bg`.
- Right border: 1px solid `border`.
- Items: `body` font, `text-secondary` color. Padding: `sm` vertical, `md` horizontal. Border radius: `md`.
- Active item: `accent` text, `accent-muted` background, 2px left border in `accent`.
- Badge: `badge` font, `severity-critical` for alert count, right-aligned in the nav item.
- Bottom section: version number in `body-sm` `text-muted`, status dot (green = connected, `text-muted` = disconnected).


---


## Do's and Don'ts

**Do:**
- Use severity colors only for severity. Never use red for a cancel
  button or green for a confirm button unless the action has security
  implications.
- Show the raw evidence (matched lines, file paths, version ranges)
  alongside every finding. Developers dismiss tools they can't verify.
- Use monospace for anything that could be copied to a terminal: package
  names, versions, file paths, script content, commands.
- Keep the information density high. Developers are comfortable scanning
  tables and lists. Don't pad with whitespace or decorative elements.
- Default to the alerts view. The most important information should be
  visible within one second of opening the dashboard.

**Don't:**
- Don't use gradients, blur effects, or transparency beyond the overlay
  backdrop. Flat surfaces only.
- Don't use emoji in the UI. Not in labels, not in status messages, not
  in buttons. The severity dot is the only decorative element.
- Don't compete with severity using accent blue. Accent is for
  interaction affordance (links, selected nav), never for status.
- Don't add animations beyond hover state transitions and the alert
  detail panel slide-in. No loading spinners, no progress bars, no
  skeleton screens. The app either has data or it doesn't.
- Don't truncate package names, file paths, or advisory IDs. If the
  content is too wide, let it wrap or scroll horizontally. Truncation
  hides the information developers need most.
- Don't show empty sections. If the deep scan found no IOCs, omit the
  IOC section entirely --- don't show "IOC Files Found: 0".
- Don't add branding beyond the app name in the sidebar and the tray
  icon. No splash screen, no about dialog, no logo animation.
- Don't use light mode. There is no light theme. The dark palette is
  the identity.


---


## Responsive Behavior

Ripley is a desktop application, not a web app. There are no breakpoints
in the traditional sense, but the window is resizable.

| Window width  | Behavior |
|---------------|----------|
| 900px+ (default) | Full layout: sidebar + content area side by side |
| 720-899px (minimum) | Sidebar compresses: icons only, no labels. Content area takes remaining width. |
| Alert detail open | Content splits: alert list takes left half, detail panel takes right half. On narrow windows, detail panel replaces the list (back button to return). |

The guard interception dialog is a modal overlay, max 600px wide,
centered in the window. On narrow windows it stretches to window width
minus `xl` padding.


---


## Iteration Guide

When extending the design system:

1. **New color?** Add it to the YAML header first. If it's a severity
   variant, it must have both foreground and background (20% opacity)
   tokens. Reference it as `{colors.token-name}` in components.

2. **New component?** Add it to the YAML `components` section with all
   states (default, hover, active, disabled). Document which view uses
   it in the markdown body.

3. **New view?** Add the wireframe to [UI.md](UI.md), not here. This
   file defines how things look. UI.md defines what appears where.

4. **Changing a token?** Update the YAML header and grep the markdown
   body for any prose that references the old value.

5. **Adding a severity level?** Update the severity table, add foreground
   + background tokens, add a semantic alias if appropriate, and verify
   every component that uses severity (badges, dots, alert rows, summary
   cards, CLI output colors) handles the new level.
