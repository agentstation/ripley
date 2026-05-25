# Phase 6 design notes

Judgment calls made during the M24-M28 autonomous run. Each entry: what, why,
source-of-truth path.

## M24.1 — stub `ripley-desktop` crate

PLAN.md M24.1 adds `apps/desktop/src-tauri` to the root Cargo.toml `members`
array, but the directory only gets its full Tauri scaffold in M24.2. To keep
`cargo build --workspace` green at the M24.1 boundary, M24.1 lands a minimal
stub: `apps/desktop/src-tauri/Cargo.toml` with workspace-inherited metadata
and a single `fn main() {}` binary. M24.2 overwrites both files with the real
Tauri 2 scaffold.

Source: PLAN.md M24.1 verify command requires exit 0 on cargo build.

## M24.1 — `just check` skips JS steps when `apps/desktop/package.json` absent

`just check` is wired in M24.1 but the React app doesn't exist until M24.2.
The recipe runs Rust checks unconditionally and JS checks (`pnpm -F desktop
typecheck`, `lint`) only when `apps/desktop/package.json` exists. `pnpm audit`
runs against the root workspace either way. Same conditional applied to
`fmt`, `lint`, and `test` recipes.

Source: PLAN.md M24.1 verify command requires `just check` exit 0; PLAN.md
M24.6 fully wires the JS lint/format/test rigging.
