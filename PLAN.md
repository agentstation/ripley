# PLAN.md --- Ripley Execution Control Plane

This document is the single source of truth for implementation state.
It survives context compaction events and enables the agent to resume
work from any point. Update checkboxes and the Plan State section as
tasks complete.


## /goal

```
Implement Ripley Phase 2: Ecosystem Breadth. Execute milestones M6
through M9, pass every gate, and commit. PLAN.md is the control plane.

════════════════════════════════════════════════════════════════
 PROJECT CONTEXT
════════════════════════════════════════════════════════════════

Ripley is a supply chain defense tool — Rust workspace (edition 2024),
4 crates: ripley-core (library, thiserror), ripley-guard (CLI binary,
anyhow), ripley-ipc (IPC layer), ripley-app (iced tray app).

Phase 1 is complete: npm lockfile parser, npm guard shims (PATH shim +
script-shell), macOS tray app, remediation pipeline, forensic scan.
121 tests passing, all 5 milestones (M1-M5) gated and committed.

Phase 2 extends the foundation to all ecosystems and platforms:
  M6: 6 lockfile parsers (yarn, pnpm, pip, cargo, go, gem)
  M7: 2 detection rule files + 6 guard shim binaries + multi-PM install
  M8: 2 feed sources (GHSA GraphQL, Socket.dev REST)
  M9: platform abstraction + Linux build + Windows build + CI matrix

Spec documents (read before implementing — they are the source of truth):
  CLAUDE.md       — conventions, build commands, scope guardrails
  ARCHITECTURE.md — component design, lockfile indexer, feed poller,
                    per-PM shim strategy table, project structure
  ROADMAP.md      — Phase 2 deliverable descriptions
  SETTINGS.md     — config schema, rule file format, feeds config
  WORKFLOW.md     — user workflow streams (esp. §4 install interception)

════════════════════════════════════════════════════════════════
 EXECUTION LOOP — repeat until Phase 2 Gate passes
════════════════════════════════════════════════════════════════

1. READ STATE
   Read PLAN.md §"Plan State". Find the first unchecked `- [ ]` task
   in the current milestone. Read that task's full description — it
   specifies files, types, functions, tests, and verify commands.

2. READ SPECS before implementing. Each milestone has `> Spec:` refs.
   Do not guess field names, formats, or API shapes when a spec exists.

3. IMPLEMENT the task.
   - Read the existing npm.rs or ripley-npm-shim code first. Match its
     patterns exactly: same function signatures, same error types, same
     test structure. A yarn parser should look like npm.rs with different
     parsing logic — not a new architecture.
   - Write every test listed in the task description.
   - No `unwrap()` or `expect()` in ripley-core — return Result.
   - No `unsafe`. No new crates except `serde_yaml` (for pnpm/yarn).
   - Detection rules: TOML in `rules/`, compiled via `include_str!`.
   - Test fixtures in `tests/fixtures/` at workspace root.
   - New shim binaries: `[[bin]]` in ripley-guard/Cargo.toml.
   - Snapshot test every parser output with `insta::assert_json_snapshot!`.
   - File writes to config/RC files: atomic (write temp, rename).
   - Platform-specific code: `cfg(target_os)`, never hardcode macOS paths.

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

── M6 GATE (Lockfile Parsers) ─────────────────────────────────

  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo deny check
  cargo test -p ripley-core -- yarn
  cargo test -p ripley-core -- pnpm
  cargo test -p ripley-core -- pip
  cargo test -p ripley-core -- cargo_lock
  cargo test -p ripley-core -- go
  cargo test -p ripley-core -- gem
  cargo run -p ripley-guard -- scan tests/fixtures/

  VERIFY: scan output includes findings from yarn.lock, pnpm-lock.yaml,
  Pipfile.lock, poetry.lock, Cargo.lock, go.sum, Gemfile.lock fixtures.
  VERIFY: insta snapshots exist for every parser (7 total including npm).

  Pass → commit `M6: Lockfile parsers`, update Plan State to M7.

── M7 GATE (Detection Rules & Guard Shims) ────────────────────

  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo deny check
  cargo build -p ripley-guard --bin ripley-pnpm-shim
  cargo build -p ripley-guard --bin ripley-yarn-shim
  cargo build -p ripley-guard --bin ripley-pip-shim
  cargo build -p ripley-guard --bin ripley-cargo-shim
  cargo build -p ripley-guard --bin ripley-go-shim
  cargo build -p ripley-guard --bin ripley-gem-shim
  cargo run -p ripley-guard -- guard install
  cargo run -p ripley-guard -- guard status
  cargo run -p ripley-guard -- guard uninstall

  VERIFY: guard status shows interception state for every PM found.
  VERIFY: pypi_setup.toml and cargo_build.toml rules load and match
  their respective malicious test fixtures.

  Pass → commit `M7: Detection rules and guard shims`, update to M8.

── M8 GATE (Feed Integration) ─────────────────────────────────

  cargo test --workspace
  cargo clippy --workspace
  cargo test -p ripley-core -- ghsa
  cargo test -p ripley-core -- socket

  VERIFY: mock response parsing tests pass for both GHSA and Socket.
  VERIFY: advisory deduplication merges same CVE from multiple sources.
  VERIFY: config `[feeds] sources = [...]` controls which feeds poll.

  Pass → commit `M8: Feed integration`, update to M9.

── M9 GATE (Platform Builds) ──────────────────────────────────

  # macOS (primary dev machine):
  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo deny check

  VERIFY: platform.rs trait has MacOs, Linux, Windows implementations.
  VERIFY: persistence_paths() returns OS-correct paths per platform.
  VERIFY: CI workflow file exists at .github/workflows/ with 3-OS matrix.

  Pass → commit `M9: Platform builds`, update to Phase 2 Gate.

════════════════════════════════════════════════════════════════
 PHASE 2 GATE — final verification before completion
════════════════════════════════════════════════════════════════

All four milestone gates must have passed. Then run the final check:

  cargo build --workspace --release
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
  cargo deny check
  cargo run -p ripley-guard -- scan tests/fixtures/
  cargo run -p ripley-guard -- scan --format json tests/fixtures/
  cargo run -p ripley-guard -- guard install
  cargo run -p ripley-guard -- guard status
  cargo run -p ripley-guard -- guard uninstall

  VERIFY: scan finds and parses all 7 lockfile types
  VERIFY: guard status shows per-PM interception state
  VERIFY: JSON output includes all ecosystems
  VERIFY: no unwrap()/expect() in ripley-core:
    grep -rn 'unwrap()' crates/ripley-core/src/ | grep -v '#\[cfg(test)\]' | grep -v 'mod tests'
    grep -rn 'expect(' crates/ripley-core/src/ | grep -v '#\[cfg(test)\]' | grep -v 'mod tests'
  VERIFY: all public functions have tests:
    (review each pub fn in ripley-core — every one has ≥1 test)
  VERIFY: insta snapshots cover all parser outputs and rule matches

When the Phase 2 Gate passes:
  1. Commit: `Phase 2: Ecosystem breadth`
  2. Update CLAUDE.md "Current work" to Phase 3.
  3. Update Plan State: Phase = 3, Last gate = Phase 2.
  4. The goal is COMPLETE.

════════════════════════════════════════════════════════════════
 DEPENDENCY GRAPH
════════════════════════════════════════════════════════════════

M6.F  Test fixtures ──► M6.1-M6.6  Parsers ──► M6 Gate
                                                   │
M7.1  Detection rules ◄───────────────────────────┘
  │
M7.2-M7.7  Guard shims ──► M7.8  Multi-PM install ──► M7 Gate
                                                          │
M8.1  GHSA client ──► M8.2  Socket client ──► M8 Gate ◄──┘
                                                 │
M9.1  Platform trait ──► M9.2 Linux ──► M9.3 Windows ──► M9.4 CI
  │                                                          │
  └──────────────────────────────────────────────────── M9 Gate
                                                          │
                                                   Phase 2 Gate

M6 tasks can be done in any order within M6.
M7 shims can be done in any order within M7.
M8 can start after M7 Gate (feeds don't depend on shims, but
the gate ensures a clean workspace before changing feed code).
M9 starts after M8 Gate.

════════════════════════════════════════════════════════════════
 COMMIT STRATEGY
════════════════════════════════════════════════════════════════

Commit at each milestone gate — 4 milestone commits + 1 final:
  M6 Gate → `M6: Lockfile parsers`
  M7 Gate → `M7: Detection rules and guard shims`
  M8 Gate → `M8: Feed integration`
  M9 Gate → `M9: Platform builds`
  Phase 2 Gate → `Phase 2: Ecosystem breadth`

Within a milestone, commit after completing a logical group of tasks
if the session is long and you want to checkpoint progress. Use
descriptive messages: `M6: Add yarn lockfile parser` etc.

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
- Phase 2 only. Do not implement ripley fix, ripley audit, ripley
  harden, ripley monitor, sandboxing, or behavioral analysis.
- Use trait/enum extension points where Phase 3+ will need them.

CODE QUALITY
- No unwrap() or expect() in ripley-core — always return Result.
- No unsafe unless measured and documented (there should be none).
- Every public function in ripley-core gets at least one unit test.
- insta snapshot tests for all parser outputs and rule match results.
- cargo deny check must pass at every milestone gate.

PATTERNS
- New lockfile parsers match npm.rs: pub fn parse_*_lock(content: &str)
  -> Result<ParsedLockfile>. Same ParsedLockfile struct, same error
  type, same test structure.
- New shims match ripley-npm-shim: resolve real binary via which -a,
  advisory check, delegate with passthrough, preserve exit code.
- Detection rules: TOML in rules/ dir, compiled via include_str!,
  user rules loaded from {config_dir}/rules/.
- Guard log: append-only JSONL at {data_dir}/guard.jsonl.
- Config/RC writes: atomic (write temp file, then rename).

DEPENDENCIES
- Only new crate allowed: serde_yaml (for pnpm/yarn YAML lockfiles).
- All others must already be in [workspace.dependencies].

PLATFORM
- cfg(target_os) for platform-specific paths and behaviors.
- Never hardcode macOS paths in shared code.
- Factor platform-specific logic into platform.rs trait impl.

FIXTURES
- All test fixtures in tests/fixtures/ at workspace root.
- Each lockfile fixture should be realistic (15-20 packages minimum),
  include scoped/namespaced packages, and have at least one entry
  that exercises risky-spec detection.
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

If uncommitted changes exist that appear to be a partially completed
task, review them, finish the task, verify, and mark complete. Do not
discard partial work.


## Plan State

```
Phase:     2 --- Ecosystem Breadth
Milestone: M6 --- Lockfile Parsers
Task:      M6.1.1
Status:    not started
Last gate: Phase 1
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

