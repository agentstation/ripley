# Roadmap

Execution plan for Ripley. Each phase produces a usable tool at a wider scope.
Phase 1 is the MVP. See [ARCHITECTURE.md](ARCHITECTURE.md) for the full system
design, [DESIGN.md](DESIGN.md) for the design system,
[UI.md](UI.md) for view wireframes, [DECISIONS.md](DECISIONS.md) for technical
rationale, [WORKFLOW.md](WORKFLOW.md) for user workflows,
[SETTINGS.md](SETTINGS.md) for configuration reference, and
[README.md](README.md) for the project overview.

## How to use this plan

1. Start from the current milestone (see CLAUDE.md "Current work").
2. Work through tasks in the listed order — earlier tasks produce types and
   APIs that later tasks depend on.
3. Run all verification criteria before moving to the next milestone.
4. Commit after each milestone: `M{n}: {milestone name}`.
5. Update CLAUDE.md "Current work" to point to the next milestone.
6. Do not start Phase 2+ until all Phase 1 milestones pass.


## Overview

| Phase | Focus | Outcome |
|-------|-------|---------|
| **1** | Foundation (before + after) | npm guard, macOS tray, forensic scan, AI remediation |
| **2** | Ecosystem breadth (before) | All major PMs and lockfiles, Windows + Linux |
| **3** | Response depth (after) | `ripley fix`, `ripley exposure`, `ripley audit`, `ripley harden` |
| **4** | Active detection (during) | Process/filesystem monitoring, containment |
| **5** | Advanced analysis (all) | Sandboxed execution, community rules, CI/CD |
| **6** | UI rewrite (Tauri 2) | Cross-platform GUI v1: macOS + Linux + Windows; type-safe IPC; retire iced |


---


## Phase 1: Foundation

**Goal (as shipped):** A developer on macOS can install Ripley, get a tray
notification when a package they depend on is compromised, click "Fix" to
launch an AI harness with a scoped remediation prompt, and run `ripley scan
--deep` to forensically audit their machine after a breach.

**Phase 6 update:** the macOS-only constraint was a Phase 1 scoping decision,
not a product decision. Phase 6 reworks the UI on Tauri 2 to make the dashboard
cross-platform v1 (macOS + Linux + Windows) — see Phase 6 below. The CLI
(`ripley-guard`) has been cross-platform since Phase 2.

**Not in scope (Phase 1):** other lockfile formats, other PM shims,
Windows/Linux tray builds (delivered in Phase 6), `ripley fix`/`exposure`/`harden`
commands, process monitoring, sandboxing.


### M1: Core Data Pipeline

Build the data backbone: platform directories, configuration, feed client,
lockfile parser, matcher, and the `ripley scan` CLI. Everything lives in
`ripley-core` except the CLI wiring.

**0. Platform directories and configuration**
Create modules `crates/ripley-core/src/dirs.rs` and `crates/ripley-core/src/config.rs`.

- Use `directories::ProjectDirs::from("com", "agentstation", "ripley")` to
  get platform-appropriate paths. Expose: `config_dir()`, `data_dir()`,
  `cache_dir()`. Create directories on first use.
- Define a `Config` struct with `#[serde(default)]` on all fields:
  `poll_interval_secs: u64` (default 300), `harness: Option<String>`,
  `project_roots: Vec<PathBuf>`, `guard_mode: GuardMode` (strict/audit/off),
  `trust: Vec<String>`, `posture: PostureConfig`.
- Define `PostureConfig` struct: `strict: bool` (default false),
  `require_lockfile: bool`, `require_exact_versions: bool`,
  `require_integrity_hashes: bool`, `block_exotic_sources: bool`,
  `allowed_registries: Vec<String>` (default `["https://registry.npmjs.org"]`).
  See [SETTINGS.md](SETTINGS.md) for the full settings reference.
- Load config with layering: read `{config_dir}/config.toml` (user defaults),
  then `.ripley.toml` in the working directory (project overrides), then
  `RIPLEY_*` environment variables, then CLI flags. Each layer overrides
  previous values.
- Write a default `config.toml` to `{config_dir}/` on first run if none exists.
- Implement `ripley config` subcommand: `--path` prints config file location,
  `--show` prints resolved configuration (all layers merged), `--init` writes
  default config, no flag opens config in `$VISUAL`/`$EDITOR`.

**0.5. Test fixtures**
Create baseline fixtures in `tests/fixtures/` for use across M1 and M2.

- `tests/fixtures/package-lock.json` --- a realistic lockfile with a mix of
  pinned versions, range specifiers, scoped packages, and integrity hashes.
  Include at least one package that has a known OSV advisory so the matcher
  can produce results.
- `tests/fixtures/package-lock-clean.json` --- a lockfile with no known
  vulnerabilities, for testing the exit-code-0 path.
- `tests/fixtures/package-lock-risky.json` --- a lockfile with `git+` sources,
  missing integrity hashes, `http://` resolved URLs, and `latest` specifiers.
  For posture check testing.
- `tests/fixtures/malicious-postinstall.sh` --- a script exhibiting multiple
  high-risk signals: network call, pipe to shell, base64 decode, env
  harvesting. For M2 static analyzer snapshot tests.
- `tests/fixtures/benign-postinstall.sh` --- a script with normal build
  operations: `node-gyp rebuild`, file copies, `mkdir`. Should score Low.
- Fixtures are referenced by unit and integration tests. Organize by type:
  lockfiles in `tests/fixtures/`, scripts in `tests/fixtures/scripts/`,
  attack-specific fixtures in `tests/fixtures/attacks/`.

**1. OSV.dev API client**
Create module `crates/ripley-core/src/feed/osv.rs` (and `mod.rs`).

