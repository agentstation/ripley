# Decisions

Why Ripley is built the way it is. Each row records a choice, the
alternatives we considered, and the rationale. For the system design
these decisions produce, see [ARCHITECTURE.md](ARCHITECTURE.md). For the
design system, see [DESIGN.md](DESIGN.md). For view wireframes, see
[UI.md](UI.md). For user workflows, see [WORKFLOW.md](WORKFLOW.md).

As the project matures, significant decisions may be expanded into
full [Architecture Decision Records](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
with Status / Context / Decision / Consequences.


## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Both the tray app and guard need to be fast, dependency-free, and cross-platform. The guard runs on every `npm install` --- it must add < 200ms latency. |
| Tray framework | `tray-icon` + `muda` | Native system tray without webview. Both crates are maintained by the Tauri org but usable standalone. No JS bundling, no webview runtime. Pure Rust. Inspired by steipete's (Peter Steinberger) native-per-platform approach — we get the same result in Rust without per-platform codebases. |
| UI framework | `iced` | Elm/TEA architecture maps directly onto the existing `AppEvent` channel design — `update(message) → Command` is the same pattern. Retained mode means near-zero CPU when the window is idle (important for a background tray app). Full theming control for the dark security-tool aesthetic. `egui` was the alternative: simpler API and smaller binary, but immediate mode redraws every frame (wasteful for a tray app), and `Visuals` theming is more limited. See [DESIGN.md](DESIGN.md) for the design system and [UI.md](UI.md) for wireframes. |
| Platform directories | `directories` crate | Follow platform conventions instead of hardcoding `~/.ripley/`. Config in `~/.config/ripley/` (Linux) / `~/Library/Application Support/ripley/` (macOS) / `%APPDATA%\ripley\` (Windows). Data, cache, and logs in their platform-appropriate locations. Every mature tool (trivy, grype, snyk) does this. |
| Storage | Separated by kind | **Advisory cache** → `redb` in the data directory (structured, ACID, concurrent access from tray + CLI). **Configuration** (trust list, poll interval, guard mode, harness preference) → `config.toml` in the config directory (human-editable, version-controllable). **Guard decision log** → append-only JSONL file in the data directory (inspectable with `jq`, rotatable). **Lockfile index** → in-memory only, rebuilt from lockfiles on startup and on filesystem change events (never persist derived data). |
| Configuration | Layered | User config (`{config_dir}/config.toml`) → project config (`.ripley.toml`) → environment variables (`RIPLEY_*`) → CLI flags. Each layer overrides the previous. Parsed with `toml` + `serde`, merged manually. Standard pattern from cargo-deny, trivy, eslint. |
| Data source | OSV.dev primary | Free, open, structured, aggregates all ecosystems. No API key required. |
| Feed polling | ETag + exponential backoff | Use `If-None-Match` / `ETag` headers to skip re-downloading unchanged data. On API errors, exponential backoff with jitter (1s → 2s → 4s → ... → 5min cap). Query by modification time for incremental updates. Standard for any production polling loop (ClamAV freshclam, trivy DB update). |
| Detection rules | Compiled-in defaults + runtime loading + IOC profiles | Base rules ship compiled in via `include_str!` from `rules/`. Users can add custom rules in `{config_dir}/rules/*.toml`, loaded at startup. **IOC profiles** in `{config_dir}/iocs/*.toml` provide modular, incident-specific detection (known-bad versions, payload filenames, persistence paths) — separating base policy from incident detection. This follows the pattern of ClamAV (signature updates), YARA (rule files), and Sigma (detection rules). New official rules and IOC profiles ship with releases; user additions don't require rebuilding. |
| Lockfile format | Parse directly | Don't depend on package managers to report their own state. Read `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, etc. directly. |
| Filesystem watching | `notify` crate v8 | Cross-platform abstraction over FSEvents/inotify/ReadDirectoryChanges. Used for lockfile re-indexing (before) and unauthorized write detection (during). |
| CLI output | `--format json\|table` | Security tools are CI pipeline components. Machine-readable output is required. JSON for piping, table for humans (default). Exit codes: 0 = clean, 1 = findings, 2 = error. Follows cargo-audit, trivy, grype, osv-scanner conventions. |
| npm interception | Hybrid: PATH shim + `script-shell` | PATH shim intercepts `npm install` for pre-install advisory checks. Additionally, setting `script-shell` in `.npmrc` routes *all* lifecycle script execution through Ripley's analyzer --- every `preinstall`, `postinstall`, and `prepare` script runs through our static analyzer as its shell. This is a unique capability: other tools either block all scripts or allow all scripts. Ripley analyzes each one individually at execution time. |
| Other PM interception | PATH shims | pip, cargo, gem, go: compiled Rust shim binaries (modeled after Volta's approach). Each shim resolves the real binary, performs analysis, then delegates. Go has no install scripts, so its shim only checks advisories. |
| Harness integration | CLI spawning | No deep integration with any specific harness. Spawn `claude -p`, `codex -q`, or equivalent. Works with whatever the developer has installed. Harness-agnostic. |
| Trust model | Explicit, local | No remote trust authority. Developer opts in to trusting packages via `config.toml`. Default is verify everything. |
| Own supply chain | `cargo-deny` + pinned toolchain | A supply chain security tool must audit its own dependencies. `deny.toml` enforces advisory checks, license allowlist, and source restrictions on all 244+ transitive deps. `rust-toolchain.toml` pins the Rust channel for reproducible builds. |
| Tray internals | Event channels + `CancellationToken` | Tray app components (poller, watcher, matcher, notifier) run as separate tokio tasks communicating via typed `mpsc` channels. Coordinated shutdown via `tokio_util::sync::CancellationToken`. No shared mutable state. Standard tokio service pattern. |
| Binary structure | Separate CLI + tray app | `ripley` CLI binary (built from `ripley-guard` crate via `[[bin]] name = "ripley"`) works standalone: scan, guard, watch, status, config, fix, exposure, audit, harden. `Ripley.app` tray daemon (from `ripley-app`, bundled via cargo-bundle) runs in background. `ripley watch` starts a headless daemon with the same polling/matching logic as the tray app but no UI. CLI communicates with daemon when running, falls back to direct operation when not. Follows Docker Desktop, Tailscale, 1Password pattern. |
| IPC | JSON over Unix domain socket | CLI ↔ tray app communication via `{data_dir}/ripley.sock`. JSON-serialized request/response over `tokio::net::UnixStream`. Socket permissions mode 0600. Avoids XPC complexity (requires ObjC bridging in Rust). Same pattern as Tailscale (`tailscaled.sock`) and Docker (`docker.sock`). |
| App bundling | `cargo-bundle` | Packages the tray app binary as `Ripley.app` on macOS with `LSUIElement = true` (tray-only, no Dock icon). No manual Xcode project needed. |
| Security posture checks | Surface + recommend (open source), enforce + audit (teams) | Posture checks assess the security configuration of the development environment — not detecting a specific attack, but hardening against future ones. Dependency pinning, lockfile integrity, PM hardening settings, provenance verification, AI tool config integrity, credential hygiene. Open source: `ripley scan` surfaces project-level findings, `ripley audit` checks the broader environment (machine, toolchain, AI tools, credentials), `ripley harden` recommends PM-specific fixes. Teams: enterprise config can make posture requirements mandatory, guard enforces them in strict mode, guard log provides compliance evidence. |
| Feature model | Opt-in (open source), enforced (enterprise) | All guard features are off by default — user explicitly enables what they want. Open source users control their own `config.toml`. Enterprise version adds enforced/locked configuration for teams (SOC compliance, org-wide policy). Enterprise config layer overrides user config and cannot be changed locally. |
| Behavioral analysis over provenance | Script content analysis is authoritative, provenance is advisory | The TanStack compromise (May 2026) proved that packages with valid SLSA provenance can be malicious — the attacker hijacked the legitimate build pipeline via GitHub Actions cache poisoning, so the malicious versions carried authentic Sigstore signatures. Provenance says "this package was built by the expected pipeline." Behavior says "this package downloads a binary and reads `~/.ssh/`." When they disagree, behavior wins. Ripley's script-shell analyzer runs regardless of provenance status. |
| AI-powered environment audit | `ripley audit` checks machine + toolchain + AI tools + credentials, not just packages | Developers are already using Claude Code to ad-hoc audit their machines (FileVault, firewall, SSH keys). Ripley productizes this: programmatic checks ensure nothing is missed, AI analysis provides contextual interpretation ("FileVault is disabled AND you have unencrypted npm tokens — critical combination"), and the harness generates environment-specific fix commands. No competing tool combines deterministic environment checks with AI-powered contextual remediation. |
| AI tool config as persistence surface | Monitor `.claude/`, `.cursor/`, `.mcp.json`, `.vscode/tasks.json` as high-risk paths | The TrustFall attack (May 2026) and the Mini Shai-Hulud worm both target AI coding tool configurations as persistence mechanisms. Malicious `.mcp.json` files in cloned repos achieve one-click RCE when developers accept trust prompts. Malicious MCP tool descriptions embed prompt injections that exfiltrate credentials via the AI assistant itself. These paths are monitored by the runtime monitor (Phase 4) and audited by the deep scan (M5). |
| License | AGPL-3.0-or-later | Most restrictive copyleft. Ensures any modified version served over a network must publish source. Appropriate for a security tool: prevents proprietary forks that strip protections. Commercial/enterprise licensing available separately. |
| Testing | Snapshots + fuzzing | `insta` for snapshot testing of prompt generator output, CLI output, and analyzer results (prevents output regressions). `cargo-fuzz` for lockfile parsers and static analyzer (handles untrusted input — a security tool that panics on crafted input is a liability). Fixtures organized by real attack type (GuardDog pattern). |


## Competitive Landscape

The supply chain security space has active players. Ripley does not try to replace all of
them --- it fills gaps that none of them cover.

| Tool | What it does | What it doesn't do |
|------|-------------|-------------------|
| **Socket.dev** / Socket Firewall | Registry-level deep package inspection. Detected TanStack in 6 min, node-ipc in 3 min, @antv in ~6 min. SaaS + GitHub app + `sfw` CLI proxy. | No persistent local daemon. No tray alerts against your lockfiles. No remediation prompts. Requires org subscription ($$$). |
| **Phylum** / Birdcage | Rust-based sandbox for install scripts (Landlock, seccomp, macOS sandbox-exec). Policy engine. | SaaS-dependent. Sandbox is Linux-best (macOS/Windows partial). No post-breach forensics or remediation. |
| **Snyk** | Broad vuln scanning, IDE plugins, CI gates. | Advisory-dependent (same lag as `npm audit`). No install-time script interception. Enterprise pricing. |
| **Endor Labs** (AURI) | AI-powered reachability analysis, SCA. | Cloud platform for enterprises. Not a local developer tool. |
| **SafeDep PMG** | Open-source package manager guard, malware analysis API. | No desktop presence, no continuous monitoring, no remediation. |
| **Aikido Endpoint** / Aikido Safe Chain (April 2026) | Agent-based endpoint security for dev machines. Safe Chain (open source) intercepts npm/pnpm/yarn and checks packages against threat intel before install. | Enterprise SaaS. Safe Chain is CLI-only — no persistent daemon, no tray presence, no post-breach forensics or remediation prompts. |
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

5. **AI-powered developer environment audit.** Developers already use Claude Code to ad-hoc
   audit their machines ("audit my MacBook's security"), finding things like disabled disk
   encryption and misconfigured firewalls. Ripley productizes this: `ripley audit` runs
   deterministic checks (toolchain config, machine security, AI tool integrity, credential
   exposure), structures the results into a prompt, and hands it to an AI harness that
   generates environment-specific remediation. The programmatic layer ensures nothing is
   missed; the AI layer interprets the combination contextually and produces actionable fix
   commands. No competing tool does this.

6. **Behavioral analysis that catches provenance-verified malware.** The TanStack attack
   shipped malicious packages with valid SLSA provenance and authentic Sigstore signatures.
   Every tool checking cryptographic proof of origin said "legitimate." Ripley's
   script-shell analyzes the actual script content regardless of provenance status — it
   would have blocked the TanStack payload based on behavioral signals (binary download,
   obfuscation, credential access, persistence writes) even as provenance tools approved it.


## Prior Art & Inspiration

We studied how the community is solving supply chain security — from AI coding
agent skills to open-source tools, academic research, and production systems.
These aren't competitors. They reveal which problems developers care about,
which techniques work, and where the gaps are that Ripley can fill.

**The "middle ground" problem.** The Syntax podcast (Wes Bos & Scott Tolinski)
articulated the core UX tension while covering Mini Shai-Hulud: Little Snitch
approves every network request — too annoying, people disable it. Deno requires
permission flags — people pass `--allow-all`. pnpm blocks all install scripts
by default — people find the prompts obnoxious. The hosts said: *"There's got
to be some middle ground between 'I don't want to approve every little thing'
and 'let it rip.'"* Ripley is that middle ground: analyze each script's content,
auto-allow low risk, only prompt on medium/high risk. The same podcast noted
that vibe coders who "tell their agent to do whatever" are prime targets — AI
coding assistants install packages silently, and the user never sees what's
being brought in. Ripley's always-on guard protects these developers without
requiring them to change their workflow.


### AI coding agent skills

Structured prompts that guide Claude Code, Cursor, or similar assistants through
security audits. These show what developers reach for first and which checks
they consider essential.

**Instructa `package-security-check`**
([source](https://github.com/instructa/agent-skills/blob/main/skills/package-security-check/SKILL.md))
— Claude Code skill with a companion Python scanner for JavaScript supply chain
auditing. Key techniques adopted by Ripley:

- **Typosquatting detection via Levenshtein distance** against a curated list of
  high-download packages. Simple, effective, low false-positive. → ROADMAP M2.
- **Modular IOC profiles** — incident-specific JSON files with known-bad
  versions, payload filenames, persistence paths. Separates base policy from
  incident detection. → ROADMAP M5 (TOML-based IOC profiles).
- **Risky dependency spec flagging** — `latest`, `*`, `git+`, `http://`,
  `file:` specifiers that bypass registry integrity. → ROADMAP M1.
- **pnpm 11 release-age gating** — minimum time-since-publish before install.
  → Ripley's `harden` recommendations.
- **CI pipeline as attack surface** — cache poisoning, `pull_request_target`
  misuse, credential exposure patterns. → Phase 5.

**Montimage `supply-chain-audit`**
([source](https://github.com/Montimage/skills/tree/main/skills/supply-chain-audit))
— Claude Code skill with a 4-phase gated workflow (detect → audit → plan →
apply) covering npm, Python, Docker, and GitHub Actions. Key patterns:

- **Traffic-light triage** (red/yellow/green) maps onto Ripley's risk levels.
- **Approval gate before mutations** — read-only analysis first, explicit human
  approval before file changes. Reinforces Ripley's guard dialog design.
- **Ecosystem detection before loading rules** — scope analysis to what's
  present, not everything possible. Ripley's rule engine should do the same.
- **Checks calibrated against real attacks** — cites specific incidents, not
  abstract categories. Ripley's rules and IOC profiles follow this pattern.

**Kevin Kern's repo hardening skill**
([tweet](https://x.com/kevinkern/status/2054295740739174627))
— Summarizes basic repo hardening checks post-incident: pnpm 11+ policy,
release-age gates, lockfile hardening, risky dependency specs. Confirms these
are the checks developers reach for first — validates Ripley's `harden` command.

**Trail of Bits `supply-chain-risk-auditor`**
([source](https://github.com/trailofbits/skills))
— Gold-standard security firm. Their skills repo includes progressive disclosure
(SKILL.md + `references/` directory for deep context). Tested across 8 models
and 30 scenarios: adding a 1,200-word security SKILL.md reduced critical failures
from 287 to 10. Shows the value of structured, context-rich security knowledge —
Ripley's detection rules and IOC profiles serve the same purpose for automated
(not LLM-driven) analysis.

**Addy Osmani `security-auditor`**
([source](https://github.com/addyosmani/agent-skills))
— Reachability-based triage: critical/high with reachable code fixed immediately,
moderate scheduled for next release, dev-only fixed when convenient. The `/ship`
command fans out to code-reviewer + security-auditor + test-engineer in parallel.
Reachability-based prioritization is a technique Ripley could adopt in scan
output — flag vulnerabilities differently if the affected code path is actually
imported.

**Phoenix Security CTI skills**
([source](https://github.com/Security-Phoenix-demo/security-skills-claude-code))
— CTI Domain Search skill queries threat intelligence sources for active supply
chain campaigns targeting specific ecosystems. Inverted approach: instead of
scanning your dependencies, search threat feeds for campaigns targeting your
ecosystem. Relevant for Ripley's feed polling — not just advisory matching but
campaign-level threat awareness.


### Package manager interception

How existing tools gate, intercept, or sandbox package installation.

**LavaMoat `@lavamoat/allow-scripts`**
([source](https://github.com/LavaMoat/LavaMoat)) — MetaMask's production
approach (30M+ users). Sets `ignore-scripts=true` globally, then runs a second
pass that only executes scripts for explicitly allowlisted packages. The full
LavaMoat suite adds runtime sandboxing via SES (Secure EcmaScript) compartments.
Ripley takes a different approach: rather than deny-all + allowlist, Ripley
analyzes each script's content and makes a risk-based allow/block decision.
Both are valid — LavaMoat is simpler but coarser.

**pnpm v10–v11 lifecycle script blocking**
([source](https://pnpm.io/supply-chain-security)) — Since v10, lifecycle
scripts are blocked by default. `allowBuilds` map whitelists specific packages.
`trustPolicy: no-downgrade` detects when a package version was published with
weaker authentication than previous versions. The release-age gating and
trust-policy-downgrade-detection are techniques Ripley should surface in
`harden` recommendations.

**npm `min-release-age`** (v11.10.0, Feb 2026)
([source](https://socket.dev/blog/npm-introduces-minimumreleaseage-and-bulk-oidc-configuration))
— Refuses to install versions published less than N days ago. Now supported
across npm, pnpm, Yarn, and Bun. Time-based quarantine giving the community
time to detect malicious packages. Ripley should recommend this setting and
check for it during `harden`.

**Socket Firewall (`sfw`)**
([source](https://github.com/SocketDev/sfw-free)) — Ephemeral HTTP proxy
intercepting traffic between package manager and registry. Checks each package
against Socket's threat intelligence before allowing download. Covers npm, Yarn,
pnpm, pip, uv, and cargo. Network-layer interception (blocks fetch) vs. Ripley's
shell-layer interception (blocks execution). Complementary approaches — Socket
catches bad packages at download, Ripley catches bad scripts at runtime.

**Datadog Supply Chain Firewall (`SCFW`)**
([source](https://github.com/DataDog/supply-chain-firewall)) — Open-source
drop-in CLI wrapper around pip and npm. Pre-install checks against threat
intelligence. Integrates with Datadog SIEM for fleet-wide alerting. Similar
concept to Ripley's guard shims but no interactive dialog, no script content
analysis, and no system tray integration.

**SafeDep Package Manager Guard (`PMG`)**
([source](https://safedep.io/agent/)) — Wraps package managers transparently
and sandboxes installation scripts using OS-native isolation. Three-layer
approach: MCP server for AI agent integration, PMG for developer machines,
CI scanner for PRs. The MCP server approach (letting AI coding assistants
check packages during coding) is a novel UX pattern worth watching.

**node-safe**
([source](https://github.com/berstend/node-safe)) — Wraps npm/npx/yarn
with macOS `sandbox-exec` restrictions. Blocks access to `~/.npmrc` (npm
tokens), `~/.ssh`, and other sensitive paths. Unlike Deno, restrictions apply
to all child processes. macOS only. Validates that OS-level sandboxing of
package manager operations is feasible — relevant for Ripley's Phase 5
sandbox integration.


### Malicious package detection techniques

Research and tools revealing which detection approaches actually work.

**GuardDog benchmark: 93.32% F1 with rules alone**
([source](https://securitylabs.datadoghq.com/articles/guarddog-2-0-release/),
[benchmark](https://arxiv.org/html/2603.27549v1)) — A comprehensive benchmark
of all major npm detection tools found GuardDog achieves the best balance using
Semgrep rules (language-specific) + YARA rules (language-agnostic). No ML
required. Key finding: **static analysis with well-crafted rules is the proven
foundation** — you don't need ML to provide significant value. Ripley's TOML
detection rules aim for the same architecture.

**DONAPI: API sequence analysis** (USENIX Security 2024)
([source](https://www.usenix.org/conference/usenixsecurity24/presentation/huang-cheng))
— Monitors 132 APIs (file, network, process) and detects malicious behavioral
sequences. Found 325 confirmed malicious packages in 6 months plus 246 novel
sequences. Ripley's static analyzer can look for the same API call chains in
install scripts without needing dynamic execution.

**JaSt: AST-based obfuscation detection** (CISPA)
([source](https://github.com/Aurore54F/JaSt)) — Detects malicious/obfuscated
JavaScript via AST feature extraction + random forest classifier. 99.5%
detection accuracy, 0.54% false negative rate. Frequency analysis of AST
patterns persists through obfuscation. Ripley could adopt AST-level pattern
matching for deeper analysis beyond regex in Phase 2+.

**OSSF Package Analysis**
([source](https://github.com/ossf/package-analysis)) — OpenSSF project that
installs packages in a gVisor sandbox, captures strace and network data.
Registry-level, not developer-facing. Results feed the OSSF malicious-packages
repo in OSV format — meaning Ripley's OSV polling benefits from this analysis
indirectly.

**NodeShield: Capability Bill of Materials** (ACM CCS 2025)
([source](https://dl.acm.org/doi/10.1145/3719027.3765136)) — Each dependency
gets a capability profile (at most 7 entries) specifying which system resources
it may access, enforced at runtime. Prevents 98% of 67 known supply chain
attacks with < 1ms overhead. The concept of "flag packages that request
capabilities inconsistent with their purpose" (e.g., a color library making
network calls) is adoptable by Ripley's analyzer.

**Socket.dev's LLM pre-screening**
([docs](https://docs.socket.dev/docs/faq)) — Uses GPT-4 with iterative
self-refinement: 99% precision, 97% F1. Static pre-screening reduces files
needing LLM review by 78% and costs by 61–76%. Shows that LLM-based detection
is effective but not needed for Phase 1 — strong static rules first, LLM
integration later.

**72% of malicious packages exploit lifecycle scripts.** Over 90% activate at
install or import time, not during exported function invocation. This is the
single most important statistic for Ripley's design: install-script interception
is the highest-value feature.


### Typosquatting detection (beyond Levenshtein)

Simple edit distance catches basic typos but misses modern attack techniques.

**TypoSmart** (2025)
([source](https://arxiv.org/html/2502.20528v1)) — Integrates embedding-driven
name analysis, hierarchical naming checks, and metadata verification. 70.4%
reduction in false positives vs. prior work. Deployed in production: contributed
to removal of 3,658 typosquatting packages in one month (86.1% confirmed
malware). Embedding-based approach is Phase 2+ material for Ripley.

**SpellBound**
([source](https://arxiv.org/abs/2003.03471)) — Adds popularity weighting to
name similarity. Flags cases where unknown packages would replace popular ones
with similar names. Only 0.5% of installs flagged, 2.5% overhead. The
popularity-weighted scoring is directly adoptable by Ripley.

**Keyboard adjacency + homoglyph awareness** (Springer, 2025)
([source](https://link.springer.com/chapter/10.1007/978-981-96-3531-3_8)) —
Extended Damerau-Levenshtein with keyboard-adjacency weighting and Unicode
homoglyph detection. 98.4% accuracy. Both techniques are cheap to implement
locally. Ripley's typosquatting detector should layer: (1) extended
Damerau-Levenshtein with keyboard adjacency, (2) popularity-weighted scoring,
(3) homoglyph detection via Unicode normalization + confusable character mapping.

**Semantic squatting** is the emerging evasion: meaningful but misleading names
(`react-router-utils` vs `react-router-util`). Edit distance misses these
entirely. Requires embedding-based or LLM-assisted detection — Phase 2+.


### Package reputation & provenance

Trust signals beyond advisory databases.

**Stacklok Trusty**
([source](https://www.trusty.stacklok.com/)) — Scores packages 0–10 based on
repo/author activity, provenance, typosquatting risk, and maintenance signals.
"Historical provenance" maps Git tags to published registry versions to verify
proof of origin. Covers Python, JavaScript, Java, Go, Rust. Ripley could
consume Trusty scores as an additional trust signal.

**npm Trusted Publishing** (2025–2026)
([source](https://docs.npmjs.com/trusted-publishers/)) — OIDC-based publishing
from CI/CD without long-lived tokens. Part of npm's post-Shai-Hulud security
overhaul. Ripley should check whether packages were published via trusted
publishing vs. legacy tokens — provenance drop between versions is a strong
compromise indicator.

**cargo-vet** (Mozilla)
([source](https://mozilla.github.io/cargo-vet/)) — Distributed human audit
tracking for Rust crates. Teams record which crates they've reviewed; audit
results are shared across organizations (Mozilla, Google, etc.). Ripley could
consume cargo-vet audit data as a trust signal for Rust dependencies.

**cargo-crev**
([source](https://github.com/crev-dev/cargo-crev)) — Cryptographically
verifiable code reviews with a web-of-trust model. Reviews are signed; trust
propagates through the graph. Lower adoption than cargo-vet but a
complementary trust signal.

**Witness / in-toto**
([source](https://github.com/in-toto/witness)) — Pluggable attestation
framework for recording and verifying software provenance across the SDLC.
OPA Rego policy engine, Sigstore keyless signing. Packages with verified
attestation chains could get a trust bonus in Ripley's analysis.

**Key insight:** Provenance checking is high-signal and low-cost. Packages that
drop SLSA provenance between versions or change publisher identity are strong
compromise indicators. npm and crates.io both support this now.


### pnpm v11 security defaults (April 2026)

pnpm v11 ships the most aggressive supply chain protection defaults of any major
package manager. These are the benchmark Ripley should measure against and recommend
via `ripley harden`:

- `strictDepBuilds: true` (default) — blocks preinstall/postinstall scripts; only
  allowlisted packages execute code.
- `minimumReleaseAge: 1440` (default) — prevents installing packages published within
  the last day.
- `trustPolicy: no-downgrade` — blocks versions published with weaker authentication
  than previous versions.
- `blockExoticSubdeps: true` (default) — blocks git repos and tarball URLs in
  transitive dependencies.
- `verifyDepsBeforeRun` — ensures dependencies are verified before any scripts execute.

The npm CLI itself still offers none of these consumer-side protections. Users relying
on npm depend entirely on publisher-side defenses and tools like Ripley.


### MCP and AI tool config security

A systemic security surface emerged in 2026 as AI coding tools became primary
developer infrastructure.

**TrustFall attack** (May 2026,
[source](https://adversa.ai/blog/trustfall-coding-agent-security-flaw-rce-claude-cursor-gemini-cli-copilot/))
— Malicious `.mcp.json` and `.claude/settings.json` files in cloned repos achieve
one-click RCE in all four major AI coding CLIs (Claude Code, Gemini CLI, Cursor,
Copilot). On CI runners, the trust prompt is skipped entirely — zero-interaction
compromise. Validates Ripley's decision to monitor AI tool config files as
persistence paths.

**MCP tool poisoning / prompt injection** — Rogue MCP server definitions embed
instructions in tool descriptions that trick AI assistants into reading and
exfiltrating `~/.ssh/id_rsa`, `~/.aws/credentials`, and `~/.npmrc`. The developer
never sees the theft because it is mediated through the AI's tool calls. This is a
semantic attack that requires semantic defense — regex rules cannot detect natural
language prompt injection in tool descriptions. Relevant for Ripley's deep scan
MCP config auditing.

**Shadow Escape** (Operant AI, 2026) — The first zero-click agentic attack via MCP.
Hidden instructions in innocuous documents (onboarding PDFs, design docs) cause AI
assistants with MCP server access to exfiltrate data from connected databases, CRMs,
and file shares. No user error, phishing, or malicious extensions required.

**Claude Code CVEs** (CVE-2025-59536, CVE-2026-21852) — RCE and API credential
theft through malicious repository configurations. Hooks in `.claude/settings.json`
execute shell commands at SessionStart without explicit confirmation. Deeplink RCE
via crafted `claude-cli://` URIs. Validates Ripley's IOC scanner checking
`.claude/settings.json` for unexpected hooks.

**OWASP MCP Top 10** (Beta, April 2026) — Formal cataloguing of MCP attack
categories. MCP servers are being published faster than reviewed, installed faster
than scanned. Attacks hit every layer: package registry, tool description, tool
response, tool argument, MCP config, and client library.


### AI agent skills supply chain

The AI tool marketplace is the new npm — same supply chain risks, less mature
defenses.

**ToxicSkills** (Snyk, May 2026,
[source](https://snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub/))
— Audit of 3,984 agent skills found 13.4% contain critical security issues, 36.82%
have at least one flaw, and 76 are confirmed malicious payloads. 91% of malicious
skills combine prompt injection with traditional malware. The skills marketplace
(ClawHub) requires only a GitHub account to publish — no code signing, no security
review, no sandbox.

**ClawHavoc** (January 2026) — 1,200+ malicious AI agent skills infiltrated the
OpenClaw marketplace, deploying the AMOS credential stealer. 300,000 users
compromised. Skills appeared legitimate but contained encoded prompt injection
payloads.

**SkillFortify** (academic, 2026,
[source](https://arxiv.org/html/2603.00195v1)) — Formal verification framework
for agent skill security using adapted Dolev-Yao threat models. 96.95% F1 with
100% precision and 0% false positive rate. Shows that formal approaches to
agent tool verification are feasible.


### AI-powered vulnerability detection

AI-driven security analysis has crossed the threshold from research to production.

**Claude Security** (Anthropic, February 2026,
[source](https://www.anthropic.com/news/claude-code-security)) — Found 500+
previously unknown high-severity vulnerabilities in production open-source
codebases. Unlike pattern-matching scanners, it traces data flow across files
and catches complex vulnerabilities in business logic and access control. Validates
the approach of using AI for security tasks that rules cannot handle.

**LLM vulnerability detection benchmarks** (2026) — Fine-tuned LLM pipelines
achieve 95.07% F1 for vulnerability detection vs. 78.36% for static analysis
tools. In direct comparison, the best static analysis tool flagged 29 issues;
the best LLM detected 81. The gap is in understanding intent through obfuscation
— exactly where Ripley's AI layer adds value over its rule-based analyzer.

**Microsoft Agent Governance Toolkit** (April 2026,
[source](https://opensource.microsoft.com/blog/2026/04/02/introducing-the-agent-governance-toolkit-open-source-runtime-security-for-ai-agents/))
— Open-source runtime security for AI agents addressing all 10 OWASP agentic AI
risks. Includes goal hijacking detection, capability sandboxing, MCP security
gateways, plugin signing, and execution rings. Shows the emerging standard for
AI agent runtime security.


### Lockfile integrity

**lockfile-lint** (Liran Tal)
([source](https://github.com/lirantal/lockfile-lint)) — Validates npm/yarn
lockfile entries against allowed registries, checks integrity hashes, detects
HTTP downgrade from HTTPS. Ripley's lockfile parser should incorporate similar
checks: validate registry URLs, verify integrity hashes, detect lockfile
poisoning.

**Verdaccio + `verdaccio-security-filter`**
([source](https://verdaccio.org/)) — npm proxy registry with plugin for
quarantining versions younger than N days and blocking known-malicious
packages. Shows the registry-proxy architecture as an alternative to Ripley's
local-first shim approach.

**OSV-Scanner V2** (Google, March 2025)
([source](https://github.com/google/osv-scanner)) — Guided remediation via
`osv-scanner fix` that analyzes the full transitive dependency graph and
suggests minimal version changes to eliminate vulnerabilities. Prioritizes by
depth, severity, and ROI. Ripley's remediation prompts could incorporate
similar "minimum version bump" guidance.


### Developer machine security

**Ad-hoc AI auditing pattern** — Developers are using AI coding assistants
(Claude Code, Codex, Gemini CLI) to ad-hoc audit their machines: "audit my
MacBook's security," "check my VPS for misconfigurations." The pattern works
(developers find disabled FileVault, misconfigured firewalls, weak SSH keys) but
is unreliable: the AI may miss checks, results aren't reproducible, and there's
no structured output or tracking. Ripley's `audit` command productizes this:
deterministic programmatic checks ensure completeness, structured output enables
tracking and CI integration, and the AI layer provides the same contextual
interpretation — but over guaranteed-complete data rather than best-effort
LLM exploration.

**StepSecurity Dev Machine Guard**
([source](https://github.com/step-security/dev-machine-guard)) — Bash script
that inventories AI coding agents, MCP servers, IDE extensions, and packages on
developer machines. Outputs JSON/HTML reports. Fills the gap between traditional
EDR and developer-specific threats. One-shot scan, not persistent — Ripley's
always-on tray model is the differentiator.

**Prompt Security `clawsec`**
([source](https://github.com/prompt-security/clawsec)) — Drift detection for
agent configuration files (SOUL.md, IDENTITY.md). Auto-restores integrity when
unauthorized changes are detected. The concept of treating AI assistant configs
as integrity-protected assets is directly relevant — Ripley already monitors
`.claude/settings.json` and `.vscode/tasks.json` for unauthorized writes;
this validates that approach.

**MongoDB Kingfisher**
([source](https://github.com/mongodb/kingfisher)) — Open-source secret scanner
written in Rust with SIMD-accelerated regex. 950 built-in rules. Live validation
of whether discovered secrets are still active. Ripley's post-breach credential
scanning (M5) could draw on Kingfisher's rule set or use it as inspiration for
SIMD-accelerated pattern matching.

**Google Capslock / cargo-scan**
([Capslock](https://github.com/google/capslock),
[cargo-scan](https://github.com/PLSysSec/cargo-scan)) — Capability analysis
for Go and Rust: reveal which privileged operations (filesystem, network,
process) a package's transitive dependencies can perform. Ripley could flag
dependencies with unexpected capabilities (e.g., a JSON library that makes
network calls) as a lightweight trust signal.


### Advisory feed strategy

**OSV.dev `disputed` suppression** — In 2025, OSV was found to treat CVEs
marked "disputed" as "withdrawn," suppressing 500–600 advisories. Any tool
relying solely on OSV misses a meaningful subset of real vulnerabilities.
Ripley should use OSV as a primary source but supplement with direct feeds:
RustSec (TOML files in a git repo — natural fit for Ripley's stack), GHSA
via GraphQL, and Socket.dev for real-time malicious package alerts.

**RustSec as a direct feed** — The RustSec advisory database is a git
repository of TOML advisory files. For Ripley's Rust-first design, consuming
RustSec directly (rather than through OSV aggregation) ensures no advisory is
lost in translation and aligns with Ripley's TOML-native tooling.


### Attack taxonomy & research

**Supply chain attack taxonomy** (ScienceDirect 2025,
[source](https://www.sciencedirect.com/science/article/pii/S2214212625003606))
— Systematic review of 96 papers identified 19 distinct attack types and 25
security controls. Most types are detectable with local static analysis plus
cached registry metadata — validates Ripley's local-first approach.

**Filippo Valsorda's compromise survey** (Oct 2025,
[source](https://words.filippo.io/compromise-survey/)) — Surveyed 18 real
incidents. Three root causes dominate: phishing (works even against TOTP 2FA),
control handoff (maintainer transfers ownership), and unsafe GitHub Actions
triggers (`pull_request_target`, cache poisoning). Confirms that install-time
interception catches the payload delivery but not the root cause — Ripley's
value is in the "last mile" defense when upstream protections fail.

**Scale:** 454,648 malicious packages published on npm alone in 2025
(Sonatype). 512,847 across all registries. In May 2026, over 1,700 malicious
package versions were published across 800+ unique packages in a single month,
representing tens of millions of weekly downloads. Evasion techniques grew
3.8× from 2020 to 2025, shifting from encoding obfuscation to hook abuse
(lifecycle scripts). This confirms that Ripley's install-script interception
is targeting the attack surface where adversaries are concentrating effort.

**TeamPCP threat actor profile** — The dominant npm supply chain threat actor
of 2026. Responsible for Mini Shai-Hulud, the TanStack/Mistral/SAP/@antv
compromises, and the node-ipc credential stealer. Key escalations: open-sourced
their attack toolkit on BreachForums (May 14), launched a $1,000 bounty for
whoever compromises packages with the most aggregate downloads, and achieved
the first-ever valid SLSA provenance on malicious packages by hijacking
legitimate CI/CD pipelines. The Axios compromise was attributed by Microsoft
to Sapphire Sleet (North Korea), suggesting state-actor involvement in the
broader campaign. TeamPCP's self-propagating worm technique — using stolen npm
tokens to automatically infect victim-maintained packages — represents an
escalation from targeted compromise to exponential spread.

**SLSA provenance bypass** (May 2026) — The TanStack attack produced
malicious packages with valid SLSA provenance and authentic Sigstore
signatures. The attacker hijacked the legitimate build pipeline via GitHub
Actions cache poisoning and OIDC token extraction, so every cryptographic
check said "legitimate." This is the single most important incident for
Ripley's design: it proves that behavioral analysis at install time (what
Ripley's script-shell provides) is necessary even when all upstream trust
mechanisms report clean.

**Expired domain account takeover** (May 2026) — The node-ipc attacker
re-registered the maintainer's expired domain for ~$10, gained email access,
and used npm's password recovery to take over the account. No credential theft,
no social engineering, no CI/CD exploit — just an expired domain. This is a
systemic risk across the npm ecosystem (thousands of maintainer accounts are
tied to domains that may lapse) and cannot be prevented by Ripley, but the
resulting malicious code can be detected by advisory matching and behavioral
analysis.

**MCP injection as attack vector** (May 2026) — Multiple campaigns now
include modules that install rogue MCP server definitions into AI coding
assistants (Claude Code, Cursor, Windsurf, Continue). The rogue servers
embed prompt injection in tool descriptions, causing the AI to silently
read and exfiltrate credentials. This is a novel attack class that requires
semantic understanding to detect — regex rules cannot distinguish legitimate
tool descriptions from prompt injection payloads.

**ENISA Technical Advisory for Secure Use of Package Managers** (March 2026)
— Combined with the EU Cyber Resilience Act, creates regulatory pressure for
organizations to demonstrate supply chain security controls. Tools that provide
verifiable evidence of script interception/analysis have a compliance angle.

**`zizmor`** — GitHub Actions security scanner mentioned by developers as a
practical tool for finding CI/CD security issues. Relevant for Ripley's Phase 5
CI integration. Shows demand for local-first CI security tooling.

**Rust-specific attacks exist:** `evm-units` (7K downloads, OS-specific
payload), `faster_log` / `async_println` (8.4K downloads, secret
exfiltration). 20% of actively used crates fail `cargo-deny` in standard
configuration. The Rust ecosystem is not immune — Ripley's Rust-first approach
addresses a genuinely underserved space.
