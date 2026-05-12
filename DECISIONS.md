# Decisions

Why Ripley is built the way it is. Each row records a choice, the
alternatives we considered, and the rationale. For the system design
these decisions produce, see [ARCHITECTURE.md](ARCHITECTURE.md).

As the project matures, significant decisions may be expanded into
full [Architecture Decision Records](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
with Status / Context / Decision / Consequences.


## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Both the tray app and guard need to be fast, dependency-free, and cross-platform. The guard runs on every `npm install` --- it must add < 200ms latency. |
| Tray framework | Tauri v2 | Native webview, ~5MB binary, 30-40MB idle RAM, ships on macOS/Windows/Linux. Avoids bundling Chromium. Uses `tray-icon` crate internally --- if Tauri proves too heavy, we can drop to pure Rust (`tray-icon` + `notify-rust` + `reqwest`) at 5-15MB idle with the same tray primitives. |
| Platform directories | `directories` crate | Follow platform conventions instead of hardcoding `~/.ripley/`. Config in `~/.config/ripley/` (Linux) / `~/Library/Application Support/ripley/` (macOS) / `%APPDATA%\ripley\` (Windows). Data, cache, and logs in their platform-appropriate locations. Every mature tool (trivy, grype, snyk) does this. |
| Storage | Separated by kind | **Advisory cache** → `redb` in the data directory (structured, ACID, concurrent access from tray + CLI). **Configuration** (trust list, poll interval, guard mode, harness preference) → `config.toml` in the config directory (human-editable, version-controllable). **Guard decision log** → append-only JSONL file in the data directory (inspectable with `jq`, rotatable). **Lockfile index** → in-memory only, rebuilt from lockfiles on startup and on filesystem change events (never persist derived data). |
| Configuration | Layered | User config (`{config_dir}/config.toml`) → project config (`.ripley.toml`) → environment variables (`RIPLEY_*`) → CLI flags. Each layer overrides the previous. Parsed with `toml` + `serde`, merged manually. Standard pattern from cargo-deny, trivy, eslint. |
| Data source | OSV.dev primary | Free, open, structured, aggregates all ecosystems. No API key required. |
| Feed polling | ETag + exponential backoff | Use `If-None-Match` / `ETag` headers to skip re-downloading unchanged data. On API errors, exponential backoff with jitter (1s → 2s → 4s → ... → 5min cap). Query by modification time for incremental updates. Standard for any production polling loop (ClamAV freshclam, trivy DB update). |
| Detection rules | Compiled-in defaults + runtime loading | Base rules ship compiled in via `include_str!` from `rules/`. Users can add custom rules in `{config_dir}/rules/*.toml`, loaded at startup. This follows the pattern of ClamAV (signature updates), YARA (rule files), and Sigma (detection rules). New official rules ship with releases; user rules don't require rebuilding. |
| Lockfile format | Parse directly | Don't depend on package managers to report their own state. Read `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, etc. directly. |
| Filesystem watching | `notify` crate v8 | Cross-platform abstraction over FSEvents/inotify/ReadDirectoryChanges. Used for lockfile re-indexing (before) and unauthorized write detection (during). |
| CLI output | `--format json\|table` | Security tools are CI pipeline components. Machine-readable output is required. JSON for piping, table for humans (default). Exit codes: 0 = clean, 1 = findings, 2 = error. Follows cargo-audit, trivy, grype, osv-scanner conventions. |
| npm interception | Hybrid: PATH shim + `script-shell` | PATH shim intercepts `npm install` for pre-install advisory checks. Additionally, setting `script-shell` in `.npmrc` routes *all* lifecycle script execution through Ripley's analyzer --- every `preinstall`, `postinstall`, and `prepare` script runs through our static analyzer as its shell. This is a unique capability: other tools either block all scripts or allow all scripts. Ripley analyzes each one individually at execution time. |
| Other PM interception | PATH shims | pip, cargo, gem, go: compiled Rust shim binaries (modeled after Volta's approach). Each shim resolves the real binary, performs analysis, then delegates. Go has no install scripts, so its shim only checks advisories. |
| Harness integration | CLI spawning | No deep integration with any specific harness. Spawn `claude -p`, `codex -q`, or equivalent. Works with whatever the developer has installed. Harness-agnostic. |
| Trust model | Explicit, local | No remote trust authority. Developer opts in to trusting packages via `config.toml`. Default is verify everything. |
| Own supply chain | `cargo-deny` + pinned toolchain | A supply chain security tool must audit its own dependencies. `deny.toml` enforces advisory checks, license allowlist, and source restrictions on all 244+ transitive deps. `rust-toolchain.toml` pins the Rust channel for reproducible builds. |
| Tray internals | Event channels + `CancellationToken` | Tray app components (poller, watcher, matcher, notifier) run as separate tokio tasks communicating via typed `mpsc` channels. Coordinated shutdown via `tokio_util::sync::CancellationToken`. No shared mutable state. Standard tokio service pattern. |
| Testing | Snapshots + fuzzing | `insta` for snapshot testing of prompt generator output, CLI output, and analyzer results (prevents output regressions). `cargo-fuzz` for lockfile parsers and static analyzer (handles untrusted input — a security tool that panics on crafted input is a liability). Fixtures organized by real attack type (GuardDog pattern). |


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
