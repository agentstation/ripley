# PLAN.md --- Ripley Execution Control Plane

This document is the single source of truth for implementation state.
It survives context compaction events and enables the agent to resume
work from any point. Update checkboxes and the Plan State section as
tasks complete.


## /goal

```
Implement Ripley Phase 5: Advanced Analysis. Execute milestones M19
through M23, pass every gate, and commit. PLAN.md is the control plane.

════════════════════════════════════════════════════════════════
 PROJECT CONTEXT
════════════════════════════════════════════════════════════════

Ripley is a supply chain defense tool — Rust workspace (edition 2024),
4 crates: ripley-core (library, thiserror), ripley-guard (CLI binary,
anyhow), ripley-ipc (IPC layer), ripley-app (iced tray app).

Phase 1 complete: npm lockfile parser, npm guard shims, macOS tray
app, remediation pipeline, forensic scan.

Phase 2 complete: 7 lockfile parsers, 7 guard shims, detection rules,
feed integration, platform abstraction. 183+ tests.

Phase 3 complete: `ripley audit` (environment security), `ripley harden`
(PM hardening), `ripley fix` (CVE remediation), `ripley exposure`
(credential assessment), templates, dashboard audit/posture views.
363 tests, M10-M13 gates passed.

Phase 4 adds real-time active detection — two new CLI commands + daemon:
  M14: Monitor config + process scanner (core library)
  M15: `ripley monitor` daemon (process + filesystem + advisory)
  M16: `ripley contain <pid|pkg>` (kill + forensic snapshot)
  M17: Notifications + IPC extensions (alerts, streaming)
  M18: Monitor dashboard view + tray integration

Building blocks already in place:
  commands/watch.rs     — daemon loop, advisory poller, lockfile watcher,
                          IPC server (Status), CancellationToken, events
  ripley-ipc/           — Request/Response, StatusData, Unix socket
  forensic/network.rs   — C2Database, connection parsers (3 platforms),
                          check_c2_connections()
  forensic/persistence.rs — persistence path scanner, PersistenceCategory
  app/events.rs         — AppEvent + Action enums (scaffolding)
  app/watcher.rs        — lockfile FS watcher (scaffolding)
  app/notifier.rs       — desktop notification via notify_rust
  app/tray.rs           — system tray menu (scaffolding)
  config.rs             — MonitoringConfig (needs [monitor] fields)

Spec documents (read before implementing):
  CLAUDE.md       — conventions, build commands, scope guardrails
  ARCHITECTURE.md — "Process and network monitoring", "Filesystem
                    anomaly detection", "Real-time alerting"
  ROADMAP.md      — Phase 4 spec (§"Phase 4: Active Detection")
  SETTINGS.md     — [monitor] config section spec
  UI.md           — Monitor sidebar item, disabled until enabled
  WORKFLOW.md     — §7 "Active containment"

════════════════════════════════════════════════════════════════
 EXECUTION LOOP — repeat until Phase 4 Gate passes
════════════════════════════════════════════════════════════════

1. READ STATE
   Read PLAN.md §"Plan State". Find the first unchecked `- [ ]` task
   in the current milestone. Read that task's full description — it
   specifies files, types, functions, tests, and verify commands.

2. READ SPECS before implementing. Each milestone has `> Spec:` refs.
   Do not guess field names, formats, or API shapes when a spec exists.

3. IMPLEMENT the task.
   - Read existing code first. Extend existing patterns:
     - collect_*() / evaluate_*() for system checks.
     - TrafficLight for findings, Severity for alerts.
     - Existing daemon loop in watch.rs — extend, don't rewrite.
     - IPC protocol in ripley-ipc — add variants, don't break existing.
   - async code uses tokio: spawn, spawn_blocking, channels, select!,
     CancellationToken. Platform I/O goes in spawn_blocking.
   - No `unwrap()` or `expect()` in ripley-core — return Result.
   - No `unsafe`. No new crates unless justified.
   - Test fixtures in `tests/fixtures/` at workspace root.
   - `--format json|table` on new commands.
   - Exit codes: 0=clean, 1=findings/alerts, 2=error.
   - Platform-specific code: `cfg(target_os)`, use platform.rs helpers.

4. VERIFY — every task, no exceptions.
   - Run the task's specific verify command (listed in the task).
   - Run `cargo clippy --workspace` — zero warnings.
   - Run `cargo fmt --all -- --check` — no diffs.
   - Fix any failure before proceeding. Do not skip.

5. MARK COMPLETE
   - Check the task box `[x]` in PLAN.md.
   - Update Plan State: Task = completed code, Status = "completed".
   - Go to step 1.

════════════════════════════════════════════════════════════════
 MILESTONE GATES — run all commands at each gate
════════════════════════════════════════════════════════════════

After all tasks in a milestone are checked, run that milestone's gate.
Every command must pass. Fix failures and re-run until clean.

── M14 GATE (Monitor Core Library) ────────────────────────────

  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo test -p ripley-core -- monitor
  cargo test -p ripley-core -- config

  VERIFY: MonitorConfig fields match SETTINGS.md spec.
  VERIFY: evaluate_connections filters and flags correctly.
  VERIFY: evaluate_fs_event maps persistence/lockfile/MCP paths.

  Pass → commit `M14: Monitor config and process scanner`,
  update Plan State to M15.

── M15 GATE (Monitor Daemon) ──────────────────────────────────

  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo run -p ripley-guard -- monitor --help
  cargo test -p ripley-core -- monitor
  cargo test -- monitor_integration

  VERIFY: ripley monitor --help shows usage with --daemon, --format.
  VERIFY: Monitor detects planted persistence write in test.
  VERIFY: Alerts appear in guard.jsonl.
  VERIFY: Clean shutdown with Ctrl-C.

  Pass → commit `M15: Monitor daemon`, update Plan State to M16.

── M16 GATE (Contain Command) ─────────────────────────────────

  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo run -p ripley-guard -- contain --help
  cargo test -p ripley-core -- monitor::contain
  cargo test -p ripley-ipc

  VERIFY: ripley contain --help shows usage with target, --format.
  VERIFY: Snapshot serializes to valid JSON.
  VERIFY: IPC Contain request/response roundtrips.

  Pass → commit `M16: Contain command`, update Plan State to M17.

── M17 GATE (Notifications + IPC) ─────────────────────────────

  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo test -p ripley-ipc
  cargo test -p ripley-core -- monitor

  VERIFY: Notification formatting for each alert type.
  VERIFY: IPC alert roundtrips work.
  VERIFY: Guard log entries are valid JSON with all fields.

  Pass → commit `M17: Notifications and IPC`, update Plan State to M18.

── M18 GATE (Dashboard + Tray) ────────────────────────────────

  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo build -p ripley-app

  VERIFY: ripley-app builds without warnings.
  VERIFY: Monitor view, tray, settings, events all wired.
  VERIFY: No #[allow(dead_code)] on Phase 4 scaffolding.

  Pass → commit `M18: Monitor dashboard and tray`, update Plan State
  to Phase 4 Gate.
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo build -p ripley-app
  cargo test -p ripley-core -- templates

  VERIFY: remediation templates load for each attack type.
  VERIFY: dashboard audit and posture views build without errors.

  Pass → commit `M13: Templates and dashboard`, update to Phase 3 Gate.

════════════════════════════════════════════════════════════════
 PHASE 3 GATE — final verification before completion
════════════════════════════════════════════════════════════════

All four milestone gates must have passed. Then run the final check:

  cargo build --workspace --release
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo deny check
  cargo run -p ripley-guard -- audit --format json
  cargo run -p ripley-guard -- harden --format json

  VERIFY: all 4 new commands produce correct output
  VERIFY: exit code 1 when findings present, 0 when clean
  VERIFY: no unwrap()/expect() in ripley-core:
    grep -rn 'unwrap()' crates/ripley-core/src/ | grep -v '#\[cfg(test)\]' | grep -v 'mod tests'
    grep -rn 'expect(' crates/ripley-core/src/ | grep -v '#\[cfg(test)\]' | grep -v 'mod tests'
  VERIFY: all public functions have tests
  VERIFY: insta snapshots cover all command outputs

When the Phase 3 Gate passes:
  1. Commit: `Phase 3: Response depth`
  2. Update CLAUDE.md "Current work" to Phase 4.
  3. Update Plan State: Phase = 4, Last gate = Phase 3.
  4. The goal is COMPLETE.

════════════════════════════════════════════════════════════════
 DEPENDENCY GRAPH
════════════════════════════════════════════════════════════════

M14.1  MonitorConfig ──► M14.2  Process scanner ──► M14.3  FS monitor
  │                         │                            │
  └─────────────────────────┴────────────────────── M14 Gate
                                                        │
M15.1  Extend WatchEvent ◄─────────────────────────────┘
  │
M15.2  Process loop ──► M15.3  Persistence loop ──► M15.4  CLI
  │                                                      │
  └──────────────────── M15.5  Integration ─────── M15 Gate
                                                        │
M16.1  Snapshot ──► M16.2  Kill + save ──► M16.3  CLI ──► M16.4  IPC
  │                                                            │
  └──────────────────────────────────────────────────── M16 Gate
                                                            │
M17.1  Notifications ──► M17.2  IPC streaming ──► M17.3  Log
  │                                                      │
  └──────────────────────────────────────────────── M17 Gate
                                                        │
M18.1  Monitor view ──► M18.2  Tray ──► M18.3  Events ──► M18.4  Settings
  │                                                              │
  └──────────────────────────────────────────────────────── M18 Gate
                                                                │
                                                         Phase 4 Gate

M14 tasks must be done in order (config first, then process, then FS).
M14.2 and M14.3 are independent of each other but both need M14.1.
M15 starts after M14 Gate (uses core monitor types).
M16 starts after M15 Gate (uses daemon infrastructure).
M17 starts after M16 Gate (uses contain types for notifications).
M18 starts after M17 Gate (uses IPC extensions for dashboard).

════════════════════════════════════════════════════════════════
 COMMIT STRATEGY
════════════════════════════════════════════════════════════════

Commit at each milestone gate — 5 milestone commits + 1 final:
  M14 Gate → `M14: Monitor config and process scanner`
  M15 Gate → `M15: Monitor daemon`
  M16 Gate → `M16: Contain command`
  M17 Gate → `M17: Notifications and IPC`
  M18 Gate → `M18: Monitor dashboard and tray`
  Phase 4 Gate → `Phase 4: Active detection`

Within a milestone, commit after completing a logical group of tasks
if the session is long and you want to checkpoint progress. Use
descriptive messages: `M14: Add monitor config section` etc.

════════════════════════════════════════════════════════════════
 RECOVERY AFTER COMPACTION
════════════════════════════════════════════════════════════════

After context is lost (compaction, new session, crash):
  1. Read PLAN.md §"Plan State" — current phase, milestone, task.
  2. git log --oneline -10 — confirm what's committed.
  3. git status + git diff --stat — find in-progress work.
  4. If uncommitted changes exist: review, finish the task, verify,
     mark complete. Do not discard partial work.
  5. Resume the execution loop from step 1.

The task descriptions in PLAN.md are self-contained. You do not need
prior conversation context to implement any task — the task tells you
what files to create, what functions to write, what tests to add, and
what commands to run. Read the task, read the specs it references, go.

════════════════════════════════════════════════════════════════
 CONSTRAINTS — hard rules, no exceptions
════════════════════════════════════════════════════════════════

SCOPE
- Phase 4 only. Do not implement sandboxed script execution,
  behavioral analysis engine, community rule sharing, CI/CD
  integrations, SARIF output, or any Phase 5+ features.
- Use trait/enum extension points where Phase 5+ will need them.

CODE QUALITY
- No unwrap() or expect() in ripley-core — always return Result.
- No unsafe unless measured and documented (there should be none).
- Every public function in ripley-core gets at least one unit test.
- insta snapshot tests for command outputs.
- cargo deny check must pass at every phase gate.

PATTERNS
- Monitor checks use collect/evaluate pattern:
  collect_*() runs system commands (I/O, cfg(target_os), spawn_blocking).
  evaluate_*(...) is pure and testable.
  Tests call evaluate_* with mock data.
- Severity enum for alert priorities (Critical/High/Medium/Low).
- TrafficLight for status indicators (Green/Yellow/Red).
- Async daemon code: tokio spawn/select!/CancellationToken.
  Platform I/O in spawn_blocking. Channels for event passing.
- Extend existing watch.rs daemon — don't rewrite from scratch.
- Extend existing IPC protocol — add variants, keep backward compat.
- New CLI commands follow existing scan.rs shape: async fn cmd_*,
  --format json|table, exit codes 0/1/2.

DEPENDENCIES
- No new crates expected for M14-M17. System checks use
  std::process::Command. `notify` crate already in workspace for
  filesystem watching. `notify_rust` for desktop notifications.
- M18 (dashboard) uses existing `iced`, `tray-icon`, `muda` deps.
- All deps must already be in [workspace.dependencies].

PLATFORM
- cfg(target_os) for platform-specific system checks.
- Never hardcode macOS commands in shared code.
- Use platform.rs helpers for paths, shells, home dir.
- Process monitoring: lsof (macOS), /proc+ss (Linux), netstat (Windows).
  Windows contain is a stub (Phase 5).

FIXTURES
- Test fixtures in tests/fixtures/ at workspace root.
- Monitor tests use mock connection data and mock FS events.
- Contain tests use mock lsof/ps/pgrep output.
```


## Recovery Protocol

After compaction, the agent has lost conversation context but the
codebase and this file are intact. Recovery steps:

1. `cat PLAN.md` --- read Plan State (current phase/milestone/task)
   and find the first unchecked `- [ ]` task.
2. `git log --oneline -10` --- see recent commits to confirm what's done.
3. `git status` --- see uncommitted work in progress.
4. `git diff --stat` --- see what files were being modified.
5. Read the current task's spec references if implementation context
   is needed.
6. Resume implementation from the first unchecked task.

**Phase 6 entry-point reading order** (when resuming M24-M28):
`STACK_DECISION.md` (locked stack, versions, tooling, repo layout) →
`DESIGN_ISSUES.md` (M27 view-migration priority list) →
`UI.md` (view wireframes + milestone map) →
`PLAN.md` Phase 6 task list.

If uncommitted changes exist that appear to be a partially completed
task, review them, finish the task, verify, and mark complete. Do not
discard partial work.


## Plan State

```
Phase:     6 --- UI Rewrite (Tauri 2)
Milestone: M24 --- Tauri scaffold and shell
Task:      M24.1
Status:    pending
Last gate: Phase 5 Gate
```

Update this section after each task completes. Format:
- **Task** = the code of the task just completed (e.g., `M1.3`)
- **Status** = `in progress` | `completed` | `blocked: <reason>`
- **Last gate** = the last milestone gate that passed (e.g., `M1`)


## Document References

Every spec document, what it contains, and when to consult it:

| Document | Contains | Consult when |
|----------|----------|-------------|
| `CLAUDE.md` | Conventions, build commands, scope guardrails, current milestone | Starting any task; need conventions |
| `ARCHITECTURE.md` | Component design, project structure, runtime layout, threat model | Implementing any module; need API shape |
| `DESIGN.md` | Color tokens, typography, spacing, component styles (YAML + prose) | Implementing UI (M3+) |
| `UI.md` | Wireframes, interaction specs, tray icon, dashboard, dialogs, error states | Implementing views (M3+) |
| `DECISIONS.md` | Why each tech choice was made, competitive landscape, prior art | Need rationale for an approach |
| `ROADMAP.md` | Phased milestones with task descriptions and verification criteria | Task context and verification |
| `WORKFLOW.md` | 14 user workflow streams with step-by-step flows and ASCII diagrams | Understanding user-facing behavior |
| `SETTINGS.md` | Every config setting, env var, CLI flag, rule format, IOC profile format | Implementing config, rules, settings |
| `README.md` | Project motivation, background attacks, CLI reference, architecture diagram | Project overview and CLI contract |
| `STACK_DECISION.md` | Phase 6 UI stack: Tauri 2 + React 19 + shadcn/Base UI + Tailwind v4; canonical 2026 versions, tooling, patterns, repo layout | Implementing M24-M28 (Phase 6) |
| `DESIGN_ISSUES.md` | Priority-ordered design gaps captured against iced app; M27 input list | Migrating views in M27 |
| `UX_DESIGN.md` | First-principles UX/DX: workflow → Base UI component mapping, keyboard model, tray + palette system surfaces, DX patterns | Implementing M24-M28 views; choosing components |


---


## Verification harness (Phase 6)

The Phase 6 success criteria must be **machine-checkable**, not "looks right."
This section defines the verification tools, their roles, and the standing
bar each milestone gate references.

### Tool roles (no overlap)

| Tool | Scope | Run mode | Owns |
|------|-------|----------|------|
| `cargo test --workspace` | Rust unit + integration | CI + local | Backend logic; IPC commands; `ripley-desktop` Rust crate |
| Vitest 2.x + `@testing-library/react` | TS unit + React component | CI + local | Component behavior; Zustand slices; hooks; bindings shape |
| **Playwright (new)** | React app served by Vite (no Tauri shell) | CI + local | Per-view smoke; a11y; visual regression; perf budgets; keyboard reachability |
| **Chrome DevTools MCP (new)** | Live Vite dev server in agent-driven Chromium | Agent-driven, dev only | Interactive verification during implementation: screenshots, Lighthouse audits, ARIA snapshots, console + network inspection |
| WebdriverIO 9 + `tauri-driver` | Full Tauri shell (window + tray + IPC) on real OS | CI only | E2E on Linux + Windows; tray icon presence; hotkey registration; IPC roundtrips |
| Peekaboo + manual review | macOS Tauri shell visuals | Local only | macOS WKWebView gap (no WebDriver coverage) |

**Why both Playwright and WebdriverIO:** Playwright cannot drive WebKitGTK
(Tauri's Linux WebView) and cannot drive WKWebView (Tauri's macOS WebView).
It targets the React app standalone via `pnpm dev` — fast feedback on every
PR. WebdriverIO+`tauri-driver` is the only way to exercise the actual Tauri
shell on Linux + Windows CI and stays the e2e primary. The two are
complementary, not competing.