- POST to `https://api.osv.dev/v1/query` with body:
  ```json
  { "package": { "name": "express", "ecosystem": "npm" } }
  ```
- POST to `https://api.osv.dev/v1/querybatch` for bulk queries (batch of
  the above, max 1000 per request).
- Parse the response `vulns` array. Key fields per vulnerability:
  - `id` (e.g., "GHSA-xxxx-yyyy")
  - `summary`
  - `details`
  - `affected[].package.ecosystem`
  - `affected[].package.name`
  - `affected[].ranges[].events[]` — `introduced` and `fixed` versions
  - `severity[].score` (CVSS)
  - `references[].url`
- Define an `Advisory` struct in `crates/ripley-core/src/feed/mod.rs`
  (or a dedicated types module) that normalizes the OSV response into:
  `id, ecosystem, package, affected_ranges: Vec<(introduced, fixed)>,
  severity, summary, references, iocs`.
- Use `reqwest` for HTTP, `serde` for deserialization.
- **ETag caching:** Store the `ETag` response header in `{cache_dir}/feeds/`.
  On subsequent requests, send `If-None-Match`. If the API returns 304, skip
  parsing. This avoids re-downloading unchanged data on every poll cycle.
- **Exponential backoff:** On API errors (5xx, timeout, network), retry with
  exponential backoff and jitter: 1s → 2s → 4s → ... capped at 5 minutes.
  Do not hammer a failing API every 5 minutes.
- The client should accept a list of `(ecosystem, package_name)` pairs
  and return `Vec<Advisory>`.

**2. Advisory storage**
Create module `crates/ripley-core/src/db.rs`.

- Use `redb` with database at `{data_dir}/advisories.redb` (use the `dirs`
  module from task 0, not a hardcoded path).
- Table `advisories`: key = `"{ecosystem}:{package_name}"` (String),
  value = serialized `Vec<Advisory>` (JSON bytes via `serde_json`).
- Table `meta`: key = `"last_poll"`, value = timestamp (u64, unix seconds).
- Operations: `store_advisories`, `get_advisories(ecosystem, package)`,
  `get_all_advisories`, `get_last_poll`, `set_last_poll`.
- Accept a `&Path` for the database location (testable with a temp dir
  instead of the real data directory).

**3. Lockfile parser**
Create module `crates/ripley-core/src/lockfile/npm.rs` (and `mod.rs`).

- Parse `package-lock.json` lockfileVersion 2 and 3.
- Both versions store packages in the `"packages"` object. Each key is a
  path like `"node_modules/express"`. Extract the package name from the
  path (strip `node_modules/` prefix, handle scoped packages like
  `node_modules/@scope/name`).
- Return `Vec<InstalledPackage>` where `InstalledPackage` has:
  `name: String, version: semver::Version, ecosystem: Ecosystem::Npm`.
- Additionally, flag **risky dependency specifiers**: entries using `latest`,
  `*`, broad ranges, `git+` URLs, `http://` tarballs, or `file:` paths.
  These bypass the normal registry integrity chain and are common vectors
  for supply chain attacks. Return as `Vec<RiskySpec>` alongside the
  installed packages.
- **Lockfile integrity checks** (inspired by `lockfile-lint`): validate that
  resolved URLs point to expected registries (e.g., `registry.npmjs.org`),
  verify `integrity` hashes are present and use SHA-512, detect HTTP
  downgrade from HTTPS. Return integrity issues as `Vec<LockfileWarning>`.
- Define `InstalledPackage`, `RiskySpec`, and `Ecosystem` enum in the
  lockfile module (or a shared types module). `Ecosystem` should have
  variants for all supported ecosystems even though only `Npm` is
  implemented now.
- Use the fixture at `tests/fixtures/package-lock.json` for testing.

**4. Matcher**
Create module `crates/ripley-core/src/matcher.rs`.

- Input: `&[Advisory]` and `&[InstalledPackage]`.
- For each advisory, check if any installed package matches the ecosystem +
  name, and if its version falls within any affected range.
- Use `semver::VersionReq` or manual range comparison. OSV ranges use
  `introduced` (inclusive) and `fixed` (exclusive): a version `v` is
  affected if `v >= introduced && v < fixed`.
- Output: `Vec<Match>` where `Match` contains the advisory, the installed
  package info, and the project path.

**5. Wire up `ripley scan`**
In `crates/ripley-guard/src/main.rs`, implement the `Scan` command:

- Add `--format` flag: `json` or `table` (default: `table`).
- Walk the given path for lockfiles (look for `package-lock.json`).
- Parse each lockfile → `Vec<InstalledPackage>`.
- Load advisories from local DB. If DB is empty or stale (> 1 hour),
  fetch from OSV.dev first (using ETag cache).
- Run matcher → `Vec<Match>`.
- **Posture checks:** Alongside advisory matching, report security posture
  findings from the lockfile parse: risky dependency specs, missing
  integrity hashes, unexpected registry URLs, HTTP downgrade. These are
  warnings, not advisory matches — they don't affect the exit code but
  are included in both table and JSON output under a separate
  `posture_warnings` key.
- **Table output** (default): for each match, show advisory ID, package
  name, installed version, severity, and summary. Use `colored` for
  severity highlighting. Below the advisory matches, show a posture
  summary: "3 packages use range specifiers, 1 resolves from git+,
  47 entries missing integrity hashes."
- **JSON output** (`--format json`): serialize `Vec<Match>` and
  `Vec<PostureWarning>` to JSON. Machine-parseable.