- [ ] **M6.1.1** Create `crates/ripley-core/src/lockfile/yarn.rs`
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

- [ ] **M6.2.1** Create `crates/ripley-core/src/lockfile/pnpm.rs`
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

- [ ] **M6.3.1** Create `crates/ripley-core/src/lockfile/pip.rs`
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

- [ ] **M6.4.1** Create `crates/ripley-core/src/lockfile/cargo_lock.rs`
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

- [ ] **M6.5.1** Create `crates/ripley-core/src/lockfile/go.rs`
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

- [ ] **M6.6.1** Create `crates/ripley-core/src/lockfile/gem.rs`
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

- [ ] **M6.F.1** Create lockfile test fixtures
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

- [ ] **M7.1.1** Create `rules/pypi_setup.toml`
  - Patterns for malicious `setup.py`: `os.system()`, `subprocess.Popen`,
    `exec()`, `eval()`, `__import__('os')`, base64 decoding, network calls
    in `setup()`, `.pth` file creation, `distutils.command` overrides
  - Weight: high/critical for execution, medium for suspicious imports
  - Test: `rules::tests::test_pypi_rules_load`
  - Verify: `cargo test -p ripley-core -- rules`

- [ ] **M7.1.2** Create `rules/cargo_build.toml`
  - Patterns for malicious `build.rs`: `Command::new`, network calls via
    `reqwest`/`ureq`/`std::net`, file writes outside `OUT_DIR`, reading
    home directory, environment variable harvesting, binary downloads
  - Weight: high for network + file writes, critical for credential access
  - Test: `rules::tests::test_cargo_rules_load`
  - Verify: `cargo test -p ripley-core -- rules`


