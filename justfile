default:
    @just --list

# --- Top-level recipes --------------------------------------------------------

dev:
    pnpm -F desktop tauri dev

build:
    cargo build --workspace --release
    pnpm -F desktop tauri build

test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test --workspace
    if [ -f apps/desktop/package.json ]; then
      pnpm -F desktop test -- --run
    fi

check:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo deny check
    if [ -f apps/desktop/package.json ]; then
      pnpm -F desktop typecheck
      pnpm -F desktop lint
    fi
    pnpm audit

lint:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo clippy --workspace --all-targets -- -D warnings
    if [ -f apps/desktop/package.json ]; then
      pnpm -F desktop lint
    fi

fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo fmt --all
    if [ -f apps/desktop/package.json ]; then
      pnpm -F desktop format
    fi

e2e:
    pnpm -F desktop test:e2e

test-browser:
    pnpm -F desktop test:browser

test-browser-ui:
    pnpm -F desktop exec playwright test --ui

test-browser-prod:
    pnpm -F desktop test:browser:prod

guard-bench:
    cargo bench -p ripley-core --bench guard_latency
