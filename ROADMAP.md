# Roadmap

The execution plan for Ripley. Each phase produces a usable tool at a wider
scope. Phase 1 is the MVP. See [ARCHITECTURE.md](ARCHITECTURE.md) for the full
system design and [README.md](README.md) for the project overview.


## Overview

| Phase | Focus | What ships |
|-------|-------|-----------|
| **1** | Foundation (before + after) | npm guard, macOS tray, forensic scan, AI remediation |
| **2** | Ecosystem breadth (before) | All major PMs and lockfiles, Windows + Linux |
| **3** | Response depth (after) | `ripley fix`, `ripley exposure`, `ripley harden` |
| **4** | Active detection (during) | Process/filesystem monitoring, containment |
| **5** | Advanced analysis (all) | Sandboxed execution, community rules, CI/CD |


---


## Phase 1: Foundation

**Goal:** A developer on macOS can install Ripley, get a tray notification when a package
they depend on is compromised, click "Fix" to launch an AI harness with a scoped
remediation prompt, and run `ripley scan --deep` to forensically audit their machine after
a breach.

### Milestones

**M1: Workspace + core data pipeline**
- Cargo virtual workspace scaffolding (`ripley-core`, `ripley-guard`, `src-tauri`)
- OSV.dev API client with feed polling and response normalization
- `redb` local advisory cache (ecosystem, package, affected versions, severity, CVE, IOCs)
- `package-lock.json` parser → in-memory lockfile index
- Matcher: version range intersection between advisories and installed packages
- `ripley scan [path]` --- one-shot CLI: parse lockfiles, check against cached advisories

**M2: Guard MVP (npm)**
- TOML detection rules under `rules/` embedded via `include_str!` at build time
- Static analyzer / risk scorer consuming compiled rules
- npm PATH shim binary (`~/.ripley/bin/npm`): advisory check pre-install, delegate to real npm
- npm `script-shell` binary (`~/.ripley/bin/ripley-script-shell`): receive lifecycle scripts,
  run static analysis, allow/prompt/block based on risk score
- Script extraction from registry tarballs (parse `package.json` lifecycle hooks, follow
  file references)
- `ripley guard install` / `ripley guard uninstall` (PATH + `.npmrc` setup)
- `ripley guard trust` / `ripley guard untrust` / `ripley guard log`
- CI mode: non-interactive strict/audit/off via `.ripley.toml`

**M3: Tray app MVP (macOS)**
- Tauri v2 tray-only app (no window at launch, `"windows": []`)
- System tray icon + menu (status, recent alerts, configure, quit)
- Background feed polling loop (configurable interval, default 5 min)
- Lockfile watcher via `notify` crate → re-index on change
- Match → native macOS notification with [View] [Fix] [Dismiss] actions
- Alert detail panel (webview, opens on "View"): advisory info, affected projects, IOC list
- `ripley status` CLI: show monitored projects, last poll time, alert count

**M4: Remediation pipeline**
- Prompt generator: template engine consuming advisory data + lockfile context
  (CVE, affected version, project path, IOC file paths, clean version, test command)
- Harness launcher: detect `claude` / `codex` / `opencode` in PATH, spawn in new terminal
  with generated prompt
- Wire "Fix" notification action → prompt generator → harness launcher
- `ripley config` for harness preference and project root configuration

**M5: Forensic scan**
- `ripley scan --deep [path]` implementation:
  - Installed package versions vs. known-compromised version lists
  - IOC file search (persistence markers, exfil staging, dropped binaries) from rule DB
  - Shell RC integrity review (`.bashrc`, `.zshrc`, `.profile` for injected commands)
  - Persistence mechanism audit (LaunchAgents, cron, systemd units, login items)
  - SSH key audit (unexpected keys in `~/.ssh/authorized_keys`)
  - Active network connection scan against known C2 infrastructure
- Credential exposure mapping: based on attack-specific behavior, report which credential
  stores were likely accessed (AWS, GitHub, npm, SSH, `.env`, k8s)
- Report output: structured JSON + human-readable summary
- Feed results into prompt generator for "Remediate" flow

### Not in scope for Phase 1
- Any lockfile format other than `package-lock.json`
- Any package manager other than npm
- Windows or Linux tray builds
- `ripley fix`, `ripley exposure`, `ripley harden` commands (Phase 3)
- Process monitoring or runtime detection (Phase 4)
- Sandboxed script execution (Phase 5)


---


## Phase 2: Ecosystem Breadth

**Goal:** Ripley guards all major package managers and runs on all three desktop platforms.
The lockfile index covers every ecosystem, and the guard intercepts installs across the
full stack.

**Depends on:** Phase 1 complete.

### Deliverables

**Lockfile parsers:**
- `yarn.lock` (v1 + Berry)
- `pnpm-lock.yaml`
- `Pipfile.lock`, `poetry.lock`
- `Cargo.lock`
- `go.sum`
- `Gemfile.lock`

