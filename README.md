# Ripley

**Supply chain defense before, during, and after the breach.**

Ripley is a cross-platform system tray application and package manager guardian that
defends developers across the full lifecycle of a supply chain attack: intercepting
malicious packages before they execute, detecting active compromise in real time, and
driving automated remediation after a breach is discovered.

Named after Ellen Ripley --- who enforced quarantine before the threat got aboard, fought
it when it did, and came back to finish the job.

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

The current security toolchain has gaps at every phase. Most tools cover one moment in the
timeline and leave the rest to someone else:

| Tool | Phase | The gap it leaves |
|------|-------|-------------------|
| `npm audit` | Before (partial) | Advisory must exist first. Only checks *after* install. Doesn't stop you from running the malicious postinstall script right now. |
| CVE databases | Before (partial) | Median time from compromise to CVE: days to weeks. Attackers have already moved. |
| GitHub Dependabot | Before (partial) | Opens a PR. Doesn't block installation of the bad version in the meantime. |
| EDR / antivirus | During (partial) | Optimized for known malware signatures, not developer-specific attack patterns like npm postinstall exfiltration or `.claude/settings.json` injection. |
| Security blogs | After (partial) | You learn about it from Twitter or Hacker News hours or days later. No structured IOCs, no automated remediation. |
| Incident response | After (manual) | Credential rotation, forensics, OS reinstall. Effective but entirely manual, slow, and requires expertise most developers don't have. |

**Before:** Almost no tooling operates between "a malicious package is published" and "a
developer installs it." The postinstall script already ran. The `.pth` file already loaded.

**During:** Developer machines have no runtime monitoring tuned for supply chain attack
patterns --- not a Node process writing to `.claude/settings.json`, not a Python import
exfiltrating `~/.aws/credentials`.

**After:** Incident response requires security expertise most developers don't have. Which
credential stores did this specific attack target? Which IOC files should you search for?
How do you verify that remediation was complete?


## Full-Spectrum Defense

Most security tools pick one phase. Ripley covers the full timeline. The framing comes
from two traditions:

- **Forward secrecy** in cryptography, where compromise of current keys does not compromise
  past or future sessions. The system is designed so that security holds *going forward*
  regardless of what is breached today.

- **Left of boom** in military doctrine, where effort concentrates on the period *before*
  the detonation event --- but the doctrine doesn't stop there. It also covers actions
  *during* the event (containment) and *after* (recovery, attribution, hardening).

"Boom" is the moment a malicious install script executes on your machine, or a trojaned
package gets imported into your runtime. Ripley operates across all three phases:

**Before the breach: forward defense.** Polling structured vulnerability feeds (OSV.dev,
GitHub Advisory Database, Socket.dev) on a continuous loop and cross-referencing against
every lockfile on your machine. When you install a package, Ripley intercepts the install
scripts and analyzes them before they run. If a `postinstall` script downloads a binary,
decodes base64, or exhibits obfuscation patterns, it is flagged and blocked --- not logged
after the fact.

**During the breach: active detection.** Not every attack can be prevented. Ripley watches
for IOC patterns in real time: unexpected outbound connections from Node/Python processes,
writes to known persistence paths (`.claude/settings.json`, `.vscode/tasks.json`, shell
RC files, LaunchAgents), and filesystem anomalies in project directories. When active
compromise indicators are detected, a high-priority notification fires with containment
actions.

**After the breach: automated response.** `ripley scan --deep` performs the forensic audit
a security engineer would do manually: IOC file search, persistence mechanism review,
credential exposure mapping. Ripley then generates a scoped remediation prompt and hands
it to whichever AI coding harness the developer uses (Claude Code, Codex, OpenCode). The
response timeline shifts from "read a blog post, manually audit" (hours to days) to
"notification, one click, review diff" (minutes).

A developer should never learn about a supply chain attack from Twitter. They should learn
about it from Ripley --- ideally before it affects them, but if not, the moment it does,
with containment already in progress and a fix ready to apply.


## Architecture

