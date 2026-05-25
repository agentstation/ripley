# Ripley

Supply chain defense for developers. System tray app + package manager guard.
See README.md for project background and motivation.

## Quick reference

- **README.md** — what Ripley is and why it exists
- **ARCHITECTURE.md** — system design, component specs, tech stack, threat model
- **STACK_DECISION.md** — Phase 6 UI stack: Tauri 2 + React 19 + shadcn/Base UI + Tailwind v4 + Zustand + TanStack Query + tauri-specta. Canonical versions, tooling, software patterns, repo layout
- **DESIGN.md** — design system: colors, typography, spacing, component styling, do's/don'ts
- **DESIGN_ISSUES.md** — Phase 6 view-migration priority list (was iced audit; same priorities)
- **UI.md** — view wireframes, interaction specs, tray icon, dashboard, dialogs
- **UX_DESIGN.md** — first-principles UX/DX: workflow→component mapping, keyboard model, DX patterns for M27
- **GOAL.md** — Phase 6 autonomous-execution operating manual for `/goal`-driven runs (turn rituals, halt token grammar, condition block)
- **DECISIONS.md** — rationale behind each technical choice, competitive landscape
- **ROADMAP.md** — phased execution plan with milestones and verification criteria
- **WORKFLOW.md** — user workflow streams: setup, scan, monitoring, interception, fix, forensics, CI, audit, harden, exposure, uninstall
- **SETTINGS.md** — complete reference for all settings, config layers, env vars, defaults

## Build

