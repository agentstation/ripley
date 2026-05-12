# Roadmap

Execution plan for Ripley. Each phase produces a usable tool at a wider scope.
Phase 1 is the MVP. See [ARCHITECTURE.md](ARCHITECTURE.md) for the full system
design and [README.md](README.md) for the project overview.

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
| **3** | Response depth (after) | `ripley fix`, `ripley exposure`, `ripley harden` |
| **4** | Active detection (during) | Process/filesystem monitoring, containment |
| **5** | Advanced analysis (all) | Sandboxed execution, community rules, CI/CD |


---


## Phase 1: Foundation

**Goal:** A developer on macOS can install Ripley, get a tray notification when
a package they depend on is compromised, click "Fix" to launch an AI harness
with a scoped remediation prompt, and run `ripley scan --deep` to forensically
audit their machine after a breach.

**Not in scope:** other lockfile formats, other PM shims, Windows/Linux tray
builds, `ripley fix`/`exposure`/`harden` commands, process monitoring,
sandboxing.


### M1: Core Data Pipeline

Build the data backbone: fetch advisories, parse lockfiles, match them.
Everything lives in `ripley-core`.

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
- The client should accept a list of `(ecosystem, package_name)` pairs
  and return `Vec<Advisory>`.

**2. Advisory storage**
Create module `crates/ripley-core/src/db.rs`.

- Use `redb` with database path `~/.ripley/advisories.redb`.
- Table `advisories`: key = `"{ecosystem}:{package_name}"` (String),
  value = serialized `Vec<Advisory>` (JSON bytes via `serde_json`).
- Table `meta`: key = `"last_poll"`, value = timestamp (u64, unix seconds).
- Operations: `store_advisories`, `get_advisories(ecosystem, package)`,
  `get_all_advisories`, `get_last_poll`, `set_last_poll`.
- Accept a `&Path` for the database location (testable without touching
  `~/.ripley/`).

**3. Lockfile parser**
Create module `crates/ripley-core/src/lockfile/npm.rs` (and `mod.rs`).

- Parse `package-lock.json` lockfileVersion 2 and 3.
- Both versions store packages in the `"packages"` object. Each key is a
  path like `"node_modules/express"`. Extract the package name from the
  path (strip `node_modules/` prefix, handle scoped packages like
  `node_modules/@scope/name`).
- Return `Vec<InstalledPackage>` where `InstalledPackage` has:
  `name: String, version: semver::Version, ecosystem: Ecosystem::Npm`.
- Define `InstalledPackage` and `Ecosystem` enum in the lockfile module
  (or a shared types module). `Ecosystem` should have variants for all
  supported ecosystems even though only `Npm` is implemented now.
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

- Walk the given path for lockfiles (look for `package-lock.json`).
- Parse each lockfile → `Vec<InstalledPackage>`.
- Load advisories from local DB. If DB is empty or stale (> 1 hour),
  fetch from OSV.dev first.
- Run matcher → `Vec<Match>`.
- Print results: for each match, show the advisory ID, package name,
  installed version, severity, and summary.
- Exit code 0 if no matches, 1 if matches found.

