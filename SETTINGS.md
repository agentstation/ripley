# Settings

Complete reference for every configurable setting in Ripley. For system
design, see [ARCHITECTURE.md](ARCHITECTURE.md). For the design system,
see [DESIGN.md](DESIGN.md). For UI wireframes of the settings view,
see [UI.md](UI.md). For how settings affect user workflows,
see [WORKFLOW.md](WORKFLOW.md).


## Configuration Layers

Settings are resolved in order. Each layer overrides the previous:

```
1. Defaults        ← hardcoded in the binary
2. config.toml     ← user preferences ({config_dir}/config.toml)
3. .ripley.toml    ← project overrides (project root)
4. RIPLEY_* env    ← environment variables
5. CLI flags       ← command-line arguments
6. Enterprise      ← locked org config (cannot be overridden locally)
```

The first config.toml that provides a value wins at that layer. Later
layers override earlier ones. Enterprise config (when present) overrides
everything and cannot be changed by the user.


## Platform Directories

Ripley uses the `directories` crate (`ProjectDirs::from("com",
"agentstation", "ripley")`) for platform-appropriate paths. Never
hardcoded.

| Directory      | macOS                                       | Linux                          | Windows                    |
|----------------|---------------------------------------------|--------------------------------|----------------------------|
| `{config_dir}` | `~/Library/Application Support/ripley/`     | `~/.config/ripley/`            | `%APPDATA%\ripley\`        |
| `{data_dir}`   | `~/Library/Application Support/ripley/`     | `~/.local/share/ripley/`       | `%APPDATA%\ripley\`        |
| `{cache_dir}`  | `~/Library/Caches/ripley/`                  | `~/.cache/ripley/`             | `%LOCALAPPDATA%\ripley\`   |


---


## `config.toml` --- User Configuration

Location: `{config_dir}/config.toml`. Created automatically on first run
if missing. Human-editable, version-controllable. All writes are atomic
(write to temp file, then rename).


### `[general]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `poll_interval_secs` | `u64` | `300` | How often to poll advisory feeds (seconds). Minimum: 60. |
| `harness` | `string?` | `null` (auto-detect) | Preferred AI coding CLI for remediation prompts. Values: `"claude"`, `"codex"`, `"opencode"`. When null, Ripley checks PATH in that order and uses the first one found. |
| `launch_at_login` | `bool` | `false` | Install a LaunchAgent (macOS) to start the tray app on login. |

```toml
[general]
poll_interval_secs = 300
harness = "claude"
launch_at_login = true
```

**Environment overrides:**
- `RIPLEY_POLL_INTERVAL` --- override `poll_interval_secs`
- `RIPLEY_HARNESS` --- override `harness`


### `[monitoring]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `project_roots` | `string[]` | `[]` | Directories to watch for lockfiles. The tray app walks these roots on startup and watches for filesystem changes. |

```toml
[monitoring]
project_roots = ["~/src", "~/work"]
```

**Environment overrides:**
- `RIPLEY_PROJECT_ROOTS` --- colon-separated list of paths


### `[guard]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `mode` | `string` | `"strict"` | Guard behavior for intercepted scripts. `"strict"` = block high-risk scripts (prompt user). `"audit"` = log only, never block. `"off"` = disable guard entirely (shims pass through). |
| `trust` | `string[]` | `[]` | Packages that bypass script analysis. Supports exact names (`"express"`) and glob scopes (`"@tanstack/*"`). Advisory checks still run for trusted packages. |
| `timeout_secs` | `u64` | `30` | Seconds to wait for user response on guard prompts before defaulting to Block (fail-safe). |

```toml
[guard]
mode = "strict"
trust = ["@tanstack/*", "typescript", "esbuild"]
timeout_secs = 30
```

**Environment overrides:**
- `RIPLEY_GUARD_MODE` --- override `mode`

**CLI overrides:**
- `ripley guard trust <pkg>` --- add to `trust` list
- `ripley guard untrust <pkg>` --- remove from `trust` list


### `[posture]`

Security posture checks assess how well the development environment is
hardened against future attacks. These checks don't require advisory data
--- they examine configuration, lockfile hygiene, and dependency pinning.

