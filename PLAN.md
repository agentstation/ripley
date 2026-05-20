# PLAN.md --- Ripley Execution Control Plane

This document is the single source of truth for implementation state.
It survives context compaction events and enables the agent to resume
work from any point. Update checkboxes and the Plan State section as
tasks complete.


## /goal

```
Implement Ripley following PLAN.md as the control plane.

On each turn:
1. Read PLAN.md "Plan State" to determine current position.
2. Read the current task's description and spec references.
3. Implement the task (create files, write code, write tests).
4. Run the task's verification commands.
5. If verification passes: mark the checkbox [x], update Plan State,
   and `git add` the changed files.
6. If a milestone gate is reached: run ALL gate criteria. Do not
   proceed to the next milestone until every gate check passes.
7. When a full milestone passes its gate: commit with message
   "M{n}: {milestone name}" and update CLAUDE.md "Current work"
   to the next milestone.
8. Continue to the next unchecked task.

After context compaction or session restart:
- Read PLAN.md (especially Plan State and the first unchecked task).
- Run `git log --oneline -10` and `git status` to recover context.
- Resume from step 1 above.

Constraints:
- Follow CLAUDE.md conventions (error handling, no unwrap, etc.).
- Reference spec documents (ARCHITECTURE.md, SETTINGS.md, etc.) for
  detailed requirements — do not guess when the spec is available.
- Every public function in ripley-core gets at least one unit test.
- Use `insta` snapshot tests for any output with a defined format.
- Do not add crates beyond what is declared in workspace dependencies.
- Do not implement Phase 2+ features. Use trait/enum extension points.
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
Phase:     1 --- Foundation
Milestone: M2 --- Guard MVP (npm)
Task:      M2.1 --- Detection rules
Status:    not started
Last gate: M1
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
    - `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }`
    - `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `toml = "0.8"`
    - `clap = { version = "4", features = ["derive"] }`
    - `tracing = "0.1"`, `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`
    - `redb = "2"`, `semver = { version = "1", features = ["serde"] }`
    - `directories = "5"`, `regex = "1"`, `notify = "8"`
    - `colored = "2"`, `chrono = { version = "0.4", features = ["serde"] }`
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

- [ ] **M2.1.1** Create `crates/ripley-core/src/rules/mod.rs`
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

- [ ] **M2.1.2** Create `rules/npm_postinstall.toml`
  - Rules for: network_call, code_generation, encoding_obfuscation,
    binary_execution, shell_spawning, env_harvesting, scope_escape,
    obfuscation_tools, ai_tool_config_write, mcp_server_injection
  - Follow SETTINGS.md "Rule File Format" exactly
  - Spec: ARCHITECTURE.md "Static analysis" signal table