- Exit codes: 0 = no matches (clean), 1 = matches found, 2 = error
  (network failure, parse error, etc.). Posture warnings alone do not
  cause exit code 1 — they are informational for open source users.
  In enterprise config, posture violations can be promoted to exit code 1
  via `[posture] strict = true`. These follow cargo-audit, trivy,
  and grype conventions.

**Verification:**
```
cargo deny check                              # own supply chain audit passes
cargo test --workspace                        # all unit tests pass
cargo run -p ripley-guard -- scan tests/fixtures/  # parses the fixture lockfile,
                                               # queries OSV, prints results
cargo run -p ripley-guard -- scan --format json tests/fixtures/  # JSON output
```

---


### M2: Guard MVP (npm)

Build the package manager interceptor: detection rules, static analysis,
npm PATH shim, and the script-shell binary. Most logic lives in
`ripley-guard`, with shared analysis code in `ripley-core`.

**1. Detection rule format and loader**
Create module `crates/ripley-core/src/rules.rs` (or `rules/mod.rs`).

- Define `Rule` struct: `id, name, description, ecosystem, signal, weight
  (medium|high|critical), patterns: Vec<String>`.
- Define `RuleSet` struct holding `Vec<Rule>`.
- **Two-tier loading:**
  1. Compiled-in defaults: `include_str!("../../../rules/npm_postinstall.toml")`
     (and any other rule files in `rules/`). These are the base rules.
  2. Runtime user rules: load `*.toml` from `{config_dir}/rules/` at startup.
     User rules are appended to the compiled-in set. If a user rule has the
     same `id` as a compiled-in rule, the user rule overrides it (allows
     tuning weights or disabling specific rules).
- This follows the pattern of ClamAV (signature updates), YARA (rule files),
  and Sigma (detection rules): ship defaults, let users extend.
- Parse with `serde` + `toml` crate.
- Expose `RuleSet::for_ecosystem(Ecosystem) -> &[Rule]`.
- The rule file at `rules/npm_postinstall.toml` already exists as a reference.

**2. Static analyzer**
Create module `crates/ripley-core/src/analyzer.rs`.

- Input: script content (string) + applicable `RuleSet`.
- For each rule, compile patterns to `regex::Regex` and check if any match
  the script content. Cache compiled regexes (compile once, match many).
- Output: `AnalysisResult` with `risk_level: RiskLevel (Low|Medium|High|
  Critical)`, `matched_rules: Vec<MatchedRule>` (rule id + matched line).
- Risk level = highest weight among matched rules. No matches = Low.
- Highlight which line(s) matched for user display.
- **Typosquatting detection:** Layered approach (informed by TypoSmart,
  SpellBound, and keyboard-adjacency research):
  1. Extended Damerau-Levenshtein distance with keyboard-adjacency
     weighting against a curated top-1000 packages list per ecosystem.
  2. Popularity-weighted scoring — flag unknown packages that would
     replace popular ones with similar names (SpellBound pattern).
  3. Homoglyph detection via Unicode normalization + confusable character
     mapping (catches visual masquerading attacks).
  The curated list ships as a compiled-in asset updated with releases.
  Flag matches as High risk. Embedding-based semantic similarity is
  Phase 2+ material.
- **AI tool config and MCP patterns:** Flag scripts that write to known
  AI coding tool configuration paths (`.claude/settings.json`,
  `.cursor/mcp.json`, `.mcp.json`, `.vscode/tasks.json`,
  `.github/copilot/`) or install MCP server definitions. The TrustFall
  attack (May 2026) and Mini Shai-Hulud variants use these paths for
  persistence and for weaponizing AI assistants as exfiltration agents
  via prompt injection in MCP tool descriptions.
- **Snapshot tests with `insta`:** Use `insta::assert_json_snapshot!` to
  test analyzer output against `tests/fixtures/malicious-postinstall.sh`
  and `tests/fixtures/benign-postinstall.sh`. Snapshot files make output
  regressions immediately visible in diffs.

**3. Script extractor**
Create module `crates/ripley-guard/src/extractor.rs`.

- Given a path to a directory containing `package.json`, extract lifecycle
  scripts: `preinstall`, `install`, `postinstall`, `prepare`, `prepack`.
- Return `Vec<Script>` with `name` (which hook), `content` (the script
  body), `source_file` (where it came from).
- For now, extract from the local `package.json` in `node_modules/`.
  Tarball extraction (fetching from registry) can be deferred.

**4. npm PATH shim**
Create binary `crates/ripley-guard/src/bin/ripley-npm-shim.rs`.

- Add `[[bin]] name = "ripley-npm-shim"` to `crates/ripley-guard/Cargo.toml`.
- On invocation: parse arguments to detect `install`/`add`/`i` commands.
- For install commands:
  1. Run advisory check against local DB for the package being installed.
  2. Warn if the package has a known advisory.
  3. Delegate to real npm (`which -a npm`, skip self, pick next).
- For non-install commands: delegate directly.
- The shim must pass through all stdout/stderr and preserve the exit code.

**5. npm script-shell binary**
Create binary `crates/ripley-guard/src/bin/ripley-script-shell.rs`.

- Add `[[bin]] name = "ripley-script-shell"` to Cargo.toml.
- npm invokes script-shell as: `ripley-script-shell -c "script content"`.
- Parse the `-c` argument to get the script content.
- Run the static analyzer on the script content.
- If risk is Low: execute the script via `/bin/sh -c "..."`.
- If risk is Medium or higher: print the analysis results with colored
  output showing matched lines, then prompt the user (`dialoguer::Confirm`)
  to allow or block. If allowed, execute via `/bin/sh`. If blocked, exit
  with code 1 (fails the install).

**6. `ripley guard install` / `uninstall`**
Implement in `crates/ripley-guard/src/main.rs` GuardCommands::Install:

- Create `{data_dir}/bin/` directory (using platform dirs, not hardcoded).
- Copy (or symlink) the `ripley-npm-shim` binary to `{data_dir}/bin/npm`.
- Copy the `ripley-script-shell` binary to `{data_dir}/bin/ripley-script-shell`.
- Detect the user's shell RC file (`.zshrc`, `.bashrc`, `.profile`).
- Append `export PATH="{data_dir}/bin:$PATH"` if not already present.
- Set `script-shell={data_dir}/bin/ripley-script-shell` in `~/.npmrc`
  (create if needed).
- All file writes must be atomic: write to temp file, then rename.
- `ripley guard install` must be idempotent (safe to run multiple times).
- Print summary of what was installed.

`GuardCommands::Uninstall`: reverse all of the above.

**7. `ripley guard trust` / `untrust` / `log` / `status`**

- **Trust list:** stored in `config.toml` under `[guard] trust = [...]`.
  Human-editable, version-controllable. Use `ripley-core`'s config module
  to read/write.
- `trust <pkg>`: add to trust list in config, write file atomically
  (write to temp file, then rename — prevents corruption on crash).
- `untrust <pkg>`: remove from trust list.
- **Guard log:** append-only JSONL file at `{data_dir}/guard.jsonl`.
  Each line is a JSON object: `{timestamp, package, version, action,
  risk_level, matched_rules}`. JSONL is inspectable with `jq`, greppable,
  and rotatable with standard log tools.
- `log`: read last 20 lines of the JSONL file, render as table.
- `status`: show which shims are installed, whether script-shell is active,
  count of trusted packages.

**Verification:**
```
cargo test --workspace                            # all unit tests pass
cargo build --workspace                           # all binaries compile

# Static analyzer tests
# Unit test: malicious fixture → High/Critical risk
# Unit test: benign fixture → Low risk

# CLI test
cargo run -p ripley-guard -- guard status         # shows "not installed"
cargo run -p ripley-guard -- guard install         # installs shims
cargo run -p ripley-guard -- guard status         # shows installed shims
cargo run -p ripley-guard -- guard trust express  # adds to trust list
cargo run -p ripley-guard -- guard log            # shows recent decisions
cargo run -p ripley-guard -- guard uninstall      # removes shims
```

---


### M3: Tray App MVP (macOS)

Build the system tray application and dashboard window. The tray uses
`tray-icon` + `muda`; the dashboard uses `iced` (Elm architecture,
GPU-rendered, retained mode). The tray app uses an **event-driven
architecture**: each component runs as a separate tokio task,
communicating via typed `mpsc` channels, coordinated by a
`CancellationToken` for graceful shutdown.

No npm, no frontend tooling, no webview runtime. Pure Rust all the way.
See [DESIGN.md](DESIGN.md) for the design system (tokens, component
styling) and [UI.md](UI.md) for wireframes and interaction specs.

**1. Workspace expansion**

- Add `crates/ripley-app/` and `crates/ripley-ipc/` to workspace members
  in root `Cargo.toml`.
- `ripley-ipc/Cargo.toml`: depends on `serde`, `serde_json`, `tokio`.
- `ripley-app/Cargo.toml`: depends on `ripley-core`, `ripley-ipc`,
  `tray-icon`, `muda`, `iced`, `notify-rust`, `tokio`, `tokio-util`,
  `tracing`, `tracing-appender`.
- Both `ripley-guard` and `ripley-app` depend on `ripley-ipc`.
- Add new workspace dependencies to root `Cargo.toml`:
  `tray-icon`, `muda`, `iced`, `notify-rust`.
- Log to file using `tracing-appender` with daily rotation in
  `{data_dir}/logs/`.

**2. IPC layer**
Create `crates/ripley-ipc/src/lib.rs`.

- Define request/response types for CLI ↔ tray app communication:
  ```rust
  enum Request {
      Status,
      Scan { path: PathBuf, deep: bool },
      GetAlerts,
  }
  enum Response {
      Status { last_poll: u64, alert_count: usize, watching: Vec<PathBuf> },
      ScanResult(Vec<Match>),
      Alerts(Vec<Alert>),
      Error(String),
  }
  ```
- Unix domain socket server (in tray app) at `{data_dir}/ripley.sock`.
- Unix domain socket client (in CLI) that connects when the socket exists,
  falls back to direct operation when the tray app isn't running.
- JSON serialization over `tokio::net::UnixStream`.
- Socket permissions: mode 0600 (owner-only access).

**3. Event channel architecture**
Create `crates/ripley-app/src/events.rs`.

- Define a typed event enum:
  ```rust
  enum AppEvent {
      AdvisoriesUpdated(Vec<Advisory>),
      LockfileChanged { path: PathBuf, packages: Vec<InstalledPackage> },
      MatchFound(Match),
      UserAction(Action),  // View, Fix, Dismiss, Contain
      IpcRequest(Request, oneshot::Sender<Response>),
  }
  ```
- Create an `mpsc::channel<AppEvent>` shared by all components.
- The main loop receives events and dispatches: `MatchFound` → notification,
  `UserAction::Fix` → prompt generator → harness launcher,
  `IpcRequest` → handle and respond via oneshot channel.
- All components take a `CancellationToken` from `tokio_util`. On "Quit"
  menu action or `SIGTERM`: cancel the token, which causes all tasks to
  exit cleanly (close redb, flush logs, remove socket file).
- No shared mutable state between components. All communication via channels.

**4. System tray**
Create `crates/ripley-app/src/tray.rs`.