For open source users, posture findings are informational warnings. For
teams, they can be promoted to errors via `strict = true`.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `strict` | `bool` | `false` | When true, posture warnings cause exit code 1 (same as advisory findings). Use in CI to enforce posture requirements as build gates. |
| `require_lockfile` | `bool` | `false` | Warn (or fail in strict mode) if no lockfile is found in the scanned path. |
| `require_exact_versions` | `bool` | `false` | Flag range specifiers (`^`, `~`, `*`, `latest`) in lockfile-adjacent manifests. Exact pinning prevents silent upgrades to compromised versions. |
| `require_integrity_hashes` | `bool` | `false` | Flag lockfile entries missing SHA-512 integrity hashes. Integrity hashes prevent lockfile poisoning. |
| `block_exotic_sources` | `bool` | `false` | Flag dependencies resolved from `git+`, `http://`, or `file:` sources. These bypass registry integrity checks. |
| `allowed_registries` | `string[]` | `["https://registry.npmjs.org"]` | Lockfile resolved URLs must point to one of these registries. URLs resolving elsewhere are flagged. |

```toml
[posture]
strict = false
require_lockfile = true
require_exact_versions = true
require_integrity_hashes = false
block_exotic_sources = true
allowed_registries = ["https://registry.npmjs.org"]
```


### `[audit]`

Environment security audit configuration. Controls which checks
`ripley audit` runs and how results are reported.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `machine_security` | `bool` | `true` | Check disk encryption, firewall, OS updates, screen lock. |
| `toolchain` | `bool` | `true` | Check git signing, SSH key strength, shell RC hygiene. |
| `ai_tools` | `bool` | `true` | Check MCP configs, Claude hooks, VS Code extensions for rogue definitions. |
| `credentials` | `bool` | `true` | Check for exposed secrets in shell history, env vars, RC files, `.env` in git. |
| `min_score` | `string?` | `null` | Minimum audit score per category to pass. Values: `"green"`, `"yellow"`. When set, categories scoring below this cause exit code 1. |

```toml
[audit]
machine_security = true
toolchain = true
ai_tools = true
credentials = true
```


### `[feeds]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `osv` | `bool` | `true` | Poll OSV.dev for advisories. Primary data source. |
| `ghsa` | `bool` | `false` | Poll GitHub Advisory Database via GraphQL. Supplements OSV (catches `disputed` advisories that OSV suppresses). Phase 2. |
| `socket` | `bool` | `false` | Poll Socket.dev for real-time malicious package alerts. Requires API key. Phase 2. |
| `socket_api_key` | `string?` | `null` | API key for Socket.dev. Store securely --- do not commit to version control. |
| `stale_threshold_secs` | `u64` | `3600` | Consider the advisory cache stale after this many seconds. On-demand scans refresh if stale. |

```toml
[feeds]
osv = true
ghsa = false
socket = false
stale_threshold_secs = 3600
```

**Environment overrides:**
- `RIPLEY_SOCKET_API_KEY` --- override `socket_api_key` (preferred over config file for secrets)


### `[notifications]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `enabled` | `bool` | `true` | Fire native OS notifications for new findings. |
| `min_severity` | `string` | `"low"` | Minimum severity to notify. Values: `"low"`, `"medium"`, `"high"`, `"critical"`. |
| `sound` | `bool` | `false` | Play the system alert sound with notifications. |

```toml
[notifications]
enabled = true
min_severity = "high"
sound = false
```


### `[logging]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `level` | `string` | `"info"` | Log level filter. Values: `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`. Uses `tracing-subscriber` `env-filter` syntax. |
| `file` | `bool` | `true` | Write logs to file in daemon mode. Location: `{data_dir}/logs/`. |
| `rotation` | `string` | `"daily"` | Log file rotation. Values: `"daily"`, `"hourly"`, `"never"`. Uses `tracing-appender`. |

```toml
[logging]
level = "info"
file = true
rotation = "daily"
```

**Environment overrides:**
- `RUST_LOG` --- override `level` (standard `tracing` convention)
- `RIPLEY_LOG_LEVEL` --- override `level` (takes precedence over `RUST_LOG`)


### Full `config.toml` Example