- [ ] **M2.1.3** Create `rules/credential_exfil.toml`
  - Cross-ecosystem patterns for credential theft:
    reading ~/.npmrc, ~/.aws/credentials, ~/.ssh/*, ~/.env,
    process.env bulk access
  - Weight: high or critical

- [ ] **M2.1.4** Create `rules/persistence_write.toml`
  - Patterns for writes to .claude/, .vscode/tasks.json, .mcp.json,
    .cursor/mcp.json, LaunchAgents, crontab
  - Weight: critical
  - Spec: ARCHITECTURE.md "AI tool config as persistence surface"


#### M2.2: Static analyzer

> Spec: ROADMAP.md M2 task 2
> Spec: ARCHITECTURE.md "Static analysis"

- [ ] **M2.2.1** Create `crates/ripley-core/src/analyzer.rs`
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

- [ ] **M2.2.2** Typosquatting detector
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

- [ ] **M2.3.1** Create `crates/ripley-guard/src/extractor.rs`
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

- [ ] **M2.4.1** Create `crates/ripley-guard/src/bin/ripley-npm-shim.rs`
  - Add `[[bin]] name = "ripley-npm-shim"` to Cargo.toml
  - Parse args to detect `install`/`add`/`i` commands
  - For install: run advisory check against local DB, warn if known advisory
  - Resolve real npm: `which -a npm`, skip self, pick next
  - Delegate to real npm, pass through stdout/stderr, preserve exit code
  - Verify: `cargo build -p ripley-guard --bin ripley-npm-shim`


#### M2.5: npm script-shell

> Spec: ROADMAP.md M2 task 5
> Spec: WORKFLOW.md "4. Install interception"

- [ ] **M2.5.1** Create `crates/ripley-guard/src/bin/ripley-script-shell.rs`
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

- [ ] **M2.6.1** Implement `guard install` subcommand
  - Create `{data_dir}/bin/` directory
  - Copy `ripley-npm-shim` binary to `{data_dir}/bin/npm`
  - Copy `ripley-script-shell` binary to `{data_dir}/bin/ripley-script-shell`
  - Detect shell RC file (.zshrc, .bashrc, .profile)
  - Append `export PATH="{data_dir}/bin:$PATH"` if not present
  - Set `script-shell={data_dir}/bin/ripley-script-shell` in `~/.npmrc`
  - All writes atomic (write temp, then rename)
  - Must be idempotent
  - Verify: `cargo run -p ripley-guard -- guard install` succeeds

- [ ] **M2.6.2** Implement `guard uninstall` subcommand
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

- [ ] **M2.7.1** Implement trust management
  - `guard trust <pkg>`: add to `[guard] trust` in config.toml (atomic write)
  - `guard untrust <pkg>`: remove from trust list
  - Support exact names and glob scopes (`@tanstack/*`)
  - Verify: `cargo run -p ripley-guard -- guard trust express`

- [ ] **M2.7.2** Implement guard log
  - Guard log: append-only JSONL at `{data_dir}/guard.jsonl`
  - Entry format: { timestamp, package, version, script, action,
    risk_level, matched_rules, source, user_decision }
  - `guard log`: read last 20 lines, render as table
  - Spec: SETTINGS.md "Guard Log" entry format
  - Verify: `cargo run -p ripley-guard -- guard log`

- [ ] **M2.7.3** Implement guard status
  - Show: which shims installed, script-shell active, trusted package count
  - Verify: `cargo run -p ripley-guard -- guard status`


#### M2 Gate

**All must pass before starting M3:**

- [ ] `cargo build --workspace` --- compiles (all binaries)
- [ ] `cargo test --workspace` --- all tests pass
- [ ] `cargo clippy --workspace` --- no warnings
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo deny check`
- [ ] Analyzer: malicious fixture -> High/Critical risk
- [ ] Analyzer: benign fixture -> Low risk
- [ ] `cargo run -p ripley-guard -- guard status` --- shows "not installed"
- [ ] `cargo run -p ripley-guard -- guard install` --- installs shims
- [ ] `cargo run -p ripley-guard -- guard status` --- shows installed
- [ ] `cargo run -p ripley-guard -- guard trust express` --- adds to trust
- [ ] `cargo run -p ripley-guard -- guard log` --- shows entries
- [ ] `cargo run -p ripley-guard -- guard uninstall` --- removes shims
- [ ] Commit: `M2: Guard MVP (npm)`
- [ ] Update CLAUDE.md "Current work" to M3


---


### M3: Tray App MVP (macOS)

Build the system tray application and dashboard window.

> Spec: ROADMAP.md "M3: Tray App MVP (macOS)"
> Spec: ARCHITECTURE.md "Component 1: ripley-app"
> Spec: DESIGN.md (full design system)
> Spec: UI.md (all wireframes)


#### M3.1: Workspace expansion

- [ ] **M3.1.1** Add `crates/ripley-app/` and `crates/ripley-ipc/`
  - Add to workspace members in root Cargo.toml
  - `ripley-ipc`: depends on serde, serde_json, tokio, thiserror
  - `ripley-app`: depends on ripley-core, ripley-ipc, tray-icon, muda,
    iced, notify-rust, tokio, tokio-util, tracing, tracing-appender
  - Add new workspace deps: tray-icon, muda, iced, notify-rust,
    tokio-util, tracing-appender
  - Verify: `cargo build --workspace`


#### M3.2: IPC layer

> Spec: ROADMAP.md M3 task 2

- [ ] **M3.2.1** Create `crates/ripley-ipc/src/lib.rs`
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

- [ ] **M3.3.1** Create `crates/ripley-app/src/events.rs`
  - `AppEvent` enum: AdvisoriesUpdated, LockfileChanged, MatchFound,
    UserAction, IpcRequest
  - `mpsc::channel<AppEvent>` shared by all components
  - All components take `CancellationToken` for graceful shutdown
  - Verify: `cargo build -p ripley-app`


#### M3.4: System tray

> Spec: ROADMAP.md M3 task 4
> Spec: UI.md "Tray Icon", "Tray Menu"

- [ ] **M3.4.1** Create `crates/ripley-app/src/tray.rs`
  - System tray icon with `tray-icon` crate
  - Context menu with `muda`: Show Dashboard, Scan Now, Quit
  - Icon variants: shield outline (idle), shield+dot (alert), shield+! (critical)
  - Template images for macOS
  - Bridge tray events to iced via channel
  - Spec: UI.md "Tray Icon" and "Tray Menu" for states and menu structure


#### M3.5: Background poller

> Spec: ROADMAP.md M3 task 5

- [ ] **M3.5.1** Create `crates/ripley-app/src/poller.rs`
  - Tokio task: poll OSV.dev on configured interval
  - Use ETag caching and exponential backoff from M1 feed client
  - On new advisories: send AdvisoriesUpdated event
  - Respect CancellationToken


#### M3.6: Lockfile watcher

> Spec: ROADMAP.md M3 task 6

- [ ] **M3.6.1** Create `crates/ripley-app/src/watcher.rs`
  - Walk configured project roots, index all lockfiles (in-memory)
  - Watch for changes via `notify` v8
  - On change: re-parse, send LockfileChanged event


#### M3.7: Notifications

> Spec: ROADMAP.md M3 task 7
> Spec: UI.md "Notifications" (3 variants)

- [ ] **M3.7.1** Create `crates/ripley-app/src/notifier.rs`
  - `notify-rust` for native OS notifications
  - Three variants: Before (Fix/View/Dismiss), During (Contain/View/Investigate),
    After (Remediate/View Report)
  - Click opens dashboard to relevant alert
  - Spec: UI.md notification wireframes


#### M3.8: Dashboard window

> Spec: ROADMAP.md M3 task 8
> Spec: UI.md (Dashboard, Alerts, Guard Log, Settings, First Run, Error States)
> Spec: DESIGN.md (all component specs)

- [ ] **M3.8.1** Create `crates/ripley-app/src/theme.rs`
  - Custom `iced::Theme` implementing DESIGN.md color system
  - Severity colors, surface colors, text hierarchy, accent
  - System fonts (SF Pro/SF Mono on macOS)
  - Spec: DESIGN.md "Colors", "Typography"

- [ ] **M3.8.2** Create `crates/ripley-app/src/app.rs`
  - `iced::Application` implementation
  - Message enum mapping to AppEvent
  - Sidebar navigation: Alerts (default), Guard, Settings
  - Window: 900x640 default, 720x480 minimum
  - Close hides window (does not quit)

- [ ] **M3.8.3** Create `crates/ripley-app/src/views/alerts.rs`
  - Alert list sorted by severity then recency
  - Row: severity dot + badge, package@version, advisory, project path, time, action
  - Alert detail slide-in panel
  - Empty state: green shield, "No findings"
  - First-run state: welcome screen with setup steps
  - Spec: UI.md "Alerts View", "Alert Detail", "First Run"

- [ ] **M3.8.4** Create `crates/ripley-app/src/views/guard_log.rs`
  - Table: time, package, script, risk, action
  - Expandable rows with script excerpts
  - Spec: UI.md "Guard Log View"

- [ ] **M3.8.5** Create `crates/ripley-app/src/views/settings.rs`
  - Form reads/writes config.toml
  - Sections: General, Monitoring, Guard, Posture, Advanced
  - Saves immediately, atomic writes
  - Spec: UI.md "Settings View", SETTINGS.md full schema

- [ ] **M3.8.6** Guard interception dialog
  - Modal overlay, max 600px wide
  - Script content with line numbers and highlighted matched lines
  - Buttons: Allow Once, Block, Always Trust, Inspect
  - 30-second countdown timer, defaults to Block
  - Spec: UI.md "Guard Interception Dialog", DESIGN.md "Countdown Timer"

- [ ] **M3.8.7** Error states
  - Network failure banner
  - No lockfiles found state
  - Spec: UI.md "Error States"


#### M3.9: App bundling

> Spec: ROADMAP.md M3 task 9

- [ ] **M3.9.1** Configure cargo-bundle for macOS
  - Package `ripley-app` as `Ripley.app`
  - `Info.plist`: `LSUIElement = true`
  - App icon in `crates/ripley-app/icons/`
  - Verify: `cargo bundle --release -p ripley-app`


#### M3.10: Headless daemon

> Spec: ROADMAP.md M3 task 10
> Spec: ARCHITECTURE.md "ripley watch"

- [ ] **M3.10.1** Implement `watch` subcommand in ripley-guard
  - Start poller, watcher, matcher, IPC server (no UI)
  - Fire notifications via notify-rust
  - `--daemon` flag: fork to background, log to file
  - Graceful shutdown on SIGTERM/SIGINT
  - Verify: `cargo run -p ripley-guard -- watch` starts daemon


#### M3 Gate

- [ ] `cargo build --workspace` --- all 4 crates compile
- [ ] `cargo test --workspace` --- all tests pass
- [ ] `cargo clippy --workspace`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo deny check`
- [ ] `cargo bundle --release -p ripley-app` --- produces Ripley.app
- [ ] Manual: launch Ripley.app, tray icon appears with menu
- [ ] Manual: `ripley status` shows IPC connection to daemon
- [ ] Manual: `ripley watch` starts headless daemon, responds to `ripley status`
- [ ] Commit: `M3: Tray app MVP (macOS)`
- [ ] Update CLAUDE.md "Current work" to M4


---


### M4: Remediation Pipeline

Wire the "Fix" button: prompt generation and AI harness launching.

> Spec: ROADMAP.md "M4: Remediation Pipeline"
> Spec: ARCHITECTURE.md "Prompt generator", "Harness launcher"
> Spec: WORKFLOW.md "5. Alert-driven fix"


#### M4.1: Prompt generator

- [ ] **M4.1.1** Create `crates/ripley-core/src/prompt.rs`
  - `pub fn generate_remediation_prompt(m: &Match) -> String`
  - Template: project path, package@version, CVE, clean version,
    IOC file paths, credential hints, test instruction
  - Snapshot test: `insta::assert_snapshot!` on generated prompt
  - Spec: ARCHITECTURE.md "Prompt generator" example format
  - Verify: `cargo test -p ripley-core -- prompt`


#### M4.2: Harness launcher

- [ ] **M4.2.1** Create `crates/ripley-core/src/harness.rs`
  - `pub fn detect_harness() -> Option<Harness>` --- check PATH for
    claude, codex, opencode (in order, or config preference)
  - `pub fn launch(harness: &Harness, prompt: &str) -> Result<Child>`
  - Spawn in new terminal: `open -a Terminal.app` + harness command
  - Return process handle (don't wait)
  - Test: `harness::tests::test_detect_harness` (with mocked PATH)
  - Verify: `cargo test -p ripley-core -- harness`


#### M4.3: Wire Fix action

- [ ] **M4.3.1** Connect Fix in tray app
  - Notification "Fix" -> prompt generator -> harness launcher
  - Alert detail panel "Fix with Claude" -> same flow
  - Dashboard "Remediate All" -> batch prompt generation

- [ ] **M4.3.2** Wire `--fix` flag on `ripley scan`
  - Generate prompts for all matches
  - Launch harness for each (or print to stdout if no harness)
  - Verify: `cargo run -p ripley-guard -- scan --fix tests/fixtures/`


#### M4 Gate

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace`
- [ ] Prompt generator snapshot tests pass
- [ ] `ripley scan --fix tests/fixtures/` generates prompt (prints to stdout)
- [ ] Commit: `M4: Remediation pipeline`
- [ ] Update CLAUDE.md "Current work" to M5


---


### M5: Forensic Scan

Build `ripley scan --deep` --- the post-breach audit.

> Spec: ROADMAP.md "M5: Forensic Scan"
> Spec: WORKFLOW.md "6. Post-breach forensics"
> Spec: UI.md "Deep Scan Report"
> Spec: SETTINGS.md "IOC Profiles"


#### M5.1: IOC scanner

- [ ] **M5.1.1** Create `crates/ripley-core/src/forensic/mod.rs` and `ioc.rs`
  - IOC file patterns: .claude/execution.js, .claude/setup.mjs,
    .claude/settings.json (unexpected hooks), .vscode/tasks.json (runOn),
    .mcp.json (rogue servers), .cursor/mcp.json, node_modules/.cache/ binaries,
    ~/.ssh/authorized_keys (unexpected keys)
  - `pub fn scan_iocs(path: &Path, profiles: &[IocProfile]) -> Vec<IocFinding>`
  - `IocFinding` { path, description, severity, profile_id }
  - Add `pub mod forensic;` to lib.rs
  - Test with temp dir containing planted IOC files
  - Verify: `cargo test -p ripley-core -- forensic`

- [ ] **M5.1.2** IOC profile loader
  - Parse TOML profiles from `{config_dir}/iocs/` (runtime) + compiled-in
  - `IocProfile` struct matching SETTINGS.md "IOC Profile Format"
  - Ship `iocs/tanstack-2026-05.toml` and `iocs/mini-shai-hulud.toml`


#### M5.2: Persistence auditor

- [ ] **M5.2.1** Create `crates/ripley-core/src/forensic/persistence.rs`
  - Check macOS: ~/Library/LaunchAgents/*.plist, crontab, shell RC files
  - MCP/AI config audit: .mcp.json, .cursor/mcp.json, .claude/settings.json
  - Dead man switch detection: processes polling external APIs
  - `pub fn audit_persistence(home: &Path) -> Vec<PersistenceFinding>`
  - Verify: `cargo test -p ripley-core -- persistence`


#### M5.3: Credential exposure mapper

- [ ] **M5.3.1** Create `crates/ripley-core/src/forensic/credentials.rs`
  - Map attack -> targeted credential stores (from IOC profiles)
  - Check which files exist on this machine
  - Generate rotation commands per credential
  - Dead man switch warning if applicable
  - `pub fn assess_exposure(profiles: &[IocProfile], home: &Path) -> Vec<CredentialFinding>`
  - Verify: `cargo test -p ripley-core -- credentials`


#### M5.4: Deep scan CLI

- [ ] **M5.4.1** Wire `--deep` flag in scan subcommand
  - Run lockfile scan + IOC scanner + persistence auditor + credential mapper
  - Table output: summary, findings by category, recommended actions
  - JSON output: structured report
  - Generate remediation prompt if findings exist
  - Spec: UI.md "Deep Scan Report" for output structure
  - Verify: `cargo run -p ripley-guard -- scan --deep tests/fixtures/`

- [ ] **M5.4.2** Deep scan report in dashboard
  - Summary cards: Vulns, IOCs, Persistence, Creds Risk, MCP
  - Collapsible sections per category
  - Dead man switch warning panel
  - Action buttons: Remediate All, Export Report, Copy Rotation Checklist
  - Spec: UI.md "Deep Scan Report" wireframe


#### M5 Gate

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo deny check`
- [ ] `cargo run -p ripley-guard -- scan --deep ~` on clean machine: "no findings"
- [ ] Deep scan with planted IOC fixtures: findings reported
- [ ] IOC profile loading works (compiled-in + runtime)
- [ ] Commit: `M5: Forensic scan`
- [ ] Update CLAUDE.md "Current work" to Phase 2


#### Phase 1 Gate

**All must pass before starting Phase 2:**

- [ ] All M1-M5 gates passed
- [ ] `cargo build --workspace --release` --- release build succeeds
- [ ] `cargo bundle --release -p ripley-app` --- Ripley.app bundle works
- [ ] Full workflow test: install guard, run scan, trigger notification,
      click Fix, review prompt output, run deep scan
- [ ] `cargo deny check` --- clean
- [ ] No `unwrap()` or `expect()` in ripley-core source
- [ ] All public functions in ripley-core have at least one test
- [ ] Snapshot tests exist for: analyzer output, CLI output, prompt format


---


## Phase 2: Ecosystem Breadth

**Goal:** All major PMs and lockfiles, Windows + Linux.

> Spec: ROADMAP.md "Phase 2: Ecosystem Breadth"

#### Lockfile parsers
- [ ] `crates/ripley-core/src/lockfile/yarn.rs` (v1 + v2/Berry)
- [ ] `crates/ripley-core/src/lockfile/pnpm.rs`
- [ ] `crates/ripley-core/src/lockfile/pip.rs` (Pipfile.lock, poetry.lock)
- [ ] `crates/ripley-core/src/lockfile/cargo.rs`
- [ ] `crates/ripley-core/src/lockfile/go.rs`
- [ ] `crates/ripley-core/src/lockfile/gem.rs`

#### Guard shims
- [ ] `ripley-pip-shim` (intercept pip install, analyze setup.py)
- [ ] `ripley-cargo-shim` (intercept cargo build/install, analyze build.rs)
- [ ] `ripley-gem-shim` (shim + Rubygems pre_install hook)
- [ ] `ripley-go-shim` (advisory check only)
- [ ] `ripley-yarn-shim` (PnP-aware lifecycle handling)
- [ ] `ripley-pnpm-shim` (shim + script-shell in .npmrc)

#### Detection rules
- [ ] `rules/pypi_setup.toml`
- [ ] `rules/cargo_build.toml`

#### Feed integration
- [ ] GHSA via GraphQL API
- [ ] Socket.dev API (requires API key)

#### Platform builds
- [ ] Windows tray app + installer
- [ ] Linux tray app + .deb/.AppImage
- [ ] Platform-specific test matrices

#### Phase 2 Gate
- [ ] All lockfile parsers pass tests with real-world fixtures
- [ ] All shims build and pass integration tests
- [ ] Windows and Linux builds compile and run
- [ ] Commit: `Phase 2: Ecosystem breadth`


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
