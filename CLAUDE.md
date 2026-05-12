# Ripley

Supply chain defense for developers. System tray app + package manager guard.
See README.md for project background and motivation.

## Quick reference

- **README.md** — what Ripley is and why it exists
- **ARCHITECTURE.md** — system design, component specs, technical decisions
- **ROADMAP.md** — phased execution plan with milestones and verification criteria

## Build

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
```

## Run

```
cargo run -p ripley-guard -- --help
cargo run -p ripley-guard -- scan .
cargo run -p ripley-guard -- scan --deep .
cargo run -p ripley-guard -- guard status
```

## Current work

**Phase 1, Milestone 1: Core Data Pipeline**

See ROADMAP.md for the full task list with verification criteria.
Work through milestones in order (M1 → M2 → M3 → M4 → M5).
Do not start a milestone until the previous one's verification criteria all pass.

After completing a milestone:
1. Run all verification criteria listed in ROADMAP.md for that milestone.
2. Commit with message: `M{n}: {milestone name}`
3. Update this "Current work" section to the next milestone.

## Conventions

- **Error handling:** `thiserror` in `ripley-core` (library), `anyhow` in `ripley-guard` (binary)
- **Async runtime:** `tokio` (multi-threaded, `features = ["full"]`)
- **HTTP client:** `reqwest` with `rustls-tls` (no OpenSSL/C dependency)
- **Serialization:** `serde` + `serde_json` with derive macros
- **CLI parsing:** `clap` v4 with derive macros
- **Logging:** `tracing` + `tracing-subscriber` with `env-filter`
- **Local storage:** `redb` (pure Rust embedded KV store)
- **Version matching:** `semver` crate
- **Filesystem watching:** `notify` crate v6
- **Tests:** unit tests in `#[cfg(test)] mod tests` blocks, integration tests in `tests/`
- No `unwrap()` or `expect()` in `ripley-core` — always return `Result`
- No `unsafe` unless there is a documented, measured reason
- Detection rules are TOML files in `rules/`, compiled in via `include_str!`
- Test fixtures live in `tests/fixtures/` at the workspace root

## Scope guardrails

- **Phase 1 only.** Do not implement Phase 2+ features (other lockfile formats, other
  PM shims, Windows/Linux builds, sandboxing). When the design needs an extension
  point for later, use a trait or enum variant — don't build the implementation.
- **No Tauri until M3.** Milestones M1 and M2 are pure CLI. The tray app comes in M3.
  Do not install tauri-cli or scaffold src-tauri until M3.
- **Test as you go.** Every public function in `ripley-core` should have at least one
  unit test. Write the test before or alongside the implementation, not after.
- **Keep dependencies minimal.** The workspace Cargo.toml already declares all needed
  dependencies. Do not add new crates without a clear reason.