```
  BEFORE                     DURING                      AFTER
  ──────                     ──────                      ─────

  ┌────────────────┐    ┌─────────────────┐    ┌──────────────────┐
  │  Feed Poller   │    │ Process Monitor │    │  Forensic Scan   │
  │                │    │                 │    │                  │
  │  OSV.dev       │    │  outbound conn  │    │  IOC file check  │
  │  GHSA          │    │  persistence    │    │  lockfile audit   │
  │  Socket.dev    │    │  writes         │    │  shell RC review │
  │                │    │  privilege esc  │    │  cred exposure   │
  └───────┬────────┘    └────────┬────────┘    └────────┬─────────┘
          │                      │                      │
          ▼                      ▼                      ▼
  ┌────────────────┐    ┌─────────────────┐    ┌──────────────────┐
  │ Lockfile Index │    │ Anomaly Engine  │    │ Prompt Generator │
  │                │    │                 │    │                  │
  │ every project  │    │ known C2 IPs    │    │  CVE + versions  │
  │ on this machine│    │ IOC patterns    │    │  project path    │
  │ watched live   │    │ signature DB    │    │  IOC file paths  │
  └───────┬────────┘    └────────┬────────┘    │  cred checklist  │
          │                      │             │  clean version   │
          ▼                      ▼             └────────┬─────────┘
  ┌────────────────┐    ┌─────────────────┐             │
  │    Matcher     │    │   Containment   │             ▼
  │                │    │                 │    ┌──────────────────┐
  │  new advisory  │    │  kill process   │    │ Harness Launcher │
  │  × installed   │    │  revoke network │    │                  │
  │  = alert       │    │  snapshot state │    │ claude | codex   │
  └───────┬────────┘    └────────┬────────┘    │ | opencode       │
          │                      │             └────────┬─────────┘
          ▼                      ▼                      ▼
  ┌──────────────────────────────────────────────────────────────┐
  │                  Native OS Notification                      │
  │                                                              │
  │  BEFORE: [View] [Fix] [Dismiss]                              │
  │  DURING: [View] [Contain] [Investigate]                      │
  │  AFTER:  [View] [Remediate] [Rotation Checklist]             │
  └──────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────┐
  │              ripley-guard (package manager shim)             │
  │                                                              │
  │  npm install ──►┌──────────────┐    ┌───────────────────┐    │
  │  pip install ──►│   Script     │───►│  Static Analyzer  │    │
  │  cargo build ──►│   Extractor  │    │                   │    │
  │  gem install ──►│              │    │  network calls?   │    │
  │  go install  ──►└──────────────┘    │  eval / exec?     │    │
  │                                     │  base64 decode?   │    │
  │                                     │  binary download? │    │
  │                                     │  obfuscation?     │    │
  │                                     │  known patterns?  │    │
  │                                     │                   │    │
  │                                     │  risk: lo|med|hi  │    │
  │                                     └────────┬──────────┘    │
  │                                         low  │  med/high     │
  │                                         ┌────┴────┐          │
  │                                         ▼         ▼          │
  │                                    auto-allow   prompt user  │
  │                                                 with details │
  └──────────────────────────────────────────────────────────────┘
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for component details, technical decisions, project
structure, and threat model. See [ROADMAP.md](ROADMAP.md) for the phased execution plan.


## CLI

```
# tray app
ripley                              # launch the tray app
ripley watch                        # headless daemon mode (servers, CI)
ripley config                       # open configuration
ripley status                       # show monitored projects, last poll time, alert count

# before: forward defense
ripley scan [path]                  # one-shot scan: check lockfiles against advisories
ripley guard install                # set up package manager shims in PATH
ripley guard uninstall              # remove shims, restore original behavior
ripley guard status                 # show which package managers are intercepted
ripley guard trust <pkg>            # add a package to the trust list
ripley guard untrust <pkg>          # remove from trust list
ripley guard log                    # show recent interceptions and decisions

# during: active detection
ripley monitor                      # watch processes and filesystem for IOC patterns
ripley contain <pid|pkg>            # kill process, snapshot state for investigation

# after: response and recovery
ripley scan --deep [path]           # full forensic audit (IOC files, persistence,
                                    # shell RC, network, creds)
ripley exposure <cve>               # assess credential exposure for a specific attack
ripley fix <cve> [path]             # generate and launch remediation prompt
ripley harden                       # suggest forward-defense measures based on
                                    # recent incidents
```


## Why "Ripley"

Ellen Ripley is the full-spectrum defender.

**Before.** On the Nostromo, she enforced quarantine protocol. She refused to let Kane back
aboard with the facehugger attached. She saw the threat hiding inside something everyone
else wanted to trust, and she tried to stop it at the door. She was overridden --- by Ash,
by the company, by people who prioritized other goals over safety. The threat got in anyway.

**During.** When the xenomorph was loose on the ship, she didn't freeze. She made tactical
decisions under pressure, adapted to a threat no one had seen before, and kept fighting when
every system around her had failed. She activated the self-destruct when containment was no
longer possible.

**After.** She survived. She went into cryo with the knowledge of what happened. And in
*Aliens*, she went back. Not because she had to --- because she knew the threat was still
out there and no one else understood it. She came back to finish the job, to protect others,
and to make sure it couldn't happen again.

Software supply chains have the same structure. Package registries prioritize growth and
convenience (Weyland-Yutani prioritized the specimen). The threat hides inside things that
look safe --- a routine dependency update, a trusted package name, a familiar postinstall
script (a crew member returning from a routine survey). The systems we rely on (npm audit,
CVE databases, security advisories) are like the Nostromo's crew: well-intentioned, but
operating on assumptions that no longer hold.

Most security tools pick one phase. Ripley doesn't. She shows up before the threat boards,
fights it when it does, and comes back to make sure it's finished.


## License

TBD
