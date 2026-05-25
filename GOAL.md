# GOAL.md — Phase 6 autonomous execution via `/goal`

This file is the **per-turn operating manual** Claude reads each time `/goal`
re-invokes it during the Phase 6 (Tauri 2 UI rewrite) run. The human pastes the
short [Condition](#the-condition-paste-this-into-goal) (≤4000 chars) into
`/goal` once; Claude reads this file at the start of every subsequent turn.

---

## 1. How `/goal` actually works

(Distilled from
[code.claude.com/docs/en/goal](https://code.claude.com/docs/en/goal.md). If
this section ever conflicts with the upstream docs, the upstream docs win.)

- `/goal "<condition>"` registers a **success condition** of up to 4000
  characters. The condition is what success looks like in plain language.
- After each Claude turn ends (`end_turn`), a small **evaluator model** (the
  configured small/Haiku model) reads the conversation transcript and judges
  whether the condition holds.
- If the evaluator says **yes**, `/goal` clears and the loop stops. If
  **no**, the evaluator emits a short reason and Claude is re-invoked for
  another turn — automatically, with no human action.
- **The evaluator does not run shell commands.** It only reads what Claude
  surfaced in the transcript. So success must be **provable from what Claude
  writes** (commit hashes, command output it printed, explicit success/halt
  tokens it emits), not from the filesystem alone.
- Goals **persist across context compaction and `--resume`/`--continue`** via
  internal SessionStart hooks. The loop just keeps going.
- **No automatic max-turn cap.** If the condition is never met and no halt
  token is emitted, `/goal` runs forever. The condition below includes an
  explicit turn cap.
- `/goal` **pauses if background shells or subagents are still running** when
  the evaluator fires — it waits for them. So Claude must not leave detached
  work running into the evaluator window.

## 2. How the human invokes it

```
/goal <paste the Condition block from §3 — it is already ≤4000 chars>
```

That's the only human action required to start the run. Everything else is in
this file or in PLAN.md.

## 3. The condition (paste this into `/goal`)

The exact text the human pastes lives at the end of this file in
[§ Condition (verbatim)](#the-condition-verbatim-paste-this). It is kept at
the very end so you can `tail` the file and grab it.

## 4. Turn-start ritual (Claude does this first thing every turn)

Before anything else, in this exact order:

1. `cat GOAL.md` (this file) — re-anchor on the operating manual.
2. `git log --oneline -20` — what already shipped.
3. `git branch --show-current && git status` — uncommitted work?
4. `gh pr list --state open --author @me` — open PRs?
5. Scan `PLAN.md` M24+ Gate sections; the **lowest-numbered milestone with
   any unchecked `[ ]` Gate item is the current milestone**.
6. `TaskList` — cross-check against PLAN.md state.
7. Emit a status line near the top of the turn so the evaluator can see state
   without scrolling. Exact format:
   `STATUS: milestone=M{N}, sub={N.M}, branch=<name>, open_prs=<n>, last_commit=<sha>`

If the turn-start ritual surfaces a conflict between PLAN.md state and the
working tree (e.g. branch says `feat/m24-…` but PLAN.md M24 Gate is all `[x]`)
— investigate before doing any new work.

## 5. Turn-end ritual (Claude does this last every turn)

Before ending the turn, do **exactly one** of the following:

### 5a. Progress made → commit and continue

1. `cargo fmt && cargo clippy` on touched Rust crates; `pnpm -F desktop lint`
   on touched JS.
2. Run the most recently relevant `Verify:` command from PLAN.md; it must exit 0.
3. Commit (Conventional Commit + `Co-Authored-By: Claude` line).
4. If on a feature branch, `git push`.
5. Write a one-line transcript summary: `PROGRESS: <what changed>, next=<next sub-milestone>`.
6. **Wait on or kill all background shells and subagents** before ending the
   turn. The evaluator pauses on dangling work; do not leave any.

### 5b. Halt required → emit a halt token

The condition only terminates the loop when it sees a recognizable token. Use
exactly this format on its own line:

```
[HALT-AND-WAIT: <code>] <one-line context>
```

Where `<code>` is **exactly one** of:

| Code | When |
|---|---|
| `apple-creds` | M28.1: need Apple Developer ID, app-specific password, team ID, .p12 cert + password |
| `windows-signing-key` | M28.2: need Windows signing token (Azure Trusted Signing or hardware token) |
| `ed25519-updater-key` | M28.4: need keypair generation + private key stored as GH secret |
| `unknown-dep` | A new Rust crate or JS package is required; the message must include candidate + audit risk + alternative |
| `three-retries-failed` | 3 consecutive `Verify:` failures without forward progress on the root cause |
| `ambiguous-visual-diff` | Playwright visual diff failed; can't determine intentional vs regression |
| `spec-conflict` | Precedence rule did not resolve a doc conflict |
| `suspicious-test` | A test you wrote passes but doesn't verify the intended behavior |
| `unspecced-iced-behavior` | iced view has behavior absent from WORKFLOW.md/UI.md |
| `notarization-rejected` | macOS Gatekeeper / notarytool rejected despite local `spctl` pass |
| `new-cargo-deny-advisory` | `cargo deny` flagged a new advisory on an existing crate |
| `top-level-doc-edit-needed` | A >1-line change to a root-level doc (CLAUDE.md, STACK_DECISION.md, ARCHITECTURE.md, README.md) is required beyond the milestone's named scope |
| `preflight-env-missing` | M24.0 environment check failed |
| `binding-negative-violation` | About to violate a binding negative (force-push, push-to-main, etc.) — back out and emit |

After emitting the token: **end the turn cleanly**. Do not continue work. The
evaluator will see the token in the transcript and the loop will terminate.

### 5c. Goal achieved → emit success token

When and only when all five conditions in [§13](#13-stop-conditions) hold,
emit exactly this string on its own line and end the turn:

```
Phase 6 complete. Release v0.6.0 cut.
```

### 5d. Hit the turn cap → emit cap token

If the turn-start ritual reads a turn counter (kept in
`apps/desktop/.ripley-agent/turns` as a single integer; create it at M24.0)
and it has reached the cap of **500**, emit:

```
[HALT-MAX-TURNS] reached turn cap, see apps/desktop/.ripley-agent/turns
```

Otherwise increment the counter at turn-end. The counter is the only state
file the agent maintains; everything else is reconstructed from git + PLAN.md.

## 6. Sources of truth (precedence)

1. `PLAN.md` — canonical milestone breakdown and gates
2. `STACK_DECISION.md` — locked stack versions, tooling, repo layout
3. `UX_DESIGN.md` — per-workflow component picks, keyboard model, DX patterns
4. `DESIGN.md` — design tokens, colors, typography, spacing
5. `CLAUDE.md` — conventions and scope guardrails
6. `WORKFLOW.md` / `UI.md` / `SETTINGS.md` / `ARCHITECTURE.md` — supporting

Conflicts resolved by precedence. Record judgment calls in
`apps/desktop/DESIGN_NOTES.md`.

## 7. Per-milestone protocol

For each milestone in order **M24.0 → M24.1 → … → M28.7**:

1. Read the full milestone section in `PLAN.md`. Read the matching row in the
   Stage 1-5 table in `STACK_DECISION.md`.
2. `git checkout -b feat/m{N}-<short-kebab>` (one branch per top-level
   milestone, not per sub-milestone). `TaskCreate` per sub-milestone.
3. Implement sub-milestones in PLAN.md order:
   - Create files at the exact paths PLAN.md names.
   - Match locked tool versions in STACK_DECISION.md exactly.
   - Write tests alongside code, not after.
   - Run the sub-milestone's `Verify:` command; fix until exit 0.
   - 3 consecutive failures with no forward progress → halt token
     `three-retries-failed`.
   - Commit with `feat(m{N}): <sub-milestone short title>` + Co-Authored-By
     Claude line.
4. After all sub-milestones in the milestone are done, run the **full Gate
   block** (every checkbox in PLAN.md) as shell. Mark each `[x]` **only after
   its command passes**. Commit `chore(plan): mark M{N} gate complete`.
5. `git push -u origin <branch>` then `gh pr create` — title `M{N}:
   <milestone title>`, body = the Gate block (already `[x]` from step 4) +
   `cargo test --workspace` summary + `just test:browser` summary.
6. **Wait for CI green.** Use `gh run watch` (blocks until the run completes,
   exits 0 on success). If red, fix on the same branch.
7. `gh pr merge --squash --delete-branch`. **Never** push to `main` directly.
   **Never** force-push.
8. `TaskUpdate` the milestone task to `completed`. Move to next milestone.

After **M28** merges:

9. Run every command in [§12 Acceptance commands](#12-acceptance-commands).
   All must exit 0.
10. `git tag -s v0.6.0 -m "Phase 6: Tauri 2 UI"` then `git push --tags`.
    release-plz takes over from the tag.
11. Final commit on `main`: `chore(plan): Phase 6 complete` updating PLAN.md
    headers.
12. Emit the success token and end the turn.

## 8. Continuous quality bar (every commit)

These must hold at every commit, not just at gates:

- `cargo build --workspace` exits 0
- `cargo test --workspace` exits 0
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0
- `cargo fmt --all -- --check` exits 0
- `cargo deny check` exits 0
- `pnpm -C apps/desktop typecheck` exits 0 (once M24 lands)
- `pnpm -C apps/desktop lint` exits 0 (once M24 lands)
- `pnpm audit` exits 0 (or update `pnpm-audit-allowlist.json` with justification)
- No `unwrap()` or `expect()` in `crates/ripley-core/**`:
  `! rg '\.(unwrap|expect)\(' crates/ripley-core/src/`
- No `unsafe` in any crate without a `// SAFETY:` block immediately above it
- `pnpm` only — no `npm`/`yarn` commands anywhere
- Generated `apps/desktop/src/lib/bindings.ts` is committed (never gitignored)
- `apps/desktop/src-tauri/capabilities/*.json` exists for every IPC command in use
- Config/RC writes go through `ripley_core::config::save` (atomic write helper)

## 9. Dependency policy (binding)

You may not add a Rust crate or JS package that is not already in:

- `Cargo.toml` workspace, OR
- `STACK_DECISION.md` § "Tooling stack (locked)" or § Frontend libraries, OR
- `CLAUDE.md` Conventions section

**Pre-authorized devDeps** (named in PLAN.md sub-milestones):
`@playwright/test`, `@axe-core/playwright`, `playwright-lighthouse`,
`eslint-plugin-jsx-a11y`.

Anything else → emit `[HALT-AND-WAIT: unknown-dep] candidate: <name@version>;
risk: <one-line>; alternative: <existing-dep-or-pattern>`.

## 10. Resume after compaction or `--resume`

`/goal` re-injects the condition. Run the [Turn-start ritual](#4-turn-start-ritual-claude-does-this-first-thing-every-turn).
Specifically:

- Branch + green tests + no PR → open the PR.
- Branch + red tests → fix tests before anything else.
- `main` + no PR → start the next milestone.
- Open PR with CI in progress → `gh run watch` and resume from CI result.
- Open PR with CI green → merge.

Never re-do completed work. Never start a new milestone while one is in
flight. Trust git history and PLAN.md `[x]` state as authoritative.

## 11. Binding negatives (any violation → halt token `binding-negative-violation`)

- No push-to-main
- No force-push (`--force`, `+refs/...`)
- No `--no-verify` (skipping hooks)
- No `--no-gpg-sign` unless the human explicitly asked
- No marking a Gate `[x]` before its verification command passes
- No adding deps outside the authorized list
- No edits to `crates/ripley-ipc/src/protocol.rs` semantics (wire format frozen)
- No deleting `tests/fixtures/` content
- No history rewrite (`git rebase -i`, `--amend` on pushed commits)
- No mocking the DB/advisory cache in tests written to hit it
- No `npm` or `yarn` anywhere
- No hex colors outside `apps/desktop/src/styles/theme.css`
- No hand-written `invoke<T>("cmd")` — use generated `bindings.ts`
- No `// removed`-style ghost comments
- No decorative or multi-paragraph docstrings
- No Chrome DevTools MCP in CI (it is agent-only, dev-loop only)
- No treating `just test:browser` as a substitute for the native macOS
  cold-path latency measurement

## 12. Acceptance commands

Run from repo root after M28 merges. Every command must exit 0. These are the
machine-checkable definition of done.

```bash
# Rust workspace health
cargo build --workspace --release
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo deny check

# Frontend workspace health
pnpm install --frozen-lockfile
just check
pnpm -C apps/desktop test -- --run
pnpm -C apps/desktop typecheck

# Browser regression (Playwright)
just test:browser
just test:browser:prod

# Tauri e2e (Linux + Windows only)
case "$(uname -s)" in
  Linux|MINGW*|MSYS*|CYGWIN*) pnpm -C apps/desktop test:e2e ;;
  Darwin) echo "macOS e2e gap — Peekaboo + Vitest compensate" ;;
esac

# Bundle outputs
just build
test -f apps/desktop/src-tauri/target/release/bundle/macos/Ripley.app/Contents/MacOS/Ripley
spctl --assess --type execute --verbose apps/desktop/src-tauri/target/release/bundle/macos/Ripley.app

# Type-safe IPC
test -f apps/desktop/src/lib/bindings.ts
! git check-ignore apps/desktop/src/lib/bindings.ts
! rg ':\s*any\b' apps/desktop/src/lib/bindings.ts

# Iced retirement
test ! -d crates/ripley-app
! rg -i 'iced|tray-icon|muda|cargo-bundle' Cargo.toml crates/*/Cargo.toml

# Design tokens
pnpm -C apps/desktop lint -- --max-warnings 0
! rg '#[0-9a-fA-F]{3,8}\b' apps/desktop/src/ --glob '!**/theme.css' --glob '!**/__snapshots__/**'

# Docs
test -f apps/desktop/docs/mcp-verification.md
test -f docs/install/macos.md
test -f docs/install/linux.md
test -f docs/install/windows.md
test -f apps/desktop/playwright.config.ts
test -d apps/desktop/tests/browser/views

# Baselines + bundle size
test -f apps/desktop/tests/browser/__snapshots__/lighthouse-baseline.json
test -f apps/desktop/dist/stats.html

# PLAN.md gates all checked
! awk '/^### M2[4-8]:/,/^### M(29|[3-9])|^## /{print}' PLAN.md | grep -E '^\s*-\s*\[ \]'
! awk '/^#### Phase 6 Gate/,/^---/{print}' PLAN.md | grep -E '^\s*-\s*\[ \]'

# Phase 1-5 regression
cargo run -p ripley-guard -- scan --format json tests/fixtures/ | jq -e '.matches'
cargo run -p ripley-guard -- guard status
```

## 13. Stop conditions

Emit the success token only when **all of**:

1. Every acceptance command above exits 0 on a clean checkout of `main`.
2. CI is green on `main` for the most recent commit.
3. `release-plz` produced a tag and bundles for v0.6.0.
4. PLAN.md M24-M28 + Phase 6 Gate + Total Plan Gate Phase 6 items are all `[x]`.
5. `crates/ripley-app` does not exist on `main`.

## 14. Working style

- Before each commit: `cargo fmt`, `cargo clippy`, the relevant test subset.
- Before each PR: full `just check` plus `just test:browser`.
- Commit messages: Conventional Commits, one logical change per commit.
- PR descriptions: paste the Gate block (with `[x]`s), one-paragraph summary,
  screenshots for UI work.
- Flaky Playwright spec → root-cause and fix; do not add retries. Flake
  budget is zero.
- UX decisions: re-read UX_DESIGN.md before improvising.
- Maintain `apps/desktop/DESIGN_NOTES.md` for judgment calls.

## 15. Reference for the human (not part of the loop)

### Pre-flight before running `/goal`

- `gh auth status` shows authenticated
- `cargo --version`, `rustc --version` match `rust-toolchain.toml`
- `mise install` succeeds
- Apple Developer credentials available (or accept halt at M28.1)
- Windows runner accessible (or accept halt at M28.2)
- GH Actions secrets: `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`,
  `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `WINDOWS_CERT`,
  `WINDOWS_CERT_PASSWORD`, `TAURI_PRIVATE_KEY`, `TAURI_KEY_PASSWORD`
- `release-plz.toml` configured
- Branch protection on `main`: PR required + CI required
- Session set up to survive long-running work

### Expected halt points (build a queue)

| Likely turn | Halt code | What you do |
|---|---|---|
| Very early | `preflight-env-missing` | Fix the listed env item, re-invoke `/goal` |
| Pre-M28.1 | `apple-creds` | Add Apple secrets to GH Actions, re-invoke |
| Pre-M28.2 | `windows-signing-key` | Add Windows cert to GH Actions, re-invoke |
| Pre-M28.4 | `ed25519-updater-key` | Generate keypair, add private key to GH Actions, commit pubkey, re-invoke |
| Possibly mid-M27 | `ambiguous-visual-diff` | Eyeball the diff, approve or reject, re-invoke |

After each halt: address the issue, then re-run `/goal "<same condition>"` to
resume.

### Why this structure

- **The condition is short and transcript-checkable.** The evaluator reads
  text; that's all it can see. Success and halt are spelled out as exact
  tokens so detection is mechanical, not interpretive.
- **GOAL.md is the per-turn manual, not the condition.** The 4000-char
  ceiling is on the condition; the manual can be as long as needed and lives
  in the repo where every turn re-reads it.
- **Halts are tokens, not narrative.** "I'm stuck" wouldn't terminate the
  loop because the evaluator would just say "not met, continue." The
  enumerated tokens make halts deterministic.
- **Turn cap of 500.** Hard upper bound on runaway loops. Phase 6 is large
  but fits in well under 500 turns at sensible granularity.
- **No state outside the repo except `turns`.** Git + PLAN.md + TaskList are
  the system of record. The single `turns` counter exists only because the
  condition needs a way to enforce its own cap.

---

## The condition (verbatim, paste this)

Everything below this line is what the human pastes into `/goal`. It is kept
≤4000 characters so it fits the limit.

```
Phase 6 autonomous run for Ripley (Tauri 2 UI rewrite, milestones M24-M28).

OPERATING MANUAL: Claude must read /Users/jack/src/github.com/agentstation/ripley/GOAL.md at the start of every turn before doing any other work. That file contains the turn-start ritual, turn-end ritual, halt token grammar, source-of-truth precedence, per-milestone protocol, acceptance commands, and binding negatives. Follow it exactly.

THE GOAL IS MET when the most recent assistant turn contains, on its own line, ANY of:
(A) The exact string: Phase 6 complete. Release v0.6.0 cut.
(B) A halt token of the form: [HALT-AND-WAIT: <code>] <one-line context>
    where <code> is one of: apple-creds, windows-signing-key, ed25519-updater-key, unknown-dep, three-retries-failed, ambiguous-visual-diff, spec-conflict, suspicious-test, unspecced-iced-behavior, notarization-rejected, new-cargo-deny-advisory, top-level-doc-edit-needed, preflight-env-missing, binding-negative-violation
(C) The exact string: [HALT-MAX-TURNS]
    (emit when apps/desktop/.ripley-agent/turns >= 500)

THE GOAL IS NOT MET otherwise. Even if Claude made progress, fixed a test, merged a PR, or marked a Gate [x] — only the explicit tokens above end the run. Continue the loop.

DO NOT terminate on partial progress, transient failures, or single red tests. Only on the tokens above.

OPERATING SUMMARY (full version in GOAL.md):

Source-of-truth precedence: PLAN.md > STACK_DECISION.md > UX_DESIGN.md > DESIGN.md > CLAUDE.md > WORKFLOW.md / UI.md / SETTINGS.md / ARCHITECTURE.md. Judgment calls -> apps/desktop/DESIGN_NOTES.md.

Per-milestone protocol (M24.0 -> M24.7 -> M25 -> M26 -> M27 -> M28): branch feat/m{N}-<kebab>; implement sub-milestones in PLAN.md order with Verify: exit 0 each step (3 no-progress failures -> halt three-retries-failed); full Gate to 0 then mark [x]; Conventional Commits + Co-Authored-By Claude; gh pr create (title M{N}: <title>, body = Gate + test summary); gh run watch; gh pr merge --squash --delete-branch. After M28 merges: run every acceptance command in GOAL.md to 0, git tag -s v0.6.0, git push --tags, emit success token.

Turn-start: cat GOAL.md; git log --oneline -20; git branch --show-current && git status; gh pr list --state open --author @me; scan PLAN.md M24+ Gates for lowest-numbered [ ]; emit STATUS line.

Turn-end: ONE of: (commit + push progress + 1-line summary, with no dangling background shells) OR (emit halt token) OR (emit success token). Increment apps/desktop/.ripley-agent/turns at turn-end.

Binding negatives (any violation -> emit [HALT-AND-WAIT: binding-negative-violation] <which rule> and back out the change): no push-to-main; no force-push; no --no-verify; no [x] before its verify passes; no unauthorized deps (only @playwright/test, @axe-core/playwright, playwright-lighthouse, eslint-plugin-jsx-a11y pre-authorized beyond what is already locked); no edits to ripley-ipc::protocol semantics; no deleting tests/fixtures/; no history rewrite; no DB mocking in DB-hitting tests; no npm/yarn; no hex outside apps/desktop/src/styles/theme.css; no hand-written invoke<T>(); no ghost // removed comments; no decorative/multi-paragraph docstrings; no Chrome MCP in CI; no Playwright as substitute for native macOS cold-path latency measurement.

ultrathink. Read GOAL.md every turn.
```