#### M7.2: pnpm guard shim

- [ ] **M7.2.1** Create `crates/ripley-guard/src/bin/ripley-pnpm-shim.rs`
  - Add `[[bin]] name = "ripley-pnpm-shim"` to Cargo.toml
  - Same architecture as npm shim: detect `install`/`add`, advisory check,
    delegate to real `pnpm`
  - Set `script-shell` in local `.npmrc` (pnpm respects same setting)
  - Reuse `ripley-script-shell` binary for lifecycle script interception
  - Verify: `cargo build -p ripley-guard --bin ripley-pnpm-shim`


#### M7.3: Yarn guard shim

- [ ] **M7.3.1** Create `crates/ripley-guard/src/bin/ripley-yarn-shim.rs`
  - Add `[[bin]] name = "ripley-yarn-shim"` to Cargo.toml
  - Detect `add`/`install` commands
  - Yarn v1: set `script-shell` in `.yarnrc`
  - Yarn v2+/Berry: lifecycle scripts run via `yarn plugin` — shim must
    intercept at the binary level and monitor spawned processes
  - Advisory check against local DB
  - Verify: `cargo build -p ripley-guard --bin ripley-yarn-shim`


#### M7.4: pip guard shim

- [ ] **M7.4.1** Create `crates/ripley-guard/src/bin/ripley-pip-shim.rs`
  - Add `[[bin]] name = "ripley-pip-shim"` to Cargo.toml
  - Intercept `pip install`, `pip install -e`
  - Before delegating: extract `setup.py` from sdist/wheel, run static
    analyzer with `pypi_setup.toml` rules
  - Flag `.pth` file creation attempts
  - Advisory check via OSV.dev (ecosystem = "PyPI")
  - Verify: `cargo build -p ripley-guard --bin ripley-pip-shim`