**Verification:**
```
cargo test --workspace                        # all unit tests pass
cargo run -p ripley-guard -- scan tests/fixtures/  # parses the fixture lockfile,
                                               # queries OSV, prints results
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
- Load rules from TOML using `include_str!("../../../rules/npm_postinstall.toml")`
  at compile time (adjust path as needed).
- Parse with `serde` (add `toml` to workspace dependencies if needed).
- Expose `RuleSet::for_ecosystem(Ecosystem) -> &[Rule]`.
- The rule file at `rules/npm_postinstall.toml` already exists as a reference.

**2. Static analyzer**
Create module `crates/ripley-core/src/analyzer.rs`.

- Input: script content (string) + applicable `RuleSet`.
- For each rule, compile patterns to regex and check if any match the
  script content.
- Output: `AnalysisResult` with `risk_level: RiskLevel (Low|Medium|High|
  Critical)`, `matched_rules: Vec<MatchedRule>` (rule id + matched line).
- Risk level = highest weight among matched rules. No matches = Low.
- Highlight which line(s) matched for user display.

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

- Create `~/.ripley/bin/` directory.
- Copy (or symlink) the `ripley-npm-shim` binary to `~/.ripley/bin/npm`.
- Copy the `ripley-script-shell` binary to `~/.ripley/bin/ripley-script-shell`.
- Detect the user's shell RC file (`.zshrc`, `.bashrc`, `.profile`).
- Append `export PATH="$HOME/.ripley/bin:$PATH"` if not already present.
- Set `script-shell=/Users/<user>/.ripley/bin/ripley-script-shell` in
  `~/.npmrc` (create if needed).
- Print summary of what was installed.

`GuardCommands::Uninstall`: reverse all of the above.

**7. `ripley guard trust` / `untrust` / `log` / `status`**

- Trust list: store in redb table `trust` in the same database.
  Key = package name/scope, value = timestamp when trusted.
- `trust <pkg>`: add to trust list.
- `untrust <pkg>`: remove from trust list.
- `log`: store recent guard decisions (allow/block/trust) in a redb table
  `guard_log`. Show last 20 entries.
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

Build the Tauri v2 system tray application. This is the first time the
project needs npm/frontend tooling.

**1. Tauri scaffolding**

- Install tauri-cli: `cargo install tauri-cli --version "^2"`.
- Add `src-tauri` to workspace members in root `Cargo.toml`.
- Create `src-tauri/` with `Cargo.toml`, `tauri.conf.json`, and `src/main.rs`.
- Create `frontend/` with minimal `index.html` (the alert detail panel UI
  is built here later).
- In `tauri.conf.json`:
  - Set `"windows": []` for tray-only mode (no window on launch).
  - Configure the system tray with a menu.
- Verify: `cargo tauri dev` launches a tray icon with a menu.
- `src-tauri/Cargo.toml` should depend on `ripley-core = { workspace = true }`.

**2. System tray menu**

- Menu items: Status (shows last poll time + alert count), Alerts
  (submenu of recent alerts), Configure, Quit.
- "Configure" opens a webview window for settings.
- "Quit" exits the app.

**3. Background feed poller**

- On launch, start a tokio task that polls OSV.dev on an interval
  (default 5 minutes, configurable).
- On each poll: fetch advisories via `ripley-core` feed client, store
  in redb, run matcher against lockfile index, generate alerts for
  new matches.
- Update the tray icon badge / tooltip with alert count.

**4. Lockfile watcher**

- On launch, walk configured project roots and index all lockfiles.
- Use `notify` crate to watch for changes. On change, re-parse and
  re-index the affected lockfile.
- Re-run matcher against cached advisories when lockfiles change.

**5. Notifications**

- When a new match is found (not previously alerted), fire a native
  macOS notification via Tauri's notification API.
- Notification shows: package name, version, advisory summary.
- Actions: [View] opens the detail panel webview, [Fix] triggers the
  prompt generator + harness launcher, [Dismiss] marks as seen.

**6. Alert detail panel**

- Create a simple frontend page (HTML + minimal JS, no framework needed)
  that displays full advisory details: CVE ID, affected package, installed
  version, severity, summary, IOC checklist, affected projects on this
  machine.
- Opened by "View" action or tray menu → Alerts.
- Communicates with Rust backend via Tauri's IPC (`invoke`).

**Verification:**
```
cargo tauri build --debug                  # builds the .app bundle
# Manual: launch the app, verify tray icon appears
# Manual: wait for poll interval, verify notification fires (if any
#         lockfile matches an advisory)
# Manual: click View, verify detail panel opens
cargo test --workspace                     # all tests still pass
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
  - `node_modules/@tanstack/*/router_init.js`
  - `node_modules/.cache/` unexpected executables
  - `~/.ssh/authorized_keys` (unexpected keys)
- Walk the given path and home directory, check for each IOC.
- Return `Vec<IocFinding>` with path, description, severity.

**2. Persistence auditor**
Create module `crates/ripley-core/src/forensic/persistence.rs`.

- Check macOS persistence mechanisms:
  - `~/Library/LaunchAgents/*.plist` — flag any not matching known-good list
  - `crontab -l` — parse and flag suspicious entries
  - Shell RC files (`~/.zshrc`, `~/.bashrc`, `~/.profile`) — flag lines
    that curl/wget, eval, or reference suspicious domains
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
- Windows tray app (Tauri v2 + Windows notification API)
- Linux tray app (Tauri v2 + D-Bus notifications)
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
- `ripley harden` — post-incident hardening recommendations based on recent
  alerts: guard rules to tighten, paths to monitor, domains to block.
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
