# Ripley desktop

Tauri 2 + React 19 shell for the Ripley supply-chain defense app. The
shell renders the guard dialog, the dashboard, and the settings UI; the
heavy lifting (scan, guard, audit, behavioral analysis, sandbox) lives
in the Rust workspace at `crates/`.

See `STACK_DECISION.md` (repo root) for the locked stack and
`PLAN.md` for the Phase 6 milestone breakdown.

## Layout

```
apps/desktop/
├── src-tauri/                 # Rust shell (Tauri commands, IPC bridge, tray)
│   ├── src/                   # ripley-desktop crate
│   ├── tests/                 # Rust integration tests
│   ├── capabilities/          # Tauri ACL JSON
│   └── icons/                 # Bundle icon set (16/32/64/128/256/512 + ico/icns)
├── src/                       # React UI
│   ├── components/ui/         # shadcn primitives (copied in, not depended on)
│   ├── routes/                # Page-level views (GuardDialog, etc.)
│   ├── store/                 # Zustand slices
│   ├── lib/bindings.ts        # tauri-specta generated bindings (committed)
│   └── styles/theme.css       # DESIGN.md design tokens (only place hex lives)
└── tests/
    ├── browser/               # Playwright (Vite-served React, all 3 OSes)
    ├── e2e/                   # WebdriverIO (Tauri shell, Linux + Windows)
    └── peekaboo/              # macOS screenshot scripts (developer loop)
```

## Tests

| Suite             | Where                              | Runs on                    | Drives                                      |
| ----------------- | ---------------------------------- | -------------------------- | ------------------------------------------- |
| Rust unit + integ | `src-tauri/tests/`, `#[cfg(test)]` | macOS + Linux + Windows    | Rust code                                   |
| Vitest            | `src/**/*.test.tsx`                | macOS + Linux + Windows    | React components in jsdom                   |
| Playwright        | `tests/browser/`                   | macOS + Linux + Windows    | Vite-served React (no Tauri shell)          |
| WebdriverIO       | `tests/e2e/`                       | **Linux + Windows only**   | Tauri shell (via `tauri-driver`)            |
| Peekaboo          | `tests/peekaboo/`                  | macOS dev loop, **not CI** | Tauri shell (via macOS AX + screen capture) |

## The macOS WKWebView WebDriver gap

WebdriverIO drives Tauri via the platform's WebView WebDriver server:

- Linux WebKitGTK → `WebKitWebDriver` (works)
- Windows WebView2 → `msedgedriver` (works)
- macOS WKWebView → **no WebDriver server exists**

This is not a "we'll fix it later" item — Apple does not ship a WebDriver
binding for WKWebView, and there is no working community fork. Tauri's
own docs call this out as an accepted constraint.

### How we compensate

We layer three independent signals so the macOS gap does not become a
quality gap:

1. **Playwright on all three OSes.** `tests/browser/` drives the
   Vite-served React app under Chromium. The WKWebView limitation does
   not apply because Playwright does not touch the Tauri shell. On
   macOS, this is the only browser-level signal but it covers a11y
   (axe-core), perf (Lighthouse ≥90), and DESIGN.md token conformance
   via `getComputedStyle`.

2. **Rust integration tests on macOS CI.** `src-tauri/tests/` exercises
   the IPC bridge, the capabilities ACL surface, and the prewarm
   monitor-selection math. These run on `macos-14` in CI alongside
   Linux and Windows.

3. **Peekaboo screenshot loop, manual.** macOS-only AX + screen-capture
   automation for the visual layer (tray icon, dialog framing on
   Retina, dark/light mode). Scripts at `tests/peekaboo/` are a
   developer tool, not a CI gate.

We do not "skip" macOS e2e — we replace WebDriver-style tests with this
three-signal stack. The trade-off: no auto-clicking the guard dialog on
macOS in CI. That trade is bounded by the WebdriverIO Linux + Windows
runs, which exercise the same React + IPC + Rust paths.

## Running locally

```
just dev              # Tauri dev server, hot reload (pnpm + vite + cargo)
just check            # clippy + fmt + tsc + eslint + cargo deny + pnpm audit
just test             # cargo test --workspace + pnpm -F desktop test
just test:browser     # Playwright against Vite preview
pnpm -F desktop test:e2e   # WebdriverIO (Linux + Windows only)
just guard-bench      # latency bench against a running Ripley
```

## Bundles

`pnpm -F desktop tauri build` emits the platform-native bundle:

- macOS: `.app` + `.dmg`
- Windows: `.msi` (NSIS `.exe` if configured)
- Linux: `.AppImage` + `.deb` + `.rpm`

CI uploads all of these per-target. Code signing (Apple Developer ID,
Azure Trusted Signing for Windows) lands in M28.