- [ ] **M7.4.2** Create `tests/fixtures/scripts/malicious-setup.py`
  - `os.system("curl ...")`, `base64.b64decode(...)`, writes to `~/.bashrc`
  - For pypi_setup.toml snapshot tests


#### M7.5: Cargo guard shim

- [ ] **M7.5.1** Create `crates/ripley-guard/src/bin/ripley-cargo-shim.rs`
  - Add `[[bin]] name = "ripley-cargo-shim"` to Cargo.toml
  - Intercept `cargo build`, `cargo install`
  - Before delegating: scan `build.rs` files in dependency tree with
    `cargo_build.toml` rules
  - Advisory check via OSV.dev (ecosystem = "crates.io")
  - Verify: `cargo build -p ripley-guard --bin ripley-cargo-shim`

- [ ] **M7.5.2** Create `tests/fixtures/scripts/malicious-build.rs`
  - `Command::new("curl")`, reads `env::home_dir()`, writes outside OUT_DIR
  - For cargo_build.toml snapshot tests


#### M7.6: Go guard shim

- [ ] **M7.6.1** Create `crates/ripley-guard/src/bin/ripley-go-shim.rs`
  - Add `[[bin]] name = "ripley-go-shim"` to Cargo.toml
  - Intercept `go install`, `go get`
  - Advisory check only (Go has no install scripts)
  - Check module against OSV.dev (ecosystem = "Go")
  - Verify: `cargo build -p ripley-guard --bin ripley-go-shim`


#### M7.7: Gem guard shim

- [ ] **M7.7.1** Create `crates/ripley-guard/src/bin/ripley-gem-shim.rs`
  - Add `[[bin]] name = "ripley-gem-shim"` to Cargo.toml
  - Intercept `gem install`, `bundle install`
  - Rubygems allows `extconf.rb` and `Rakefile` execution during install
  - Static analyze `extconf.rb` with npm_postinstall rules (similar patterns)
  - Advisory check via OSV.dev (ecosystem = "RubyGems")
  - Verify: `cargo build -p ripley-guard --bin ripley-gem-shim`


#### M7.8: Guard install expansion

- [ ] **M7.8.1** Expand `guard install` for all shims
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

- [ ] **M8.1.1** Create `crates/ripley-core/src/feed/ghsa.rs`
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

- [ ] **M8.1.2** Merge GHSA advisories into feed pipeline
  - `FeedSource` enum: `Osv`, `Ghsa`, `Socket`
  - `Advisory` gains `source: FeedSource` field
  - Deduplication: same CVE from multiple sources → merge, keep richest data
  - Poller queries both OSV and GHSA on each poll cycle
  - Config: `[feeds] sources = ["osv", "ghsa"]` (default both)


#### M8.2: Socket.dev feed client