**Available today** (Phase 1-5 shipped, Rust-only workspace):

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
cargo deny check                    # audit own supply chain
cargo run -p ripley-guard -- scan --format json tests/fixtures/
```

**Becomes available with M24.1** (Phase 6 scaffolding — `justfile`, `package.json`,
`pnpm-workspace.yaml`, `.mise.toml`, `lefthook.yml` are created in that milestone):

```
just dev                            # Tauri dev (pnpm + vite + cargo, hot reload)
just build                          # cargo build --workspace + pnpm tauri build
just test                           # cargo test --workspace + pnpm -F desktop test
just check                          # clippy + fmt + tsc + eslint + cargo deny + pnpm audit
```

**Frontend commands** (M24+, run from repo root; not present on `main` until M24.1 lands):

```
pnpm install                        # install JS deps (pnpm only — never npm/yarn)
pnpm -F desktop tauri dev           # launch desktop in dev mode
pnpm -F desktop tauri build         # bundle .dmg / .msi / .deb / .rpm / AppImage
pnpm -F desktop test                # Vitest unit tests
pnpm -F desktop test:e2e            # WebdriverIO (Linux + Windows only)
pnpm audit                          # JS supply chain audit
```

## Run

```
cargo run -p ripley-guard -- --help
cargo run -p ripley-guard -- scan .
cargo run -p ripley-guard -- scan --deep .
cargo run -p ripley-guard -- scan --format json .
cargo run -p ripley-guard -- guard status
```

## Current work

**Phase 6 — UI Rewrite (Tauri 2).** Phases 1-5 shipped (M1-M23, 510+ tests).
Current milestone: **M24 — Tauri scaffold & shell**. See STACK_DECISION.md for
the locked stack/tooling/patterns and PLAN.md M24-M28 for the task breakdown.

Phase 5 delivered: SARIF output, CI/CD integration, sandboxed script execution,
behavioral analysis, community rule sharing with three-tier loading. The
`crates/ripley-app` iced dashboard is the migration target; it is retired in M28.

## Autonomous execution (`/goal`)

Phase 6 is driven by Claude Code's built-in `/goal` command. The full
operating manual is `GOAL.md`. If you are running inside a `/goal` loop:

- **Read `GOAL.md` first thing every turn** — it contains the turn-start
  ritual, turn-end ritual, halt token grammar, and source-of-truth precedence.
- **The condition (≤4000 chars) lives at the bottom of `GOAL.md`.** The human
  pastes only that block into `/goal`. Everything else is read by Claude
  per-turn from the file.
- **Loop terminates only on explicit tokens** — the success token
  `Phase 6 complete. Release v0.6.0 cut.`, a `[HALT-AND-WAIT: <code>]` halt
  token from the enumerated list in GOAL.md §5b, or `[HALT-MAX-TURNS]`.
  Partial progress does NOT terminate the loop; the evaluator just re-invokes.
- **Wait for or kill background shells before ending a turn** — `/goal` pauses
  if the evaluator fires while background work is still running.
- **Single state file**: `apps/desktop/.ripley-agent/turns` (gitignored, just
  the turn counter for the 500-turn cap). All other state is reconstructed
  from git + PLAN.md `[x]` checkboxes + `gh pr list`.

## Conventions

- **Error handling:** `thiserror` in `ripley-core` (library), `anyhow` in `ripley-guard` (binary)
- **Async runtime:** `tokio` (multi-threaded, `features = ["full"]`)
- **HTTP client:** `reqwest` with `rustls` (no OpenSSL/C dependency)
- **Serialization:** `serde` + `serde_json` with derive macros; `toml` for config files
- **CLI parsing:** `clap` v4 with derive macros
- **CLI output:** support `--format json|table`, exit codes: 0=clean, 1=findings, 2=error
- **Logging:** `tracing` + `tracing-subscriber` with `env-filter`; `tracing-appender` for file logging in daemon mode
- **Advisory cache:** `redb` v4 (pure Rust embedded KV store) — advisory data only
- **Configuration:** `config.toml` in platform config dir — human-editable, layered (user → project → env → CLI). Overlay pattern with `Option<T>` fields for merge.
- **Guard log:** append-only JSONL at `{data_dir}/guard.jsonl`
- **Lockfile index:** in-memory only, rebuilt on startup and filesystem events — never persist derived data
- **Platform directories:** `directories` crate — never hardcode `~/.ripley/`
- **Version matching:** `semver` crate
- **Filesystem watching:** `notify` crate v8
- **Pattern matching:** `regex` crate (cache compiled regexes)
- **UI framework (Phase 6, in progress):** Tauri 2.11 + React 19.2 + shadcn/ui (Base UI primitive) + Tailwind v4. The iced-based `crates/ripley-app` (`tray-icon` + `muda`) is the migration target; retired in M28
- **Frontend package manager:** pnpm only — npm and yarn are forbidden (phantom-dep risk for a supply-chain tool)
- **Frontend build tool:** Vite 8 (Rolldown + Oxc; no Babel)
- **Frontend language:** TypeScript 6 (TS 7 Go-native port still beta — defer)
- **Frontend styling:** Tailwind v4 with `@theme` block in CSS (no `tailwind.config.ts`); DESIGN.md tokens in `apps/desktop/src/styles/theme.css`
- **Frontend components:** `@base-ui/react` 1.x via shadcn CLI v4 (`pnpm dlx shadcn@latest init` + `--base base-ui`, style `base-vega`). Copy-into-repo at `apps/desktop/src/components/ui/`
- **Frontend state:** TanStack Query v5 for IPC ("server state") + Zustand 5 for UI state (slice pattern at >2 slices). Do not mirror server state into Zustand
- **Type-safe IPC:** `tauri-specta` v2 emits TS bindings from `#[tauri::command]` + `specta::Type` annotations into `apps/desktop/src/lib/bindings.ts`. **Generated bindings are committed**, not gitignored (same principle as `Cargo.lock`). No hand-written `invoke<T>("cmd")` calls
- **Frontend linting:** ESLint 9 flat config (`eslint.config.js`) + `typescript-eslint` v8 + React + jsx-a11y plugins. Biome rejected — missing `react-hooks/exhaustive-deps` and security plugin breadth
- **Frontend formatting:** Prettier 3 paired via `eslint-config-prettier`
- **Frontend unit tests:** Vitest 2.x + `@testing-library/react` 16+ + jsdom + `@vitest/coverage-v8`
- **Frontend e2e tests:** WebdriverIO 9 + `tauri-driver`. Linux + Windows CI only; macOS WKWebView gap accepted and covered by Vitest + Peekaboo screenshots
- **Browser regression tests:** Playwright 1.x against Vite-served React (NOT the Tauri shell). Complements WebdriverIO — Playwright covers per-view a11y (`@axe-core/playwright`), Lighthouse scores (`playwright-lighthouse`: a11y ≥95, perf ≥90, best-practices ≥95), visual diffs, DESIGN.md token conformance via `getComputedStyle`, perf budgets (mount-to-interactive <50ms warm), and bundle-size ceilings. Runs on all three OS runners. Specs live in `apps/desktop/tests/browser/`. Invoked via `just test:browser`
- **Interactive verification (agent-only):** Chrome DevTools MCP — agent-driven Chromium for PR self-review of any view touching DESIGN.md tokens, `KEYMAP`, or IPC bindings. Canonical loop: `new_page` → `navigate_page` → `take_snapshot` → `take_screenshot` → `list_console_messages` → `list_network_requests` → `lighthouse_audit` → `close_page`. Not a runtime dep, not part of CI; documented at `apps/desktop/docs/mcp-verification.md`
- **Git hooks:** Lefthook 1.x (`lefthook.yml` at root). Drop `lint-staged` — Lefthook handles staged filtering natively
- **Versioning:** `release-plz` for Rust + Conventional Commits as single source of truth. Changesets only if `packages/*` ever publishes JS
- **Task runner:** `just` (`justfile` at root). All top-level commands proxy through it
- **Tool version pinning:** `mise` (`.mise.toml` at root) pins node, pnpm, just, lefthook, tauri-cli
- **JS supply chain audit:** `pnpm audit` in CI; new JS deps reviewed with the same scrutiny as new Rust deps
- **Snapshot testing:** `insta` crate for analyzer output, CLI output, prompt format (Rust side); Vitest snapshots for React component output
- **Tests:** unit tests in `#[cfg(test)] mod tests` blocks (Rust), `*.test.ts(x)` colocated (TS), integration tests in `tests/`
- No `unwrap()` or `expect()` in `ripley-core` — always return `Result`
- No `unsafe` unless there is a documented, measured reason
- Detection rules: TOML files compiled in via `include_str!` (base) + loaded at runtime from `{config_dir}/rules/` (user)
- Test fixtures live in `tests/fixtures/` at the workspace root
- File writes to config/RC files must be atomic (write temp, then rename)