- Use `tray-icon` crate to create a system tray icon.
- Use `muda` crate for the context menu: Show Dashboard, Scan Now, Quit.
- Single-click on tray icon → open/focus the dashboard window.
- Tray icon variants reflect status: shield outline (idle), shield with
  dot (alerts), shield with ! (critical). Use template images on macOS.
- "Quit" cancels the `CancellationToken`, triggering graceful shutdown.
- macOS: run the event loop on the main thread (required by AppKit).
- Bridge tray events to `iced` via channel (see UI.md "Integration
  with `tray-icon`").

**5. Background feed poller**

- Spawn as a tokio task with a clone of the event sender and the
  cancellation token.
- Poll OSV.dev on the configured interval (from `config.toml`).
  Use ETag caching and exponential backoff (from M1 feed client).
- On new advisories: send `AdvisoriesUpdated` event. The main loop
  re-runs the matcher against the current lockfile index.
- On cancellation: exit the loop cleanly.

**6. Lockfile watcher**

- Spawn as a tokio task.
- On launch, walk configured project roots and index all lockfiles
  (in-memory index, not persisted).
- Use `notify` v8 to watch for changes. On change, re-parse and
  send `LockfileChanged` event.
- The main loop re-runs the matcher against cached advisories.

**7. Notifications**

- Use `notify-rust` crate for native OS notifications.
- On macOS: UNUserNotificationCenter.
- When a new match is found (not previously alerted), fire a notification.
- Notification shows: package name, version, advisory summary.
- Notification click opens the dashboard window to the relevant alert.
- Three notification variants by phase (see UI.md "Notifications"):
  Before: [Fix] [View] [Dismiss]. During: [Contain] [View] [Investigate].
  After: [Remediate] [View Report].

**8. Dashboard window**
Create `crates/ripley-app/src/app.rs`, `theme.rs`, and `views/`.

- Implement `iced::Application` with sidebar navigation (Alerts, Guard
  Log, Settings) and a main content area.
- **Theme:** implement the color system and severity scale from DESIGN.md/UI.md
  as a custom `iced::Theme`. Dark palette, system fonts, 4px grid.
- **Alerts view:** list current findings sorted by severity. Each row
  shows severity badge, package@version, advisory ID, project path,
  time, and action button ([Fix] or [Dismiss]).
- **Alert detail panel:** slide-in panel showing full advisory info,
  affected projects, IOC files, and action buttons (Fix with harness
  selection, View Advisory, Copy Prompt).
- **Guard log view:** table of recent interceptions from `guard.jsonl`.
  Expandable rows showing matched rules and script excerpts.
- **Settings view:** form that reads/writes `config.toml` — poll
  interval, harness preference, project roots, guard mode, trust list.
  Saves immediately, writes atomically.
- **Guard interception dialog:** when script-shell sends a `GuardPrompt`
  IPC request, show a modal dialog with script content, highlighted
  matched lines, matched rules, and [Allow Once] [Block] [Always Trust]
  [Inspect] buttons. 30-second timeout defaults to Block.
- Window default size: 900x640, minimum: 720x480.
- Closing the window hides it (does not quit the app).
- See UI.md for wireframes and DESIGN.md for component specifications.

**9. macOS app bundling**

- Use `cargo-bundle` to package `ripley-app` as `Ripley.app`.
- `Info.plist`: set `LSUIElement = true` (tray-only, no Dock icon).
- Include app icon in `crates/ripley-app/icons/`.
- Auto-start via LaunchAgent plist (installed by `ripley guard install`
  or a separate `ripley app install` command).

**10. Headless daemon (`ripley watch`)**

- Implement the `Watch` subcommand in `crates/ripley-guard/src/main.rs`.
- Start the same background components as the tray app: feed poller,
  lockfile watcher, matcher, IPC server. Use the same event channel
  architecture (`AppEvent` + `mpsc`) and `CancellationToken`.
- Fire native OS notifications via `notify-rust` (works without a GUI).
- Create the IPC socket at `{data_dir}/ripley.sock` so `ripley status`
  and other CLI commands can query state.
- `--daemon` flag: fork to background and log to `{data_dir}/logs/`.
  Without the flag, run in foreground logging to stdout.
- Respond to SIGTERM/SIGINT for graceful shutdown (cancel token, close
  redb, remove socket file).
- No `iced`, no `tray-icon`, no `muda` dependency. The `Watch` command
  lives in `ripley-guard`, not `ripley-app`.

**Verification:**
```
cargo build --workspace                    # all crates compile
cargo test --workspace                     # all tests pass
cargo bundle --release -p ripley-app       # produces Ripley.app
# Manual: launch Ripley.app, verify tray icon appears with menu
# Manual: wait for poll interval, verify notification fires (if any
#         lockfile matches an advisory)
# Manual: run `ripley status`, verify IPC with tray app via socket
# Manual: run `ripley watch`, verify headless daemon starts, creates
#         socket, polls feeds, and responds to `ripley status`
# Manual: Ctrl-C `ripley watch`, verify graceful shutdown (socket removed)
```

---


### M4: Remediation Pipeline

Wire the "Fix" button: prompt generation and AI harness launching.

**1. Prompt generator**
Create module `crates/ripley-core/src/prompt.rs`.

- Input: a `Match` (advisory + installed package + project path).
- Output: a self-contained remediation prompt string.
- Template includes:
  - Project path
  - Package name and compromised version
  - CVE / advisory ID
  - Clean version to upgrade to (from advisory `fixed` field)
  - IOC file paths to check (from advisory or rule DB)
  - Credential exposure hints (if available)
  - Instruction to run tests after fix
- See ARCHITECTURE.md "Prompt generator" section for the example format.

**2. Harness launcher**
Create module `crates/ripley-core/src/harness.rs`.

- Detect available harnesses by checking PATH for: `claude`, `codex`,
  `opencode` (in that order, or user-configured preference).
- Spawn the harness in a new terminal window:
  - macOS: `open -a Terminal.app` or `osascript` to open a new terminal
    tab, then run the harness command.
  - The prompt is passed via the harness's prompt flag (`claude -p "..."`,
    `codex -q "..."`, etc.).