- [ ] **M8.2.1** Create `crates/ripley-core/src/feed/socket.rs`
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

- [ ] **M9.1.1** Create platform abstraction layer
  - `crates/ripley-core/src/platform.rs` with trait `Platform`
  - Methods: `persistence_paths()`, `shell_rc_paths()`, `notify()`,
    `open_editor()`, `detect_pms()`
  - Implementations: `MacOsPlatform`, `LinuxPlatform`, `WindowsPlatform`
  - Compile-time dispatch via `cfg(target_os)` or runtime detection
  - Migrate macOS-specific code from persistence.rs, guard install, etc.
  - Test: each platform impl returns valid paths on its OS
  - Verify: `cargo test -p ripley-core -- platform`


#### M9.2: Linux build

- [ ] **M9.2.1** Linux tray app
  - `tray-icon` + `muda` work on Linux (X11/Wayland via `libappindicator`)
  - D-Bus notifications via `notify-rust` (already cross-platform)
  - Persistence auditor: check systemd user services, cron, shell RC
  - Add `~/.config/systemd/user/` to persistence scan paths
  - Test on Ubuntu 22.04+ and Fedora 39+
  - Verify: `cargo build --workspace` on Linux

- [ ] **M9.2.2** Linux installer
  - `.deb` package via `cargo-deb`
  - `.AppImage` via `linuxdeploy`
  - Desktop entry file for app launcher integration
  - Verify: `.deb` installs cleanly on Ubuntu


#### M9.3: Windows build

- [ ] **M9.3.1** Windows tray app
  - `tray-icon` + `muda` work on Windows (Win32 API)
  - Windows notification API via `notify-rust` or `windows-rs` bindings
  - Persistence auditor: check Registry Run keys, Task Scheduler,
    Startup folder, PowerShell profiles
  - Guard shims: `.cmd` wrapper scripts for PATH interception
  - Verify: `cargo build --workspace` on Windows

- [ ] **M9.3.2** Windows installer
  - `.msi` via WiX toolset or `cargo-wix`
  - Add to PATH during install
  - Start menu shortcut
  - Verify: `.msi` installs cleanly on Windows 10+


#### M9.4: CI matrix

- [ ] **M9.4.1** GitHub Actions CI workflow
  - Matrix: macOS-latest, ubuntu-latest, windows-latest
  - Steps: build, test, clippy, fmt, deny
  - Cache: Cargo registry + target directory
  - Artifact upload: release binaries per platform


#### M9 Gate

- [ ] `cargo build --workspace` on macOS, Linux, Windows
- [ ] `cargo test --workspace` on all three platforms
- [ ] `cargo clippy --workspace` clean on all three
- [ ] Platform-specific persistence paths correct per OS
- [ ] Installers build and install cleanly
- [ ] CI matrix green on all three platforms
- [ ] Commit: `M9: Platform builds`


#### Phase 2 Gate

**All must pass before starting Phase 3:**

- [ ] All M6-M9 gates passed
- [ ] `cargo build --workspace --release` on macOS, Linux, Windows
- [ ] `ripley scan` finds and parses all 7 lockfile types
- [ ] `guard install` installs shims for all detected PMs
- [ ] `guard status` shows per-PM state on all platforms
- [ ] Advisory sources configurable (OSV, GHSA, Socket)
- [ ] `cargo deny check` clean
- [ ] No `unwrap()` or `expect()` in ripley-core
- [ ] All public functions have tests
- [ ] Snapshot tests for all parser outputs and detection rules
- [ ] Commit: `Phase 2: Ecosystem breadth`
- [ ] Update CLAUDE.md "Current work" to Phase 3


---


## Phase 3: Response Depth

**Goal:** Structured remediation after a breach.

> Spec: ROADMAP.md "Phase 3: Response Depth"

#### Commands
- [ ] `ripley fix <cve> [path]` --- Spec: WORKFLOW.md "12. Standalone fix"
- [ ] `ripley exposure <cve>` --- Spec: WORKFLOW.md "13. Credential exposure assessment"
- [ ] `ripley audit` --- Spec: WORKFLOW.md "10. Environment security audit"
  - [ ] Machine security checks (FileVault, firewall, OS updates, screen lock)
  - [ ] Developer toolchain checks (git signing, SSH keys, shell RC)
  - [ ] AI tool config integrity (MCP configs, Claude hooks, VS Code extensions)
  - [ ] Credential exposure (shell history, env vars, .env, npm tokens)
  - [ ] Traffic-light output per category
  - [ ] `--fix` flag: prompt generation + harness launch