```toml
[general]
poll_interval_secs = 300
harness = "claude"
launch_at_login = true

[monitoring]
project_roots = ["~/src", "~/work"]

[guard]
mode = "strict"
trust = ["@tanstack/*", "typescript", "esbuild"]
timeout_secs = 30

[posture]
strict = false
require_lockfile = true
require_exact_versions = false
require_integrity_hashes = false
block_exotic_sources = true
allowed_registries = ["https://registry.npmjs.org"]

[audit]
machine_security = true
toolchain = true
ai_tools = true
credentials = true

[feeds]
osv = true
ghsa = false
socket = false
stale_threshold_secs = 3600

[notifications]
enabled = true
min_severity = "low"
sound = false

[logging]
level = "info"
file = true
rotation = "daily"
```


---


## `.ripley.toml` --- Project Configuration

Location: project root directory. Checked into version control. Overrides
`config.toml` for the project it lives in. Designed for team-wide settings
that travel with the repo.

This file uses a subset of the `config.toml` schema. Only settings that
make sense at the project level are supported.


### `[guard]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `mode` | `string` | (from config.toml) | Override guard mode for this project. |
| `trust` | `string[]` | (from config.toml) | Project-specific trust list. Merged with (not replacing) the user's trust list. |

```toml
[guard]
mode = "strict"
trust = ["@tanstack/*", "typescript", "esbuild"]
```


### `[posture]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `strict` | `bool` | (from config.toml) | Override posture strictness for this project. Commonly `true` in CI configs. |
| `require_lockfile` | `bool` | (from config.toml) | Override per-project. |
| `require_exact_versions` | `bool` | (from config.toml) | Override per-project. |
| `require_integrity_hashes` | `bool` | (from config.toml) | Override per-project. |
| `block_exotic_sources` | `bool` | (from config.toml) | Override per-project. |
| `allowed_registries` | `string[]` | (from config.toml) | Override per-project. Useful for teams with private registries. |

```toml
[posture]
strict = true
require_lockfile = true
require_exact_versions = true
require_integrity_hashes = true
block_exotic_sources = true
allowed_registries = [
    "https://registry.npmjs.org",
    "https://npm.internal.example.com"
]
```


### `[audit]`

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `min_score` | `string?` | (from config.toml) | Override minimum audit score for this project. Commonly set in CI to enforce environment security as a build gate. |

```toml
[audit]
min_score = "yellow"
```


### Full `.ripley.toml` Example

```toml
[guard]
mode = "strict"
trust = ["@tanstack/*", "typescript", "esbuild"]

[posture]
strict = true
require_lockfile = true
require_exact_versions = true
require_integrity_hashes = true
block_exotic_sources = true

[audit]
min_score = "yellow"
```


---


## Environment Variables

Environment variables override both `config.toml` and `.ripley.toml`.
Useful for CI pipelines and containerized environments.

| Variable | Overrides | Type | Notes |
|----------|-----------|------|-------|
| `RIPLEY_POLL_INTERVAL` | `general.poll_interval_secs` | integer | Seconds. |
| `RIPLEY_HARNESS` | `general.harness` | string | `claude`, `codex`, `opencode`. |
| `RIPLEY_PROJECT_ROOTS` | `monitoring.project_roots` | string | Colon-separated paths. |
| `RIPLEY_GUARD_MODE` | `guard.mode` | string | `strict`, `audit`, `off`. |
| `RIPLEY_SOCKET_API_KEY` | `feeds.socket_api_key` | string | Preferred over config file for secrets. |
| `RIPLEY_LOG_LEVEL` | `logging.level` | string | `trace`, `debug`, `info`, `warn`, `error`. |
| `RUST_LOG` | `logging.level` | string | Standard `tracing` env filter. Lower precedence than `RIPLEY_LOG_LEVEL`. |
| `CI` | --- | any | When set to any value, Ripley runs non-interactively. Guard prompts are auto-resolved: `strict` mode blocks, `audit` mode allows. No tray app, no notifications. |
| `NO_COLOR` | --- | any | Disable colored terminal output. Standard convention. |


---


## CLI Flags

CLI flags override everything except enterprise config. Available on
the `ripley scan` command.