- Return the process handle (don't wait for completion).

**3. Wire "Fix" action**

- In the tray app: when "Fix" is clicked on a notification, call the
  prompt generator with the match data, then call the harness launcher.
- In the CLI: `ripley scan` could offer a `--fix` flag that generates
  prompts for all matches and launches the harness for each.

**Verification:**
```
# Unit test: prompt generator produces expected format for a test Match
# Unit test: harness launcher detects `claude` in PATH (mock or real)
cargo test --workspace

# Integration test: run `ripley scan --fix tests/fixtures/` and verify
# it generates a prompt (even if no harness is available, print the
# prompt to stdout as fallback)
```

---


### M5: Forensic Scan

Build `ripley scan --deep` — the post-breach audit.

**1. IOC scanner**
Create module `crates/ripley-core/src/forensic/ioc.rs` (and `mod.rs`).

- Define a list of IOC file patterns to check, sourced from known attacks:
  - `.claude/execution.js`, `.claude/setup.mjs`, `.claude/settings.json`
    (check for unexpected `hooks.SessionStart`)
  - `.vscode/tasks.json` (check for `runOn: folderOpen` entries not in git)
  - `.mcp.json`, `.cursor/mcp.json`, `.github/copilot/` (check for
    rogue MCP server definitions or prompt injection in tool descriptions)
  - `node_modules/@tanstack/*/router_init.js`
  - `node_modules/.cache/` unexpected executables
  - `~/.ssh/authorized_keys` (unexpected keys)
- Walk the given path and home directory, check for each IOC.
- **IOC profiles:** Support modular, incident-specific detection profiles
  stored as TOML files in `{config_dir}/iocs/`. Each profile contains:
  known-bad package versions, payload filenames, persistence paths, and
  workflow markers for a specific attack (e.g., `tanstack-2026-05.toml`,
  `mini-shai-hulud.toml`). Base profiles ship compiled-in; users can add
  custom profiles for emerging incidents before an official release.
- Return `Vec<IocFinding>` with path, description, severity, and the
  source profile that triggered the finding.

**2. Persistence auditor**
Create module `crates/ripley-core/src/forensic/persistence.rs`.

- Check macOS persistence mechanisms:
  - `~/Library/LaunchAgents/*.plist` — flag any not matching known-good list
  - `crontab -l` — parse and flag suspicious entries
  - Shell RC files (`~/.zshrc`, `~/.bashrc`, `~/.profile`) — flag lines
    that curl/wget, eval, or reference suspicious domains
- **MCP and AI tool config audit:** Check MCP config files (`.mcp.json`,
  `.cursor/mcp.json`, `.github/copilot/`) for rogue server definitions
  that weren't present before a suspected breach. Check MCP tool
  descriptions for embedded prompt injection payloads that instruct the
  AI to read sensitive files. Check `.claude/settings.json` for
  unauthorized hooks, environment variables, or `enableAllProjectMcpServers`.
  (TrustFall May 2026, MCP injection campaigns)
- **Dead man switch detection:** Check for processes or scripts that poll
  external APIs (e.g., GitHub API) to monitor credential validity. Mini
  Shai-Hulud installed a dead man switch that ran `rm -rf ~` when the
  victim's GitHub token was revoked. The deep scan should detect this
  pattern and warn users to **back up before rotating credentials**.
- Return `Vec<PersistenceFinding>`.

**3. Credential exposure mapper**
Create module `crates/ripley-core/src/forensic/credentials.rs`.

- Given a list of advisory IDs (or attack names), map to which credential
  stores that attack is known to target.
- Check if those credential files exist on this machine.
- Output: which credentials are at risk, with rotation instructions
  (e.g., "Rotate npm token: `npm token revoke && npm login`").

**4. Deep scan CLI**
Wire up in `crates/ripley-guard/src/main.rs` for `Scan { deep: true }`:

- Run the lockfile scan (from M1).
- Run the IOC scanner.
- Run the persistence auditor.
- Run the credential exposure mapper.
- Print a structured report: summary, findings by category, recommended
  actions.
- Also generate a remediation prompt if findings exist (reuse M4 prompt
  generator).

**Verification:**
```
cargo test --workspace

# Test with a clean machine — should report "no findings"
cargo run -p ripley-guard -- scan --deep ~

# Test with fixtures — create a temp dir with IOC files planted,
# run deep scan against it, verify findings are reported
```


---


## Phase 2: Ecosystem Breadth

**Goal:** Ripley guards all major package managers and runs on all three
desktop platforms. The lockfile index covers every ecosystem, and the guard
intercepts installs across the full developer stack.

**Depends on:** Phase 1 complete.

### Deliverables

**Lockfile parsers** (add to `crates/ripley-core/src/lockfile/`):
- `yarn.rs` — `yarn.lock` (v1 classic + v2/Berry)
- `pnpm.rs` — `pnpm-lock.yaml`
- `pip.rs` — `Pipfile.lock`, `poetry.lock`
- `cargo.rs` — `Cargo.lock`
- `go.rs` — `go.sum`
- `gem.rs` — `Gemfile.lock`

**Guard shims** (add `[[bin]]` entries in ripley-guard):
- `ripley-pip-shim` — intercept `pip install`, analyze `setup.py` / build backends
- `ripley-cargo-shim` — intercept `cargo build`/`install`, analyze `build.rs`
- `ripley-gem-shim` — shim + Rubygems `pre_install` hook
- `ripley-go-shim` — advisory check only (Go has no install scripts)
- `ripley-yarn-shim` — Yarn PnP-aware lifecycle handling
- `ripley-pnpm-shim` — shim + `script-shell` in `.npmrc` (same trick as npm)

**Detection rules** (add to `rules/`):
- `pypi_setup.toml` — patterns for malicious `setup.py` / `.pth` files
- `cargo_build.toml` — patterns for malicious `build.rs`
- `credential_exfil.toml` — cross-ecosystem credential theft patterns

**Feed integration:**
- GitHub Advisory Database (GHSA) via GraphQL API
- Socket.dev API for real-time malicious package alerts

**Platform builds:**
- Windows tray app (`tray-icon` + `muda` + Windows notification API)
- Linux tray app (`tray-icon` + `muda` + D-Bus notifications)
- Platform-specific installers (`.dmg`, `.msi`, `.deb`/`.AppImage`)


---


## Phase 3: Response Depth

**Goal:** After a breach is discovered, Ripley provides structured
remediation: specific fix commands, credential rotation checklists, and
hardening recommendations tailored to the attack that just hit.

**Depends on:** Phase 1 M4 and M5 (prompt generator + forensic scan).

### Deliverables

- `ripley fix <cve> [path]` — generate and launch a remediation prompt for
  a specific CVE. If no path given, fix all affected projects.
- `ripley exposure <cve>` — credential exposure assessment for a specific
  attack. Which stores were targeted, which exist on this machine, rotation
  commands for each.
- `ripley audit` — developer environment security audit. Checks the machine
  itself, not just packages. Programmatic checks collect data deterministically;
  the AI analysis layer interprets it contextually and generates environment-
  specific fix commands via the harness.

  **Machine security:**
  - Disk encryption (FileVault on macOS, LUKS on Linux, BitLocker on Windows)
  - Firewall enabled and configured
  - OS security updates current
  - Screen lock timeout ≤ 5 minutes
  - Remote login / SSH server disabled unless intentional

  **Developer toolchain:**
  - Git commit signing configured (SSH or GPG keys)
  - SSH keys use strong algorithms (Ed25519 over RSA-1024)
  - SSH keys are passphrase-protected
  - Shell RC files clean of injected `eval`/`curl`/suspicious domains
  - Shell history permissions (not world-readable)

  **AI tool config integrity:**
  - MCP configs (`.mcp.json`, `.cursor/mcp.json`) free of rogue server
    definitions and prompt injection in tool descriptions
  - `.claude/settings.json` hooks authorized (no unexpected SessionStart)
  - VS Code extensions from verified publishers
  - No unauthorized `enableAllProjectMcpServers` settings

  **Credential exposure:**
  - No secrets in shell history or environment variables
  - No `.env` files committed to git
  - npm tokens in `.npmrc` scoped narrowly with expiry
  - SSH agent forwarding disabled where not needed
  - No plaintext tokens in shell RC files
  - 2FA enabled on registry accounts (npm, PyPI)

  **AI-powered analysis:** `ripley audit --fix` structures all findings into
  a remediation prompt and hands it to the configured AI harness. The AI
  generates environment-specific fix commands (tailored to macOS version,
  shell, PM version), prioritizes by actual risk considering the combination
  of findings (disabled disk encryption + unencrypted npm tokens = critical),
  and explains why each finding matters in the context of supply chain defense.

  **Output:** Traffic-light summary per category (green/yellow/red). `--format
  json` for CI integration. `--fix` to generate and launch AI remediation.
  For teams: enterprise config can mandate minimum audit scores per category.

- `ripley harden` — PM-specific hardening recommendations. A focused subset of
  `ripley audit` that detects the active package manager and recommends
  PM-specific settings, normalizing fragmented setting names:

  **Dependency pinning:**
  - Exact versions: `npm config set save-exact true` / pnpm `savePrefix: ""`
  - Lockfile committed to git
  - Integrity hashes present (SHA-512)
  - No exotic sources (git+, http, file:)

  **PM hardening:**
  - Release-age gating: `minimumReleaseAge` (pnpm) / `minReleaseAge` (npm
    .npmrc) / equivalent (Yarn `npmMinimalAgeGate`, Bun `minReleaseAge`)
  - Script execution: `ignore-scripts` + allowlist, or Ripley's guard
  - Exotic subdeps: pnpm `blockExoticSubdeps: true`
  - Trust policy: pnpm `trustPolicy: no-downgrade`
  - pnpm `allowBuilds` map for explicit script approval

  **Provenance:**
  - npm Trusted Publishing enabled for your own packages
  - Provenance drop detection across dependency updates
  - Publisher change alerts

  **Credential hygiene:**
  - npm tokens in `.npmrc`: scoped vs broad, expiry set
  - Tokens accidentally committed to project directories
  - 2FA enabled on registry accounts

  **Output:** Traffic-light summary per category (green/yellow/red) with
  specific commands to run for each recommendation. `--format json` for
  CI integration. For teams: enterprise config can mandate minimum posture
  scores per category.
- Remediation templates: structured per-attack playbooks for common patterns
  (npm worm, PyPI `.pth` injection, credential exfiltration, IDE config
  poisoning). Each template: IOC-specific file checks, credential rotation
  checklists, verification steps.
- Network connection audit against known C2 IP/domain databases.


---


## Phase 4: Active Detection

**Goal:** Ripley monitors the developer's machine in real time for signs of
active compromise: unexpected network connections, unauthorized writes to
sensitive paths, and C2 communication. When detected, containment is one
click away.

**Depends on:** Phase 1 M3 (tray app + notifications).

### Deliverables

- `ripley monitor` — background daemon watching for IOC patterns:
  - Outbound connections from Node, Python, Ruby, Go processes
  - Unauthorized writes to `.claude/`, `.vscode/`, shell RCs, LaunchAgents,
    cron, systemd
  - Processes masquerading as system services
  - Connections to known C2 infrastructure (IP + domain matching)
- `ripley contain <pid|pkg>` — kill process, snapshot state (open files,
  network connections, environment) for forensic analysis.
- Real-time high-priority notifications with [View] [Contain] [Investigate]
  actions.
- Filesystem anomaly detection: new files in `.claude/` or `.vscode/` not
  in git, lockfile edits outside explicit install, unexpected binaries in
  `node_modules/.cache/`.


---


## Phase 5: Advanced Analysis

**Goal:** Move from static rule matching to behavioral analysis. Enable
community-driven rule development. Ship CI/CD integrations for teams.

**Depends on:** Phase 2 (multi-PM support) for full CI value.

### Deliverables

- **Sandboxed script execution:**
  - macOS `sandbox-exec` profiles restricting network + filesystem
  - Linux bubblewrap (`bwrap`) namespaces with network isolation
  - Run scripts with network disabled, observe filesystem + process activity
  - Compare observed vs. declared behavior
- **Behavioral analysis engine:**
  - Record filesystem writes, network attempts, process spawns
  - Flag anomalies (CSS library reading `~/.ssh/`)
- **Community rule sharing:**
  - Publish/subscribe detection rules to a public registry
  - Versioned TOML with metadata (author, confidence, source attack)
- **CI/CD integration:**
  - GitHub Action: `ripley-guard` as a build step
  - GitLab CI template
  - `ripley guard --ci` with SARIF output for code scanning dashboards


---


## Phase 6: UI Rewrite (Tauri 2)

**Goal:** Replace the iced-based `crates/ripley-app` (Phase 1-5) with a Tauri 2
desktop app delivering cross-platform tray + dashboard + guard dialog on
macOS + Linux + Windows. Type-safe IPC end-to-end via `tauri-specta`. shadcn/ui
(Base UI primitive) + Tailwind v4 views matching DESIGN.md tokens. Retire iced.

**Depends on:** Phase 1 M3 (existing UDS IPC protocol, view inventory),
[DESIGN_ISSUES.md](DESIGN_ISSUES.md) (priority-ordered view migration list).

**Stack:** locked in [STACK_DECISION.md](STACK_DECISION.md). Tauri 2.11 +
React 19.2 + shadcn/ui (`@base-ui/react` 1.x, style `base-vega`) + Tailwind v4
+ Zustand 5 + TanStack Query v5 + `tauri-specta` v2. pnpm + Vite 8 + TypeScript 6.
ESLint 9 flat config + Prettier 3 + Vitest 2 + WebdriverIO 9. Lefthook +
release-plz + just + mise.

### Deliverables

- **M24 — Tauri scaffold & shell:**
  - `apps/desktop/` Tauri 2 scaffold + Vite + React 19 + pnpm + TS 6
  - Root `package.json` (private), `pnpm-workspace.yaml`, `justfile`,
    `lefthook.yml`, `release-plz.toml`, `.mise.toml`
  - shadcn init (`--base base-ui`, style `base-vega`) + Tailwind v4 with
    DESIGN.md tokens in `theme.css`
  - `tauri-specta` v2 codegen in `build.rs` → committed `bindings.ts`
  - `TrayIconBuilder` on all 3 OSes; hidden pre-warm window; `set_activation_policy(.Accessory)` on macOS
  - Cmd/Ctrl+Shift+R global shortcut shows the pre-warm window; Cmd/Ctrl+K reserved for the in-window command palette (wired in M27.5)
  - 3 placeholder typed `tauri::command` invokes wired via TanStack Query
  - CI matrix (macOS/Linux/Windows) builds + smoke-test green
- **M25 — Guard-dialog critical path:**
  - Bridge existing `ripley-script-shell` UDS protocol into Tauri events
  - shadcn `Dialog` on Base UI renders prompt; Allow/Block/Trust roundtrip
  - Pre-warm tuning until <500ms first show + <50ms warm on all 3 OSes
  - Latency instrumented; insta snapshot of prompt text
- **M26 — Cross-platform parity:**
  - Linux primary: Ubuntu LTS + Fedora; Arch/Hyprland documented
  - Windows: Win10 22H2+ (WebView2 evergreen + bootstrap fallback)
  - Tray verified: GNOME (extension docs), KDE, Hyprland, Win 11
  - Tauri bundler: `.dmg` + `.app`, `.msi` + `.exe`, AppImage + `.deb` + `.rpm`
  - WebdriverIO + `tauri-driver` e2e on Linux + Windows CI runners
- **M27 — View migration (priority-ordered per DESIGN_ISSUES.md):**
  - Alerts + `AlertCard` + `SeverityBadge`; Guard Log virtualized DataTable
  - Deep Scan, Monitor, Audit, Posture, Settings views
  - Command palette: Base UI `Combobox` + `match-sorter` + recency (no cmdk)
  - All DESIGN.md tokens consumed; no `#[allow(dead_code)]` remaining
- **M28 — Release ops & retire iced:**
  - macOS notarization (Developer ID hardened runtime), Windows code signing
  - Tauri updater with Ed25519 signed manifests
  - `release-plz` Rust automation; JS supply chain audit (`pnpm audit`) in CI
  - Delete `crates/ripley-app`; drop iced/tray-icon/muda/cargo-bundle from `Cargo.toml`
  - Per-platform install docs