## Scope guardrails

- **Phase 6 in progress** (UI rewrite to Tauri 2). Do not implement Phase 7+
  features (plugin system, hosted dashboard, deep tracing). Use trait/enum
  extension points where the design needs them for later phases.
- **New deps:** Evaluate carefully. Do not add crates *or* JS packages without
  a clear reason. JS deps cost the same audit attention as Rust deps for a
  supply-chain tool.
- **Follow existing patterns.** Audit checks use collect/evaluate pattern: `collect_*()`
  runs system commands (platform-specific via `cfg(target_os)`), `evaluate_*(output)`
  is pure and testable. Traffic-light output uses `TrafficLight` enum across audit and
  harden. New CLI commands follow existing `scan.rs` shape (`cmd_*`, `--format`, exit
  codes). Prompt generation extends `prompt.rs`. Harness reuses `harness.rs`.
- **Cross-platform.** Use `cfg(target_os)` for platform-specific system checks. Never
  hardcode macOS commands in shared code. Use `platform.rs` helpers for paths, shells.
- **Test as you go.** Every public function in `ripley-core` should have at least one
  unit test. Audit checks test via `evaluate_*` with mock command output. Use `insta`
  snapshot tests for all command outputs.
- **Keep dependencies minimal.** Do not add new crates without a clear reason.
- **Own supply chain.** Run `cargo deny check` as part of verification. A supply chain
  security tool that doesn't audit its own dependencies has no credibility.