| Flag | Overrides | Type | Description |
|------|-----------|------|-------------|
| `--format` | --- | `json` or `table` | Output format. Default: `table`. |
| `--deep` | --- | flag | Run full forensic scan (IOC scan, persistence audit, credential exposure). |
| `--fix` | --- | flag | Generate remediation prompts for all findings and launch the configured AI harness. Falls back to printing prompts to stdout if no harness is available. |
| `--no-cache` | --- | flag | Force fresh advisory fetch, ignoring ETag cache and stale threshold. |

**`ripley config` flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--path` | flag | Print the resolved config file path and exit. |
| `--show` | flag | Print the fully resolved configuration (all layers merged). |
| `--init` | flag | Write a default `config.toml` if none exists. |

**`ripley watch` flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--daemon` | flag | Fork to background. Logs to `{data_dir}/logs/` instead of stdout. |

**`ripley audit` flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--format` | `json` or `table` | Output format. Default: `table`. |
| `--fix` | flag | Structure all findings into a prompt and launch the configured AI harness for remediation. |


---


## `[monitor]` --- Runtime Monitoring (Phase 4)

Configuration for `ripley monitor` and the tray app's active detection
mode. These settings control the runtime process and filesystem monitor.
Not implemented until Phase 4.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `enabled` | `bool` | `false` | Enable runtime monitoring when the daemon is running. |
| `watch_processes` | `bool` | `true` | Monitor outbound connections from Node/Python/Ruby/Go processes. |
| `watch_persistence` | `bool` | `true` | Monitor writes to `.claude/`, `.vscode/`, shell RCs, LaunchAgents, MCP configs. |
| `watch_lockfiles` | `bool` | `true` | Flag lockfile edits outside of explicit install commands. |
| `c2_domains` | `string[]` | `[]` | Additional C2 domains to match against. Loaded alongside compiled-in list. |
| `c2_ips` | `string[]` | `[]` | Additional C2 IP addresses to match against. |

```toml
[monitor]
enabled = true
watch_processes = true
watch_persistence = true
watch_lockfiles = true
```


## Enterprise Configuration (Future)

Enterprise config is a locked layer that overrides all user and project
settings. It cannot be changed locally. Designed for SOC 2 compliance
and team-wide security baselines.

Pushed by an organization admin. Stored at a well-known path or fetched
from a management endpoint.


### `[enterprise]`

| Setting | Type | Description |
|---------|------|-------------|
| `org_id` | `string` | Organization identifier. |
| `policy_url` | `string?` | URL to fetch updated enterprise policy from. |
| `locked` | `bool` | When true, all settings below cannot be overridden by the user. |


### `[enterprise.guard]`

| Setting | Type | Description |
|---------|------|-------------|
| `mode` | `string` | Enforced guard mode. Cannot be weakened by user. |
| `required_rules` | `string[]` | Rule IDs that cannot be disabled. |
| `trust` | `string[]` | Org-wide trust list. Merged with project trust lists. |
| `max_trust` | `string[]?` | If set, users cannot trust packages outside this list. |


### `[enterprise.posture]`

| Setting | Type | Description |
|---------|------|-------------|
| `strict` | `bool` | Enforced posture strictness. |
| `require_lockfile` | `bool` | Enforced. |
| `require_exact_versions` | `bool` | Enforced. |
| `require_integrity_hashes` | `bool` | Enforced. |
| `block_exotic_sources` | `bool` | Enforced. |
| `min_release_age` | `u64?` | Minimum seconds since package publication before install is allowed. Surfaces as a `harden` recommendation even without enterprise config. |
| `min_posture_score` | `string?` | Minimum posture score per category to pass CI. Values: `"green"`, `"yellow"`. |


### `[enterprise.audit]`

| Setting | Type | Description |
|---------|------|-------------|
| `require_guard_log` | `bool` | Guard log must be enabled and retained. |
| `log_retention_days` | `u64` | Minimum days to retain guard log entries. |
| `report_endpoint` | `string?` | URL to POST audit reports to (compliance evidence). |


---


## Detection Rules

TOML files defining patterns for the static analyzer. Two sources:

| Source | Location | Loaded | Override behavior |
|--------|----------|--------|-------------------|
| Base rules | `rules/` in source tree | Compiled in via `include_str!` | Always active. |
| User rules | `{config_dir}/rules/*.toml` | At startup (runtime) | Same `id` as a base rule → user rule overrides it. |


### Rule File Format

```toml
id = "npm_network_call"
name = "Network call in postinstall"
description = "Script makes outbound network requests"
ecosystem = "npm"
signal = "network_call"
weight = "high"                     # "low", "medium", "high", "critical"

patterns = [
    "curl\\s",
    "wget\\s",
    "fetch\\(",
    "http\\.get\\(",
    "net\\.connect\\(",
    "XMLHttpRequest",
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | yes | Unique identifier. User rules with the same ID override base rules. |
| `name` | `string` | yes | Human-readable name shown in guard prompts. |
| `description` | `string` | yes | Longer description shown in guard detail view. |
| `ecosystem` | `string` | yes | Which ecosystem this rule applies to: `"npm"`, `"pypi"`, `"cargo"`, `"gem"`, `"go"`, `"all"`. |
| `signal` | `string` | yes | Signal category for grouping. |
| `weight` | `string` | yes | Risk level: `"low"`, `"medium"`, `"high"`, `"critical"`. |
| `patterns` | `string[]` | yes | Regex patterns. If any pattern matches the script content, the rule fires. Patterns are compiled once and cached. |


### Disabling a Base Rule

Create a user rule with the same `id` and set `weight = "low"` and empty
`patterns`:

```toml
# {config_dir}/rules/disable_network_call.toml
id = "npm_network_call"
name = "Network call in postinstall (disabled)"
description = "Disabled by user"
ecosystem = "npm"
signal = "network_call"
weight = "low"
patterns = []
```


---


## IOC Profiles

TOML files defining incident-specific indicators of compromise. Loaded
alongside detection rules but serve a different purpose: rules detect
suspicious behavior patterns; IOC profiles detect artifacts from specific,
known attacks.

| Source | Location | Loaded |
|--------|----------|--------|
| Base profiles | `iocs/` in source tree | Compiled in via `include_str!` |
| User profiles | `{config_dir}/iocs/*.toml` | At startup (runtime) |


### IOC Profile Format

```toml
id = "tanstack-2026-05"
name = "TanStack Router Compromise (May 2026)"
description = "CVE-2026-45321 — pnpm store cache poisoning via pull_request_target"
date = "2026-05-11"
references = [
    "https://socket.dev/blog/tanstack-compromise",
]

[packages]
ecosystem = "npm"
names = ["@tanstack/react-router", "@tanstack/router", "@tanstack/start"]
bad_versions = ["1.169.4", "1.169.5"]
clean_version = "1.170.0"

[indicators]
files = [
    ".claude/execution.js",
    ".claude/setup.mjs",
    ".vscode/setup.mjs",
    "node_modules/@tanstack/*/router_init.js",
]
persistence_paths = [
    "~/.claude/settings.json",
    "~/.vscode/tasks.json",
]
mcp_configs = [
    ".mcp.json",
    ".cursor/mcp.json",
    ".github/copilot/",
]
domains = ["evil-c2.example.com"]
ips = ["185.x.x.x"]