**Guard shims:**
- pip (`~/.ripley/bin/pip`): intercept install, analyze `setup.py` / build backend scripts
- cargo (`~/.ripley/bin/cargo`): intercept build/install, analyze `build.rs` and proc macros
- gem (`~/.ripley/bin/gem`): shim + `pre_install` Rubygems hook
- go (`~/.ripley/bin/go`): advisory check only (no install scripts)
- yarn (`~/.ripley/bin/yarn`): shim with Yarn PnP-aware lifecycle handling
- pnpm (`~/.ripley/bin/pnpm`): shim + `script-shell` in `.npmrc`

**Feed integration:**
- GitHub Advisory Database (GHSA) via GraphQL API
- Socket.dev API for real-time malicious package alerts

**Platform builds:**
- Windows tray app (Tauri v2 + Windows notification API)
- Linux tray app (Tauri v2 + D-Bus notifications)
- Platform-specific installers (`.dmg`, `.msi`, `.deb`/`.rpm`/`.AppImage`)

### Not in scope for Phase 2
- `ripley fix` or other after-phase CLI commands (Phase 3)
- Any runtime monitoring (Phase 4)


---


## Phase 3: Response Depth

**Goal:** After a breach is discovered, Ripley provides structured remediation: specific
fix commands, credential rotation checklists, and hardening recommendations tailored to
the attack that just hit.

**Depends on:** Phase 1 complete. Phase 2 not required (can work with npm-only).

### Deliverables

- `ripley fix <cve> [path]` --- generate and launch a remediation prompt for a specific
  CVE in a specific project. If no path given, fix all affected projects.
- `ripley exposure <cve>` --- assess credential exposure for a specific attack. Output:
  which credential stores the attack targets, which ones exist on this machine, rotation
  commands / links for each.
- `ripley harden` --- post-incident hardening recommendations based on recent alerts and
  scan results. Suggests: guard rules to tighten, paths to monitor, domains to block,
  packages to watch.
- Remediation templates: structured per-attack playbooks for common attack patterns
  (npm worm, PyPI `.pth` injection, credential exfiltration, IDE config poisoning).
  Each template includes IOC-specific file checks, credential rotation checklists,
  and verification steps.
- Network connection audit against known C2 IP/domain databases.

### Not in scope for Phase 3
- Runtime monitoring (Phase 4)
- Automated containment (Phase 4)


---


## Phase 4: Active Detection

**Goal:** Ripley monitors the developer's machine in real time for signs of active
compromise: unexpected network connections, unauthorized writes to sensitive paths, and
C2 communication patterns. When detected, containment is one click away.

**Depends on:** Phase 1 (tray app + notifications). Phase 2 and 3 are independent.

### Deliverables

- `ripley monitor` --- background daemon watching for IOC patterns:
  - Unexpected outbound connections from Node, Python, Ruby, Go, Cargo processes
  - Unauthorized writes to `.claude/`, `.vscode/`, shell RC files, LaunchAgents,
    cron, systemd units
  - Processes masquerading as system services
  - Active connections to known C2 infrastructure (IP + domain matching)
- `ripley contain <pid|pkg>` --- immediate response:
  - Kill suspicious process
  - Snapshot process state (open files, network connections, environment) for forensic
    analysis
  - Optional: revoke network access for the process's user
- Real-time high-priority notifications with [View] [Contain] [Investigate] actions
- Filesystem anomaly detection via `notify` crate: new files in `.claude/` or `.vscode/`
  that aren't in git, lockfile modifications outside of explicit install commands,
  unexpected binaries in `node_modules/.cache/` or Python site-packages
- Integration with tray app: "During" phase alerts in the alert panel, distinct from
  "Before" and "After" alerts


---


## Phase 5: Advanced Analysis

**Goal:** Move from static rule matching to behavioral analysis. Enable community-driven
rule development. Ship CI/CD integrations for teams.

**Depends on:** Phase 2 (multi-PM support) for full CI value.

### Deliverables

- **Sandboxed script execution:**
  - macOS: `sandbox-exec` profiles restricting network + filesystem
  - Linux: bubblewrap (`bwrap`) namespaces with network isolation
  - Run install scripts with network disabled, observe filesystem and process activity
  - Compare observed behavior against declared behavior in package manifest
- **Behavioral analysis engine:**
  - Record filesystem writes, network attempts, process spawns during sandboxed execution
  - Diff against expected behavior for the package type
  - Flag anomalies (e.g., a CSS library trying to read `~/.ssh/`)
- **Community rule sharing:**
  - Publish detection rules to a public registry
  - Subscribe to community rule feeds (curated sets, organization-specific)
  - Rule format: versioned TOML with metadata (author, confidence, source attack)
- **CI/CD integration:**
  - GitHub Action: `ripley-guard` as a step, fails on high-risk scripts
  - GitLab CI template
  - `ripley guard --ci` with structured SARIF output for code scanning dashboards