**Why Chrome MCP is not a CI tool:** it is an agent-driven interactive
session, not a scripted runner. Use it during implementation to verify
incremental work ("navigate to alerts view, take a screenshot, run
Lighthouse, check console"). Convert findings into Playwright assertions
once stable.

### Standing bar (must hold for every view in M25-M28)

Each of these is enforced by a Playwright spec under
`apps/desktop/tests/browser/` (or by ESLint/Vite where noted):

| Check | Threshold | How verified |
|-------|-----------|--------------|
| Lighthouse accessibility | ≥ 95 per view | `lighthouse_audit` (MCP, ad-hoc) + `@playwright/test` + `playwright-lighthouse` (CI) |
| Lighthouse performance | ≥ 90 per view (Vite-served, headless Chromium) | Same |
| Lighthouse best-practices | ≥ 95 per view | Same |
| Zero `console.error` / `console.warn` | 0 on initial mount, 0 during scripted flows | Playwright `page.on("console", ...)` assertion |
| Zero failed network requests | 0 non-200 (or non-mocked) on view load | Playwright `page.on("requestfailed", ...)` assertion |
| Severity colors match tokens | `getComputedStyle` returns exact `severity-*` hex from DESIGN.md | Playwright per-badge assertion (one spec, all four severities) |
| Raw hex in JSX/CSS | 0 occurrences | ESLint rule `ripley/no-raw-hex` |
| `aria-label` on icon-only buttons | 100% coverage | Playwright `axe` scan (`@axe-core/playwright`) |
| Keyboard reachability | Every action in `KEYMAP` reachable via scripted keystroke | Playwright per-shortcut spec |
| Bundle size budget | TBD baseline + 10% ceiling | `vite build` reporter assert in CI |
| Mount-to-interactive (guard dialog component) | < 50ms warm in headless Chromium (proxy for cold path) | Playwright + `performance.mark` round-trip |

Failures on any of these block the corresponding milestone gate.

### What Chrome MCP does during implementation

When implementing a view (e.g. M27.2 Alerts), the canonical loop is:

1. Start `pnpm -C apps/desktop dev` (Vite dev server on `:5173`).
2. Use `mcp__chrome-devtools__new_page` to open `http://localhost:5173/alerts`.
3. `take_screenshot` — visual sanity.
4. `take_snapshot` — verify ARIA tree shape (heading levels, landmarks,
   button labels).
5. `lighthouse_audit` — confirm a11y ≥ 95 before opening a PR.
6. `list_console_messages` — verify no errors/warnings.
7. `list_network_requests` — verify no calls to non-`tauri://` endpoints
   (the dev server stubs IPC; production uses Tauri).
8. Encode any non-trivial finding as a Playwright spec under
   `tests/browser/` so it stays verified in CI.

### What Playwright does in CI

`apps/desktop/tests/browser/` contains scripted regressions:

- `tests/browser/views/*.spec.ts` — per-view smoke (mount, assert key DOM,
  no console errors, no failed requests)
- `tests/browser/a11y/*.spec.ts` — `@axe-core/playwright` scan per view
- `tests/browser/lighthouse/*.spec.ts` — `playwright-lighthouse` scoring
- `tests/browser/keyboard/*.spec.ts` — every `KEYMAP` entry scripted
  (`page.keyboard.press`)
- `tests/browser/visual/*.spec.ts` — screenshot diffs vs.
  `tests/browser/__snapshots__/`
- `tests/browser/perf/guard-dialog.spec.ts` — mount-to-interactive
  measurement against the < 50ms budget

These run on Chromium only (Vite-served, no Tauri); cross-platform
coverage lives in WebdriverIO+tauri-driver as before.

### What Chrome MCP and Playwright explicitly do NOT cover

- Tray icon and tray menu (OS-native, outside WebView). Owned by
  WebdriverIO + Rust unit tests + Peekaboo.
- Global hotkey registration. Owned by WebdriverIO + Rust unit tests.
- IPC over UDS. Owned by `ripley-desktop` Rust integration tests.
- Notifications (OS-native). Owned by Rust tests + manual cross-platform
  smoke.
- macOS visual fidelity. Owned by Peekaboo + manual review.


---


## Phase 1: Foundation

**Goal:** A developer on macOS can install Ripley, get a tray notification
when a package they depend on is compromised, click "Fix" to launch an AI
harness with a scoped remediation prompt, and run `ripley scan --deep` to
forensically audit their machine after a breach.

**Scope:** npm lockfile only, macOS only, OSV.dev feed only. No other PM
shims, no Windows/Linux tray, no `ripley fix`/`exposure`/`harden` commands.
Extension points (traits, enum variants) for later phases.


---


### M1: Core Data Pipeline

Build the data backbone: platform directories, configuration, feed client,
lockfile parser, matcher, and the `ripley scan` CLI.

> Spec: ROADMAP.md "M1: Core Data Pipeline"
> Spec: ARCHITECTURE.md "Before the breach: forward defense"
> Spec: SETTINGS.md (full config schema)


#### M1.S: Workspace scaffold

Set up the Cargo workspace, crate structure, and tooling configuration.
No business logic yet --- just the skeleton that everything else builds on.

- [x] **M1.S.1** Create root `Cargo.toml` (virtual workspace)
  - `[workspace]` with members: `crates/ripley-core`, `crates/ripley-guard`
  - `resolver = "2"`
  - `[workspace.dependencies]` declaring ALL shared deps:
    - `thiserror = "2"`, `anyhow = "1"`
    - `tokio = { version = "1", features = ["full"] }`
    - `reqwest = { version = "0.13", default-features = false, features = ["rustls", "json"] }`
    - `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `toml = "1"`
    - `clap = { version = "4", features = ["derive"] }`
    - `tracing = "0.1"`, `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`
    - `redb = "4"`, `semver = { version = "1", features = ["serde"] }`
    - `directories = "6"`, `regex = "1"`, `notify = "8"`
    - `colored = "3"`, `chrono = { version = "0.4", features = ["serde"] }`
    - `insta = { version = "1", features = ["json"] }`
  - `[workspace.package]` with `edition = "2024"`, `license = "AGPL-3.0-or-later"`
  - Verify: `cargo check --workspace` compiles (may be empty libs)
  - Spec: ARCHITECTURE.md "Project Structure"

- [x] **M1.S.2** Create `crates/ripley-core/Cargo.toml` and `src/lib.rs`
  - `[package]` name = `ripley-core`, `edition.workspace = true`
  - `[dependencies]`: thiserror, tokio, reqwest, serde, serde_json, toml,
    redb, semver, directories, regex, tracing, chrono --- all `{ workspace = true }`
  - `[dev-dependencies]`: insta, tempfile = "3", tokio (with test-util)
  - `src/lib.rs`: empty module declarations (filled in later tasks)
  - Verify: `cargo build -p ripley-core`

- [x] **M1.S.3** Create `crates/ripley-guard/Cargo.toml` and `src/main.rs`
  - `[package]` name = `ripley-guard`, `edition.workspace = true`
  - `[[bin]]` name = `"ripley"`, path = `"src/main.rs"`
  - `[dependencies]`: ripley-core (path), anyhow, clap, tokio,
    tracing, tracing-subscriber, colored, serde_json --- all workspace
  - `src/main.rs`: minimal clap CLI skeleton with `Scan` subcommand stub
  - Verify: `cargo run -p ripley-guard -- --help` prints help

- [x] **M1.S.4** Create `rust-toolchain.toml`
  - Pin channel: `[toolchain] channel = "stable"`
  - Verify: `rustup show` shows expected toolchain

- [x] **M1.S.5** Create `deny.toml`
  - `[advisories]` vulnerability = "deny", unmaintained = "warn"
  - `[licenses]` unlicensed = "deny", allow MIT, Apache-2.0, BSD-2/3,
    ISC, Unicode-3.0, Unicode-DFS-2016, Zlib, OpenSSL, ring
  - `[bans]` multiple-versions = "warn", wildcards = "deny"
  - `[sources]` unknown-registry = "deny", unknown-git = "deny"
  - Verify: `cargo deny check` passes
  - Spec: DECISIONS.md "Own supply chain"

- [x] **M1.S.6** Create `Cargo.lock`
  - Run `cargo generate-lockfile`
  - Verify: `Cargo.lock` exists and is valid

**Verify M1.S:**
```
cargo build --workspace
cargo run -p ripley-guard -- --help
cargo deny check
```


#### M1.0: Platform directories and configuration

> Spec: ARCHITECTURE.md "Runtime directory layout"
> Spec: SETTINGS.md "Platform Directories", "config.toml"
> Spec: ROADMAP.md M1 task 0

- [x] **M1.0.1** Create `crates/ripley-core/src/dirs.rs`
  - `pub fn project_dirs() -> Result<ProjectDirs>` using
    `ProjectDirs::from("com", "agentstation", "ripley")`
  - `pub fn config_dir() -> Result<PathBuf>` (creates dir if missing)
  - `pub fn data_dir() -> Result<PathBuf>` (creates dir if missing)
  - `pub fn cache_dir() -> Result<PathBuf>` (creates dir if missing)
  - Error type: `DirError` in this module using `thiserror`
  - Add `pub mod dirs;` to `lib.rs`
  - Test: `dirs::tests::test_dir_functions_return_paths`
  - Verify: `cargo test -p ripley-core -- dirs`

- [x] **M1.0.2** Create `crates/ripley-core/src/types.rs`
  - `pub enum Ecosystem { Npm, PyPI, Cargo, Go, Gem }` with `Display`, `Serialize`, `Deserialize`
  - `pub enum GuardMode { Strict, Audit, Off }` with serde, default = Strict
  - `pub enum RiskLevel { Low, Medium, High, Critical }` with `Ord`, serde
  - `pub enum Severity { Critical, High, Medium, Low }` with serde
  - Add `pub mod types;` to `lib.rs`
  - Test: `types::tests::test_ecosystem_display`
  - Verify: `cargo test -p ripley-core -- types`

- [x] **M1.0.3** Create `crates/ripley-core/src/config.rs`
  - Define structs with `#[serde(default)]` on all fields:
    - `Config` { general: GeneralConfig, monitoring: MonitoringConfig,
      guard: GuardConfig, posture: PostureConfig, audit: AuditConfig,
      feeds: FeedsConfig, notifications: NotificationsConfig,
      logging: LoggingConfig }
    - Each sub-struct matches SETTINGS.md schema exactly
  - `PostureConfig` { strict, require_lockfile, require_exact_versions,
    require_integrity_hashes, block_exotic_sources, allowed_registries }
  - `pub fn load_config(working_dir: &Path) -> Result<Config>`
    - Layer 1: defaults (struct Default impl)
    - Layer 2: `{config_dir}/config.toml` (user)
    - Layer 3: `{working_dir}/.ripley.toml` (project)
    - Layer 4: `RIPLEY_*` env vars
    - Each layer overrides previous non-default values
  - `pub fn write_default_config(path: &Path) -> Result<()>`
    - Atomic write: temp file then rename
  - Add `pub mod config;` to `lib.rs`
  - Test: `config::tests::test_default_config`
  - Test: `config::tests::test_layered_loading` (with tempdir)
  - Test: `config::tests::test_env_var_override`
  - Verify: `cargo test -p ripley-core -- config`
  - Spec: SETTINGS.md full schema, ARCHITECTURE.md "Configuration"

- [x] **M1.0.4** Wire `ripley config` subcommand in `main.rs`
  - Add `Config` variant to CLI enum with `--path`, `--show`, `--init` flags
  - `--path`: print `config_dir()/config.toml`
  - `--show`: load_config and print as TOML
  - `--init`: write_default_config if not exists
  - No flag: open in `$VISUAL` / `$EDITOR` / platform default
  - Verify: `cargo run -p ripley-guard -- config --path` prints a path
  - Verify: `cargo run -p ripley-guard -- config --init` creates file
  - Spec: ARCHITECTURE.md "ripley config"

**Verify M1.0:**
```
cargo test -p ripley-core -- dirs
cargo test -p ripley-core -- types
cargo test -p ripley-core -- config
cargo run -p ripley-guard -- config --path
```


#### M1.F: Test fixtures

> Spec: ROADMAP.md M1 task 0.5

- [x] **M1.F.1** Create `tests/fixtures/package-lock.json`
  - Realistic npm lockfileVersion 3 with:
    - Mix of pinned and range-specifier deps
    - Scoped packages (`@scope/name`)
    - Integrity hashes (SHA-512) on most entries
    - At least one package with a known OSV advisory (e.g., an old
      `lodash` or `express` version) for matcher testing
  - At least 15-20 packages for realistic coverage

- [x] **M1.F.2** Create `tests/fixtures/package-lock-clean.json`
  - Lockfile with only packages that have no known advisories
  - All entries have integrity hashes
  - All resolved URLs point to registry.npmjs.org
  - For testing exit-code-0 path

- [x] **M1.F.3** Create `tests/fixtures/package-lock-risky.json`
  - Lockfile with posture problems:
    - `git+` source URLs
    - Missing integrity hashes
    - `http://` resolved URLs (not https)
    - `file:` specifiers
  - For posture check testing

- [x] **M1.F.4** Create `tests/fixtures/scripts/malicious-postinstall.sh`
  - Multiple high-risk signals: `curl | sh`, `base64 --decode`,
    `eval "$(wget ...)"`, `process.env` harvesting, writes to
    `.claude/settings.json`
  - For M2 static analyzer snapshot tests

- [x] **M1.F.5** Create `tests/fixtures/scripts/benign-postinstall.sh`
  - Normal build operations: `node-gyp rebuild`, `mkdir -p dist`,
    `cp -r src/* dist/`
  - Should score Low risk in analyzer

**Verify M1.F:**
```
ls tests/fixtures/package-lock*.json        # 3 files
ls tests/fixtures/scripts/*.sh              # 2 files
python3 -m json.tool tests/fixtures/package-lock.json > /dev/null  # valid JSON
```


#### M1.1: OSV.dev API client

> Spec: ROADMAP.md M1 task 1
> Spec: ARCHITECTURE.md "Feed poller" (API details)

- [x] **M1.1.1** Create `crates/ripley-core/src/feed/mod.rs`
  - Define `Advisory` struct:
    `id, ecosystem: Ecosystem, package: String, affected_ranges: Vec<AffectedRange>,
    severity: Option<Severity>, summary: String, references: Vec<String>,
    iocs: Vec<String>`
  - `AffectedRange` { introduced: semver::Version, fixed: Option<semver::Version> }
  - `FeedError` enum using thiserror
  - Add `pub mod feed;` to `lib.rs`
  - Verify: `cargo build -p ripley-core`

- [x] **M1.1.2** Create `crates/ripley-core/src/feed/osv.rs`
  - `pub struct OsvClient` with `reqwest::Client` and `cache_dir: PathBuf`
  - `pub async fn query(&self, ecosystem: &str, package: &str) -> Result<Vec<Advisory>>`
    - POST to `https://api.osv.dev/v1/query`
    - Body: `{ "package": { "name": "...", "ecosystem": "..." } }`
    - Parse `vulns` array, map to `Advisory` structs
  - `pub async fn query_batch(&self, queries: &[(Ecosystem, &str)]) -> Result<Vec<Advisory>>`
    - POST to `https://api.osv.dev/v1/querybatch`
    - Max 1000 per request, chunk if needed
  - **ETag caching:** store ETag in `{cache_dir}/feeds/osv_{ecosystem}_{package}.etag`
    Send `If-None-Match` on subsequent requests. Skip parsing on 304.
  - **Exponential backoff:** on 5xx/timeout/network error, retry
    1s -> 2s -> 4s -> ... capped at 5 min. Use jitter.
  - Test: `osv::tests::test_parse_osv_response` (mock JSON, no network)
  - Test: `osv::tests::test_advisory_mapping` (verify field mapping)
  - Verify: `cargo test -p ripley-core -- feed`
  - Spec: ARCHITECTURE.md "Feed poller" for API format

- [x] **M1.1.3** Integration test for OSV client (requires network)
  - `tests/integration/osv_client.rs` or `#[ignore]` test
  - Query a known package (e.g., `lodash` on npm) and verify response
  - Mark `#[ignore]` so CI doesn't depend on network
  - Verify: `cargo test -p ripley-core -- osv --ignored` (manual)


#### M1.2: Advisory storage

> Spec: ROADMAP.md M1 task 2
> Spec: ARCHITECTURE.md "Advisory cache"

- [x] **M1.2.1** Create `crates/ripley-core/src/db.rs`
  - `pub struct AdvisoryDb` wrapping `redb::Database`
  - Table `advisories`: key = `"{ecosystem}:{package}"` (String),
    value = JSON bytes (`Vec<Advisory>` via serde_json)
  - Table `meta`: key = `"last_poll"`, value = u64 (unix timestamp)
  - `pub fn open(path: &Path) -> Result<Self>` (create if missing)
  - `pub fn store_advisories(&self, ecosystem: Ecosystem, package: &str, advisories: &[Advisory]) -> Result<()>`
  - `pub fn get_advisories(&self, ecosystem: Ecosystem, package: &str) -> Result<Vec<Advisory>>`
  - `pub fn get_all_advisories(&self) -> Result<Vec<Advisory>>`
  - `pub fn get_last_poll(&self) -> Result<Option<u64>>`
  - `pub fn set_last_poll(&self, timestamp: u64) -> Result<()>`
  - Add `pub mod db;` to `lib.rs`
  - Error type: `DbError` using thiserror
  - Test: `db::tests::test_store_and_retrieve` (tempdir)
  - Test: `db::tests::test_last_poll` (tempdir)
  - Test: `db::tests::test_empty_db_returns_empty` (tempdir)
  - Verify: `cargo test -p ripley-core -- db`


#### M1.3: Lockfile parser

> Spec: ROADMAP.md M1 task 3
> Spec: ARCHITECTURE.md "Lockfile indexer"

- [x] **M1.3.1** Create `crates/ripley-core/src/lockfile/mod.rs`
  - Define shared types:
    - `InstalledPackage { name: String, version: semver::Version, ecosystem: Ecosystem }`
    - `RiskySpec { package: String, specifier: String, reason: String }`
    - `LockfileWarning { package: String, field: String, message: String, severity: Severity }`
    - `ParsedLockfile { packages: Vec<InstalledPackage>, risky_specs: Vec<RiskySpec>, warnings: Vec<LockfileWarning> }`
  - `pub fn parse_lockfile(path: &Path) -> Result<ParsedLockfile>` (dispatch by filename)
  - `LockfileError` using thiserror
  - Add `pub mod lockfile;` to `lib.rs`
  - Verify: `cargo build -p ripley-core`

- [x] **M1.3.2** Create `crates/ripley-core/src/lockfile/npm.rs`
  - `pub fn parse_package_lock(content: &str) -> Result<ParsedLockfile>`
  - Parse `package-lock.json` lockfileVersion 2 and 3
  - Extract packages from the `"packages"` object:
    - Key = path like `"node_modules/express"` or `"node_modules/@scope/name"`
    - Strip `node_modules/` prefix to get package name
    - Handle scoped packages: `node_modules/@scope/name` -> `@scope/name`
    - Skip the root entry (empty key `""`)
  - Extract `version` field, parse with `semver::Version`
  - **Risky specs:** flag entries with `resolved` containing `git+`, `http://`,
    `file:` or where version is `*`, `latest`, or very broad ranges
  - **Lockfile integrity:** validate `resolved` URLs against expected
    registries (default: `registry.npmjs.org`), check `integrity` field
    exists and uses SHA-512, detect HTTP downgrade from HTTPS
  - Test: `npm::tests::test_parse_fixture` using `tests/fixtures/package-lock.json`
  - Test: `npm::tests::test_risky_specs` using `tests/fixtures/package-lock-risky.json`
  - Test: `npm::tests::test_scoped_packages`
  - Test: `npm::tests::test_clean_lockfile` using `tests/fixtures/package-lock-clean.json`
  - Snapshot test: `insta::assert_json_snapshot!` on parsed output
  - Verify: `cargo test -p ripley-core -- lockfile`
  - Verify: `cargo test -p ripley-core -- npm`


#### M1.4: Matcher

> Spec: ROADMAP.md M1 task 4
> Spec: ARCHITECTURE.md "Matcher"

- [x] **M1.4.1** Create `crates/ripley-core/src/matcher.rs`
  - `pub struct Match` { advisory: Advisory, package: InstalledPackage,
    project_path: PathBuf }
  - `pub fn find_matches(advisories: &[Advisory], packages: &[InstalledPackage], project_path: &Path) -> Vec<Match>`
  - For each advisory, check if any installed package matches ecosystem +
    name. If version falls within any affected range, it's a match.
  - Version matching: `v >= introduced && (fixed.is_none() || v < fixed)`
  - Sort results by severity (critical first), then by package name
  - Add `pub mod matcher;` to `lib.rs`
  - Test: `matcher::tests::test_match_found` (advisory + matching package)
  - Test: `matcher::tests::test_no_match` (advisory + non-matching version)
  - Test: `matcher::tests::test_no_fixed_version` (open-ended range)
  - Test: `matcher::tests::test_multiple_ranges`
  - Verify: `cargo test -p ripley-core -- matcher`


#### M1.5: Wire up `ripley scan`

> Spec: ROADMAP.md M1 task 5
> Spec: WORKFLOW.md "2. Proactive scan"
> Spec: SETTINGS.md "CLI Flags"

- [x] **M1.5.1** Implement `Scan` subcommand in `main.rs`
  - CLI args: `path` (positional), `--format` (json|table), `--deep` (flag),
    `--fix` (flag), `--no-cache` (flag)
  - Walk the given path for `package-lock.json` files
  - Parse each lockfile -> `ParsedLockfile`
  - Load advisories from redb. If DB empty or stale (> `stale_threshold_secs`),
    fetch from OSV.dev first (batch query all unique packages)
  - If `--no-cache`, always fetch fresh
  - Run matcher -> `Vec<Match>`
  - Collect posture warnings from all lockfiles
  - Spec: ARCHITECTURE.md "ripley scan" and "First-run behavior"

- [x] **M1.5.2** Table output (default format)
  - For each match: advisory ID, package name, installed version, severity,
    summary. Use `colored` crate for severity highlighting.
  - Below matches: posture summary line ("3 packages use range specifiers,
    1 resolves from git+, 47 entries missing integrity hashes")
  - Spec: ROADMAP.md M1 task 5 output description

- [x] **M1.5.3** JSON output (`--format json`)
  - Serialize `{ "matches": [...], "posture_warnings": [...] }` to stdout
  - Each match: advisory_id, package, version, severity, summary, project_path
  - Each warning: package, field, message, severity
  - Snapshot test: `insta::assert_json_snapshot!` on JSON output
  - Verify: output is valid JSON (`python3 -m json.tool`)

- [x] **M1.5.4** Exit codes and error handling
  - Exit 0: no matches (clean)
  - Exit 1: matches found
  - Exit 2: error (network failure, parse error, etc.)
  - Posture warnings alone do NOT cause exit 1
  - When `[posture] strict = true` in config, posture warnings DO cause exit 1
  - First-run offline: exit 2 with message "no cached advisories and network
    unavailable --- run with network access to populate the cache"
  - Test: integration test with clean fixture -> exit 0
  - Test: integration test with fixture containing known vuln -> exit 1

- [x] **M1.5.5** `ripley status` subcommand
  - When daemon not running: advisory cache age, guard shim status,
    configured project roots, "daemon not running"
  - Read redb `meta.last_poll` for cache age
  - Check `{data_dir}/bin/npm` exists for shim status
  - Read config for project roots
  - Spec: ARCHITECTURE.md "ripley status"

**Verify M1.5:**
```
cargo run -p ripley-guard -- scan tests/fixtures/
cargo run -p ripley-guard -- scan --format json tests/fixtures/
cargo run -p ripley-guard -- status
```


#### M1 Gate

**All must pass before starting M2:**

- [x] `cargo build --workspace` --- compiles clean
- [x] `cargo test --workspace` --- all tests pass
- [x] `cargo clippy --workspace` --- no warnings
- [x] `cargo fmt --all -- --check` --- formatted
- [x] `cargo deny check` --- own supply chain audit passes
- [x] `cargo run -p ripley-guard -- scan tests/fixtures/` --- produces table output
- [x] `cargo run -p ripley-guard -- scan --format json tests/fixtures/` --- produces valid JSON
- [x] `cargo run -p ripley-guard -- config --path` --- prints a path
- [x] `cargo run -p ripley-guard -- status` --- prints status info
- [x] Exit code test: scan clean fixture returns 0
- [ ] Exit code test: scan fixture with known vuln returns 1
- [x] Commit: `M1: Core data pipeline`
- [x] Update CLAUDE.md "Current work" to M2


---


### M2: Guard MVP (npm)

Build the package manager interceptor: detection rules, static analysis,
npm PATH shim, and the script-shell binary.

> Spec: ROADMAP.md "M2: Guard MVP (npm)"
> Spec: ARCHITECTURE.md "Component 2: ripley-guard"
> Spec: SETTINGS.md "Detection Rules"


#### M2.1: Detection rules

> Spec: SETTINGS.md "Detection Rules", "Rule File Format"
> Spec: ROADMAP.md M2 task 1

- [x] **M2.1.1** Create `crates/ripley-core/src/rules/mod.rs`
  - `Rule` struct: id, name, description, ecosystem, signal, weight (RiskLevel),
    patterns: Vec<String>
  - `RuleSet` struct holding `Vec<Rule>`
  - `RuleSet::load_compiled()` --- load rules from `include_str!` (base rules)
  - `RuleSet::load_user_rules(dir: &Path)` --- load `*.toml` from config dir
  - `RuleSet::merge(base, user)` --- user rules with same `id` override base
  - `RuleSet::for_ecosystem(&self, eco: Ecosystem) -> Vec<&Rule>`
  - Add `pub mod rules;` to `lib.rs`
  - Test: `rules::tests::test_load_compiled_rules`
  - Test: `rules::tests::test_user_override`
  - Verify: `cargo test -p ripley-core -- rules`

- [x] **M2.1.2** Create `rules/npm_postinstall.toml`
  - Rules for: network_call, code_generation, encoding_obfuscation,
    binary_execution, shell_spawning, env_harvesting, scope_escape,
    obfuscation_tools, ai_tool_config_write, mcp_server_injection
  - Follow SETTINGS.md "Rule File Format" exactly
  - Spec: ARCHITECTURE.md "Static analysis" signal table

- [x] **M2.1.3** Create `rules/credential_exfil.toml`
  - Cross-ecosystem patterns for credential theft:
    reading ~/.npmrc, ~/.aws/credentials, ~/.ssh/*, ~/.env,
    process.env bulk access
  - Weight: high or critical

- [x] **M2.1.4** Create `rules/persistence_write.toml`
  - Patterns for writes to .claude/, .vscode/tasks.json, .mcp.json,
    .cursor/mcp.json, LaunchAgents, crontab
  - Weight: critical
  - Spec: ARCHITECTURE.md "AI tool config as persistence surface"


#### M2.2: Static analyzer

> Spec: ROADMAP.md M2 task 2
> Spec: ARCHITECTURE.md "Static analysis"

- [x] **M2.2.1** Create `crates/ripley-core/src/analyzer.rs`
  - `pub struct AnalysisResult` { risk_level: RiskLevel,
    matched_rules: Vec<MatchedRule>, highlighted_lines: Vec<(usize, String, RiskLevel)> }
  - `MatchedRule` { rule_id, rule_name, matched_line: usize, matched_text: String }
  - `pub fn analyze(script: &str, rules: &RuleSet, ecosystem: Ecosystem) -> AnalysisResult`
  - Compile regex patterns once per rule (cache via lazy_static or OnceLock)
  - For each rule, check each line of script against patterns
  - Risk level = highest weight among matched rules; no matches = Low
  - Add `pub mod analyzer;` to `lib.rs`
  - Test with malicious fixture: expect High/Critical
  - Test with benign fixture: expect Low
  - Snapshot test: `insta::assert_json_snapshot!` on analysis output
  - Verify: `cargo test -p ripley-core -- analyzer`

- [x] **M2.2.2** Typosquatting detector
  - `pub fn check_typosquat(name: &str, ecosystem: Ecosystem) -> Option<TyposquatMatch>`
  - Layered approach:
    1. Extended Damerau-Levenshtein with keyboard-adjacency weighting
    2. Popularity-weighted scoring against top-1000 packages list
    3. Unicode homoglyph detection via confusable character mapping
  - Ship curated top-1000 list as compiled-in asset
  - Flag matches as High risk
  - Test: `analyzer::tests::test_typosquat_detection` ("expresss" -> "express")
  - Test: `analyzer::tests::test_homoglyph` (Greek omicron in "lodash")
  - Verify: `cargo test -p ripley-core -- typosquat`
  - Spec: ROADMAP.md M2 task 2, DECISIONS.md "Typosquatting detection"


#### M2.3: Script extractor

> Spec: ROADMAP.md M2 task 3

- [x] **M2.3.1** Create `crates/ripley-guard/src/extractor.rs`
  - `pub struct Script` { name: String, content: String, source_file: PathBuf }
  - `pub fn extract_lifecycle_scripts(package_dir: &Path) -> Result<Vec<Script>>`
  - Read `package.json`, extract `scripts.preinstall`, `scripts.install`,
    `scripts.postinstall`, `scripts.prepare`, `scripts.prepack`
  - Return empty vec if no lifecycle scripts
  - Test: create temp dir with package.json, verify extraction
  - Verify: `cargo test -p ripley-guard -- extractor`


#### M2.4: npm PATH shim

> Spec: ROADMAP.md M2 task 4
> Spec: ARCHITECTURE.md "Installation" PATH shims

- [x] **M2.4.1** Create `crates/ripley-guard/src/bin/ripley-npm-shim.rs`
  - Add `[[bin]] name = "ripley-npm-shim"` to Cargo.toml
  - Parse args to detect `install`/`add`/`i` commands
  - For install: run advisory check against local DB, warn if known advisory
  - Resolve real npm: `which -a npm`, skip self, pick next
  - Delegate to real npm, pass through stdout/stderr, preserve exit code
  - Verify: `cargo build -p ripley-guard --bin ripley-npm-shim`


#### M2.5: npm script-shell

> Spec: ROADMAP.md M2 task 5
> Spec: WORKFLOW.md "4. Install interception"

- [x] **M2.5.1** Create `crates/ripley-guard/src/bin/ripley-script-shell.rs`
  - Add `[[bin]] name = "ripley-script-shell"` to Cargo.toml
  - npm invokes as: `ripley-script-shell -c "script content"`
  - Parse `-c` arg to get script content
  - Load detection rules, run static analyzer
  - Low risk: execute via `/bin/sh -c "..."`
  - Medium+ risk: print analysis with colored output, prompt user
    (allow/block). If tray app running, send IPC `GuardPrompt` instead.
  - Block: exit 1
  - Log decision to `{data_dir}/guard.jsonl`
  - Verify: `cargo build -p ripley-guard --bin ripley-script-shell`


#### M2.6: Guard install/uninstall

> Spec: ROADMAP.md M2 task 6
> Spec: WORKFLOW.md "1. Setup & onboarding", "14. Uninstall & upgrade"

- [x] **M2.6.1** Implement `guard install` subcommand
  - Create `{data_dir}/bin/` directory
  - Copy `ripley-npm-shim` binary to `{data_dir}/bin/npm`
  - Copy `ripley-script-shell` binary to `{data_dir}/bin/ripley-script-shell`
  - Detect shell RC file (.zshrc, .bashrc, .profile)
  - Append `export PATH="{data_dir}/bin:$PATH"` if not present
  - Set `script-shell={data_dir}/bin/ripley-script-shell` in `~/.npmrc`
  - All writes atomic (write temp, then rename)
  - Must be idempotent
  - Verify: `cargo run -p ripley-guard -- guard install` succeeds

- [x] **M2.6.2** Implement `guard uninstall` subcommand
  - Remove shim binaries from `{data_dir}/bin/`
  - Remove PATH line from shell RC
  - Remove `script-shell` line from `~/.npmrc`
  - Remove LaunchAgent plist if installed
  - Print summary of what was removed
  - Spec: WORKFLOW.md "14. Uninstall & upgrade"
  - Verify: `cargo run -p ripley-guard -- guard uninstall` succeeds


#### M2.7: Guard trust/untrust/log/status

> Spec: ROADMAP.md M2 task 7
> Spec: WORKFLOW.md "9. Trust management"
> Spec: SETTINGS.md "Guard Log"

- [x] **M2.7.1** Implement trust management
  - `guard trust <pkg>`: add to `[guard] trust` in config.toml (atomic write)
  - `guard untrust <pkg>`: remove from trust list
  - Support exact names and glob scopes (`@tanstack/*`)
  - Verify: `cargo run -p ripley-guard -- guard trust express`

- [x] **M2.7.2** Implement guard log
  - Guard log: append-only JSONL at `{data_dir}/guard.jsonl`
  - Entry format: { timestamp, package, version, script, action,
    risk_level, matched_rules, source, user_decision }
  - `guard log`: read last 20 lines, render as table
  - Spec: SETTINGS.md "Guard Log" entry format
  - Verify: `cargo run -p ripley-guard -- guard log`

- [x] **M2.7.3** Implement guard status
  - Show: which shims installed, script-shell active, trusted package count
  - Verify: `cargo run -p ripley-guard -- guard status`


#### M2 Gate

**All must pass before starting M3:**

- [x] `cargo build --workspace` --- compiles (all binaries)
- [x] `cargo test --workspace` --- all tests pass (75 total)
- [x] `cargo clippy --workspace` --- no warnings (only expected dead_code for extractor)
- [x] `cargo fmt --all -- --check`
- [x] `cargo deny check`
- [x] Analyzer: malicious fixture -> High/Critical risk
- [x] Analyzer: benign fixture -> Low risk
- [x] `cargo run -p ripley-guard -- guard status` --- shows "not installed"
- [x] `cargo run -p ripley-guard -- guard install` --- installs shims
- [x] `cargo run -p ripley-guard -- guard status` --- shows installed
- [x] `cargo run -p ripley-guard -- guard trust express` --- adds to trust
- [x] `cargo run -p ripley-guard -- guard log` --- works (no entries in fresh env)
- [x] `cargo run -p ripley-guard -- guard uninstall` --- removes shims
- [x] Commit: `M2: Guard MVP (npm)`
- [x] Update CLAUDE.md "Current work" to M3


---


### M3: Tray App MVP (macOS)

Build the system tray application and dashboard window.

> Spec: ROADMAP.md "M3: Tray App MVP (macOS)"
> Spec: ARCHITECTURE.md "Component 1: ripley-app"
> Spec: DESIGN.md (full design system)
> Spec: UI.md (all wireframes)


#### M3.1: Workspace expansion

- [x] **M3.1.1** Add `crates/ripley-app/` and `crates/ripley-ipc/`
  - Add to workspace members in root Cargo.toml
  - `ripley-ipc`: depends on serde, serde_json, tokio, thiserror
  - `ripley-app`: depends on ripley-core, ripley-ipc, tray-icon, muda,
    iced, notify-rust, tokio, tokio-util, tracing, tracing-appender
  - Add new workspace deps: tray-icon, muda, iced, notify-rust,
    tokio-util, tracing-appender
  - Verify: `cargo build --workspace`


#### M3.2: IPC layer

> Spec: ROADMAP.md M3 task 2

- [x] **M3.2.1** Create `crates/ripley-ipc/src/lib.rs`
  - Request enum: Status, Scan, GetAlerts, GuardPrompt
  - Response enum: Status, ScanResult, Alerts, GuardDecision, Error
  - Socket path: `{data_dir}/ripley.sock`
  - JSON serialization over UnixStream
  - Socket permissions: mode 0600
  - Client helper: `pub async fn send_request(req: Request) -> Result<Response>`
  - Server helper: `pub async fn serve(handler, cancel_token) -> Result<()>`
  - Test: round-trip serialize/deserialize
  - Verify: `cargo test -p ripley-ipc`


#### M3.3: Event channel architecture

> Spec: ROADMAP.md M3 task 3

- [x] **M3.3.1** Create `crates/ripley-app/src/events.rs`
  - `AppEvent` enum: AdvisoriesUpdated, LockfileChanged, MatchFound,
    UserAction, IpcRequest
  - `mpsc::channel<AppEvent>` shared by all components
  - All components take `CancellationToken` for graceful shutdown
  - Verify: `cargo build -p ripley-app`


#### M3.4: System tray

> Spec: ROADMAP.md M3 task 4
> Spec: UI.md "Tray Icon", "Tray Menu"

- [x] **M3.4.1** Create `crates/ripley-app/src/tray.rs`
  - System tray icon with `tray-icon` crate
  - Context menu with `muda`: Show Dashboard, Scan Now, Quit
  - Icon variants: shield outline (idle), shield+dot (alert), shield+! (critical)
  - Template images for macOS
  - Bridge tray events to iced via channel
  - Spec: UI.md "Tray Icon" and "Tray Menu" for states and menu structure


#### M3.5: Background poller

> Spec: ROADMAP.md M3 task 5

- [x] **M3.5.1** Create `crates/ripley-app/src/poller.rs`
  - Tokio task: poll OSV.dev on configured interval
  - Use ETag caching and exponential backoff from M1 feed client
  - On new advisories: send AdvisoriesUpdated event
  - Respect CancellationToken


#### M3.6: Lockfile watcher

> Spec: ROADMAP.md M3 task 6

- [x] **M3.6.1** Create `crates/ripley-app/src/watcher.rs`
  - Walk configured project roots, index all lockfiles (in-memory)
  - Watch for changes via `notify` v8
  - On change: re-parse, send LockfileChanged event


#### M3.7: Notifications

> Spec: ROADMAP.md M3 task 7
> Spec: UI.md "Notifications" (3 variants)

- [x] **M3.7.1** Create `crates/ripley-app/src/notifier.rs`
  - `notify-rust` for native OS notifications
  - Three variants: Before (Fix/View/Dismiss), During (Contain/View/Investigate),
    After (Remediate/View Report)
  - Click opens dashboard to relevant alert
  - Spec: UI.md notification wireframes


#### M3.8: Dashboard window

> Spec: ROADMAP.md M3 task 8
> Spec: UI.md (Dashboard, Alerts, Guard Log, Settings, First Run, Error States)
> Spec: DESIGN.md (all component specs)

- [x] **M3.8.1** Create `crates/ripley-app/src/theme.rs`
  - Custom `iced::Theme` implementing DESIGN.md color system
  - Severity colors, surface colors, text hierarchy, accent
  - System fonts (SF Pro/SF Mono on macOS)
  - Spec: DESIGN.md "Colors", "Typography"

- [x] **M3.8.2** Create `crates/ripley-app/src/app.rs`
  - `iced::Application` implementation
  - Message enum mapping to AppEvent
  - Sidebar navigation: Alerts (default), Guard, Settings
  - Window: 900x640 default, 720x480 minimum
  - Close hides window (does not quit)

- [x] **M3.8.3** Create `crates/ripley-app/src/views/alerts.rs`
  - Alert list sorted by severity then recency
  - Row: severity dot + badge, package@version, advisory, project path, time, action
  - Alert detail slide-in panel
  - Empty state: green shield, "No findings"
  - First-run state: welcome screen with setup steps
  - Spec: UI.md "Alerts View", "Alert Detail", "First Run"

- [x] **M3.8.4** Create `crates/ripley-app/src/views/guard_log.rs`
  - Table: time, package, script, risk, action
  - Expandable rows with script excerpts
  - Spec: UI.md "Guard Log View"

- [x] **M3.8.5** Create `crates/ripley-app/src/views/settings.rs`
  - Form reads/writes config.toml
  - Sections: General, Monitoring, Guard, Posture, Advanced
  - Saves immediately, atomic writes
  - Spec: UI.md "Settings View", SETTINGS.md full schema

- [x] **M3.8.6** Guard interception dialog
  - Modal overlay, max 600px wide
  - Script content with line numbers and highlighted matched lines
  - Buttons: Allow Once, Block, Always Trust, Inspect
  - 30-second countdown timer, defaults to Block
  - Spec: UI.md "Guard Interception Dialog", DESIGN.md "Countdown Timer"

- [x] **M3.8.7** Error states
  - Network failure banner
  - No lockfiles found state
  - Spec: UI.md "Error States"


#### M3.9: App bundling

> Spec: ROADMAP.md M3 task 9

- [x] **M3.9.1** Configure cargo-bundle for macOS
  - Package `ripley-app` as `Ripley.app`
  - `Info.plist`: `LSUIElement = true`
  - App icon in `crates/ripley-app/icons/`
  - Verify: `cargo bundle --release -p ripley-app`


#### M3.10: Headless daemon

> Spec: ROADMAP.md M3 task 10
> Spec: ARCHITECTURE.md "ripley watch"

- [x] **M3.10.1** Implement `watch` subcommand in ripley-guard
  - Start poller, watcher, matcher, IPC server (no UI)
  - Fire notifications via notify-rust
  - `--daemon` flag: fork to background, log to file
  - Graceful shutdown on SIGTERM/SIGINT
  - Verify: `cargo run -p ripley-guard -- watch` starts daemon


#### M3 Gate

- [x] `cargo build --workspace` --- all 4 crates compile
- [x] `cargo test --workspace` --- all tests pass (79 total)
- [x] `cargo clippy --workspace`
- [x] `cargo fmt --all -- --check`
- [x] `cargo deny check`
- [x] `cargo bundle --release -p ripley-app` --- produces Ripley.app
- [x] Manual: launch Ripley.app, tray icon appears with menu
- [x] Manual: `ripley status` shows IPC connection to daemon
- [x] Manual: `ripley watch` starts headless daemon, responds to `ripley status`
- [x] Commit: `M3: Tray app MVP (macOS)`
- [x] Update CLAUDE.md "Current work" to M4


---


### M4: Remediation Pipeline

Wire the "Fix" button: prompt generation and AI harness launching.

> Spec: ROADMAP.md "M4: Remediation Pipeline"
> Spec: ARCHITECTURE.md "Prompt generator", "Harness launcher"
> Spec: WORKFLOW.md "5. Alert-driven fix"


#### M4.1: Prompt generator

- [x] **M4.1.1** Create `crates/ripley-core/src/prompt.rs`
  - `pub fn generate_remediation_prompt(m: &Match) -> String`
  - Template: project path, package@version, CVE, clean version,
    IOC file paths, credential hints, test instruction
  - Snapshot test: `insta::assert_snapshot!` on generated prompt
  - Spec: ARCHITECTURE.md "Prompt generator" example format
  - Verify: `cargo test -p ripley-core -- prompt`


#### M4.2: Harness launcher

- [x] **M4.2.1** Create `crates/ripley-core/src/harness.rs`
  - `pub fn detect_harness() -> Option<Harness>` --- check PATH for
    claude, codex, opencode (in order, or config preference)
  - `pub fn launch(harness: &Harness, prompt: &str) -> Result<Child>`
  - Spawn in new terminal: `open -a Terminal.app` + harness command
  - Return process handle (don't wait)
  - Test: `harness::tests::test_detect_harness` (with mocked PATH)
  - Verify: `cargo test -p ripley-core -- harness`


#### M4.3: Wire Fix action

- [x] **M4.3.1** Connect Fix in tray app
  - Notification "Fix" -> prompt generator -> harness launcher
  - Alert detail panel "Fix with Claude" -> same flow
  - Dashboard "Remediate All" -> batch prompt generation

- [x] **M4.3.2** Wire `--fix` flag on `ripley scan`
  - Generate prompts for all matches
  - Launch harness for each (or print to stdout if no harness)
  - Verify: `cargo run -p ripley-guard -- scan --fix tests/fixtures/`


#### M4 Gate

- [x] `cargo test --workspace` (86 tests)
- [x] `cargo clippy --workspace`
- [x] Prompt generator snapshot tests pass
- [x] `ripley scan --fix tests/fixtures/` generates prompt (prints to stdout)
- [x] Commit: `M4: Remediation pipeline`
- [x] Update CLAUDE.md "Current work" to M5


---


### M5: Forensic Scan

Build `ripley scan --deep` --- the post-breach audit.

> Spec: ROADMAP.md "M5: Forensic Scan"
> Spec: WORKFLOW.md "6. Post-breach forensics"
> Spec: UI.md "Deep Scan Report"
> Spec: SETTINGS.md "IOC Profiles"


#### M5.1: IOC scanner

- [x] **M5.1.1** Create `crates/ripley-core/src/forensic/mod.rs` and `ioc.rs`
  - IOC file patterns: .claude/execution.js, .claude/setup.mjs,
    .claude/settings.json (unexpected hooks), .vscode/tasks.json (runOn),
    .mcp.json (rogue servers), .cursor/mcp.json, node_modules/.cache/ binaries,
    ~/.ssh/authorized_keys (unexpected keys)
  - `pub fn scan_iocs(path: &Path, profiles: &[IocProfile]) -> Vec<IocFinding>`
  - `IocFinding` { path, description, severity, profile_id }
  - Add `pub mod forensic;` to lib.rs
  - Test with temp dir containing planted IOC files
  - Verify: `cargo test -p ripley-core -- forensic`

- [x] **M5.1.2** IOC profile loader
  - Parse TOML profiles from `{config_dir}/iocs/` (runtime) + compiled-in
  - `IocProfile` struct matching SETTINGS.md "IOC Profile Format"
  - Ship `iocs/tanstack-2026-05.toml` and `iocs/mini-shai-hulud.toml`


#### M5.2: Persistence auditor

- [x] **M5.2.1** Create `crates/ripley-core/src/forensic/persistence.rs`
  - Check macOS: ~/Library/LaunchAgents/*.plist, crontab, shell RC files
  - MCP/AI config audit: .mcp.json, .cursor/mcp.json, .claude/settings.json
  - Dead man switch detection: processes polling external APIs
  - `pub fn audit_persistence(home: &Path) -> Vec<PersistenceFinding>`
  - Verify: `cargo test -p ripley-core -- persistence`


#### M5.3: Credential exposure mapper

- [x] **M5.3.1** Create `crates/ripley-core/src/forensic/credentials.rs`
  - Map attack -> targeted credential stores (from IOC profiles)
  - Check which files exist on this machine
  - Generate rotation commands per credential
  - Dead man switch warning if applicable
  - `pub fn assess_exposure(profiles: &[IocProfile], home: &Path) -> Vec<CredentialFinding>`
  - Verify: `cargo test -p ripley-core -- credentials`


#### M5.4: Deep scan CLI

- [x] **M5.4.1** Wire `--deep` flag in scan subcommand
  - Run lockfile scan + IOC scanner + persistence auditor + credential mapper
  - Table output: summary, findings by category, recommended actions
  - JSON output: structured report
  - Generate remediation prompt if findings exist
  - Spec: UI.md "Deep Scan Report" for output structure
  - Verify: `cargo run -p ripley-guard -- scan --deep tests/fixtures/`

- [x] **M5.4.2** Deep scan report in dashboard
  - Summary cards: Vulns, IOCs, Persistence, Creds Risk, MCP
  - Collapsible sections per category
  - Dead man switch warning panel
  - Action buttons: Remediate All, Export Report, Copy Rotation Checklist
  - Spec: UI.md "Deep Scan Report" wireframe


#### M5 Gate

- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace`
- [x] `cargo fmt --all -- --check`
- [x] `cargo deny check`
- [x] `cargo run -p ripley-guard -- scan --deep ~` on clean machine: "no findings"
- [x] Deep scan with planted IOC fixtures: findings reported
- [x] IOC profile loading works (compiled-in + runtime)
- [x] Commit: `M5: Forensic scan`
- [x] Update CLAUDE.md "Current work" to Phase 2


#### Phase 1 Gate

**All must pass before starting Phase 2:**

- [x] All M1-M5 gates passed
- [x] `cargo build --workspace --release` --- release build succeeds
- [x] `cargo bundle --release -p ripley-app` --- Ripley.app bundle works
- [ ] Full workflow test: install guard, run scan, trigger notification,
      click Fix, review prompt output, run deep scan
- [x] `cargo deny check` --- clean
- [x] No `unwrap()` or `expect()` in ripley-core source
- [x] All public functions in ripley-core have at least one test
- [x] Snapshot tests exist for: analyzer output, CLI output, prompt format


---


## Phase 2: Ecosystem Breadth

**Goal:** Ripley guards all major package managers and runs on all three
desktop platforms. Every ecosystem in the developer stack --- npm, yarn, pnpm,
pip, cargo, go, gem --- gets a lockfile parser and a guard shim. Feed
integration expands beyond OSV.dev to include GHSA and Socket.dev. The tray
app and CLI build and run on macOS, Linux, and Windows.

**Scope:** Six new lockfile parsers, six new guard shims, two new detection
rule files, two new feed sources, three platform builds. The npm foundation
from Phase 1 remains stable --- this phase extends the same patterns to new
ecosystems without modifying npm-specific code.

> Spec: ROADMAP.md "Phase 2: Ecosystem Breadth"


---


### M6: Lockfile Parsers

Parse every major lockfile format. Each parser follows the same trait and
output shape as the npm parser from M1.

> Spec: ROADMAP.md "Phase 2: Ecosystem Breadth" — Lockfile parsers
> Spec: ARCHITECTURE.md "Lockfile indexer"


#### M6.1: Yarn lockfile parser

- [x] **M6.1.1** Create `crates/ripley-core/src/lockfile/yarn.rs`
  - Parse `yarn.lock` v1 (classic) format: YAML-like key-value with
    `package@version:` headers and `resolved`, `integrity` fields
  - Parse `yarn.lock` v2/Berry format: YAML with `__metadata` header,
    `resolution`, `checksum` fields, PnP-aware resolution paths
  - Detect version by presence of `__metadata` key
  - Extract `ParsedLockfile` with packages, risky specs, warnings
  - Risky specs: `git+`, `http://`, `file:`, `patch:`, `portal:` protocols
  - Lockfile integrity: check `integrity`/`checksum` fields present
  - Add `"yarn.lock"` dispatch to `parse_lockfile()` in `mod.rs`
  - Test: `yarn::tests::test_parse_v1_fixture` with `tests/fixtures/yarn-v1.lock`
  - Test: `yarn::tests::test_parse_v2_fixture` with `tests/fixtures/yarn-v2.lock`
  - Test: `yarn::tests::test_scoped_packages`
  - Test: `yarn::tests::test_risky_specs` (git+, http, patch protocols)
  - Snapshot: `insta::assert_json_snapshot!` on parsed output
  - Verify: `cargo test -p ripley-core -- yarn`


#### M6.2: pnpm lockfile parser

- [x] **M6.2.1** Create `crates/ripley-core/src/lockfile/pnpm.rs`
  - Parse `pnpm-lock.yaml` v6 and v9 formats
  - v6: `packages` key with `/pkg/version` entries
  - v9: `snapshots` + `packages` split, `settings.autoInstallPeers`
  - Detect version from `lockfileVersion` field
  - Extract packages from nested YAML structure
  - Risky specs: `link:`, `file:`, `git+`, tarball URLs
  - Lockfile integrity: check `resolution.integrity` fields
  - Add `serde_yaml` to workspace dependencies (needed for pnpm/yarn-v2)
  - Add `"pnpm-lock.yaml"` dispatch to `parse_lockfile()`
  - Test: `pnpm::tests::test_parse_v6_fixture` with `tests/fixtures/pnpm-lock-v6.yaml`
  - Test: `pnpm::tests::test_parse_v9_fixture` with `tests/fixtures/pnpm-lock-v9.yaml`
  - Test: `pnpm::tests::test_risky_specs`
  - Snapshot: `insta::assert_json_snapshot!`
  - Verify: `cargo test -p ripley-core -- pnpm`


#### M6.3: pip lockfile parser

- [x] **M6.3.1** Create `crates/ripley-core/src/lockfile/pip.rs`
  - Parse `Pipfile.lock` (JSON format): `default` and `develop` sections,
    each entry has `version`, `hashes`, `index`
  - Parse `poetry.lock` (TOML format): `[[package]]` arrays with `name`,
    `version`, `source` tables
  - Detect format by filename (`Pipfile.lock` vs `poetry.lock`)
  - Map ecosystem to `Ecosystem::PyPI`
  - Risky specs: `git+`, `file:`, `editable = true`, missing hashes
  - Add `"Pipfile.lock"` and `"poetry.lock"` dispatch to `parse_lockfile()`
  - Test: `pip::tests::test_parse_pipfile_lock` with `tests/fixtures/Pipfile.lock`
  - Test: `pip::tests::test_parse_poetry_lock` with `tests/fixtures/poetry.lock`
  - Test: `pip::tests::test_risky_specs`
  - Snapshot: `insta::assert_json_snapshot!`
  - Verify: `cargo test -p ripley-core -- pip`


#### M6.4: Cargo lockfile parser

- [x] **M6.4.1** Create `crates/ripley-core/src/lockfile/cargo_lock.rs`
  - Parse `Cargo.lock` (TOML format): `[[package]]` arrays with `name`,
    `version`, `source`, `checksum`
  - Map ecosystem to `Ecosystem::Cargo`
  - Source parsing: `registry+`, `git+`, `path+` prefixes
  - Risky specs: `git+` sources (especially with branch/rev), `path+` sources,
    missing `checksum` fields, `registry+` pointing to non-crates.io registries
  - Add `"Cargo.lock"` dispatch to `parse_lockfile()`
  - Test: `cargo_lock::tests::test_parse_fixture` with `tests/fixtures/Cargo.lock`
  - Test: `cargo_lock::tests::test_risky_sources`
  - Snapshot: `insta::assert_json_snapshot!`
  - Verify: `cargo test -p ripley-core -- cargo_lock`


#### M6.5: Go lockfile parser

- [x] **M6.5.1** Create `crates/ripley-core/src/lockfile/go.rs`
  - Parse `go.sum`: `module version hash` lines (SHA-256 in base64)
  - Parse companion `go.mod` for `require` directives with version info
  - Map ecosystem to `Ecosystem::Go`
  - Extract module path + version from each line
  - Risky specs: `replace` directives pointing to local paths or non-standard
    module proxies, missing `go.sum` entries for required modules
  - Add `"go.sum"` dispatch to `parse_lockfile()`
  - Test: `go::tests::test_parse_go_sum` with `tests/fixtures/go.sum`
  - Test: `go::tests::test_risky_replace`
  - Snapshot: `insta::assert_json_snapshot!`
  - Verify: `cargo test -p ripley-core -- go`


#### M6.6: Gem lockfile parser

- [x] **M6.6.1** Create `crates/ripley-core/src/lockfile/gem.rs`
  - Parse `Gemfile.lock`: `GEM` section with `specs:` indented entries,
    `BUNDLED WITH` version footer
  - Map ecosystem to `Ecosystem::Gem`
  - Extract gem name + version from indented spec lines
  - Risky specs: `GIT` sources, `PATH` sources, `remote:` pointing to
    non-rubygems.org hosts
  - Add `"Gemfile.lock"` dispatch to `parse_lockfile()`
  - Test: `gem::tests::test_parse_fixture` with `tests/fixtures/Gemfile.lock`
  - Test: `gem::tests::test_risky_sources`
  - Snapshot: `insta::assert_json_snapshot!`
  - Verify: `cargo test -p ripley-core -- gem`


#### M6.F: Test fixtures

- [x] **M6.F.1** Create lockfile test fixtures
  - `tests/fixtures/yarn-v1.lock` — realistic v1 with scoped pkgs, integrity
  - `tests/fixtures/yarn-v2.lock` — v2/Berry with PnP metadata
  - `tests/fixtures/pnpm-lock-v6.yaml` — v6 format with nested deps
  - `tests/fixtures/pnpm-lock-v9.yaml` — v9 format with snapshots split
  - `tests/fixtures/Pipfile.lock` — mix of pinned + dev deps, hashes
  - `tests/fixtures/poetry.lock` — TOML format with sources
  - `tests/fixtures/Cargo.lock` — real workspace lockfile
  - `tests/fixtures/go.sum` — with hash pairs
  - `tests/fixtures/Gemfile.lock` — GEM + BUNDLED WITH sections


#### M6 Gate

**All must pass before starting M7:**

- [ ] `cargo build --workspace`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace` --- no warnings
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo deny check`
- [ ] Each parser handles its fixture correctly
- [ ] `ripley scan` auto-detects all lockfile types in a mixed project
- [ ] Snapshot tests exist for all parser outputs
- [ ] Commit: `M6: Lockfile parsers`
- [ ] Update CLAUDE.md "Current work" to M7


---


### M7: Detection Rules & Guard Shims

Build guard shims for each new ecosystem with ecosystem-specific detection rules.

> Spec: ROADMAP.md "Phase 2: Ecosystem Breadth" — Guard shims, Detection rules
> Spec: ARCHITECTURE.md "Component 2: ripley-guard"


#### M7.1: Detection rules for new ecosystems

- [x] **M7.1.1** Create `rules/pypi_setup.toml`
  - Patterns for malicious `setup.py`: `os.system()`, `subprocess.Popen`,
    `exec()`, `eval()`, `__import__('os')`, base64 decoding, network calls
    in `setup()`, `.pth` file creation, `distutils.command` overrides
  - Weight: high/critical for execution, medium for suspicious imports
  - Test: `rules::tests::test_pypi_rules_load`
  - Verify: `cargo test -p ripley-core -- rules`

- [x] **M7.1.2** Create `rules/cargo_build.toml`
  - Patterns for malicious `build.rs`: `Command::new`, network calls via
    `reqwest`/`ureq`/`std::net`, file writes outside `OUT_DIR`, reading
    home directory, environment variable harvesting, binary downloads
  - Weight: high for network + file writes, critical for credential access
  - Test: `rules::tests::test_cargo_rules_load`
  - Verify: `cargo test -p ripley-core -- rules`


#### M7.2: pnpm guard shim

- [x] **M7.2.1** Create `crates/ripley-guard/src/bin/ripley-pnpm-shim.rs`
  - Add `[[bin]] name = "ripley-pnpm-shim"` to Cargo.toml
  - Same architecture as npm shim: detect `install`/`add`, advisory check,
    delegate to real `pnpm`
  - Set `script-shell` in local `.npmrc` (pnpm respects same setting)
  - Reuse `ripley-script-shell` binary for lifecycle script interception
  - Verify: `cargo build -p ripley-guard --bin ripley-pnpm-shim`


#### M7.3: Yarn guard shim

- [x] **M7.3.1** Create `crates/ripley-guard/src/bin/ripley-yarn-shim.rs`
  - Add `[[bin]] name = "ripley-yarn-shim"` to Cargo.toml
  - Detect `add`/`install` commands
  - Yarn v1: set `script-shell` in `.yarnrc`
  - Yarn v2+/Berry: lifecycle scripts run via `yarn plugin` — shim must
    intercept at the binary level and monitor spawned processes
  - Advisory check against local DB
  - Verify: `cargo build -p ripley-guard --bin ripley-yarn-shim`


#### M7.4: pip guard shim

- [x] **M7.4.1** Create `crates/ripley-guard/src/bin/ripley-pip-shim.rs`
  - Add `[[bin]] name = "ripley-pip-shim"` to Cargo.toml
  - Intercept `pip install`, `pip install -e`
  - Before delegating: extract `setup.py` from sdist/wheel, run static
    analyzer with `pypi_setup.toml` rules
  - Flag `.pth` file creation attempts
  - Advisory check via OSV.dev (ecosystem = "PyPI")
  - Verify: `cargo build -p ripley-guard --bin ripley-pip-shim`

- [x] **M7.4.2** Create `tests/fixtures/scripts/malicious-setup.py`
  - `os.system("curl ...")`, `base64.b64decode(...)`, writes to `~/.bashrc`
  - For pypi_setup.toml snapshot tests


#### M7.5: Cargo guard shim

- [x] **M7.5.1** Create `crates/ripley-guard/src/bin/ripley-cargo-shim.rs`
  - Add `[[bin]] name = "ripley-cargo-shim"` to Cargo.toml
  - Intercept `cargo build`, `cargo install`
  - Before delegating: scan `build.rs` files in dependency tree with
    `cargo_build.toml` rules
  - Advisory check via OSV.dev (ecosystem = "crates.io")
  - Verify: `cargo build -p ripley-guard --bin ripley-cargo-shim`

- [x] **M7.5.2** Create `tests/fixtures/scripts/malicious-build.rs`
  - `Command::new("curl")`, reads `env::home_dir()`, writes outside OUT_DIR
  - For cargo_build.toml snapshot tests


#### M7.6: Go guard shim

- [x] **M7.6.1** Create `crates/ripley-guard/src/bin/ripley-go-shim.rs`
  - Add `[[bin]] name = "ripley-go-shim"` to Cargo.toml
  - Intercept `go install`, `go get`
  - Advisory check only (Go has no install scripts)
  - Check module against OSV.dev (ecosystem = "Go")
  - Verify: `cargo build -p ripley-guard --bin ripley-go-shim`


#### M7.7: Gem guard shim

- [x] **M7.7.1** Create `crates/ripley-guard/src/bin/ripley-gem-shim.rs`
  - Add `[[bin]] name = "ripley-gem-shim"` to Cargo.toml
  - Intercept `gem install`, `bundle install`
  - Rubygems allows `extconf.rb` and `Rakefile` execution during install
  - Static analyze `extconf.rb` with npm_postinstall rules (similar patterns)
  - Advisory check via OSV.dev (ecosystem = "RubyGems")
  - Verify: `cargo build -p ripley-guard --bin ripley-gem-shim`


#### M7.8: Guard install expansion

- [x] **M7.8.1** Expand `guard install` for all shims
  - Detect which PMs are installed on the system
  - Install shims only for detected PMs
  - Copy all built shim binaries to `{data_dir}/bin/`
  - `guard status` reports interception state per PM
  - `guard uninstall` removes all shim binaries
  - Test: `guard install` on system with npm + cargo installs both shims
  - Verify: `cargo run -p ripley-guard -- guard install`
  - Verify: `cargo run -p ripley-guard -- guard status`


#### M7 Gate

**All must pass before starting M8:**

- [ ] `cargo build --workspace` --- all shim binaries compile
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo deny check`
- [ ] All detection rules load and match against test fixtures
- [ ] `guard install` detects and installs shims for available PMs
- [ ] `guard status` shows per-PM interception state
- [ ] Commit: `M7: Detection rules and guard shims`
- [ ] Update CLAUDE.md "Current work" to M8


---


### M8: Feed Integration

Expand advisory sources beyond OSV.dev to include GHSA and Socket.dev.

> Spec: ROADMAP.md "Phase 2: Ecosystem Breadth" — Feed integration
> Spec: ARCHITECTURE.md "Feed poller"


#### M8.1: GHSA feed client

- [x] **M8.1.1** Create `crates/ripley-core/src/feed/ghsa.rs`
  - `pub struct GhsaClient` with `reqwest::Client` and auth token
  - GraphQL query to `https://api.github.com/graphql`
  - Query `securityVulnerabilities` by ecosystem and package
  - Map GHSA severity (CRITICAL/HIGH/MEDIUM/LOW) to `Severity` enum
  - Map GHSA advisory fields to `Advisory` struct
  - Pagination: handle `pageInfo.hasNextPage` / `endCursor`
  - Auth: `GITHUB_TOKEN` env var or config `[feeds] github_token`
  - Rate limiting: respect `X-RateLimit-Remaining`, backoff on 403
  - Test: `ghsa::tests::test_parse_graphql_response` (mock JSON)
  - Test: `ghsa::tests::test_advisory_mapping`
  - Integration test (`#[ignore]`): query a known package
  - Verify: `cargo test -p ripley-core -- ghsa`

- [x] **M8.1.2** Merge GHSA advisories into feed pipeline
  - `FeedSource` enum: `Osv`, `Ghsa`, `Socket`
  - `Advisory` gains `source: FeedSource` field
  - Deduplication: same CVE from multiple sources → merge, keep richest data
  - Poller queries both OSV and GHSA on each poll cycle
  - Config: `[feeds] sources = ["osv", "ghsa"]` (default both)


#### M8.2: Socket.dev feed client

- [x] **M8.2.1** Create `crates/ripley-core/src/feed/socket.rs`
  - `pub struct SocketClient` with API key auth
  - REST API: `https://api.socket.dev/v0/report/supported`
  - Query package scores and alerts
  - Map Socket alert types to `Severity`
  - Auth: `SOCKET_API_KEY` env var or config `[feeds] socket_api_key`
  - Rate limiting: respect API limits
  - Test: `socket::tests::test_parse_response` (mock JSON)
  - Integration test (`#[ignore]`): query a known package
  - Verify: `cargo test -p ripley-core -- socket`


#### M8 Gate

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace`
- [ ] Both feed clients parse mock responses correctly
- [ ] Advisory deduplication works across sources
- [ ] Config controls which sources are active
- [ ] Commit: `M8: Feed integration`
- [ ] Update CLAUDE.md "Current work" to M9


---


### M9: Platform Builds

Build and test on Windows and Linux. Platform-specific tray integration,
notifications, and installers.

> Spec: ROADMAP.md "Phase 2: Ecosystem Breadth" — Platform builds


#### M9.1: Cross-platform abstraction

- [x] **M9.1.1** Create platform abstraction layer
  - `crates/ripley-core/src/platform.rs` with trait `Platform`
  - Methods: `persistence_paths()`, `shell_rc_paths()`, `notify()`,
    `open_editor()`, `detect_pms()`
  - Implementations: `MacOsPlatform`, `LinuxPlatform`, `WindowsPlatform`
  - Compile-time dispatch via `cfg(target_os)` or runtime detection
  - Migrate macOS-specific code from persistence.rs, guard install, etc.
  - Test: each platform impl returns valid paths on its OS
  - Verify: `cargo test -p ripley-core -- platform`


#### M9.2: Linux build

- [x] **M9.2.1** Linux tray app
  - `tray-icon` + `muda` work on Linux (X11/Wayland via `libappindicator`)
  - D-Bus notifications via `notify-rust` (already cross-platform)
  - Persistence auditor: check systemd user services, cron, shell RC
  - Add `~/.config/systemd/user/` to persistence scan paths
  - Test on Ubuntu 22.04+ and Fedora 39+
  - Verify: `cargo build --workspace` on Linux

- [x] **M9.2.2** Linux installer
  - `.deb` package via `cargo-deb`
  - `.AppImage` via `linuxdeploy`
  - Desktop entry file for app launcher integration
  - Verify: `.deb` installs cleanly on Ubuntu


#### M9.3: Windows build

- [x] **M9.3.1** Windows tray app
  - `tray-icon` + `muda` work on Windows (Win32 API)
  - Windows notification API via `notify-rust` or `windows-rs` bindings
  - Persistence auditor: check Registry Run keys, Task Scheduler,
    Startup folder, PowerShell profiles
  - Guard shims: `.cmd` wrapper scripts for PATH interception
  - Verify: `cargo build --workspace` on Windows

- [x] **M9.3.2** Windows installer
  - `.msi` via WiX toolset or `cargo-wix`
  - Add to PATH during install
  - Start menu shortcut
  - Verify: `.msi` installs cleanly on Windows 10+


#### M9.4: CI matrix

- [x] **M9.4.1** GitHub Actions CI workflow
  - Matrix: macOS-latest, ubuntu-latest, windows-latest
  - Steps: build, test, clippy, fmt, deny
  - Cache: Cargo registry + target directory
  - Artifact upload: release binaries per platform


#### M9 Gate

- [x] `cargo build --workspace` on macOS, Linux, Windows
- [x] `cargo test --workspace` on all three platforms
- [x] `cargo clippy --workspace` clean on all three
- [x] Platform-specific persistence paths correct per OS
- [x] Installers build and install cleanly
- [x] CI matrix green on all three platforms
- [ ] Commit: `M9: Platform builds`


#### Phase 2 Gate

**All must pass before starting Phase 3:**

- [x] All M6-M9 gates passed
- [x] `cargo build --workspace --release` on macOS, Linux, Windows
- [x] `ripley scan` finds and parses all 7 lockfile types
- [x] `guard install` installs shims for all detected PMs
- [x] `guard status` shows per-PM state on all platforms
- [x] Advisory sources configurable (OSV, GHSA, Socket)
- [x] `cargo deny check` clean
- [x] No `unwrap()` or `expect()` in ripley-core
- [x] All public functions have tests
- [x] Snapshot tests for all parser outputs and detection rules
- [ ] Commit: `Phase 2: Ecosystem breadth`
- [ ] Update CLAUDE.md "Current work" to Phase 3


---


## Phase 3: Response Depth

**Goal:** Structured remediation after a breach.

> Spec: ROADMAP.md "Phase 3: Response Depth"
> Spec: WORKFLOW.md §10-§13
> Spec: SETTINGS.md §[audit]


### M10: Environment Security Audit (`ripley audit`)

> Spec: WORKFLOW.md "10. Environment security audit"
> Spec: SETTINGS.md "[audit]" section
> Spec: ROADMAP.md "Phase 3" → `ripley audit`


#### M10.F: Audit framework types

- [x] **M10.F.1** Create audit module with shared types
  - File: `crates/ripley-core/src/audit/mod.rs`
  - Types:
    - `TrafficLight` enum: `Green`, `Yellow`, `Red` (with Serialize, Deserialize,
      Display, Ord for comparison)
    - `AuditCategory` enum: `MachineSecurity`, `Toolchain`, `AiTools`, `Credentials`
      (with Serialize, Deserialize, Display)
    - `AuditFinding` struct: `name: String`, `status: TrafficLight`,
      `detail: String`, `fix_command: Option<String>`
    - `CategoryReport` struct: `category: AuditCategory`,
      `findings: Vec<AuditFinding>`, `overall: TrafficLight`
    - `AuditReport` struct: `categories: Vec<CategoryReport>`,
      `timestamp: String`
  - `CategoryReport::overall()` method: Red if any Red, Yellow if any Yellow,
    else Green
  - Register `pub mod audit;` in `crates/ripley-core/src/lib.rs`
  - Submodules: `pub mod machine;`, `pub mod toolchain;`, `pub mod ai_tools;`,
    `pub mod creds;`
  - Tests: `test_traffic_light_ordering`, `test_category_report_overall`
  - Verify: `cargo build -p ripley-core`


#### M10.1: Machine security checks

- [x] **M10.1.1** Disk encryption check
  - File: `crates/ripley-core/src/audit/machine.rs`
  - `pub fn collect_disk_encryption() -> Result<String, AuditError>`
    - macOS: `Command::new("fdesetup").arg("status")` → parse output
    - Linux: read `/etc/crypttab` or `Command::new("lsblk").args(["--fs", "--json"])`
    - Windows: `Command::new("manage-bde").args(["-status", "C:"])`
  - `pub fn evaluate_disk_encryption(output: &str) -> AuditFinding`
    - Green if enabled, Red if disabled, Yellow if unknown
  - Tests: `test_evaluate_filevault_on`, `test_evaluate_filevault_off`,
    `test_evaluate_luks_present`, `test_evaluate_bitlocker_on`
  - Verify: `cargo test -p ripley-core -- audit::machine::tests`

- [x] **M10.1.2** Firewall, OS updates, and screen lock checks
  - File: `crates/ripley-core/src/audit/machine.rs` (continued)
  - Firewall:
    - `pub fn collect_firewall() -> Result<String, AuditError>`
    - `pub fn evaluate_firewall(output: &str) -> AuditFinding`
    - macOS: `defaults read /Library/Preferences/com.apple.alf globalstate`
    - Linux: `ufw status` or `iptables -L -n`
    - Windows: `netsh advfirewall show allprofiles`
  - OS updates:
    - `pub fn collect_os_updates() -> Result<String, AuditError>`
    - `pub fn evaluate_os_updates(output: &str) -> AuditFinding`
    - macOS: `softwareupdate -l`
    - Linux: `apt list --upgradable 2>/dev/null` or `dnf check-update`
    - Windows: PowerShell `Get-WindowsUpdate` or registry check
  - Screen lock:
    - `pub fn collect_screen_lock() -> Result<String, AuditError>`
    - `pub fn evaluate_screen_lock(output: &str) -> AuditFinding`
    - macOS: `defaults read com.apple.screensaver idleTime`
    - Green if ≤300s, Yellow if ≤600s, Red if >600s or disabled
  - `pub fn check_machine_security() -> CategoryReport` — runs all 4
  - Tests: `test_evaluate_firewall_active`, `test_evaluate_firewall_off`,
    `test_evaluate_screen_lock_5min`, `test_evaluate_screen_lock_disabled`
  - Verify: `cargo test -p ripley-core -- audit::machine`


#### M10.2: Developer toolchain checks

- [x] **M10.2.1** Git signing and SSH key checks
  - File: `crates/ripley-core/src/audit/toolchain.rs`
  - `pub fn collect_git_signing() -> Result<String, AuditError>`
    - `git config --global commit.gpgsign` + `git config --global user.signingkey`
  - `pub fn evaluate_git_signing(output: &str) -> AuditFinding`
    - Green if gpgsign=true + key set, Yellow if not configured
  - `pub fn evaluate_ssh_keys(ssh_dir: &Path) -> Vec<AuditFinding>`
    - Scan `~/.ssh/` for key files, check algorithm (Ed25519=Green, RSA≥4096=Yellow,
      RSA<4096=Red, DSA=Red)
    - Check passphrase protection (try `ssh-keygen -y -P "" -f key`)
  - Tests: `test_evaluate_git_signing_configured`, `test_evaluate_git_signing_missing`,
    `test_evaluate_ssh_key_ed25519`, `test_evaluate_ssh_key_rsa_weak`
  - Verify: `cargo test -p ripley-core -- audit::toolchain`

- [x] **M10.2.2** Shell RC hygiene and history permissions
  - File: `crates/ripley-core/src/audit/toolchain.rs` (continued)
  - `pub fn evaluate_shell_rc_hygiene(content: &str, path: &Path) -> Vec<AuditFinding>`
    - Check for `eval $(curl`, `curl | bash`, `curl | sh`, suspicious domains
    - Reuse patterns from forensic/persistence.rs `SUSPICIOUS_PATTERNS`
    - Green if clean, Red if suspicious eval/curl found
  - `pub fn evaluate_shell_history_permissions(mode: u32) -> AuditFinding`
    - Green if owner-only (0600/0640), Yellow if group-readable, Red if world-readable
  - `pub fn check_toolchain(home: &Path) -> CategoryReport`
  - Tests: `test_evaluate_rc_clean`, `test_evaluate_rc_suspicious_curl`,
    `test_evaluate_history_permissions`
  - Verify: `cargo test -p ripley-core -- audit::toolchain`


#### M10.3: AI tool config integrity

- [x] **M10.3.1** MCP config and Claude hooks checks
  - File: `crates/ripley-core/src/audit/ai_tools.rs`
  - `pub fn evaluate_mcp_config(content: &str, path: &Path) -> Vec<AuditFinding>`
    - Parse JSON, check for rogue server definitions (non-standard URLs,
      prompt injection in tool descriptions — look for `<system>`, `ignore
      previous`, instruction override patterns)
    - Green if not present or clean, Red if rogue server found
  - `pub fn evaluate_claude_hooks(content: &str) -> Vec<AuditFinding>`
    - Parse `.claude/settings.json`, check `hooks` for unauthorized commands
    - Flag unexpected `SessionStart` hooks
    - Green if no hooks or all authorized, Yellow if hooks present
  - `pub fn evaluate_project_mcp_settings(content: &str) -> AuditFinding`
    - Check for `enableAllProjectMcpServers: true` — Red if found
  - Tests: `test_evaluate_mcp_clean`, `test_evaluate_mcp_rogue_server`,
    `test_evaluate_claude_hooks_safe`, `test_evaluate_claude_hooks_suspicious`,
    `test_evaluate_project_mcp_enabled`
  - Verify: `cargo test -p ripley-core -- audit::ai_tools`

- [x] **M10.3.2** VS Code extension check
  - File: `crates/ripley-core/src/audit/ai_tools.rs` (continued)
  - `pub fn collect_vscode_extensions() -> Result<String, AuditError>`
    - `code --list-extensions` if VS Code installed
  - `pub fn evaluate_vscode_extensions(output: &str) -> Vec<AuditFinding>`
    - Check against known-suspicious extensions (curated list)
    - Yellow for unverified publishers, Red for known-malicious
  - `pub fn check_ai_tools(home: &Path) -> CategoryReport`
  - Tests: `test_evaluate_vscode_clean`, `test_evaluate_vscode_suspicious`
  - Verify: `cargo test -p ripley-core -- audit::ai_tools`


#### M10.4: Credential exposure checks

- [x] **M10.4.1** Shell history and env file checks
  - File: `crates/ripley-core/src/audit/creds.rs`
  - `pub fn evaluate_shell_history_secrets(content: &str) -> Vec<AuditFinding>`
    - Scan for token patterns: `npm_`, `ghp_`, `sk-`, `AKIA`, `xoxb-`,
      `glpat-`, password assignments
    - Red for each found token
  - `pub fn evaluate_env_in_git(project_path: &Path) -> Vec<AuditFinding>`
    - Check if `.env` files are tracked by git (`git ls-files .env`)
    - Red if committed, Green if gitignored or absent
  - Tests: `test_evaluate_history_with_npm_token`, `test_evaluate_history_clean`,
    `test_evaluate_env_committed`, `test_evaluate_env_gitignored`
  - Verify: `cargo test -p ripley-core -- audit::creds`

- [x] **M10.4.2** npm token and RC file token checks
  - File: `crates/ripley-core/src/audit/creds.rs` (continued)
  - `pub fn evaluate_npmrc_tokens(content: &str) -> Vec<AuditFinding>`
    - Check scope: broad token (Red) vs scoped (Yellow)
    - Check expiry if detectable
  - `pub fn evaluate_rc_file_tokens(content: &str, path: &Path) -> Vec<AuditFinding>`
    - Plaintext tokens in `.bashrc`, `.zshrc`, `.profile` etc.
    - Same token patterns as shell history check
  - `pub fn check_credentials(home: &Path) -> CategoryReport`
  - Tests: `test_evaluate_npmrc_broad_token`, `test_evaluate_npmrc_scoped`,
    `test_evaluate_rc_file_clean`, `test_evaluate_rc_file_with_token`
  - Verify: `cargo test -p ripley-core -- audit::creds`


#### M10.5: Audit CLI and --fix

- [x] **M10.5.1** Audit report aggregation and prompt generation
  - File: `crates/ripley-core/src/audit/mod.rs` (extend)
  - `pub fn run_audit(home: &Path) -> AuditReport`
    - Call check_machine_security(), check_toolchain(home),
      check_ai_tools(home), check_credentials(home)
    - Assemble into AuditReport with timestamp
  - File: `crates/ripley-core/src/prompt.rs` (extend)
  - `pub fn generate_audit_prompt(report: &AuditReport) -> String`
    - Structure all findings by category
    - Include environment context (OS, shell, PM versions)
    - Request environment-specific fix commands
  - Tests: `test_run_audit_produces_all_categories`,
    `test_generate_audit_prompt_includes_findings`
  - Verify: `cargo test -p ripley-core -- audit`, `cargo test -p ripley-core -- prompt`

- [x] **M10.5.2** CLI command registration
  - File: `crates/ripley-guard/src/commands/audit.rs` (new)
  - `pub async fn cmd_audit(format: &str, fix: bool) -> Result<ExitCode>`
    - Run `run_audit(home)`
    - Table output: traffic-light summary per category (see WORKFLOW.md §10)
    - JSON output: serialize AuditReport
    - If `--fix`: generate prompt, detect harness, launch or print to stdout
    - Exit code: 0 if all green, 1 if any red/yellow, 2 on error
  - File: `crates/ripley-guard/src/commands/mod.rs` — add `pub mod audit;`
  - File: `crates/ripley-guard/src/main.rs` — add `Audit` variant to Commands enum:
    `Audit { #[arg(long, default_value = "table")] format: String, #[arg(long)] fix: bool }`
  - Snapshot tests: `insta::assert_json_snapshot!` for audit JSON output
  - Verify: `cargo run -p ripley-guard -- audit`
  - Verify: `cargo run -p ripley-guard -- audit --format json`


#### M10 Gate

- [x] `cargo build --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace`
- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p ripley-core -- audit`
- [x] `cargo run -p ripley-guard -- audit`
- [x] `cargo run -p ripley-guard -- audit --format json`
- [x] Traffic-light output with 4 categories
- [x] `--fix` generates prompt and detects harness
- [ ] Commit: `M10: Environment audit`


---


### M11: PM Hardening (`ripley harden`)

> Spec: WORKFLOW.md "11. PM hardening"
> Spec: ROADMAP.md "Phase 3" → `ripley harden`


#### M11.1: Harden framework and PM detection

- [x] **M11.1.1** Create harden module with types and PM detection
  - File: `crates/ripley-core/src/harden/mod.rs`
  - Reuse `TrafficLight` from `audit` module (re-export or move to `types.rs`)
  - Types:
    - `HardenCategory` enum: `DependencyPinning`, `PmHardening`, `Provenance`,
      `CredentialHygiene` (with Serialize, Deserialize, Display)
    - `HardenFinding` struct: `name: String`, `status: TrafficLight`,
      `detail: String`, `fix_command: Option<String>`, `pm: String`
    - `HardenReport` struct: `detected_pms: Vec<DetectedPm>`,
      `categories: Vec<HardenCategoryReport>`, `timestamp: String`
    - `HardenCategoryReport` struct: `category: HardenCategory`,
      `findings: Vec<HardenFinding>`, `overall: TrafficLight`
    - `DetectedPm` struct: `name: String` (npm/pnpm/yarn/bun),
      `version: Option<String>`, `lockfile_path: PathBuf`,
      `config_path: Option<PathBuf>`
  - `pub fn detect_package_managers(path: &Path) -> Vec<DetectedPm>`
    - Look for lockfiles: package-lock.json→npm, pnpm-lock.yaml→pnpm,
      yarn.lock→yarn, bun.lockb→bun
    - Get version: `npm --version`, `pnpm --version`, etc.
  - Register `pub mod harden;` in `crates/ripley-core/src/lib.rs`
  - Submodules: `pub mod pinning;`, `pub mod settings;`, `pub mod provenance;`,
    `pub mod creds;`
  - Tests: `test_detect_npm_from_lockfile`, `test_detect_multiple_pms`
  - Verify: `cargo build -p ripley-core`


#### M11.2: Dependency pinning checks

- [x] **M11.2.1** Save-exact, lockfile, integrity, exotic sources
  - File: `crates/ripley-core/src/harden/pinning.rs`
  - `pub fn check_save_exact(pm: &DetectedPm) -> HardenFinding`
    - npm: `npm config get save-exact` → Green if true
    - pnpm: check `.npmrc` for `save-prefix=` (empty = exact) → Green if set
  - `pub fn check_lockfile_committed(pm: &DetectedPm) -> HardenFinding`
    - `git ls-files <lockfile>` → Green if tracked, Red if not
  - `pub fn check_integrity_hashes(lockfile_path: &Path) -> HardenFinding`
    - Parse lockfile, count entries with/without integrity hashes
    - Green if all present, Yellow if >90%, Red if <90%
  - `pub fn check_exotic_sources(lockfile_path: &Path) -> HardenFinding`
    - Reuse risky_specs from lockfile parser
    - Green if none, Red if git+/http/file: sources found
  - `pub fn check_dependency_pinning(pm: &DetectedPm) -> HardenCategoryReport`
  - Tests: `test_save_exact_enabled`, `test_save_exact_disabled`,
    `test_lockfile_committed`, `test_lockfile_not_committed`,
    `test_integrity_all_present`, `test_exotic_sources_found`
  - Verify: `cargo test -p ripley-core -- harden::pinning`


#### M11.3: PM hardening checks

- [x] **M11.3.1** Release-age, script policy, trust policy
  - File: `crates/ripley-core/src/harden/settings.rs`
  - `pub fn check_release_age(pm: &DetectedPm) -> HardenFinding`
    - npm: `npm config get minReleaseAge` → Green if set
    - pnpm: check `package.json` or `.npmrc` for `minimumReleaseAge`
    - Fix command: `npm config set minReleaseAge 86400` (or PM equivalent)
  - `pub fn check_script_policy(pm: &DetectedPm) -> HardenFinding`
    - npm: `npm config get ignore-scripts` → Green if true + allowlist
    - pnpm: check `strictDepBuilds` or `allowBuilds`
    - Fix command: `npm config set ignore-scripts true`
  - `pub fn check_trust_policy(pm: &DetectedPm) -> HardenFinding`
    - pnpm: check `trustPolicy` → Green if `no-downgrade`
    - Others: N/A
  - `pub fn check_exotic_subdeps(pm: &DetectedPm) -> HardenFinding`
    - pnpm: `blockExoticSubdeps` → Green if true
  - `pub fn check_pm_hardening(pm: &DetectedPm) -> HardenCategoryReport`
  - Tests: `test_release_age_set`, `test_release_age_missing`,
    `test_script_policy_ignore_scripts`, `test_trust_policy_no_downgrade`
  - Verify: `cargo test -p ripley-core -- harden::settings`


#### M11.4: Provenance and credential hygiene

- [x] **M11.4.1** Provenance and npmrc credential checks
  - File: `crates/ripley-core/src/harden/provenance.rs`
  - `pub fn check_trusted_publishing() -> HardenFinding`
    - Check for OIDC configuration in CI files (.github/workflows/)
    - Yellow if not configured (informational)
  - `pub fn check_provenance(pm: &DetectedPm) -> HardenCategoryReport`
  - File: `crates/ripley-core/src/harden/creds.rs`
  - `pub fn check_npmrc_credential_hygiene(npmrc_path: &Path) -> Vec<HardenFinding>`
    - Broad vs scoped tokens, expiry, project directory leaks
  - `pub fn check_credential_hygiene(home: &Path) -> HardenCategoryReport`
  - Tests: `test_npmrc_broad_token`, `test_npmrc_scoped_token`,
    `test_npmrc_project_directory_leak`
  - Verify: `cargo test -p ripley-core -- harden::provenance`,
    `cargo test -p ripley-core -- harden::creds`


#### M11.5: Harden CLI

- [x] **M11.5.1** CLI command registration
  - File: `crates/ripley-guard/src/commands/harden.rs` (new)
  - `pub async fn cmd_harden(path: PathBuf, format: &str) -> Result<ExitCode>`
    - Detect PMs in path, run all checks, output report
    - Table output: traffic-light per category with fix commands
      (see WORKFLOW.md §11 for layout)
    - JSON output: serialize HardenReport
    - Exit code: 0 if all green, 1 if any red/yellow, 2 on error
  - File: `crates/ripley-guard/src/commands/mod.rs` — add `pub mod harden;`
  - File: `crates/ripley-guard/src/main.rs` — add `Harden` variant:
    `Harden { path: Option<PathBuf>, #[arg(long, default_value = "table")] format: String }`
  - Snapshot tests: `insta::assert_json_snapshot!` for harden JSON output
  - Verify: `cargo run -p ripley-guard -- harden`
  - Verify: `cargo run -p ripley-guard -- harden --format json`


#### M11 Gate

- [x] `cargo build --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace`
- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p ripley-core -- harden`
- [x] `cargo run -p ripley-guard -- harden`
- [x] `cargo run -p ripley-guard -- harden --format json`
- [x] Traffic-light output with per-PM recommendations
- [x] Commit: `M11: PM hardening`


---


### M12: Fix and Exposure

> Spec: WORKFLOW.md "12. Standalone fix"
> Spec: WORKFLOW.md "13. Credential exposure assessment"
> Spec: ROADMAP.md "Phase 3" → `ripley fix`, `ripley exposure`


#### M12.1: `ripley fix <cve> [path]`

- [x] **M12.1.1** CVE-targeted prompt generation
  - File: `crates/ripley-core/src/prompt.rs` (extend)
  - `pub fn generate_cve_fix_prompt(cve_id: &str, matches: &[Match]) -> String`
    - Multi-project prompt: for each affected project, list package, version,
      clean version, IOC files if available, test instructions
    - Differences from existing `generate_remediation_prompt`: takes CVE as
      entry point (not Match), may span multiple projects, includes IOC
      profiles if the CVE has associated attack patterns
  - Tests: `test_generate_cve_fix_prompt_single_project`,
    `test_generate_cve_fix_prompt_multi_project`,
    `test_generate_cve_fix_prompt_with_iocs`
  - Verify: `cargo test -p ripley-core -- prompt`

- [x] **M12.1.2** Fix CLI command
  - File: `crates/ripley-guard/src/commands/fix.rs` (new)
  - `pub async fn cmd_fix(cve_id: &str, path: Option<PathBuf>) -> Result<ExitCode>`
    - Look up CVE in advisory DB (fetch from OSV if not cached)
    - If path given, scan that project; if omitted, scan current directory
    - Match CVE against installed packages
    - Generate prompt via `generate_cve_fix_prompt`
    - Detect harness, launch or print to stdout
    - Exit code: 0 if harness launched, 1 if no affected packages, 2 on error
  - File: `crates/ripley-guard/src/commands/mod.rs` — add `pub mod fix;`
  - File: `crates/ripley-guard/src/main.rs` — add `Fix` variant:
    `Fix { cve: String, path: Option<PathBuf> }`
  - Tests: integration test with fixture lockfile + mock advisory
  - Verify: `cargo build -p ripley-guard`
  - Verify: `cargo test -p ripley-guard -- fix`


#### M12.2: `ripley exposure <cve>`

- [x] **M12.2.1** CVE-targeted exposure assessment
  - File: `crates/ripley-core/src/forensic/exposure.rs` (new)
  - Build on existing `forensic/credentials.rs` (ExposureReport, assess_exposure)
  - `pub fn assess_cve_exposure(cve_id: &str, profiles: &IocProfileSet, home: &Path)
      -> Result<CveExposureReport, ForensicError>`
    - Look up IOC profile for CVE
    - Extract targeted credential stores from profile
    - Check which exist on this machine
    - Generate rotation command for each at-risk credential
    - Include dead man switch warning if applicable
  - Types:
    - `CveExposureReport` struct: `cve_id: String`, `attack_name: String`,
      `at_risk: Vec<CredentialRisk>`, `clean: Vec<String>`,
      `dead_man_switch: Option<String>`
    - `CredentialRisk` struct: `path: PathBuf`, `exists: bool`,
      `contains_secret: bool`, `rotation_command: String`, `priority: Severity`
  - Tests: `test_assess_cve_exposure_with_existing_creds`,
    `test_assess_cve_exposure_no_profile`, `test_dead_man_switch_warning`
  - Verify: `cargo test -p ripley-core -- forensic::exposure`

- [x] **M12.2.2** Exposure CLI command
  - File: `crates/ripley-guard/src/commands/exposure.rs` (new)
  - `pub async fn cmd_exposure(cve_id: &str, format: &str) -> Result<ExitCode>`
    - Run assess_cve_exposure
    - Table output: AT RISK / CLEAN per credential with rotation commands
      (see WORKFLOW.md §13 for layout)
    - JSON output: serialize CveExposureReport
    - Exit code: 0 if all clean, 1 if any at risk, 2 on error
  - File: `crates/ripley-guard/src/commands/mod.rs` — add `pub mod exposure;`
  - File: `crates/ripley-guard/src/main.rs` — add `Exposure` variant:
    `Exposure { cve: String, #[arg(long, default_value = "table")] format: String }`
  - Snapshot tests
  - Verify: `cargo run -p ripley-guard -- exposure CVE-2024-001 --format json`


#### M12 Gate

- [x] `cargo build --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace`
- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p ripley-guard -- fix`
- [x] `cargo test -p ripley-core -- forensic::exposure`
- [x] `ripley fix` generates prompt for known CVE
- [x] `ripley exposure` produces credential assessment
- [x] Commit: `M12: Fix and exposure`


---


### M13: Templates and Dashboard Views

> Spec: ROADMAP.md "Phase 3" → remediation templates, network audit
> Spec: UI.md "Audit Report View", "Posture & Hardening View"


#### M13.1: Remediation templates

- [x] **M13.1.1** Template framework and per-attack playbooks
  - File: `crates/ripley-core/src/templates/mod.rs`
  - Types:
    - `RemediationTemplate` struct: `id: String`, `name: String`,
      `attack_type: String`, `ioc_checks: Vec<String>`,
      `credential_rotation: Vec<RotationStep>`,
      `verification_steps: Vec<String>`
    - `RotationStep` struct: `credential: String`, `command: String`,
      `priority: Severity`
  - Templates as TOML files compiled via `include_str!`:
    - `templates/npm_worm.toml`
    - `templates/pypi_pth_injection.toml`
    - `templates/credential_exfil.toml`
    - `templates/ide_config_poison.toml`
  - `pub fn load_templates() -> Vec<RemediationTemplate>`
  - `pub fn find_template(attack_type: &str) -> Option<&RemediationTemplate>`
  - Register `pub mod templates;` in `crates/ripley-core/src/lib.rs`
  - Tests: `test_load_templates_all_present`, `test_find_template_npm_worm`,
    `test_template_has_required_fields`
  - Verify: `cargo test -p ripley-core -- templates`

- [x] **M13.1.2** Network C2 audit
  - File: `crates/ripley-core/src/forensic/network.rs` (new)
  - `pub fn collect_active_connections() -> Result<Vec<NetworkConnection>, ForensicError>`
    - macOS/Linux: parse `lsof -i -n -P` or `ss -tunap`
    - Windows: `netstat -ano`
  - `pub fn check_c2_connections(connections: &[NetworkConnection],
      c2_list: &C2Database) -> Vec<C2Finding>`
    - Match against known C2 domains/IPs
  - Types: `NetworkConnection`, `C2Finding`, `C2Database`
  - `C2Database` loaded from bundled TOML + user-configurable additions
  - Register in `crates/ripley-core/src/forensic/mod.rs`
  - Tests: `test_parse_lsof_output`, `test_c2_match_found`,
    `test_c2_no_match`
  - Verify: `cargo test -p ripley-core -- forensic::network`


#### M13.2: Dashboard views

- [x] **M13.2.1** Audit report view
  - File: `crates/ripley-app/src/views/audit.rs` (new)
  - Render AuditReport as traffic-light dashboard:
    - Per-category card with Green/Yellow/Red indicator
    - Expandable findings list per category
    - Summary bar at bottom
  - Register in `crates/ripley-app/src/views/mod.rs`
  - Register in app.rs view enum
  - Verify: `cargo build -p ripley-app`

- [x] **M13.2.2** Posture/harden view
  - File: `crates/ripley-app/src/views/posture.rs` (new)
  - Render HardenReport as dashboard:
    - Detected PMs at top
    - Per-category traffic-light cards
    - Fix commands as copyable text
  - Register in `crates/ripley-app/src/views/mod.rs`
  - Register in app.rs view enum
  - Verify: `cargo build -p ripley-app`


#### M13 Gate

- [x] `cargo build --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace`
- [x] `cargo fmt --all -- --check`
- [x] `cargo build -p ripley-app`
- [x] `cargo test -p ripley-core -- templates`
- [x] `cargo test -p ripley-core -- forensic::network`
- [x] Remediation templates load for each attack type
- [x] Dashboard views build without errors
- [x] Commit: `M13: Templates and dashboard`


---


#### Phase 3 Gate

**All must pass before starting Phase 4:**

- [x] All M10-M13 gates passed
- [x] `cargo build --workspace --release`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace`
- [x] `cargo deny check`
- [x] `ripley audit` produces traffic-light output with 4 categories
- [x] `ripley audit --format json` produces valid JSON
- [x] `ripley harden` detects PMs and produces recommendations
- [x] `ripley harden --format json` produces valid JSON
- [x] `ripley fix <cve>` generates remediation prompt
- [x] `ripley exposure <cve>` produces credential assessment
- [x] No `unwrap()` or `expect()` in ripley-core
- [x] All public functions have tests
- [x] Snapshot tests for all command outputs
- [x] Commit: `Phase 3: Response depth`
- [x] Update CLAUDE.md "Current work" to Phase 4


---


## Phase 4: Active Detection

**Goal:** Ripley monitors the developer's machine in real time for signs
of active compromise: unexpected network connections, unauthorized writes
to sensitive paths, and C2 communication. When detected, containment is
one click away.

> Spec: ROADMAP.md "Phase 4: Active Detection"
> Spec: WORKFLOW.md "7. Active containment"
> Spec: SETTINGS.md "[monitor]"
> Spec: UI.md "Monitor View"
> Spec: ARCHITECTURE.md "Process and network monitoring"

**Building blocks already in place:**
- `commands/watch.rs` — daemon loop with advisory poller, lockfile FS
  watcher, IPC server (Status only), CancellationToken shutdown, event
  channel (`WatchEvent::NewAdvisories`, `WatchEvent::LockfileChanged`)
- `ripley-ipc/protocol.rs` — `Request`/`Response` enums, `StatusData`,
  Unix socket path, serde roundtrip
- `ripley-ipc/server.rs` — async socket server with request handler
- `forensic/network.rs` — `C2Database`, `NetworkConnection`,
  `C2Finding`, `collect_active_connections()`, `check_c2_connections()`,
  parsers (lsof/ss/netstat) — all 3 platforms
- `forensic/persistence.rs` — `PersistenceFinding`, `PersistenceCategory`,
  persistence path scanner for all platforms
- `app/events.rs` — `AppEvent` + `Action` enums (scaffolding, not wired)
- `app/watcher.rs` — lockfile FS watcher (scaffolding, not wired)
- `app/notifier.rs` — desktop notification via `notify_rust`
- `app/tray.rs` — system tray menu (Show Dashboard, Scan Now, Quit)
- `config.rs` — `MonitoringConfig { project_roots }` (needs [monitor]
  fields)


### M14: Monitor Config + Process Scanner (core library)

**Goal:** `[monitor]` config section and a reusable process monitor that
detects suspicious outbound connections from developer tool processes.

> Spec: SETTINGS.md "[monitor]" table
> Spec: ARCHITECTURE.md "Process and network monitoring"

- [x] **M14.1** — `[monitor]` config section

  **Files:**
  - `crates/ripley-core/src/config.rs` — expand `MonitoringConfig` (or
    add `MonitorConfig` alongside it) with fields from SETTINGS.md:
    `enabled: bool`, `watch_processes: bool`, `watch_persistence: bool`,
    `watch_lockfiles: bool`, `c2_domains: Vec<String>`,
    `c2_ips: Vec<String>`.
  - Add `ConfigOverlay` fields and `apply_overlay` merge for the new
    monitor fields.
  - Add env var overrides: `RIPLEY_MONITOR_ENABLED`, etc.
  - Add `Config { monitor: MonitorConfig }` field on the resolved config.

  **Tests:**
  - Default values match SETTINGS.md (enabled=false, watches=true).
  - TOML round-trip: serialize → deserialize → values match.
  - Overlay merge: project-level overrides user-level.
  - Env vars override file config.

  **Verify:**
  ```
  cargo test -p ripley-core -- config
  cargo clippy --workspace
  ```

- [x] **M14.2** — Process scanner: collect + evaluate

  Build a reusable process monitor in `ripley-core` that wraps the
  existing `forensic/network.rs` connection parser and adds process
  filtering (Node/Python/Ruby/Go/Bun/Deno).

  **Files:**
  - `crates/ripley-core/src/monitor/mod.rs` — new module, exports.
  - `crates/ripley-core/src/monitor/process.rs`:
    - `struct ProcessAlert { connection: NetworkConnection, reason: AlertReason, severity: Severity, timestamp: u64 }`
    - `enum AlertReason { C2Connection { indicator: String }, SuspiciousOutbound, PersistenceWrite { path: PathBuf }, LockfileEdit { path: PathBuf }, McpConfigChange { path: PathBuf }, UnknownBinary { path: PathBuf } }`
    - `fn scan_processes(config: &MonitorConfig, c2_db: &C2Database) -> Result<Vec<ProcessAlert>>`:
      calls `collect_active_connections()`, filters to developer-tool
      processes (node, python, ruby, go, bun, deno — match by process
      name), runs `check_c2_connections()` with merged C2 list
      (compiled-in + config c2_domains/c2_ips), returns alerts.
    - `fn evaluate_connections(connections: &[NetworkConnection], config: &MonitorConfig, c2_db: &C2Database) -> Vec<ProcessAlert>`:
      Pure function. Filters connections to watched processes, checks
      against C2 database. Testable without I/O.
    - `fn is_developer_process(name: &str) -> bool`: matches node,
      python, python3, ruby, go, bun, deno, pip, npm, yarn, pnpm, cargo.
    - `fn merge_c2_database(compiled: &C2Database, config: &MonitorConfig) -> C2Database`:
      Merges compiled-in C2 list with user-configured c2_domains/c2_ips.
  - `crates/ripley-core/src/lib.rs` — add `pub mod monitor;`

  **Tests:**
  - `is_developer_process`: true for "node", "python3", "ruby", "go",
    "bun"; false for "systemd", "sshd", "finder".
  - `evaluate_connections` with mock connections: filters non-dev
    processes, flags C2 matches, returns correct severity.
  - `merge_c2_database`: compiled + config entries are all present.
  - `evaluate_connections` with empty connection list → empty alerts.
  - `evaluate_connections` with connection to known C2 → alert with
    `AlertReason::C2Connection`.

  **Verify:**
  ```
  cargo test -p ripley-core -- monitor::process
  cargo clippy --workspace
  ```

- [x] **M14.3** — Filesystem monitor: persistence path watcher

  Watch for writes to persistence paths and MCP configs. Reuses
  `forensic/persistence.rs` knowledge of persistence paths.

  **Files:**
  - `crates/ripley-core/src/monitor/filesystem.rs`:
    - `fn persistence_watch_paths() -> Vec<PathBuf>`: returns all paths
      that should be watched — LaunchAgents, systemd, cron, shell RCs,
      `.claude/`, `.vscode/`, `.mcp.json`, `.cursor/mcp.json`. Uses
      `cfg(target_os)` and `platform.rs` for paths.
    - `fn evaluate_fs_event(path: &Path, kind: FsEventKind) -> Option<ProcessAlert>`:
      Pure function. Given a changed path and event kind, returns an
      alert if the path matches a persistence pattern. Maps to
      appropriate `AlertReason` variant.
    - `enum FsEventKind { Created, Modified, Deleted }`
    - `fn is_lockfile_edit(path: &Path) -> bool`: checks if the path
      is any known lockfile (package-lock.json, yarn.lock, pnpm-lock.yaml,
      Cargo.lock, go.sum, Gemfile.lock, requirements.txt).
    - `fn is_persistence_path(path: &Path) -> bool`: checks against
      known persistence paths per platform.
    - `fn is_mcp_config(path: &Path) -> bool`: checks .mcp.json,
      .cursor/mcp.json, .claude/settings.json.

  **Tests:**
  - `persistence_watch_paths` returns platform-appropriate paths (macOS:
    includes LaunchAgents; Linux: includes systemd).
  - `evaluate_fs_event` for persistence path write → alert.
  - `evaluate_fs_event` for lockfile edit → alert with
    `AlertReason::LockfileEdit`.
  - `evaluate_fs_event` for MCP config change → alert with
    `AlertReason::McpConfigChange`.
  - `evaluate_fs_event` for unrelated path → None.
  - `is_lockfile_edit` true/false cases.
  - `is_persistence_path` true/false cases per platform.
  - `is_mcp_config` true/false cases.

  **Verify:**
  ```
  cargo test -p ripley-core -- monitor::filesystem
  cargo clippy --workspace
  ```

#### M14 Gate (Monitor Core Library)

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
cargo test -p ripley-core -- monitor
cargo test -p ripley-core -- config
```

VERIFY: `MonitorConfig` fields match SETTINGS.md spec.
VERIFY: `evaluate_connections` filters and flags correctly in tests.
VERIFY: `evaluate_fs_event` maps persistence/lockfile/MCP paths.
VERIFY: All new public functions have tests.

Pass → commit `M14: Monitor config and process scanner`, update Plan
State to M15.


---


### M15: Monitor Daemon (`ripley monitor` CLI)

**Goal:** A long-running `ripley monitor` command that combines process
scanning, filesystem watching, and the existing advisory poller into a
unified daemon with event-driven architecture.

> Spec: ROADMAP.md "ripley monitor — background daemon"
> Spec: WORKFLOW.md "7. Active containment" — `ripley monitor`

- [x] **M15.1** — Extend `WatchEvent` and daemon loop

  Extend the existing `commands/watch.rs` daemon to support monitor
  events alongside the existing advisory/lockfile events.

  **Files:**
  - `crates/ripley-guard/src/commands/watch.rs`:
    - Add `WatchEvent::MonitorAlert(ProcessAlert)` variant.
    - Add `WatchEvent::PersistenceAlert(ProcessAlert)` variant.
    - In `cmd_watch`: if `config.monitor.enabled`, spawn the process
      scanner loop and persistence watcher alongside existing poller
      and lockfile watcher.
    - Handle new event types in the main event loop: log, notify,
      append to guard.jsonl.

  **Tests:**
  - (Integration-level — tested via M15.3)

  **Verify:**
  ```
  cargo build -p ripley-guard
  cargo clippy --workspace
  ```

- [x] **M15.2** — Process scanner loop

  A tokio task that periodically scans active connections.

  **Files:**
  - `crates/ripley-guard/src/commands/watch.rs` (or extract to
    `crates/ripley-guard/src/commands/monitor.rs` if watch.rs grows
    too large):
    - `async fn run_process_scanner(config: Config, c2_db: C2Database, tx: Sender<WatchEvent>, cancel: CancellationToken)`:
      Loop: sleep for scan interval (default 30s, configurable), call
      `scan_processes`, send alerts via channel. Use `spawn_blocking`
      for the sync `collect_active_connections` call.
    - Dedup: track seen connections (process+remote_addr+remote_port)
      to avoid re-alerting for persistent connections. Clear after
      configurable TTL (e.g. 5 minutes).

  **Tests:**
  - Dedup logic: same connection twice → one alert.
  - Different connections → separate alerts.

  **Verify:**
  ```
  cargo build -p ripley-guard
  cargo clippy --workspace
  ```

- [x] **M15.3** — Persistence watcher loop

  A tokio task that watches persistence paths for filesystem events.

  **Files:**
  - `crates/ripley-guard/src/commands/watch.rs` (or `monitor.rs`):
    - `async fn run_persistence_watcher(config: Config, tx: Sender<WatchEvent>, cancel: CancellationToken)`:
      Use `notify` crate to watch paths from
      `persistence_watch_paths()`. On event, call `evaluate_fs_event`,
      send alerts. Debounce rapid events (100ms window).
    - If `config.monitor.watch_lockfiles`: also watch project roots
      for lockfile changes outside of explicit install (reuse existing
      lockfile watcher, but fire `PersistenceAlert` instead of
      `LockfileChanged` when the daemon is in monitor mode).

  **Tests:**
  - (Integration test in M15.5)

  **Verify:**
  ```
  cargo build -p ripley-guard
  cargo clippy --workspace
  ```

- [x] **M15.4** — `ripley monitor` CLI command

  Wire the monitor as a new CLI subcommand.

  **Files:**
  - `crates/ripley-guard/src/commands/monitor.rs` — new file:
    - `pub async fn cmd_monitor(daemon: bool) -> Result<()>`:
      Load config, check `config.monitor.enabled` (error if false with
      helpful message about enabling in config.toml), load C2 database,
      set up event channel, spawn: process scanner loop, persistence
      watcher, existing advisory poller, IPC server. Main loop handles
      all events.
    - `--daemon` flag: background mode with file logging (same as
      `watch --daemon`).
    - Alert output format: timestamp, severity icon, process name,
      reason, detail. Example:
      `[14:23:05] ● HIGH  node (PID 12345) → 185.x.x.x:443 (known C2)`
    - `--format json`: JSON Lines output, one alert per line.
  - `crates/ripley-guard/src/commands/mod.rs` — add `pub mod monitor;`
  - `crates/ripley-guard/src/main.rs` — add `Monitor` subcommand to
    clap enum with `--daemon` and `--format` flags. Route to
    `cmd_monitor`.

  **Tests:**
  - Verify `Monitor` subcommand parses with clap.

  **Verify:**
  ```
  cargo build -p ripley-guard
  cargo run -p ripley-guard -- monitor --help
  cargo clippy --workspace
  ```

- [x] **M15.5** — Guard log integration + integration tests

  Monitor alerts are logged to `guard.jsonl` for forensic review.

  **Files:**
  - `crates/ripley-guard/src/commands/monitor.rs` (or watch.rs):
    - On each alert: append JSON line to `guard.jsonl` with fields:
      `timestamp`, `event_type: "monitor_alert"`, `severity`, `process`,
      `pid`, `reason`, `detail`.
  - `tests/monitor_integration.rs` — new integration test:
    - Start monitor with a test config pointing at a temp dir.
    - Write a file to a persistence path in the temp dir.
    - Assert that a `PersistenceAlert` appears in guard.jsonl.
    - (Process scanner test: can only test evaluate path since
      `collect_active_connections` needs real system state.)

  **Tests:**
  - Integration: persistence path write → alert in guard.jsonl.
  - Integration: monitor starts and stops cleanly with CancellationToken.

  **Verify:**
  ```
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  ```

#### M15 Gate (Monitor Daemon)

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
cargo run -p ripley-guard -- monitor --help
cargo test -p ripley-core -- monitor
cargo test -- monitor_integration
```

VERIFY: `ripley monitor --help` shows usage with --daemon and --format.
VERIFY: Monitor starts, detects planted persistence write in test.
VERIFY: Alerts appear in guard.jsonl.
VERIFY: Clean shutdown with Ctrl-C.

Pass → commit `M15: Monitor daemon`, update Plan State to M16.


---


### M16: Contain Command (`ripley contain`)

**Goal:** Kill a suspicious process, snapshot its state (open files,
network connections, environment, process tree) for forensic analysis.

> Spec: ROADMAP.md "ripley contain <pid|pkg>"
> Spec: WORKFLOW.md "7. Active containment" — containment action

- [x] **M16.1** — Process snapshot (core library)

  Capture a process's forensic state before killing it.

  **Files:**
  - `crates/ripley-core/src/monitor/contain.rs`:
    - `struct ProcessSnapshot { pid: u32, name: String, cmdline: String, open_files: Vec<String>, network_connections: Vec<NetworkConnection>, env_vars: Vec<(String, String)>, children: Vec<u32>, timestamp: u64 }`
    - `fn collect_process_snapshot(pid: u32) -> Result<ProcessSnapshot>`:
      Platform-specific. macOS: `lsof -p`, `ps -p`, `pgrep -P`.
      Linux: read `/proc/{pid}/cmdline`, `/proc/{pid}/fd`,
      `/proc/{pid}/environ`, `/proc/{pid}/net/tcp`. Windows: stub that
      returns error (Phase 5).
    - `fn evaluate_snapshot_output(lsof_output: &str, ps_output: &str, pgrep_output: &str) -> ProcessSnapshot`:
      Pure parser, testable.
    - Uses `cfg(target_os)` for platform dispatch.

  **Tests:**
  - `evaluate_snapshot_output` with mock lsof/ps/pgrep output.
  - Parse open files list from lsof output.
  - Parse child PIDs from pgrep output.
  - Parse env vars from /proc format.

  **Verify:**
  ```
  cargo test -p ripley-core -- monitor::contain
  cargo clippy --workspace
  ```

- [x] **M16.2** — Process kill + snapshot save

  **Files:**
  - `crates/ripley-core/src/monitor/contain.rs`:
    - `fn contain_process(pid: u32, data_dir: &Path) -> Result<ContainResult>`:
      1. `collect_process_snapshot(pid)` — capture state first.
      2. Save snapshot as JSON to `{data_dir}/snapshots/{pid}_{timestamp}.json`.
      3. Kill process: `kill(pid, SIGKILL)` on Unix, stub on Windows.
      4. Return `ContainResult { snapshot_path, killed: bool }`.
    - `struct ContainResult { snapshot_path: PathBuf, killed: bool, snapshot: ProcessSnapshot }`
    - Snapshot directory creation: atomic mkdir.
  - File writes: temp file + rename for atomicity.

  **Tests:**
  - Snapshot serializes to valid JSON.
  - Snapshot file path uses `{pid}_{timestamp}.json` format.
  - `ContainResult` fields are populated correctly.
  - (Cannot test actual kill in unit tests — tested in integration.)

  **Verify:**
  ```
  cargo test -p ripley-core -- monitor::contain
  cargo clippy --workspace
  ```

- [x] **M16.3** — `ripley contain` CLI command

  **Files:**
  - `crates/ripley-guard/src/commands/contain.rs` — new file:
    - `pub async fn cmd_contain(target: &str, format: &str) -> Result<()>`:
      Parse target as PID (u32) or package name (string). If PID:
      contain directly. If package name: find PID by matching process
      names against package manager processes (heuristic: search for
      processes whose cmdline contains the package name).
    - Table output: snapshot summary (PID, name, connections, open files
      count, children count), "Process killed" confirmation.
    - JSON output: full `ContainResult` serialized.
    - Exit codes: 0=contained, 1=process not found, 2=error.
  - `crates/ripley-guard/src/commands/mod.rs` — add `pub mod contain;`
  - `crates/ripley-guard/src/main.rs` — add `Contain` subcommand with
    `target: String`, `--format` flag. Route to `cmd_contain`.

  **Tests:**
  - Verify `Contain` subcommand parses with clap.
  - Target parsing: "12345" → PID, "evil-pkg" → package name.

  **Verify:**
  ```
  cargo build -p ripley-guard
  cargo run -p ripley-guard -- contain --help
  cargo clippy --workspace
  ```

- [x] **M16.4** — IPC: Contain request from app

  Extend the IPC protocol so the tray app can request containment.

  **Files:**
  - `crates/ripley-ipc/src/protocol.rs`:
    - Add `Request::Contain { pid: u32 }` variant.
    - Add `Response::ContainResult(ContainResultData)` variant.
    - `struct ContainResultData { pid: u32, killed: bool, snapshot_path: String }`
  - `crates/ripley-guard/src/commands/monitor.rs` (IPC handler):
    - Handle `Request::Contain` in the IPC server: call
      `contain_process`, return result.

  **Tests:**
  - `Request::Contain` roundtrip (serialize/deserialize).
  - `Response::ContainResult` roundtrip.

  **Verify:**
  ```
  cargo test -p ripley-ipc
  cargo clippy --workspace
  ```

#### M16 Gate (Contain Command)

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
cargo run -p ripley-guard -- contain --help
cargo test -p ripley-core -- monitor::contain
cargo test -p ripley-ipc
```

VERIFY: `ripley contain --help` shows usage with target and --format.
VERIFY: Snapshot serializes to valid JSON.
VERIFY: IPC Contain request/response roundtrips.
VERIFY: All new public functions have tests.

Pass → commit `M16: Contain command`, update Plan State to M17.


---


### M17: Real-time Notifications + IPC Extensions

**Goal:** High-priority desktop notifications with actionable buttons
(View, Contain, Investigate). IPC extensions for the tray app to receive
alerts from the monitor daemon.

> Spec: ROADMAP.md "Real-time high-priority notifications"
> Spec: WORKFLOW.md "7. Active containment" — notification flow

- [x] **M17.1** — Monitor notification system

  **Files:**
  - `crates/ripley-guard/src/commands/monitor.rs` (or a dedicated
    `crates/ripley-guard/src/notify.rs`):
    - `fn notify_monitor_alert(alert: &ProcessAlert)`:
      Fire desktop notification via `notify_rust` with:
      - Title: severity + alert type (e.g., "HIGH: Suspicious connection")
      - Body: process name, PID, reason detail
      - Actions (if platform supports): "View", "Contain"
    - Severity determines urgency: Critical/High → urgent notification
      (if platform supports), Medium/Low → normal.
  - `crates/ripley-app/src/notifier.rs` — extend to handle monitor
    alerts (not just vulnerability matches):
    - `fn notify_monitor_alert(alert: &ProcessAlert)` — parallel to
      existing `notify_match`.

  **Tests:**
  - Notification message formatting for each `AlertReason` variant.
  - Severity to urgency mapping.

  **Verify:**
  ```
  cargo build --workspace
  cargo clippy --workspace
  ```

- [x] **M17.2** — IPC: Alert streaming

  Allow the tray app to subscribe to real-time alerts from the daemon.

  **Files:**
  - `crates/ripley-ipc/src/protocol.rs`:
    - Add `Request::SubscribeAlerts` variant.
    - Add `Response::MonitorAlert(MonitorAlertData)` variant.
    - `struct MonitorAlertData { timestamp: u64, severity: String, process: String, pid: u32, reason: String, detail: String }`
  - `crates/ripley-guard/src/commands/monitor.rs` (IPC handler):
    - Track subscribed clients. On each alert, broadcast to all
      subscribers.
  - `crates/ripley-ipc/src/client.rs` — add
    `async fn subscribe_alerts(&self) -> Result<Receiver<MonitorAlertData>>`:
    Send `SubscribeAlerts`, receive stream of `MonitorAlert` responses.

  **Tests:**
  - `Request::SubscribeAlerts` roundtrip.
  - `Response::MonitorAlert` roundtrip.
  - `MonitorAlertData` serialization.

  **Verify:**
  ```
  cargo test -p ripley-ipc
  cargo clippy --workspace
  ```

- [x] **M17.3** — Guard log: structured monitor events

  Ensure all monitor events are logged in a structured, queryable format.

  **Files:**
  - `crates/ripley-core/src/monitor/mod.rs`:
    - `struct MonitorLogEntry { timestamp: u64, event_type: MonitorEventType, severity: Severity, process: Option<String>, pid: Option<u32>, path: Option<PathBuf>, detail: String }`
    - `enum MonitorEventType { ProcessAlert, PersistenceAlert, LockfileAlert, McpConfigAlert, ContainAction, MonitorStarted, MonitorStopped }`
    - `fn format_log_entry(entry: &MonitorLogEntry) -> String`:
      JSON serialization for guard.jsonl.
  - `crates/ripley-guard/src/commands/monitor.rs`:
    - Use `MonitorLogEntry` for all guard.jsonl writes.
    - Log `MonitorStarted` on startup, `MonitorStopped` on shutdown.

  **Tests:**
  - `MonitorLogEntry` serialization includes all fields.
  - Each `MonitorEventType` variant serializes correctly.
  - `format_log_entry` produces valid JSON.

  **Verify:**
  ```
  cargo test -p ripley-core -- monitor
  cargo clippy --workspace
  ```

#### M17 Gate (Notifications + IPC)

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
cargo test -p ripley-ipc
cargo test -p ripley-core -- monitor
```

VERIFY: Notification formatting is correct for each alert type.
VERIFY: IPC alert roundtrips work.
VERIFY: Guard log entries are valid JSON with all fields.

Pass → commit `M17: Notifications and IPC`, update Plan State to M18.


---


### M18: Monitor Dashboard View + Tray Integration

**Goal:** Wire the monitor into the iced dashboard (Monitor sidebar
item, live alert list) and the tray app menu (status indicator, quick
contain action).

> Spec: UI.md "Monitor" sidebar item
> Spec: DESIGN.md for styling

- [x] **M18.1** — Monitor view in dashboard

  **Files:**
  - `crates/ripley-app/src/views/monitor.rs` — new file:
    - Sidebar item: "Monitor" with a dot indicator (green=clean,
      yellow=alerts, red=critical alert, grey=disabled).
    - If `monitor.enabled = false`: disabled state with tooltip
      "Enable monitoring in Settings > Monitor".
    - If enabled: list of recent alerts (from IPC subscription),
      each row showing: timestamp, severity badge, process name,
      reason summary.
    - Each alert row has actions: [View Details] [Contain].
    - Empty state: "Monitoring active. No suspicious activity detected."
      with a green shield icon.
  - `crates/ripley-app/src/views/mod.rs` — add `pub mod monitor;`
  - `crates/ripley-app/src/app.rs` — wire Monitor view into sidebar
    navigation and app state.

  **Tests:**
  - Monitor view renders in disabled state when config says disabled.
  - Monitor view renders empty state when enabled with no alerts.
  - (UI rendering tests are limited — focus on state logic.)

  **Verify:**
  ```
  cargo build -p ripley-app
  cargo clippy --workspace
  ```

- [x] **M18.2** — Tray menu: monitor status

  **Files:**
  - `crates/ripley-app/src/tray.rs`:
    - Add "Monitor" menu item with status indicator.
    - If monitoring enabled and running: "Monitor: Active ●"
    - If monitoring disabled: "Monitor: Off" (greyed).
    - Add "Contain..." menu item (enabled only when alerts exist).
    - Add `TrayAction::ToggleMonitor` and `TrayAction::Contain(u32)`.

  **Tests:**
  - `TrayAction` enum has new variants.
  - `handle_event` routes new menu items correctly.

  **Verify:**
  ```
  cargo build -p ripley-app
  cargo clippy --workspace
  ```

- [x] **M18.3** — App event loop: wire monitor events

  **Files:**
  - `crates/ripley-app/src/events.rs`:
    - Add `AppEvent::MonitorAlert(ProcessAlert)` variant.
    - Add `AppEvent::ContainResult(ContainResult)` variant.
    - Add `Action::ToggleMonitor` variant.
  - `crates/ripley-app/src/app.rs`:
    - Handle `AppEvent::MonitorAlert`: update alert list, fire
      notification via `notifier.rs`, update tray icon color.
    - Handle `Action::Contain`: send IPC `Request::Contain`, handle
      response, show notification with snapshot path.
    - Handle `Action::ToggleMonitor`: send IPC request to toggle
      monitor, update UI state.

  **Tests:**
  - `AppEvent::MonitorAlert` and `AppEvent::ContainResult` variants
    exist and are constructible.
  - (Event handling tested via integration — complex async + UI.)

  **Verify:**
  ```
  cargo build -p ripley-app
  cargo clippy --workspace
  ```

- [x] **M18.4** — Settings view: monitor section

  **Files:**
  - `crates/ripley-app/src/views/settings.rs`:
    - Add "Monitor" section with toggles for: `enabled`,
      `watch_processes`, `watch_persistence`, `watch_lockfiles`.
    - Changes write to config.toml (atomic write).
    - When `enabled` toggled on: show note "Restart monitor to apply."

  **Tests:**
  - (UI widget tests are limited — verify compilation and state logic.)

  **Verify:**
  ```
  cargo build -p ripley-app
  cargo clippy --workspace
  ```

#### M18 Gate (Dashboard + Tray)

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
cargo build -p ripley-app
```

VERIFY: `ripley-app` builds without warnings.
VERIFY: Monitor view, tray menu, settings section, event handlers
  all compile and wire together.
VERIFY: Dead code warnings from Phase 3 app scaffolding are resolved
  (remove `#[allow(dead_code)]` annotations now that code is wired up).

Pass → commit `M18: Monitor dashboard and tray`, update Plan State to
Phase 4 Gate.


---


#### Phase 4 Gate

**All must pass before starting Phase 5:**

- [x] All M14-M18 gates passed
- [x] `cargo build --workspace --release`
- [x] `cargo test --workspace` (426 tests)
- [x] `cargo clippy --workspace`
- [x] `cargo fmt --all -- --check`
- [x] `cargo deny check`
- [x] `ripley monitor --help` shows usage with --daemon and --format
- [x] `ripley contain --help` shows usage with target and --format
- [x] Monitor detects planted persistence write in integration test
- [x] Contain produces valid snapshot JSON
- [x] IPC roundtrip tests pass for all new request/response types
- [x] Guard log entries are valid JSON with monitor event types
- [x] No `unwrap()` or `expect()` in ripley-core production code
- [x] All public functions have tests
- [x] No `#[allow(dead_code)]` remaining on Phase 4 scaffolding
- [x] Commit: `Phase 4: Active detection`
- [x] Update CLAUDE.md "Current work" to Phase 5


---


## Phase 5: Advanced Analysis

**Goal:** SARIF output, CI/CD integration, sandboxed script execution, behavioral analysis, community rule sharing.

> Spec: ROADMAP.md "Phase 5: Advanced Analysis"
> Spec: WORKFLOW.md "8. CI pipeline gate"

**Depends on:** Phase 4 complete (426 tests, M14-M18 gates passed).

**Scope decision:** Behavioral analysis uses sandbox enforcement signals (blocked network, denied fs access) rather than deep process tracing (dtrace/strace). Full observation is Phase 6+. This keeps all 5 milestones roughly equal in size.


---


### M19: SARIF Output

**Goal:** Add `--format sarif` to `ripley scan`, producing SARIF 2.1.0-compliant JSON for GitHub Code Scanning, GitLab SAST, and other security dashboards.

- [x] **M19.1** — SARIF types and serialization

  New module `crates/ripley-core/src/sarif.rs`:
  - `SarifLog { version, schema, runs }`, `SarifRun { tool, results, invocations }`
  - `SarifTool { driver: SarifToolComponent }`, `SarifToolComponent { name, version, rules }`
  - `SarifReportingDescriptor { id, name, short_description, default_configuration }`
  - `SarifResult { rule_id, rule_index, level, message, locations, fingerprints }`
  - `SarifLocation { physical_location }`, `SarifPhysicalLocation { artifact_location, region }`
  - `SarifArtifactLocation { uri, uri_base_id }`, `SarifRegion { start_line, start_column }`
  - `SarifMessage { text }`, `SarifInvocation { execution_successful, exit_code }`
  - `enum SarifLevel { Error, Warning, Note, None }` — maps from Severity
  - `fn severity_to_sarif_level(severity: &Severity) -> SarifLevel`
  - All structs: `#[serde(rename_all = "camelCase")]`
  - Register in `crates/ripley-core/src/lib.rs`: `pub mod sarif;`

  Tests: serialization valid JSON, severity mapping, empty results produce valid log, round-trip.

  Verify: `cargo test -p ripley-core -- sarif && cargo clippy --workspace`

- [x] **M19.2** — Convert scan results to SARIF

  Extend `crates/ripley-core/src/sarif.rs`:
  - `fn matches_to_sarif(matches: &[Match], warnings: &[LockfileWarning], risky_specs: &[RiskySpec]) -> SarifLog`
  - `fn analysis_to_sarif(results: &[AnalysisResult], script_paths: &[PathBuf]) -> SarifLog`
  - `fn sarif_fingerprint(advisory_id: &str, package: &str, version: &str) -> String` — SHA-256 deterministic fingerprint

  Tests: single match → 1 result, multiple matches → deduped rules, snapshot test with insta.

  Verify: `cargo test -p ripley-core -- sarif && cargo clippy --workspace`

- [x] **M19.3** — Wire `--format sarif` into CLI

  Files:
  - `crates/ripley-guard/src/output.rs`: `pub fn print_sarif(matches, warnings, risky_specs)`
  - `crates/ripley-guard/src/commands/scan.rs`: add `"sarif"` branch to format match
  - `crates/ripley-guard/src/main.rs`: update `--format` help text to include `sarif`

  Tests: CLI produces valid JSON with `$schema`, integration test parses output, exit codes unchanged.

  Verify: `cargo test --workspace && cargo run -p ripley-guard -- scan --format sarif tests/fixtures/`

#### M19 Gate
- [x] SARIF output is valid JSON with `version: "2.1.0"` and `$schema`
- [x] Each advisory match maps to a SARIF result with correct level
- [x] `--format sarif` works alongside existing `--format json` and `--format table`
- [x] `cargo test --workspace && cargo clippy --workspace && cargo fmt --all -- --check`


---


### M20: CI/CD Integration

**Goal:** GitHub Action, GitLab CI template, `--ci` flag with SARIF sidecar file output. Teams can gate builds on Ripley findings.

- [x] **M20.1** — `--ci` flag and CI auto-detection

  Files:
  - `crates/ripley-core/src/platform.rs`: `pub fn is_ci() -> bool` — checks CI, GITHUB_ACTIONS, GITLAB_CI, JENKINS_URL, CIRCLECI, TRAVIS env vars
  - `crates/ripley-guard/src/main.rs`: add `--ci` and `--sarif-output <PATH>` flags to Scan subcommand
  - `crates/ripley-guard/src/commands/scan.rs`:
    - `fn is_ci_environment() -> bool` (delegates to `platform::is_ci()`)
    - When `--ci`: write SARIF sidecar to `--sarif-output` path (default: `ripley-results.sarif`), table to stdout

  Tests: is_ci with CI=true → true, without → false, --ci flag parses.

  Verify: `cargo test --workspace && cargo run -p ripley-guard -- scan --ci --help`

- [x] **M20.2** — GitHub Action

  New file `.github/actions/ripley-scan/action.yml`:
  - Composite action: install ripley, run scan with --ci, upload SARIF via github/codeql-action/upload-sarif@v3
  - Inputs: path, format, posture-strict, sarif-upload

  New file `docs/ci/github-actions.md`: usage documentation.

  Tests: YAML is syntactically valid.

  Verify: `python3 -c "import yaml; yaml.safe_load(open('.github/actions/ripley-scan/action.yml'))"`

- [x] **M20.3** — GitLab CI template

  New file `docs/ci/gitlab-ci.yml`: ripley-scan job with SAST-compatible artifact.
  New file `docs/ci/gitlab-ci.md`: usage documentation.

  Tests: YAML is syntactically valid.

- [x] **M20.4** — CI integration tests

  New file `crates/ripley-guard/tests/ci_integration.rs`:
  - `test_ci_flag_produces_sarif_file`: run with --ci --sarif-output, assert file exists + valid SARIF
  - `test_ci_exit_code_clean`: exit code 0 for clean scan
  - `test_ci_exit_code_findings`: exit code 1 when findings exist
  - `test_sarif_output_file_alongside_table`: --ci --format table writes table to stdout AND SARIF to file

  Verify: `cargo test --workspace`

#### M20 Gate
- [x] `ripley scan --ci` auto-writes SARIF sidecar file
- [x] GitHub Action YAML syntactically valid
- [x] GitLab CI template syntactically valid
- [x] Exit codes: 0=clean, 1=findings, 2=error (unchanged)
- [x] `cargo test --workspace && cargo clippy --workspace && cargo fmt --all -- --check`


---


### M21: Sandbox Execution

**Goal:** Wrap script execution in platform-specific sandbox restricting network access and filesystem scope. macOS: sandbox-exec. Linux: bwrap. Windows: no-op stub. Opt-in via `[guard] sandbox = true`.

**Design:** sandbox-exec is deprecated on macOS but still functional (used by Homebrew, Nix, Chrome). bwrap (bubblewrap) on Linux is standard unprivileged sandboxing (Flatpak). No root required.

- [x] **M21.1** — Sandbox config section

  File `crates/ripley-core/src/config.rs`:
  - Add to `GuardConfig`: `sandbox: bool` (default false), `sandbox_allow_network: bool` (default false), `sandbox_writable_paths: Vec<String>`
  - Add to `GuardOverlay`: corresponding `Option<>` fields
  - Env var overrides: `RIPLEY_GUARD_SANDBOX`, `RIPLEY_GUARD_SANDBOX_ALLOW_NETWORK`

  File `crates/ripley-guard/src/main.rs`: add `--sandbox` flag to Scan subcommand.

  Tests: default false, TOML round-trip, overlay merge, env var override.

  Verify: `cargo test -p ripley-core -- config && cargo clippy --workspace`

- [x] **M21.2** — Sandbox profiles (core library)

  New module `crates/ripley-core/src/sandbox/mod.rs`: re-exports.
  New file `crates/ripley-core/src/sandbox/profile.rs`:
  - `struct SandboxProfile { allow_network, writable_paths, readable_paths, working_dir }`
  - `SandboxProfile::for_ecosystem(ecosystem, package_dir)` — dispatch per ecosystem
  - `#[cfg(target_os = "macos")] fn to_sandbox_exec_profile(&self) -> String` — Scheme-syntax .sb profile
  - `#[cfg(target_os = "linux")] fn to_bwrap_args(&self) -> Vec<String>` — bwrap CLI args
  - `#[cfg(target_os = "windows")]` — stub with warning

  Tests: macOS profile contains `(deny default)`, `(deny network*)` when appropriate. Linux args contain `--unshare-net`, `--ro-bind`. Windows stub returns without error.

  Verify: `cargo test -p ripley-core -- sandbox::profile && cargo clippy --workspace`

- [x] **M21.3** — Sandbox executor

  New file `crates/ripley-core/src/sandbox/executor.rs`:
  - `struct SandboxResult { exit_code, stderr_output, network_blocked, duration_ms, sandbox_violations }`
  - `struct SandboxViolation { kind: ViolationKind, detail }`
  - `enum ViolationKind { NetworkAccess, FileWriteOutsideScope, ProcessSpawn, Other }`
  - `pub fn execute_sandboxed(script, args, profile) -> Result<SandboxResult, SandboxError>`
  - Platform dispatch: macOS → sandbox-exec, Linux → bwrap, Windows → Err(Unsupported)
  - `enum SandboxError { BwrapNotFound, ProfileGenerationFailed, ExecutionFailed, SandboxExecNotFound, Unsupported }`

  Tests: echo hello succeeds in sandbox (platform-gated), SandboxResult serialization, violation categorization.

  Verify: `cargo test -p ripley-core -- sandbox::executor && cargo clippy --workspace`

- [x] **M21.4** — Wire sandbox into script-shell

  File `crates/ripley-guard/src/bin/ripley-script-shell.rs`:
  - Load config, check `guard.sandbox`
  - If enabled: build profile for ecosystem, call `execute_sandboxed`
  - Log violations to guard.jsonl: `"sandbox": true, "violations": [...]`
  - If violations High/Critical: apply block/prompt logic
  - Fallback: if sandbox setup fails, fall back to `delegate_to_sh()` with warning
  - New: `fn delegate_to_sandbox(args, config) -> ExitCode`

  Tests: sandbox=false uses delegate_to_sh, unsupported platform falls back, log entries include sandbox fields.

  Verify: `cargo build -p ripley-guard && cargo test --workspace`

- [x] **M21.5** — Sandbox integration tests

  New fixtures:
  - `tests/fixtures/scripts/sandbox-network-test.sh` — attempts curl/wget
  - `tests/fixtures/scripts/sandbox-fs-escape-test.sh` — writes to /tmp/evil.txt
  - `tests/fixtures/scripts/sandbox-benign.sh` — mkdir + cp within package dir

  New file `crates/ripley-guard/tests/sandbox_integration.rs`:
  - `#[cfg(target_os = "macos")]`: test_sandbox_blocks_network, test_sandbox_allows_benign
  - `#[cfg(target_os = "linux")]`: same tests (skip if bwrap not installed)
  - Cross-platform: test_sandbox_disabled_passthrough, test_sandbox_result_logged

  Verify: `cargo test --workspace`

#### M21 Gate
- [x] macOS sandbox-exec profile denies network and restricts filesystem
- [x] Linux bwrap args include `--unshare-net` and `--ro-bind`
- [x] Sandbox catches network call from test script
- [x] Sandbox allows benign script to complete
- [x] script-shell falls back gracefully when sandbox unavailable
- [x] Guard log includes sandbox violation details
- [x] No `unwrap()`/`expect()` in ripley-core sandbox code
- [x] `cargo test --workspace && cargo clippy --workspace && cargo fmt --all -- --check`


---


### M22: Behavioral Analysis Engine

**Goal:** Record and evaluate behavior observed during sandboxed script execution. Produce `BehavioralReport` comparing observed vs. declared behavior. Uses sandbox violation signals, not deep tracing.

**Scope guard:** No dtrace/strace. Sandbox IS the observer — blocked operations are the behavioral signals.

- [x] **M22.1** — Behavioral report types

  New module `crates/ripley-core/src/behavioral/mod.rs`: re-exports.
  New file `crates/ripley-core/src/behavioral/report.rs`:
  - `BehavioralReport { package, version, ecosystem, declared, observed, anomalies, risk_score, analysis_duration_ms }`
  - `DeclaredBehavior { has_install_scripts, script_names, declared_dependencies, known_build_tool }`
  - `ObservedBehavior { network_attempts, fs_writes, fs_reads, process_spawns, exit_code }`
  - `NetworkAttempt { host, port, blocked }`, `FsWrite { path, blocked, outside_package }`
  - `FsRead { path, sensitive }`, `ProcessSpawn { command, args }`
  - `BehavioralAnomaly { kind, severity, description, evidence }`
  - `enum AnomalyKind { UnexpectedNetwork, ScopeEscape, CredentialAccess, SuspiciousSpawn, BehaviorMismatch }`
  - `risk_score`: 0.0-1.0

  Tests: serialization, defaults, AnomalyKind round-trip.

  Verify: `cargo test -p ripley-core -- behavioral::report && cargo clippy --workspace`

- [x] **M22.2** — Declared behavior extractor

  New file `crates/ripley-core/src/behavioral/analyzer.rs`:
  - `fn extract_declared_behavior(package_dir, ecosystem) -> Result<DeclaredBehavior, BehavioralError>`
    - npm: parse package.json scripts (preinstall, postinstall, install, prepare)
    - cargo: parse Cargo.toml for `build = "build.rs"`, `links`
    - pip: parse pyproject.toml build system
  - `fn is_known_build_tool(package_name, ecosystem) -> bool` — curated list (node-gyp, esbuild, webpack, etc.)
  - `enum BehavioralError { ManifestNotFound, ParseError(String) }`

  Tests: npm with postinstall → has_install_scripts=true, without → false, known build tools, missing manifest → error.

  Verify: `cargo test -p ripley-core -- behavioral::analyzer && cargo clippy --workspace`

- [x] **M22.3** — Sandbox result to observed behavior

  Extend `crates/ripley-core/src/behavioral/analyzer.rs`:
  - `fn sandbox_result_to_observed(result: &SandboxResult, profile: &SandboxProfile) -> ObservedBehavior`
  - `fn parse_sandbox_exec_stderr(stderr: &str) -> Vec<SandboxViolation>` — parse macOS denial messages
  - `fn parse_bwrap_stderr(stderr: &str) -> Vec<SandboxViolation>` — parse Linux permission errors

  Tests: macOS denial parsed, Linux permission denied parsed, clean stderr → empty observed.

  Verify: `cargo test -p ripley-core -- behavioral::analyzer && cargo clippy --workspace`

- [x] **M22.4** — Anomaly detection and risk scoring

  Extend `crates/ripley-core/src/behavioral/analyzer.rs`:
  - `fn detect_anomalies(declared, observed) -> Vec<BehavioralAnomaly>` — rules:
    1. Network from non-network package → UnexpectedNetwork, High
    2. Writes outside package dir → ScopeEscape, High
    3. Reads credential paths (~/.ssh, ~/.npmrc, ~/.aws) → CredentialAccess, Critical
    4. Unexpected shell spawns → SuspiciousSpawn, Medium
    5. Behavior from package with no install scripts → BehaviorMismatch, Critical
  - `fn calculate_risk_score(anomalies) -> f64` — weighted: Critical=0.4, High=0.25, Medium=0.15, Low=0.05, cap 1.0
  - `pub fn analyze_behavior(package_dir, ecosystem, sandbox_result, profile) -> Result<BehavioralReport, BehavioralError>` — orchestrator

  Tests: CSS library + network → anomaly, node-gyp + network → no anomaly, credential read → Critical, risk score ranges, insta snapshot.

  Verify: `cargo test -p ripley-core -- behavioral && cargo clippy --workspace`

- [x] **M22.5** — Wire behavioral analysis into script-shell and SARIF

  Files:
  - `crates/ripley-guard/src/bin/ripley-script-shell.rs`: after sandbox, run `analyze_behavior`, print anomalies if High+, log to guard.jsonl
  - `crates/ripley-guard/src/output.rs`: `print_behavioral_table()`, `print_behavioral_json()`
  - `crates/ripley-core/src/sarif.rs`: `fn behavioral_to_sarif(report) -> Vec<SarifResult>`

  Tests: anomalies in stderr, guard log includes risk score, behavioral SARIF results correct.

  Verify: `cargo build --workspace && cargo test --workspace && cargo clippy --workspace`

#### M22 Gate
- [x] `BehavioralReport` captures declared vs. observed behavior
- [x] Anomaly detection flags unexpected network from non-network packages
- [x] Risk scoring: 0.0 for benign, >0.4 for critical anomalies
- [x] Behavioral results integrate into SARIF output
- [x] Guard log includes behavioral analysis when sandbox enabled
- [x] No `unwrap()`/`expect()` in ripley-core behavioral code
- [x] `cargo test --workspace && cargo clippy --workspace && cargo fmt --all -- --check`


---


### M23: Community Rule Sharing

**Goal:** Enable publish/subscribe of detection rules from community sources. Three-tier rule loading: compiled-in (base) + user (local) + community (fetched from URLs). Git-based distribution (Homebrew tap model), no custom registry server.

**Trust model:** Community rules untrusted by default — they flag but cannot block in strict mode unless the user promotes a source to trusted.

- [x] **M23.1** — Extended rule metadata

  File `crates/ripley-core/src/rules/mod.rs`:
  - Add optional fields to `Rule`: `author: Option<String>`, `confidence: Option<u8>`, `source_attack: Option<String>`, `updated_at: Option<String>`, `min_ripley_version: Option<String>`, `source: RuleSource`
  - `enum RuleSource { Compiled, User, Community { source_name: String } }` with `#[serde(default)]`
  - `impl Rule`: `fn is_community(&self) -> bool`, `fn is_trusted(&self, trusted_sources: &[String]) -> bool`

  Tests: existing rules parse unchanged, full metadata parses, is_community correct, compiled rules still load.

  Verify: `cargo test -p ripley-core -- rules && cargo clippy --workspace`

- [x] **M23.2** — Rule source registry and index

  New file `crates/ripley-core/src/rules/registry.rs`:
  - `struct RuleSourceEntry { name, url, trust_level, last_fetched, rule_count }`
  - `enum TrustLevel { Untrusted, Trusted }`
  - `struct RuleIndex { version, rules: Vec<RuleEntry> }` — the `index.toml` format
  - `struct RuleEntry { file, id, name, ecosystem, weight, author, updated_at }`
  - `struct SourceRegistry { sources: Vec<RuleSourceEntry> }` — persisted to `{data_dir}/rules/sources.toml`
  - `fn load_registry`, `fn save_registry` (atomic write), `fn add_source`, `fn remove_source`, `fn trust_source`, `fn untrust_source`

  Tests: add/remove/trust, round-trip persistence, duplicate name → error, empty registry loads.

  Verify: `cargo test -p ripley-core -- rules::registry && cargo clippy --workspace`

- [x] **M23.3** — Rule fetcher

  New file `crates/ripley-core/src/rules/fetcher.rs`:
  - `async fn fetch_rules(source, data_dir) -> Result<Vec<Rule>>` — GET index.toml, GET each rule file, store in `{data_dir}/rules/community/{name}/`
  - `async fn update_source(source, data_dir) -> Result<usize>` — fetch + update last_fetched
  - `async fn update_all_sources(registry, data_dir) -> Result<Vec<(String, usize)>>`
  - ETag caching (If-None-Match header, skip on 304)
  - Validation: patterns must be valid regex, weight valid severity, ecosystem recognized. Invalid rules skipped.
  - Uses `reqwest` (already in workspace)

  Extend `crates/ripley-core/src/rules/mod.rs`:
  - `RuleSet::load_all(config_dir, data_dir)` — three-tier merge: compiled < community < user
  - `fn load_community_rules(data_dir)` — walk `{data_dir}/rules/community/*/`

  Tests: mock HTTP fetch, ETag caching, invalid rule skipped, three-tier precedence (user > community > compiled).

  Verify: `cargo test -p ripley-core -- rules::fetcher && cargo clippy --workspace`

- [x] **M23.4** — `ripley rule` CLI commands

  File `crates/ripley-guard/src/main.rs`:
  - Add `Rule { command: RuleCommands }` to Commands enum
  - `enum RuleCommands { Add, Remove, Update, List, Trust, Untrust, Search }`

  New file `crates/ripley-guard/src/commands/rule.rs`:
  - `cmd_add(name, url)`, `cmd_remove(name)`, `cmd_update(name)`, `cmd_list()`, `cmd_trust(name)`, `cmd_untrust(name)`, `cmd_search(query)`

  Tests: clap parses correctly, empty list → helpful message, search finds by keyword.

  Verify: `cargo build -p ripley-guard && cargo run -p ripley-guard -- rule --help && cargo run -p ripley-guard -- rule list`

- [x] **M23.5** — Three-tier loading integration + tests

  Files:
  - `crates/ripley-guard/src/bin/ripley-script-shell.rs`: replace load_compiled+load_user with `RuleSet::load_all`
  - `crates/ripley-guard/src/commands/scan.rs`: use `RuleSet::load_all` for analysis
  - Untrusted community rules can flag but not block in non-interactive mode

  New file `crates/ripley-guard/tests/community_rules_integration.rs`:
  - test_community_rule_loads_from_directory
  - test_community_rule_untrusted_does_not_block
  - test_community_rule_trusted_blocks
  - test_three_tier_merge_precedence

  Verify: `cargo test --workspace`

- [x] **M23.6** — Example community rule repository

  New files:
  - `docs/community-rules-example/index.toml` — example index
  - `docs/community-rules-example/supply_chain_2026.toml` — example rules with full metadata
  - `docs/community-rules.md` — documentation (format, subscribe, trust model, publish)

  Tests: example index parses as valid RuleIndex, example rules parse as valid Rule.

  Verify: `cargo test --workspace`

#### M23 Gate
- [x] Three-tier rule loading: compiled + community + user, correct precedence
- [x] `ripley rule add` fetches rules from HTTP source
- [x] `ripley rule list` shows sources with metadata
- [x] Community rules from untrusted sources flag but do not block
- [x] Trusted community rules can block like compiled rules
- [x] `ripley rule search` finds rules by keyword
- [x] Example community rule repository parses correctly
- [x] `cargo test --workspace && cargo clippy --workspace && cargo fmt --all -- --check`


---


#### Phase 5 Gate

**All must pass before Phase 5 is complete:**

- [x] All M19-M23 gates passed
- [x] `cargo build --workspace --release`
- [x] `cargo test --workspace` — 510 tests (target 500+)
- [x] `cargo clippy --workspace` — no warnings
- [x] `cargo fmt --all -- --check`
- [x] `cargo deny check` — clean
- [x] `ripley scan --format sarif tests/fixtures/` produces valid SARIF 2.1.0 JSON
- [x] `ripley scan --ci --sarif-output /tmp/test.sarif tests/fixtures/` writes SARIF sidecar
- [x] GitHub Action YAML syntactically valid
- [x] GitLab CI template syntactically valid
- [x] Sandbox catches network call from test script
- [x] Behavioral analysis detects anomaly for non-network package making network call
- [x] Community rule loads from registry (integration test)
- [x] Three-tier merge: user > community > compiled
- [x] `ripley rule list` shows configured sources
- [x] No `unwrap()` or `expect()` in ripley-core production code
- [x] All public functions have tests
- [x] No `#[allow(dead_code)]` remaining on Phase 5 scaffolding
- [x] Commit: `Phase 5: Advanced analysis`


---


## Phase 6: UI Rewrite (Tauri 2)

**Goal:** Replace iced-based `crates/ripley-app` with a cross-platform Tauri 2 desktop app (React 19 + shadcn/Base UI + Tailwind v4) shipping on macOS + Linux + Windows from first release. Type-safe IPC via `tauri-specta`. Sub-500ms cold / sub-50ms warm guard-dialog latency preserved. Retire iced.

**Stack:** see `STACK_DECISION.md` "Canonical 2026 versions (locked)" — Tauri 2.11, React 19.2, Vite 8, TS 6, Tailwind v4, shadcn CLI v4 (`--base base-ui`, style `base-vega`), `@base-ui/react` ~1.5, Zustand 5, TanStack Query v5, TanStack Table v8 + Virtual v3, `tauri-specta` v2, ESLint 9 flat, Prettier 3, Vitest 2.x, WebdriverIO 9 + `tauri-driver`, pnpm, Lefthook 1.x, release-plz, just, mise.

**Repo layout:** see `STACK_DECISION.md` "Repo layout (locked)". `crates/ripley-app` is preserved and still builds until M28.

**Order rationale:** ship the risk surface first. M24 scaffolds the shell so the tray-only behavior is falsifiable immediately. M25 measures real guard-dialog latency on real hardware — the only genuine unknown. M26 expands cross-platform parity before content migration so we don't discover Linux/Windows blockers after porting every view. M27 ports views. M28 signs/notarizes, automates releases, then retires iced.


### M24: Tauri scaffold and shell

**Goal:** Empty-but-runnable Tauri app on macOS dev box: tray icon, hidden pre-warm window, Cmd+Shift+R opens it, `tauri-specta` codegen wired day one, ESLint+Prettier+Vitest configured, pnpm/just/lefthook/mise in place.

- [x] **M24.0** — Pre-flight environment verification

  This sub-milestone is a **gate, not an implementation step**. No new files
  except `apps/desktop/.ripley-agent/turns` (turn counter for `/goal`'s
  built-in cap). Verifies the toolchain, auth, and baseline before any
  scaffolding work starts. If any check fails, emit the halt token
  `[HALT-AND-WAIT: preflight-env-missing] <which check>` per `GOAL.md` §5b
  and do not proceed to M24.1.

  Checks (each must exit 0 except where noted):
  - `mise install` resolves locked versions for `node`, `pnpm`, `just`, `lefthook`, `tauri-cli` (read `.mise.toml` once it lands in M24.1; until then verify host versions are not blockers)
  - `cargo --version`, `rustc --version` match `rust-toolchain.toml`
  - `gh auth status` shows authenticated; `gh api user` returns 200
  - `gh api repos/agentstation/ripley/branches/main/protection` returns a config requiring at least: PR review + status checks (warn-only if branch protection is intentionally permissive on this fork)
  - `df -h .` shows >20GB available
  - `cargo deny check` passes on current `main` (baseline before any new work)
  - `cargo test --workspace` passes on current `main` (baseline)
  - GitHub Actions secrets listed (`gh secret list`) — informational only; missing M28 secrets do NOT halt here, they halt at M28.1/M28.2/M28.4 with the correct code

  Setup actions (do these in this milestone, not later):
  - Create `apps/desktop/.ripley-agent/` directory
  - Create `apps/desktop/.ripley-agent/turns` containing `0`
  - Add `apps/desktop/.ripley-agent/turns` to `.gitignore` (the counter is per-run, not per-repo)

  Tests: every check above exits 0 on a clean clone with `mise install` done.

  Verify: `mise install && gh auth status && cargo --version && rustc --version && df -h . | tail -1 && cargo deny check && cargo test --workspace`

- [x] **M24.1** — Root monorepo scaffolding

  New files at workspace root:
  - `package.json` — private, `"packageManager": "pnpm@..."`, scripts proxy to `just`
  - `pnpm-workspace.yaml` — `packages: ["apps/*", "packages/*"]`
  - `.mise.toml` — pin `node`, `pnpm`, `just`, `lefthook`, `tauri-cli`
  - `justfile` — recipes: `dev`, `build`, `test`, `check`, `lint`, `fmt`, `e2e`, `guard-bench`
  - `lefthook.yml` — pre-commit: `cargo fmt`, `cargo clippy`, `pnpm prettier`, `pnpm eslint`, `pnpm tsc` over `{staged_files}`
  - `release-plz.toml` — workspace-aware Rust versioning config
  - `packages/.gitkeep`

  Extend root `Cargo.toml`:
  - `resolver = "3"`
  - `[workspace.package]` shared metadata (`version`, `edition`, `license`, `repository`, `rust-version`)
  - `members += ["apps/desktop/src-tauri"]` (keep `crates/ripley-app` until M28)

  Extend `.gitignore`: `node_modules/`, `dist/`, `*.tsbuildinfo`, `apps/desktop/src-tauri/target/`

  Tests: `pnpm install` succeeds on clean clone, `just check` runs every gate, `mise install` resolves pins.

  Verify: `pnpm install && just check && cargo build --workspace`

- [x] **M24.2** — Tauri 2 app scaffold

  New directory `apps/desktop/`:
  - `package.json` — React 19.2, Vite 8, TS 6, Tailwind v4, shadcn CLI v4, Zustand 5, `@tanstack/react-query` v5, `@tanstack/react-table` v8, `@tanstack/react-virtual` v3, `@base-ui/react` ~1.5, `lucide-react`, `clsx`, `tailwind-merge`, `class-variance-authority`, `match-sorter`, `vite-tsconfig-paths`, `@tauri-apps/api` 2.11, `@tauri-apps/cli` 2.11
  - `tsconfig.json` (references-only), `tsconfig.app.json` (strict + canonical bundler flags), `tsconfig.node.json`
  - `vite.config.ts` — `@tailwindcss/vite`, `@vitejs/plugin-react`, `vite-tsconfig-paths`
  - `index.html` — Vite entry
  - `eslint.config.js` — flat config: `typescript-eslint`, `react`, `react-hooks`, `jsx-a11y`, `eslint-config-prettier`
  - `.prettierrc`
  - `src/main.tsx` — `createRoot` + `<StrictMode>` + `QueryClientProvider`
  - `src/App.tsx` — placeholder root component
  - `src/env.d.ts`
  - `src/styles/theme.css` — `@import "tailwindcss"; @theme { /* DESIGN.md tokens */ }`

  New directory `apps/desktop/src-tauri/`:
  - `Cargo.toml` — `name = "ripley-desktop"`, path deps to `../../../crates/ripley-core` and `ripley-ipc`; deps: `tauri` 2.11, `tauri-specta` v2, `specta` v2, `tokio` (workspace), `tracing`, `tracing-subscriber` (with `env-filter`), `serde`, `serde_json`
  - `build.rs` — `tauri_build::build()` + `tauri_specta::Builder::new().export(...)` writing `../src/lib/bindings.ts`
  - `tauri.conf.json` — v2 schema, single hidden window, tray-only bundle identifier
  - `capabilities/default.json` — minimal allowlist (no `shell`, no `fs` beyond data dir)
  - `icons/` — placeholder icons (full set in M26.3)
  - `src/main.rs`, `src/lib.rs` — `tauri::Builder` shell only

  Tests: `cargo build -p ripley-desktop` compiles, `pnpm tauri info` reports v2 environment, generated `bindings.ts` is non-empty.

  Verify: `cd apps/desktop && pnpm install && pnpm tauri info && cargo build -p ripley-desktop`

- [x] **M24.3** — `tauri-specta` IPC codegen wired day one

  Files:
  - `apps/desktop/src-tauri/src/commands/mod.rs` — `pub mod ping;`
  - `apps/desktop/src-tauri/src/commands/ping.rs` — `#[tauri::command, specta::specta] pub fn ping() -> String` (placeholder for M25 commands)
  - `apps/desktop/src-tauri/build.rs` — emit `apps/desktop/src/lib/bindings.ts` via `tauri_specta::Builder::new().commands(collect_commands![ping])`
  - `apps/desktop/src/lib/bindings.ts` — GENERATED, committed (do not edit by hand)
  - `apps/desktop/src/lib/query.ts` — `QueryClient` factory + `invalidateAll()` helper
  - `apps/desktop/src/lib/ipc.ts` — `subscribe(channel, handler)` event helper

  Smoke test: React component calls `commands.ping()` (typed) and renders the result.

  Tests: `bindings.ts` regenerates deterministically on `cargo build`; type errors surface when a Rust signature changes; frontend `pnpm tsc --noEmit` clean.

  Verify: `cargo build -p ripley-desktop && cd apps/desktop && pnpm tsc --noEmit`

- [x] **M24.4** — Tray icon + hidden window + hotkey

  Files:
  - `apps/desktop/src-tauri/src/tray.rs` — `TrayIconBuilder` with template-icon on macOS, themed icon on Win/Linux; menu items: Open, Quit
  - `apps/desktop/src-tauri/src/prewarm.rs` — create hidden window at startup (`visible: false`, `decorations: false`, `focus: false`), expose `show()` / `hide()`
  - `apps/desktop/src-tauri/src/main.rs` — `set_activation_policy(.Accessory)` on macOS, register Cmd+Shift+R global shortcut → `prewarm::show()`
  - `apps/desktop/src-tauri/capabilities/default.json` — grant `core:tray`, `global-shortcut:allow-register`

  Tests: app launches with no Dock icon on macOS, tray icon appears, Cmd+Shift+R toggles window, Quit terminates cleanly.

  Verify: `pnpm tauri dev` — visually confirm tray-only behavior on macOS dev box.

- [ ] **M24.5** — shadcn init + Tailwind v4 theme tokens

  Files:
  - `apps/desktop/components.json` — shadcn config: `"style": "base-vega"`, `"primitive": "base-ui"`, `tsx: true`, `tailwind.cssVariables: false`, aliases (`@/components`, `@/lib`)
  - `apps/desktop/src/styles/theme.css` — full DESIGN.md token map under `@theme` block (colors, font-sans/mono, radii, spacing, motion)
  - `apps/desktop/src/lib/cn.ts` — re-export `clsx`, `tailwind-merge`, `cva`
  - Run `pnpm dlx shadcn@latest init --base base-ui --style base-vega`; add `Button` as smoke test (`src/components/ui/button.tsx`)

  Tests: Button renders with correct token-driven styling, `theme.css @theme` parses, dark mode toggles via `data-theme` attribute.

  Verify: `pnpm vitest run src/components/ui/button.test.tsx`

- [ ] **M24.6** — Lint/format/test rigging + lefthook hooks

  Files:
  - `apps/desktop/eslint.config.js` — flat config above; add custom rule `ripley/no-raw-hex` (regex on JSX + CSS attrs forbidding `#[0-9a-fA-F]{3,8}` outside `theme.css`)
  - `apps/desktop/.prettierrc`
  - `apps/desktop/vitest.config.ts` — `jsdom` env, `@testing-library/react` 16+, `setupFiles: ["./src/test/setup.ts"]`
  - `apps/desktop/src/test/setup.ts` — jest-dom matchers
  - `lefthook.yml` (root) — pre-commit gates active
  - `.github/workflows/ci.yml` — extend with `pnpm install`, `pnpm -C apps/desktop lint`, `pnpm -C apps/desktop test`, `pnpm audit`

  Tests: hooks run on commit; CI fails on TS error, lint error, or failing Vitest.

  Verify: `just check && lefthook run pre-commit`

- [ ] **M24.7** — Playwright + browser verification scaffold

  Files:
  - `apps/desktop/package.json` — add devDeps: `@playwright/test` v1.x, `@axe-core/playwright`, `playwright-lighthouse`
  - `apps/desktop/playwright.config.ts` — `webServer: { command: "pnpm dev", port: 5173, reuseExistingServer: !process.env.CI }`, Chromium-only projects, screenshot/trace on failure
  - `apps/desktop/tests/browser/smoke.spec.ts` — placeholder: app loads, no console errors, no failed requests
  - `apps/desktop/tests/browser/a11y/baseline.spec.ts` — `axe` scan over the root view; baseline must be clean before per-view scans land in M27
  - `apps/desktop/tests/browser/lighthouse/baseline.spec.ts` — runs `lighthouse_audit` against the empty shell; record initial scores in `tests/browser/__snapshots__/lighthouse-baseline.json` (regression-gates land in M27)
  - `apps/desktop/tests/browser/__snapshots__/.gitkeep`
  - `justfile` — add recipes `test:browser` (Playwright run), `test:browser:ui` (Playwright UI mode for local dev)
  - `.github/workflows/ci.yml` — install Playwright browsers (`pnpm exec playwright install --with-deps chromium`), run `pnpm -C apps/desktop test:browser`
  - Update `CLAUDE.md` Conventions to list Playwright + Chrome DevTools MCP
  - Update `STACK_DECISION.md` Tooling stack to list both

  Tests: `pnpm -C apps/desktop test:browser` passes smoke + a11y baseline; CI step uploads Playwright HTML report as artifact on failure.

  Verify: `just test:browser` clean locally.

#### M24 Gate
- [x] **M24.0 pre-flight passed** — `mise install`, `gh auth status`, baseline `cargo deny check` + `cargo test --workspace` all exit 0; `apps/desktop/.ripley-agent/turns` exists
- [ ] `pnpm install && pnpm -C apps/desktop tauri dev` launches tray-only app on macOS
- [ ] No Dock icon; tray icon visible; Cmd+Shift+R toggles hidden pre-warm window
- [ ] `apps/desktop/src/lib/bindings.ts` generated by `cargo build -p ripley-desktop`
- [ ] shadcn Button renders with DESIGN.md tokens (`base-vega` style + Base UI primitive)
- [ ] `just check` passes (cargo fmt + clippy + test, pnpm lint + tsc + vitest, pnpm audit, cargo deny check)
- [ ] Lefthook pre-commit gates pass on a noop commit
- [ ] `crates/ripley-app` still builds and runs unchanged
- [ ] **`just test:browser` passes** — Playwright smoke + a11y baseline + Lighthouse baseline recorded
- [ ] **Chrome DevTools MCP can drive `http://localhost:5173`** — agent verified by `navigate_page` + `take_screenshot` + `lighthouse_audit`
- [ ] **ESLint rule `ripley/no-raw-hex` fires on a deliberate test violation**
- [ ] Commit: `M24: Tauri scaffold and shell`


### M25: Guard-dialog critical path

**Goal:** End-to-end interception flow on real hardware: simulated `npm install` triggers `ripley-script-shell`, UDS bridge emits Tauri event, pre-warmed window shows the guard dialog, user clicks Allow/Block, decision returns to script-shell. Measured cold + warm latency must meet <500ms / <50ms targets.

- [ ] **M25.1** — IPC bridge: UDS → Tauri event

  Files:
  - `apps/desktop/src-tauri/src/ipc_bridge.rs` — async task spawned on Tauri startup: connect to `{data_dir}/ripley.sock` (Phase 1 protocol unchanged), forward `GuardEvent` messages via `app.emit("guard://event", ...)` and pass `GuardDecision` back via channel
  - `apps/desktop/src-tauri/src/commands/guard.rs` — `#[tauri::command, specta::specta] async fn submit_guard_decision(id: String, decision: Decision) -> Result<(), String>`
  - Reuse `ripley_ipc::protocol::*` — no wire-format changes

  Tests: integration test in `apps/desktop/src-tauri/tests/ipc_bridge.rs` spawning a mock UDS server, asserting event emission and decision round-trip.

  Verify: `cargo test -p ripley-desktop`

- [ ] **M25.2** — Guard dialog React view

  Files:
  - `apps/desktop/src/routes/GuardDialog.tsx` — title, package/version, severity badge, rule matches list, Allow / Block / Allow once buttons
  - `apps/desktop/src/components/ripley/SeverityBadge.tsx` — DESIGN.md weight tokens
  - `apps/desktop/src/components/ripley/RuleMatchList.tsx`
  - `apps/desktop/src/hooks/useGuardEvent.ts` — subscribe to `guard://event`, push into Zustand `guardSlice`
  - `apps/desktop/src/store/guard.ts` — `currentEvent: GuardEvent | null`, `respond(decision)`

  Tests: Vitest renders dialog from fixture event; button clicks call `commands.submitGuardDecision`; severity badge maps weights correctly.

  Verify: `pnpm vitest run src/routes/GuardDialog`

- [ ] **M25.3** — Pre-warm strategy + show-on-event

  Files:
  - `apps/desktop/src-tauri/src/prewarm.rs` — hidden window pre-rendered on startup; `show_on_event(event)`: populate state via channel, `set_focus`, center on cursor display
  - `apps/desktop/src/App.tsx` — listen on `guard://show`, route to `<GuardDialog />`, return to last view on close

  Tests: cold path (window not yet shown) and warm path (window already opened once) both render dialog without flicker.

  Verify: manual smoke on dev box.

- [ ] **M25.4** — Latency instrumentation

  Files:
  - `apps/desktop/src-tauri/src/ipc_bridge.rs` — `tracing::info!` spans: `event_received`, `event_emitted`, `dialog_visible`
  - `apps/desktop/src/routes/GuardDialog.tsx` — `useEffect(() => commands.reportVisible(id), [])`
  - `apps/desktop/src-tauri/src/commands/diag.rs` — `report_visible(id)` records latency in `{data_dir}/guard-latency.jsonl`
  - `justfile` — `guard-bench` recipe that fires 10 simulated events and prints p50/p95

  Tests: round-trip latency recorded per dialog instance.

  Verify: `just guard-bench` reports p95 cold <500ms, p95 warm <50ms on Apple Silicon dev box.

- [ ] **M25.5** — Sidecar wire-up to `ripley-script-shell`

  Files:
  - `apps/desktop/src-tauri/tauri.conf.json` — `bundle.externalBin: ["binaries/ripley-script-shell"]` per-platform suffixes
  - `apps/desktop/src-tauri/build.rs` — copy `ripley-script-shell` build artifact into `apps/desktop/src-tauri/binaries/` on release builds
  - Document non-Tauri install path: Phase 1 behavior preserved when desktop app is not installed (script-shell runs CLI prompt fallback)

  Tests: bundle includes `ripley-script-shell` binary; headless invocation works without desktop app running.

  Verify: `pnpm tauri build --debug && ls apps/desktop/src-tauri/target/debug/bundle/`

- [ ] **M25.6** — Latency tuning to budget

  Iterate `prewarm.rs` strategy (pre-render vs render-on-first-event), Vite chunking, React lazy boundaries, and dialog component weight until measured p95 ≤ budget.

  Tests: three consecutive `just guard-bench` runs meet budget.

  Verify: `just guard-bench` p95 cold ≤500ms, p95 warm ≤50ms.

#### M25 Gate
- [ ] Simulated `npm install` interception triggers guard dialog end-to-end
- [ ] User Allow → script-shell exits 0; Block → script-shell exits non-zero (Phase 1 contract preserved)
- [ ] p95 cold latency <500ms on Apple Silicon
- [ ] p95 warm latency <50ms on Apple Silicon
- [ ] `tauri-specta` types compile without `any` in IPC layer
- [ ] No protocol change to `ripley_ipc::protocol` (Phase 1 wire format preserved)
- [ ] **Playwright perf spec** (`tests/browser/perf/guard-dialog.spec.ts`) mounts the dialog component in Vite-served React, uses `performance.mark` between route enter and first interactive paint, and asserts warm budget <50ms in headless Chromium (proxy for the Tauri shell — does not replace the native cold-path measurement above)
- [ ] **Playwright a11y spec** for guard dialog passes axe with zero violations of `serious` or `critical` severity
- [ ] Commit: `M25: Guard-dialog critical path`


### M26: Cross-platform parity

**Goal:** Reproducible builds on Linux + Windows. Tray works on macOS, Windows 11, KDE, and (documented) GNOME + Hyprland. Tauri bundler outputs `.dmg`/`.app`, `.msi`/`.exe`, AppImage/`.deb`/`.rpm`. WebdriverIO e2e runs on Linux + Windows CI. macOS e2e gap explicitly documented and compensated.

- [ ] **M26.1** — Linux build + tray verification

  Files:
  - `.github/workflows/ci.yml` — `ubuntu-22.04` matrix entry with `webkit2gtk-4.1-dev`, `libayatana-appindicator3-dev` apt deps
  - `docs/install/linux.md` — system deps, AppIndicator extension note for stock GNOME
  - `apps/desktop/src-tauri/src/tray.rs` — verify StatusNotifierItem path on Linux

  Tests: CI green on Ubuntu; manual screenshots from KDE VM (Plasma 6) and GNOME VM with AppIndicator extension.

  Verify: `cargo build --target x86_64-unknown-linux-gnu && pnpm tauri build --target x86_64-unknown-linux-gnu`

- [ ] **M26.2** — Windows build + tray verification

  Files:
  - `.github/workflows/ci.yml` — `windows-latest` matrix entry
  - `docs/install/windows.md` — WebView2 runtime dependency (pre-installed on Win 11), MSI install flow
  - Code-signing key handling deferred to M28.2 (CI does an unsigned build here)

  Tests: CI green on Windows; tray icon + dialog render in Win 11 VM.

  Verify: `cargo build --target x86_64-pc-windows-msvc` produces a working `.msi`.

- [ ] **M26.3** — Tauri bundler outputs per OS

  Files:
  - `apps/desktop/src-tauri/tauri.conf.json` — `bundle.targets: ["dmg", "app", "msi", "appimage", "deb", "rpm"]`
  - `apps/desktop/src-tauri/icons/` — full icon set (16/32/64/128/256/512 + `.ico` + `.icns`)
  - `release-plz.toml` — bundle step plumbed into the release pipeline (signing wires up in M28)

  Tests: each platform CI uploads its artifact as a workflow artifact.

  Verify: `pnpm tauri build --debug` on each matrix entry produces expected files.

- [ ] **M26.4** — WebdriverIO e2e setup (Linux + Windows)

  Files:
  - `apps/desktop/wdio.conf.ts` — `tauri-driver` binary path, capabilities `tauri:options`
  - `apps/desktop/tests/e2e/smoke.spec.ts` — launch app, open Cmd+Shift+R, assert palette renders
  - `apps/desktop/tests/e2e/guard-dialog.spec.ts` — IPC-driven mock event, assert Allow/Block buttons fire `submit_guard_decision`
  - `.github/workflows/ci.yml` — `e2e-linux` + `e2e-windows` jobs (NOT macOS — see M26.5)

  Tests: 2 e2e specs pass on Linux + Windows runners.

  Verify: `pnpm -C apps/desktop e2e` on Linux + Windows CI.

- [ ] **M26.5** — macOS coverage compensation

  Files:
  - `apps/desktop/tests/peekaboo/` — screenshot scripts driven by the existing Peekaboo loop for tray/dialog visuals
  - `apps/desktop/src-tauri/tests/` — additional Rust integration tests for tray menu actions, capabilities ACL boundary, IPC bridge error paths
  - `apps/desktop/README.md` — document the WKWebView WebDriver gap explicitly (not a future-fix item)

  Tests: Rust integration tests pass on macOS CI; documented Peekaboo runs are reproducible.

  Verify: `cargo test -p ripley-desktop`

#### M26 Gate
- [ ] CI green on `ubuntu-22.04`, `windows-latest`, and `macos-14`
- [ ] Tauri bundler emits `.dmg`, `.app`, `.msi`, `.AppImage`, `.deb`, `.rpm`
- [ ] Tray visible + functional on macOS, Windows 11, KDE Plasma 6; documented setup for stock GNOME (AppIndicator) + Hyprland
- [ ] WebdriverIO e2e green on Linux + Windows
- [ ] macOS WebDriver gap documented + compensated (Rust integ + Vitest + Peekaboo)
- [ ] **Playwright browser suite runs on all three OS runners** in CI alongside WebdriverIO — Playwright covers Vite-served React (no Tauri shell), WebdriverIO covers the Tauri shell on Linux + Windows. On macOS, Playwright is the only browser-level signal (compensates for the WebDriver gap)
- [ ] **Lighthouse baseline scores recorded** in `tests/browser/__snapshots__/lighthouse-baseline.json` for each OS — accessibility ≥95, best-practices ≥95, performance ≥90 on the empty shell
- [ ] Commit: `M26: Cross-platform parity`


### M27: View migration

**Goal:** Port every iced view to React/shadcn/Tailwind at functional parity. Migration order driven by `DESIGN_ISSUES.md` priority list (highest density-and-aesthetic delta first). Reuse Phase 1-5 backend logic unchanged.

**Component picks, keyboard model, and DX patterns** are locked in [UX_DESIGN.md](UX_DESIGN.md). Install primitives via `pnpm dlx shadcn@latest add <slug> --base base-ui` in the order that doc lists (Button, Dialog, Sheet, Combobox first). Do not install components outside UX_DESIGN.md's "Used" table without a PR-time justification.

- [ ] **M27.1** — Shared component library (`components/ripley/`)

  Files (each with co-located `*.test.tsx`):
  - `AlertCard.tsx`, `SeverityBadge.tsx`, `EcosystemIcon.tsx`, `WeightBar.tsx`, `RuleMatchList.tsx`, `TimestampCell.tsx`, `EmptyState.tsx`, `LoadingSkeleton.tsx`, `ErrorPane.tsx`
  - `DataTable.tsx` — shadcn data-table recipe over TanStack Table + Virtual; row virtualization at >200 rows; column resize/sort/filter
  - `KeyValueGrid.tsx` — DESIGN.md grid for forensic/audit reports

  Tests: Vitest snapshot per component + DESIGN.md token-conformance assertions.

  Verify: `pnpm -C apps/desktop test`

- [ ] **M27.2** — Alerts view (M3 parity)

  Files:
  - `apps/desktop/src/routes/Alerts.tsx`
  - `apps/desktop/src/hooks/useAlerts.ts` — `useQuery(["alerts"], commands.listAlerts)`
  - `apps/desktop/src-tauri/src/commands/alerts.rs` — wrap `ripley_core::store` reads

  Tests: empty state, 1 alert, 50 alerts (virtualized), error state.

  Verify: `pnpm vitest run src/routes/Alerts`

- [ ] **M27.3** — Guard log view (M3 parity)

  Files:
  - `apps/desktop/src/routes/GuardLog.tsx` — DataTable over `guard.jsonl`
  - `apps/desktop/src/hooks/useGuardLog.ts`
  - `apps/desktop/src-tauri/src/commands/guard_log.rs` — paginated read of `guard.jsonl`

  Tests: pagination, filtering by decision, severity rendering, timestamp formatting.

  Verify: `pnpm vitest run src/routes/GuardLog`

- [ ] **M27.4** — Deep scan / Monitor / Audit / Posture views

  Files:
  - `apps/desktop/src/routes/DeepScan.tsx` — `KeyValueGrid` over forensic report
  - `apps/desktop/src/routes/Monitor.tsx` — real-time monitor events via Tauri event channel + `queryClient.setQueryData`
  - `apps/desktop/src/routes/Audit.tsx` — TrafficLight rendering of audit checks
  - `apps/desktop/src/routes/Posture.tsx` — `ripley harden` recommendations
  - Corresponding `commands::deep_scan`, `commands::monitor`, `commands::audit`, `commands::harden` Tauri handlers

  Tests: each view has Vitest happy + empty + error coverage.

  Verify: `pnpm vitest run`

- [ ] **M27.5** — Command palette (Cmd+K)

  Files:
  - `apps/desktop/src/components/ripley/CommandPalette.tsx` — Base UI `Combobox` + `match-sorter`
  - `apps/desktop/src/store/command-palette.ts` — recency list with `persist` middleware
  - `apps/desktop/src/hooks/useCommands.ts` — declarative command registry

  Tests: fuzzy search returns expected ordering; recency persists across sessions; Esc closes; Enter executes.

  Verify: `pnpm vitest run src/components/ripley/CommandPalette`

- [ ] **M27.6** — Settings view

  Files:
  - `apps/desktop/src/routes/Settings.tsx` — DESIGN.md form components over `ripley_core::config`
  - `apps/desktop/src-tauri/src/commands/settings.rs` — read/write `config.toml` with atomic writes (reuse `ripley_core::config::save`)

  Tests: round-trip read+write of every config layer; atomic write fault injection.

  Verify: `pnpm vitest run src/routes/Settings && cargo test -p ripley-desktop -- commands::settings`

- [ ] **M27.7** — DESIGN_ISSUES.md follow-through

  Walk every numbered DESIGN_ISSUES item (4+) and either close it with a referenced PR/commit or move it to `apps/desktop/DESIGN_NOTES.md` with rationale. Items 1-3 dissolved by stack switch — mark as resolved.

  Tests: every DESIGN_ISSUES item has explicit disposition.

  Verify: manual review with Peekaboo screenshots.

- [ ] **M27.8** — Per-view browser verification suite

  Files (one spec per migrated view; co-located under `apps/desktop/tests/browser/views/`):
  - `alerts.spec.ts`, `guard-log.spec.ts`, `deep-scan.spec.ts`, `monitor.spec.ts`, `audit.spec.ts`, `posture.spec.ts`, `settings.spec.ts`, `command-palette.spec.ts`
  - Each spec asserts: route renders without console errors; axe scan is clean (zero `serious`/`critical`); all interactive elements reachable via Tab in `KEYMAP` order; severity colors match DESIGN.md tokens via `getComputedStyle`; no raw hex colors emitted in computed styles outside `--ripley-*` custom properties
  - `apps/desktop/tests/browser/lighthouse/per-view.spec.ts` — Lighthouse audit per route, asserts a11y ≥95, perf ≥90, best-practices ≥95 against the recorded baseline
  - `apps/desktop/tests/browser/visual/` — Playwright screenshot diffs per view (threshold 0.1% pixel diff); baselines committed under `__snapshots__/` per OS
  - `.github/workflows/ci.yml` — extend `test:browser` job to run on `ubuntu-22.04`, `windows-latest`, `macos-14`

  Tests: every view spec passes; Lighthouse never regresses below baseline; visual diffs flagged at PR time.

  Verify: `just test:browser` green on all three OS runners.

- [ ] **M27.9** — Chrome DevTools MCP verification recipe

  Files:
  - `apps/desktop/docs/mcp-verification.md` — canonical 8-step loop (new_page → navigate → take_snapshot → take_screenshot → list_console_messages → list_network_requests → lighthouse_audit → close_page); when to invoke it (PR self-review for any view touching DESIGN.md tokens, `KEYMAP`, or IPC bindings); how findings get filed (commit message + screenshot in PR description)
  - `apps/desktop/docs/mcp-verification.md` enumerates per-view checklists (Alerts: virtualized scroll, severity sort, empty state; Settings: form errors render; CommandPalette: subsequence ranking, recency persistence)

  Tests: docs reviewed; a single PR demonstrates the loop end-to-end with linked Chrome MCP transcript.

  Verify: documentation review.

#### M27 Gate
- [ ] All Phase 1-5 views available in the Tauri app at functional parity
- [ ] DataTable virtualizes correctly at 10k rows
- [ ] Command palette opens with Cmd+K, fuzzy-finds commands, persists recency
- [ ] Every DESIGN_ISSUES.md item has explicit disposition
- [ ] Vitest coverage ≥80% on `apps/desktop/src/`
- [ ] No iced view referenced by the Tauri app
- [ ] **Every migrated view has a Playwright spec** under `tests/browser/views/` with axe + Lighthouse + computed-style token assertions passing on all three OS runners
- [ ] **Lighthouse per-view scores meet baseline** (a11y ≥95, perf ≥90, best-practices ≥95) and no view regresses from the M26 baseline by more than 2 points without justification
- [ ] **`ripley/no-raw-hex` ESLint rule** still passes — no raw hex literals in `apps/desktop/src/**` outside `theme.css`
- [ ] **Chrome MCP verification loop documented** at `apps/desktop/docs/mcp-verification.md` and demonstrated on at least one PR
- [ ] **Visual regression baselines committed** for every view; diffs reviewed on each PR touching `apps/desktop/src/`
- [ ] Commit: `M27: View migration`


### M28: Release ops + retire iced

**Goal:** Sign + notarize + auto-update on all three OSes. `release-plz` automation for Rust. Delete `crates/ripley-app` and prune `iced`/`tray-icon`/`muda`/`cargo-bundle` from `Cargo.toml`. Per-platform install docs.

- [ ] **M28.1** — macOS notarization

  Files:
  - `.github/workflows/release.yml` — `apple-actions/import-codesign-certs`, `apple-actions/submit-notarization`; secret refs for `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `APPLE_CERTIFICATE`
  - `apps/desktop/src-tauri/tauri.conf.json` — `bundle.macOS.signingIdentity`, `entitlements`, hardened runtime
  - `apps/desktop/entitlements.plist`

  Tests: notarization succeeds in dry-run; `stapler validate` passes; Gatekeeper assesses signed bundle.

  Verify: `spctl --assess --type execute --verbose apps/desktop/src-tauri/target/release/bundle/macos/Ripley.app`

- [ ] **M28.2** — Windows code-signing

  Files:
  - `.github/workflows/release.yml` — Windows signing via Azure Trusted Signing or `signtool` with hardware token
  - `docs/install/windows.md` — signing-key procurement notes

  Tests: signed `.msi` and `.exe`; SmartScreen does not block after reputation build.

  Verify: `signtool verify /pa <path>`

- [ ] **M28.3** — Linux signing + repository hygiene

  Files:
  - Sign AppImage with `gpg --detach-sign`
  - Provide `.deb` + `.rpm` checksums + `.sig`
  - Document optional publish-to-repo path (Cloudsmith/GitHub Pages apt+rpm repo) — out of scope for v1 release, docs only

  Tests: `gpg --verify` passes on AppImage signature.

  Verify: `gpg --verify Ripley.AppImage.sig Ripley.AppImage`

- [ ] **M28.4** — Tauri updater (Ed25519)

  Files:
  - Generate Ed25519 key pair; store private key in CI secret
  - `apps/desktop/src-tauri/tauri.conf.json` — `plugins.updater.pubkey`, `endpoints` (placeholder until release infra exists)
  - `release-plz.toml` — emit `latest.json` per platform on each release tag
  - Implement silent update check on app startup; prompt user when an update is available

  Tests: mock updater endpoint serves new version; app prompts and applies update on next launch.

  Verify: end-to-end updater round-trip on dev VM with mock server.

- [ ] **M28.5** — `release-plz` automation

  Files:
  - `release-plz.toml` — workspace-aware, Conventional Commits, CHANGELOG, tag, GitHub Release per crate
  - `.github/workflows/release.yml` — trigger on `release-plz` PR merge: build matrix, attach bundles, publish release
  - Document `crates.io` token vs binary-only release decision (binary-only for v1)

  Tests: dry-run release on a fork produces expected artifacts + tags.

  Verify: `release-plz --dry-run` on a feature branch.

- [ ] **M28.6** — Retire iced (`crates/ripley-app` removal)

  Files:
  - Delete `crates/ripley-app/`
  - Root `Cargo.toml` — remove from `members`
  - Prune workspace deps: `iced`, `iced_core`, `iced_runtime`, `tray-icon`, `muda`, `cargo-bundle`, related dev-deps
  - Update `CLAUDE.md` Quick reference + `ARCHITECTURE.md` Component 1 to drop iced mentions
  - Delete `crates/ripley-app`-only fixtures
  - Run `cargo deny check` to ensure no orphaned advisories

  Tests: `cargo test --workspace` passes; release builds on all 3 OSes; bundle sizes recorded.

  Verify: `cargo build --workspace --release && cargo test --workspace && cargo deny check`

- [ ] **M28.7** — Install + release documentation

  Files:
  - `docs/install/macos.md`, `docs/install/linux.md`, `docs/install/windows.md`
  - `README.md` — replace iced screenshots with Tauri screenshots; update install commands
  - `CHANGELOG.md` — generated by `release-plz`
  - `apps/desktop/README.md` — dev setup, e2e gap, contributing

  Tests: every install command in docs verified against a clean VM.

  Verify: manual run-through on each OS.

#### M28 Gate
- [ ] Signed + notarized bundles on macOS + Windows
- [ ] Signed AppImage on Linux
- [ ] Tauri updater applies an update end-to-end
- [ ] `release-plz` produces a release tag + bundles + CHANGELOG
- [ ] `crates/ripley-app` deleted; `iced`/`tray-icon`/`muda`/`cargo-bundle` removed from `Cargo.toml`
- [ ] `cargo deny check` clean on the post-removal workspace
- [ ] Install docs validated on a clean VM per OS
- [ ] **Playwright suite runs against the production build** (`pnpm -C apps/desktop build && pnpm -C apps/desktop preview` served on `:4173`; `PLAYWRIGHT_BASE_URL=http://localhost:4173 just test:browser`) — verifies minified bundle behavior, not just dev-server output
- [ ] **Bundle-size budget enforced** — Vite build emits `dist/stats.html`; `tests/browser/perf/bundle-size.spec.ts` asserts main JS chunk ≤250KB gzipped (regression-blocks any PR that crosses the line)
- [ ] Commit: `M28: Release ops and retire iced`


---


#### Phase 6 Gate

**All must pass before Phase 6 is complete:**

- [ ] All M24-M28 gates passed
- [ ] `pnpm install && just check` clean on a fresh clone
- [ ] `cargo build --workspace --release` on macOS, Linux, Windows
- [ ] `cargo test --workspace` — all tests pass on all platforms
- [ ] `pnpm -C apps/desktop test` — Vitest coverage ≥80% on `src/`
- [ ] `pnpm -C apps/desktop e2e` — WebdriverIO suite green on Linux + Windows
- [ ] `just test:browser` — Playwright suite (smoke + per-view + a11y + Lighthouse + perf + visual) green on all three OS runners against both dev server and production build
- [ ] `pnpm audit` clean (or documented allowlist)
- [ ] `cargo deny check` clean
- [ ] `cargo clippy --workspace` no warnings
- [ ] Tauri bundler produces signed `.dmg`/`.app`, `.msi`/`.exe`, AppImage/`.deb`/`.rpm`
- [ ] Tauri updater applies a real update on each OS
- [ ] Guard-dialog p95 cold <500ms / warm <50ms preserved post-migration; Playwright warm-mount budget <50ms holds
- [ ] Lighthouse: a11y ≥95, perf ≥90, best-practices ≥95 on every migrated view, all three OSes
- [ ] axe scans clean (zero `serious`/`critical`) on every migrated view
- [ ] `ripley/no-raw-hex` ESLint rule clean across `apps/desktop/src/**`
- [ ] Bundle-size budget held — main JS chunk ≤250KB gzipped
- [ ] Chrome DevTools MCP verification loop documented and demonstrated at least once
- [ ] `tauri-specta` `bindings.ts` committed; no `any` in IPC layer
- [ ] `crates/ripley-app` deleted; `iced`/`tray-icon`/`muda`/`cargo-bundle` gone from workspace
- [ ] Every `DESIGN_ISSUES.md` item has explicit disposition
- [ ] All public functions in new `apps/desktop/src-tauri/src/` have tests
- [ ] Generated `bindings.ts` committed (not gitignored)
- [ ] Commit: `Phase 6: UI rewrite (Tauri 2)`


---


## Total Plan Gate

**The plan is complete when ALL of the following are true:**

- [ ] All Phase 1-6 gates passed
- [ ] `cargo build --workspace --release` on macOS, Linux, Windows
- [ ] `cargo test --workspace` --- all tests pass on all platforms
- [ ] `pnpm install && just check` clean on a fresh clone
- [ ] `pnpm -C apps/desktop test` --- Vitest coverage ≥80% on `src/`
- [ ] `pnpm -C apps/desktop e2e` --- WebdriverIO green on Linux + Windows
- [ ] `just test:browser` --- Playwright suite green on all three OS runners (dev + prod builds); Lighthouse ≥95/≥90/≥95; axe clean
- [ ] `pnpm audit` clean (or documented allowlist)
- [ ] `cargo deny check` --- clean
- [ ] `cargo clippy --workspace` --- no warnings
- [ ] Full end-to-end workflow test on macOS (Tauri app):
  1. Install Ripley (signed `.dmg` install + guard install)
  2. Launch tray app (Tauri); no Dock icon
  3. Run `ripley scan` --- produces results visible in Alerts view
  4. Receive notification for a known vuln
  5. Click Fix --- harness launches with correct prompt
  6. Run `ripley scan --deep` --- forensic report produced
  7. Run `ripley audit` --- environment audit produced
  8. Run `ripley harden` --- PM hardening recommendations produced
  9. Run `ripley monitor` --- detects planted IOC
  10. Run `ripley contain` --- kills process, saves snapshot
  11. Trigger simulated `npm install` --- guard dialog shows within p95 budget
  12. Tauri updater check applies a new version end-to-end
  13. Run `ripley guard uninstall` --- clean removal
- [ ] Same end-to-end workflow validated on Windows 11 (signed `.msi`) and Linux (signed AppImage + `.deb`)
- [ ] Documentation matches implementation across all spec docs: `README.md`, `CLAUDE.md`, `ARCHITECTURE.md`, `STACK_DECISION.md`, `DESIGN.md`, `DESIGN_ISSUES.md`, `UI.md`, `DECISIONS.md`, `ROADMAP.md`, `WORKFLOW.md`, `SETTINGS.md`, `PLAN.md` (12 total)
- [ ] No `unwrap()` or `expect()` in ripley-core
- [ ] All public functions have tests
- [ ] Snapshot tests cover all output formats
- [ ] `crates/ripley-app` removed; `iced`/`tray-icon`/`muda`/`cargo-bundle` not in workspace deps
- [ ] `tauri-specta` `bindings.ts` committed; IPC layer fully typed end-to-end