[credentials]
targeted = [
    "~/.npmrc",
    "~/.aws/credentials",
    "~/.ssh/id_*",
    "~/.gitconfig",
    "~/.env",
]
dead_man_switch = true
rotation_warning = "Back up home directory before rotating. Dead man switch detected."
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | yes | Unique identifier for this incident. |
| `name` | `string` | yes | Human-readable name. |
| `description` | `string` | yes | Brief description of the attack. |
| `date` | `string` | yes | Date of the incident (YYYY-MM-DD). |
| `references` | `string[]` | no | URLs to advisories, blog posts, or analysis. |
| `packages.ecosystem` | `string` | yes | Affected ecosystem. |
| `packages.names` | `string[]` | yes | Affected package names. |
| `packages.bad_versions` | `string[]` | yes | Known-compromised versions. |
| `packages.clean_version` | `string` | yes | First clean version to upgrade to. |
| `indicators.files` | `string[]` | no | IOC files to check for. Glob patterns supported. |
| `indicators.persistence_paths` | `string[]` | no | Persistence locations to audit. |
| `indicators.mcp_configs` | `string[]` | no | MCP config files to check for rogue server definitions or prompt injection in tool descriptions. |
| `indicators.ai_tool_configs` | `string[]` | no | AI coding tool config paths to audit (`.claude/settings.json`, `.cursor/`, etc.). |
| `indicators.domains` | `string[]` | no | Known C2 domains. |
| `indicators.ips` | `string[]` | no | Known C2 IP addresses. |
| `credentials.targeted` | `string[]` | no | Credential stores this attack targets. |
| `credentials.dead_man_switch` | `bool` | no | Whether this attack installs a dead man switch. |
| `credentials.rotation_warning` | `string` | no | Warning to display before credential rotation. |


