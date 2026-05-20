# Ripley

Supply chain defense for developers. System tray app + package manager guard.
See README.md for project background and motivation.

## Quick reference

- **README.md** — what Ripley is and why it exists
- **ARCHITECTURE.md** — system design, component specs, tech stack, threat model
- **DESIGN.md** — design system: colors, typography, spacing, component styling, do's/don'ts
- **UI.md** — view wireframes, interaction specs, tray icon, dashboard, dialogs
- **DECISIONS.md** — rationale behind each technical choice, competitive landscape
- **ROADMAP.md** — phased execution plan with milestones and verification criteria
- **WORKFLOW.md** — user workflow streams: setup, scan, monitoring, interception, fix, forensics, CI, audit, harden, exposure, uninstall
- **SETTINGS.md** — complete reference for all settings, config layers, env vars, defaults

## Build

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
cargo deny check                    # audit own supply chain
cargo run -p ripley-guard -- scan --format json tests/fixtures/
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

**Phase 2: Ecosystem Breadth.** Phase 1 complete (M1-M5). Now implementing M6-M9:
lockfile parsers (yarn, pnpm, pip, cargo, go, gem), guard shims (6 PMs),
detection rules (pypi, cargo), feed integration (GHSA, Socket.dev), platform
builds (Windows, Linux).

See PLAN.md for detailed task breakdown and ROADMAP.md for deliverable descriptions.

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
- **UI framework:** `iced` for dashboard window and guard dialog; `tray-icon` + `muda` for system tray
- **Snapshot testing:** `insta` crate for analyzer output, CLI output, prompt format
- **Tests:** unit tests in `#[cfg(test)] mod tests` blocks, integration tests in `tests/`
- No `unwrap()` or `expect()` in `ripley-core` — always return `Result`
- No `unsafe` unless there is a documented, measured reason
- Detection rules: TOML files compiled in via `include_str!` (base) + loaded at runtime from `{config_dir}/rules/` (user)
- Test fixtures live in `tests/fixtures/` at the workspace root
- File writes to config/RC files must be atomic (write temp, then rename)

## Scope guardrails

- **Phase 2 only.** Do not implement Phase 3+ features (ripley fix, ripley audit,
  ripley harden, ripley monitor, sandboxing, behavioral analysis). Use trait/enum
  extension points where the design needs them for later phases.
- **New deps allowed:** `serde_yaml` for pnpm/yarn lockfile parsing. All other deps
  should already be in `[workspace.dependencies]`. Do not add crates beyond what
  Phase 2 deliverables require.
- **Follow existing patterns.** Each new lockfile parser matches npm.rs shape (return
  `ParsedLockfile`). Each new shim matches ripley-npm-shim (resolve real binary,
  advisory check, delegate with passthrough).
- **Cross-platform.** Use `cfg(target_os)` for platform-specific paths. Never hardcode
  macOS paths in shared code. Factor platform-specific logic into `platform.rs` trait.
- **Test as you go.** Every public function in `ripley-core` should have at least one
  unit test. Use `insta` snapshot tests for all parser outputs and rule matches.
- **Keep dependencies minimal.** Do not add new crates without a clear reason.
- **Own supply chain.** Run `cargo deny check` as part of verification. A supply chain
  security tool that doesn't audit its own dependencies has no credibility.