- [ ] `ripley harden` --- Spec: WORKFLOW.md "11. PM hardening"
  - [ ] PM detection (npm/pnpm/yarn/bun)
  - [ ] Dependency pinning checks
  - [ ] PM hardening checks (release-age, script policy, trust policy)
  - [ ] Provenance checks
  - [ ] Credential hygiene checks

#### Dashboard views
- [ ] Audit report view --- Spec: UI.md "Audit Report View"
- [ ] Posture/harden view --- Spec: UI.md "Posture & Hardening View"

#### Templates
- [ ] Per-attack remediation templates (npm worm, PyPI .pth, credential exfil, IDE config)
- [ ] Network connection audit against C2 databases

#### Phase 3 Gate
- [ ] All commands produce correct output with fixtures
- [ ] Dashboard views render correctly
- [ ] `ripley audit --format json` produces valid JSON
- [ ] `ripley harden --format json` produces valid JSON
- [ ] Commit: `Phase 3: Response depth`


---


## Phase 4: Active Detection

**Goal:** Real-time monitoring for active compromise.

> Spec: ROADMAP.md "Phase 4: Active Detection"

- [ ] `ripley monitor` daemon --- Spec: WORKFLOW.md "7. Active containment"
  - [ ] Process monitoring (outbound connections from Node/Python/Ruby/Go)
  - [ ] Filesystem monitoring (persistence paths, lockfile edits, .claude/, .vscode/)
  - [ ] C2 domain/IP matching
  - [ ] MCP config change detection
- [ ] `ripley contain <pid|pkg>` --- kill + snapshot state
- [ ] Real-time notifications: Contain/View/Investigate
- [ ] Monitor dashboard view --- Spec: UI.md "Monitor View"
- [ ] `[monitor]` config section --- Spec: SETTINGS.md "[monitor]"

#### Phase 4 Gate
- [ ] Monitor detects planted IOC write in test
- [ ] Contain kills target process and saves snapshot
- [ ] Dashboard monitor view renders correctly
- [ ] Commit: `Phase 4: Active detection`


---


## Phase 5: Advanced Analysis

**Goal:** Behavioral analysis, community rules, CI/CD integration.

> Spec: ROADMAP.md "Phase 5: Advanced Analysis"

- [ ] Sandboxed script execution (macOS sandbox-exec, Linux bwrap)
- [ ] Behavioral analysis engine (record filesystem/network/process activity)
- [ ] Community rule sharing (publish/subscribe TOML rules)
- [ ] GitHub Action: `ripley-guard` as build step
- [ ] GitLab CI template
- [ ] SARIF output for code scanning dashboards

#### Phase 5 Gate
- [ ] Sandbox catches network call from test script
- [ ] Community rule loads from registry
- [ ] GitHub Action passes in test repo
- [ ] Commit: `Phase 5: Advanced analysis`


---


## Total Plan Gate

**The plan is complete when ALL of the following are true:**

- [ ] All Phase 1-5 gates passed
- [ ] `cargo build --workspace --release` on macOS, Linux, Windows
- [ ] `cargo test --workspace` --- all tests pass on all platforms
- [ ] `cargo deny check` --- clean
- [ ] `cargo clippy --workspace` --- no warnings
- [ ] Full end-to-end workflow test on macOS:
  1. Install Ripley (cargo install + guard install)
  2. Launch tray app
  3. Run `ripley scan` --- produces results
  4. Receive notification for a known vuln
  5. Click Fix --- harness launches with correct prompt
  6. Run `ripley scan --deep` --- forensic report produced
  7. Run `ripley audit` --- environment audit produced
  8. Run `ripley harden` --- PM hardening recommendations produced
  9. Run `ripley monitor` --- detects planted IOC
  10. Run `ripley contain` --- kills process, saves snapshot
  11. Run `ripley guard uninstall` --- clean removal
- [ ] Documentation matches implementation (all 9 docs accurate)
- [ ] No `unwrap()` or `expect()` in ripley-core
- [ ] All public functions have tests
- [ ] Snapshot tests cover all output formats