---


## Guard Log

Location: `{data_dir}/guard.jsonl`. Append-only JSONL file. Each line is
a JSON object recording a guard decision. Inspectable with `jq`,
greppable, rotatable with standard log tools.

Not a setting file --- included here because it is a configurable output
that the enterprise layer can mandate.

### Entry Format

```json
{
    "timestamp": "2026-05-12T12:04:32Z",
    "package": "@example/pkg",
    "version": "1.2.3",
    "script": "postinstall",
    "action": "blocked",
    "risk_level": "high",
    "matched_rules": ["npm_network_call", "npm_pipe_to_shell"],
    "source": "script-shell",
    "user_decision": true
}
```

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `string` | ISO 8601 UTC timestamp. |
| `package` | `string` | Package name. |
| `version` | `string` | Package version. |
| `script` | `string` | Lifecycle hook name (`preinstall`, `postinstall`, etc.). |
| `action` | `string` | `"allowed"`, `"blocked"`, `"trusted"` (bypassed via trust list). |
| `risk_level` | `string` | `"low"`, `"medium"`, `"high"`, `"critical"`. |
| `matched_rules` | `string[]` | IDs of rules that fired. |
| `source` | `string` | `"script-shell"` or `"shim"`. |
| `user_decision` | `bool` | Whether the user was prompted (vs. auto-allow/auto-block). |

View with: `ripley guard log` or `jq '.' {data_dir}/guard.jsonl`.


---


## Precedence Summary

How a single setting resolves across all layers, using `guard.mode` as
an example:

```
Enterprise config says "strict"     → strict (locked, game over)
  ↑ overrides
CLI flag --guard-mode=audit         → audit
  ↑ overrides
RIPLEY_GUARD_MODE=audit             → audit
  ↑ overrides
.ripley.toml [guard] mode = "audit" → audit
  ↑ overrides
config.toml [guard] mode = "strict" → strict
  ↑ overrides
Hardcoded default                   → "strict"
```

When enterprise config is present and `locked = true`, all layers below
it are ignored for locked settings.


---


## Defaults Reference

Every setting with its default value, for quick scanning.

| Section | Setting | Default |
|---------|---------|---------|
| general | `poll_interval_secs` | `300` |
| general | `harness` | `null` (auto-detect) |
| general | `launch_at_login` | `false` |
| monitoring | `project_roots` | `[]` |
| guard | `mode` | `"strict"` |
| guard | `trust` | `[]` |
| guard | `timeout_secs` | `30` |
| posture | `strict` | `false` |
| posture | `require_lockfile` | `false` |
| posture | `require_exact_versions` | `false` |
| posture | `require_integrity_hashes` | `false` |
| posture | `block_exotic_sources` | `false` |
| posture | `allowed_registries` | `["https://registry.npmjs.org"]` |
| audit | `machine_security` | `true` |
| audit | `toolchain` | `true` |
| audit | `ai_tools` | `true` |
| audit | `credentials` | `true` |
| audit | `min_score` | `null` |
| feeds | `osv` | `true` |
| feeds | `ghsa` | `false` |
| feeds | `socket` | `false` |
| feeds | `socket_api_key` | `null` |
| feeds | `stale_threshold_secs` | `3600` |
| notifications | `enabled` | `true` |
| notifications | `min_severity` | `"low"` |
| notifications | `sound` | `false` |
| logging | `level` | `"info"` |
| logging | `file` | `true` |
| logging | `rotation` | `"daily"` |
