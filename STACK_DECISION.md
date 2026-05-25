# UI Stack Decision

Status: **Decided 2026-05-23.** Canonical versions + tooling locked 2026-05-23.

**Stack:** Tauri 2.11 + React 19.2 + shadcn/ui (Base UI primitive) + Tailwind v4 +
Zustand 5 + TanStack Query v5 + `tauri-specta` v2 typed IPC, on top of the
existing `ripley-core` / `ripley-ipc` / `ripley-guard` Rust workspace. pnpm + Vite 8
+ TypeScript 6. Cross-platform GUI v1: macOS, Linux, Windows.

This document captures the reasoning. Implementation begins immediately — no
pre-decision spikes (rationale at the end). Canonical versions, tooling, software
patterns, and repo layout are locked in dedicated sections below.

Companion docs: [UX_DESIGN.md](UX_DESIGN.md) maps each [WORKFLOW.md](WORKFLOW.md)
stream to the specific Base UI primitives this stack provides, and locks the
keyboard model + DX patterns for the M27 view migration.

## Hard constraints (locked)

1. **Cross-platform GUI is a v1 requirement.** macOS + Linux + Windows from first
   release. Not "macOS first, port later." The threat model (compromised packages)
   is platform-agnostic; the CLI (`ripley-guard`) is already cross-platform; the
   GUI cannot be the platform bottleneck. Retrofitting cross-platform after building
   macOS-only is the most expensive failure mode and is explicitly rejected.
2. **Preserve `ripley-core` Rust crate.** 510+ tests, business logic, advisory cache,
   detection rules. Any stack must call into it without rewriting.
3. **Tray-resident.** App lives in the menu bar / system tray, not the Dock /
   taskbar.
4. **Native-feeling dense UI** matching `DESIGN.md` (GitHub-dark palette,
   terminal-adjacent).

These constraints **eliminate any macOS-only path** (SwiftUI MenuBarExtra, the
CodexBar hybrid pattern) regardless of how nice the macOS experience would be.
The CodexBar reference review below is kept as architectural reference, not a
candidate stack.

## Why we're re-evaluating

Grounded in `DESIGN_ISSUES.md`. Symptoms surfaced during the design-iteration loop:

1. **No tray icon.** `crates/ripley-app/src/tray.rs` is fully implemented but never
   instantiated — `main.rs` calls `app::run()` directly. The structural reason is
   that mixing `tray-icon` + `muda` with iced's `winit`-driven event loop on macOS
   is awkward; there is no first-party path.
2. **No native menu bar.** Iced 0.13 has no `NSMenu` / `NSStatusItem` APIs.
3. **No AX tree.** Iced renders to a single wgpu/Metal canvas — Peekaboo's `see` /
   `click` return empty, breaking the screenshot-driven design loop we just set up.
4. **Design drift.** Current dashboard is ~15% of `DESIGN.md`. theme.rs has half
   the tokens and many are `#[allow(dead_code)]`; `button()` inherits iced's purple
   default.

(1)–(3) are structural to iced. (4) is fixable in any stack. The question is
whether to fix-in-place or migrate.

## What "fix in place" means

Iced 0.13 + bolt-on `tray-icon`. We've established this path requires:

- Manual `winit` integration to share the event loop with `tray-icon` (no first-party
  story; community examples exist but are brittle on macOS).
- No `NSMenu` / `NSStatusItem` — we'd write a thin `objc2` shim ourselves.
- No AX tree — Peekaboo automation is keyboard-only (current `Cmd+1..7` hack).
- Continuing design work proceeds, but every "native macOS feel" item in
  `DESIGN_ISSUES.md` either ships compromised or requires raw `objc2` work.

This is viable. It is not free. Estimated 2-3 weeks of structural work *before*
addressing any of the 13 cross-cutting design issues — and the AX-tree limitation
is permanent.

## Candidates

Iced is eliminated as the *long-term* stack for the reasons above. Realistic
candidates:

### Tauri 2 (React + shadcn/ui on Base UI + Tailwind frontend, Rust backend)
- **Tray + menu bar:** first-class on all three platforms. `TrayIconBuilder` with
  per-OS backends: NSStatusItem on macOS, Shell_NotifyIcon on Windows,
  StatusNotifierItem/libappindicator on Linux.
- **AX tree:** full — WebView exposes the DOM accessibility tree natively on
  every platform (WKWebView / WebView2 / WebKitGTK). Peekaboo loop works on macOS
  out of the box; for cross-platform e2e the canonical 2026 path is WebdriverIO 9
  + `tauri-driver` (Linux + Windows CI; macOS WKWebView gap documented as accepted
  constraint and covered by Vitest + Peekaboo screenshots). Playwright 1.x is
  layered on top as a **browser-regression** harness against Vite-served React
  (not the Tauri shell, so the WKWebView limitation does not apply); it runs on
  all three OS runners and on macOS is the browser-level signal that further
  shrinks the WebDriver gap (a11y via `@axe-core/playwright`, scoring via
  `playwright-lighthouse`, visual diffs, perf budgets). Chrome DevTools MCP is
  the agent-only verification surface for PR self-review — never a runtime dep,
  never in CI.
- **Component library:** shadcn/ui with **Base UI** (`@base-ui/react` 1.x) as the
  primitive layer (officially supported since shadcn's Jan-2026 release; CLI v4
  March-2026 stabilized the `--base base-ui` flag), Tailwind v4 for styling,
  copy-into-repo ownership model. Frontend library choice detailed below;
  rationale is that DESIGN.md specifies *terminal-adjacent* density, not GitHub's
  app aesthetic.
- **Performance:** guard dialog cold start is the risk. Mitigation: pre-warmed
  hidden window, shown on demand. Budget 200-400ms first show, <50ms thereafter.
  Same pre-warm pattern works on all three OSes.
- **Migration:** ~4-7 weeks. Rewrite views in TSX; `ripley-core` / `ripley-ipc` /
  `ripley-guard` unchanged.
- **Risks:** Linux tray needs libappindicator or extension on GNOME (known long-tail
  pain); `set_activation_policy` runtime bug on Tauri 2.x; notarization on macOS +
  code signing on Windows + AppImage/.deb/.rpm on Linux multiplies release ops;
  WebView memory baseline (~80-150 MB) higher than native.
- **Cross-platform:** This is Tauri's home field. Single codebase, three bundle
  targets, single design system.

### ~~SwiftUI + Rust core~~ — eliminated by cross-platform constraint
SwiftUI is macOS/iOS-only. Meeting Constraint 1 would require a second GUI stack for
Linux/Windows, which is exactly the "second project" failure mode we are rejecting.
Kept here only because the CodexBar review (below) references its architecture as
a reusable pattern in the event we ever reverse Constraint 1.

### Slint
- **Tray + menu bar:** first-party `NSMenu` via muda since Slint 1.10. Tray on
  Linux via StatusNotifierItem and on Windows via Shell_NotifyIcon. Royalty-free
  desktop license for our use case.
- **AX tree:** AccessKit integration status is the hard gate. **Verify current state
  before committing.** No AccessKit = no Peekaboo design loop = we lose the
  workflow we just established. This is a deal-breaker for design iteration speed.
- **Design system:** custom `.slint` DSL. GitHub-dark palette built once, ships
  identically on all three platforms. No off-the-shelf component library matching
  shadcn/ui's depth — every severity badge, virtualized table, command palette is
  custom.
- **Performance:** native-grade everywhere. Small binary (~5-15 MB), low memory
  (~20-40 MB baseline). Guard dialog cold start is excellent across all platforms.
- **Migration:** ~4-6 weeks for view rewrites + DSL learning curve. Pure Rust
  toolchain — no Node, no FFI shim.
- **Risks:** smaller ecosystem than Tauri/React; component library gap; AccessKit
  is the single biggest concrete unknown; designer hiring/contracting harder
  (DSL vs. industry-standard React).
- **Cross-platform:** First-class on macOS/Linux/Windows. Single binary per
  platform, no runtime dependency.

### GPUI (Zed's framework)
- **Tray + menu bar:** `NSMenu` works, no `NSStatusItem` API as of investigation.
- **AX tree:** no AccessKit.
- **Status:** pre-1.0, Zed-internal churn, breaking changes likely.
- **Verdict:** wildcard. Strong long-term potential, not appropriate for a product
  we want to ship and stabilize.

## Comparison matrix (cross-platform-v1 lens)

SwiftUI and any other macOS-only path is omitted from this matrix per Constraint 1.

| Criterion | Iced (status quo) | **Tauri 2** | **Slint** | GPUI |
|---|---|---|---|---|
| Cross-platform GUI v1 | partial (broken tray) | yes, first-class | yes, first-class | macOS-mature, others rough |
| Tray icon, all 3 OS | bolt-on, fragile | native per-OS | native per-OS | partial |
| Menu bar / status item | objc2 shim | native everywhere | native (1.10+) | manual |
| AX tree (Peekaboo loop) | none | full (WebView DOM) | **needs verification** | not yet |
| GitHub-dark palette | manual | tokens in Tailwind config | custom in `.slint` | manual |
| Component library depth | minimal | shadcn/ui on Base UI (full recipe set) | shallow, custom | shallow |
| Guard dialog cold start | excellent | 200-400ms (mitigated) | excellent | excellent |
| Memory baseline | ~30 MB | ~80-150 MB | ~20-40 MB | native |
| Toolchain | Rust | Rust + Node + system WebView | Pure Rust | Pure Rust |
| Component library depth | minimal | deep (Primer + React eco) | shallow, custom | shallow |
| Designer hireability | n/a | React (industry-standard) | `.slint` DSL (niche) | niche |
| Migration cost | 2-3 wk patch (compromised) | 4-7 wk | ~4-6 wk | unknown, high |
| Linux tray reliability | n/a | GNOME extension dep | StatusNotifierItem | unknown |
| Release ops (3 OS) | n/a | notarize+sign+AppImage | sign+package per OS | unknown |
| Ecosystem maturity | mature | mature | maturing | pre-1.0 |

## Mapping to Ripley's requirements

From `UI.md` / `ARCHITECTURE.md` / `DESIGN.md`, under Constraint 1:

1. **Cross-platform GUI on day one.** → Tauri and Slint only. Tauri is its native
   home; Slint is genuinely first-class on all three.
2. **Tray-resident across all 3 OS.** → Both candidates handle this; both have
   the same Linux long-tail risk (StatusNotifierItem / libappindicator).
3. **Guard dialog <500ms after npm install kick-off, on every OS.** → Slint native
   wins on cold start. Tauri needs pre-warm to meet the budget on first invocation
   per session — same pattern on every OS.
4. **Dense, terminal-adjacent UI with GitHub palette.** → Tauri+Primer ships
   `DESIGN.md` for free, identically on all three. Slint requires building it
   once in the DSL.
5. **Peekaboo-driven design iteration.** → Tauri works today. Slint needs
   AccessKit verification — if AccessKit is not landed and stable, the
   design-iteration loop breaks and Slint is out.
6. **Reuse `ripley-core` / `ripley-ipc` / `ripley-guard` Rust.** → Both keep the
   Rust core untouched. Tauri calls in via `tauri::command`. Slint calls in
   directly (same process, Rust-native).
7. **Designer / contractor availability.** → Tauri (React) is industry-standard;
   Slint (`.slint` DSL) limits the pool.

## Recommendation

**Tauri 2 + React + shadcn/ui (Base UI primitive) + Tailwind** — recommended without
"lean."

Under the locked cross-platform-v1 constraint the comparison narrows to Tauri vs Slint,
and Tauri wins on four axes that matter for *this* product:

- **Component depth and customizability.** `DESIGN.md` specifies "dark, dense,
  terminal-adjacent" with monospace-forward layout — explicitly *not* a designer-built
  dashboard aesthetic. shadcn/ui on Base UI + Tailwind gives us copy-into-repo
  component code we own and can re-density without fighting an opinionated library.
  Virtualized tables (TanStack), command palette (Base UI Combobox + match-sorter),
  dialogs/toasts/popovers (Base UI primitives via shadcn) are battle-tested recipes.
  Slint requires building each one from scratch in `.slint` DSL.
- **Peekaboo loop is preserved.** Tauri exposes the DOM accessibility tree on
  every platform, so the screenshot-driven design iteration workflow we just
  invested in keeps working. Slint may not, depending on current AccessKit status.
- **Designer / contractor availability.** React + Tailwind + shadcn is the
  industry-standard 2026 frontend stack. Hiring or contracting design and frontend
  work is dramatically easier than for `.slint` DSL.
- **Iteration velocity.** Hot-module reload on TSX, instant CSS via Tailwind, DOM
  inspection in DevTools. Slint has tooling but the loop is slower in practice.

Slint genuinely beats Tauri on guard-dialog cold start, memory footprint, and
deployment simplicity. Those wins are real but secondary: cold start is mitigated
by pre-warm; memory cost (~80-150 MB vs ~20-40 MB) is acceptable for a developer
tool; release ops complexity is a one-time setup.

The honest tradeoffs we are accepting by choosing Tauri:
- Linux tray reliability on GNOME without extensions is a known issue. We will
  ship clear setup docs and accept that GNOME-vanilla users see degraded tray UX.
- WebView memory baseline. Acceptable for our user (developers on workstation-class
  machines).
- WebView2 on Windows ships with the OS now (Win10 1803+), but bootstrap installer
  is still required for older targets — keep Windows target at Win10 22H2+.
- `set_activation_policy` Tauri-2.x bug needs verification on current release.
- Release-ops complexity (notarize macOS, sign Windows, AppImage/.deb/.rpm on
  Linux) is real one-time work, then automated in CI.

**Slint was the credible alternative**, kept in the Candidates section above as
architectural reference. We did not choose it because (a) AccessKit status would
break our screenshot-driven design iteration loop, (b) the DSL learning curve and
hireability cost real velocity, and (c) component-library depth in shadcn/Base UI
is a 5+ year ecosystem advantage we don't want to forfeit. If Tauri ever proves
structurally unworkable for us, we will re-decide from current data — not from a
stale contingency clause.

## Frontend component library (inside Tauri)

DESIGN.md specifies "dark, dense, terminal-adjacent — a security operations
interface … monospace-forward. It looks like it belongs next to a terminal, not a
dashboard built by designers for executives" (lines 6-14, 281). The palette is
lifted from GitHub-dark; the *layout philosophy* is the opposite of github.com's
app UI. This rules out using GitHub's own component library — Primer ships
GitHub's app aesthetic, not a terminal-density UI.

### Recommended frontend stack

Foundation-first principle applied throughout: where a long-term-correct choice
is known, we commit to it now rather than carrying a temporarily-easier dependency
we'd port out later.

| Layer | Choice | Rationale |
|---|---|---|
| Runtime | **React 19** | 2026 default; required for Tauri 2 / Vite ecosystem alignment. |
| Build tool | **Vite** (Tauri default) | Tauri ships with Vite preconfigured. |
| Package manager | **pnpm** | Foundation-correct 2026 default. Strict node_modules layout catches phantom deps — relevant for a supply chain tool. |
| Styling | **Tailwind CSS v4** | Utility-first matches DESIGN.md's token-heavy spec. `@theme` block in CSS is the right home for our palette. |
| Components | **shadcn/ui with Base UI primitive** (`pnpm dlx shadcn@latest init` then `--base base-ui`; style `base-vega`; package `@base-ui/react` 1.x) | Own the code, accessibility-grade via MUI-team-backed Base UI, cleaner TypeScript + render-prop ergonomics than Radix. |
| Command palette | **Base UI `Combobox` + `match-sorter`** | One primitive layer, one search library across the app. No cmdk (see below). |
| List filtering / sorting / fuzzy search | **`match-sorter`** | Tanstack-ecosystem-aligned, ~5KB, primitive-agnostic. Used for command palette, alert table search, audit log filter, settings search — one library covers all. |
| Virtualized tables | **TanStack Table + TanStack Virtual** | Required for large alert/audit lists; shadcn DataTable recipe wraps them; primitive-agnostic. |
| State management | **Zustand** | No providers, TS-native, primitive-agnostic. Pairs naturally with Tauri's invoke-based IPC (event handlers update store; views subscribe). |
| Icons | **Lucide React** | shadcn default. Monoline outlined style matches terminal aesthetic. |
| Fonts | system-ui + ui-monospace | Already specified in DESIGN.md — no font dep, no web font load. |
| Design tokens | Inlined from DESIGN.md into Tailwind v4 `@theme` | Zero vendor coupling. DESIGN.md is already self-contained. |

### Why Base UI primitive (not Radix)

shadcn officially supports both Radix and Base UI as the underlying primitive layer
since January 2026 — same shadcn component API, switchable via a single line in
`components.json` (`"style": "base-vega"` vs `"style": "new-york"`). For a greenfield
2026 project the calculus favors Base UI on every axis that matters to us:

- **Maintenance trajectory.** Radix slowed materially after WorkOS acquired it.
  Base UI has dedicated full-time MUI engineers. For a 5+ year product foundation,
  the active-investment delta is the dominant factor.
- **Cleaner API for our exact components.** Native multi-select Combobox with
  search (Radix Select doesn't support search natively; shadcn ships cmdk to fill
  the gap). Native Combobox + Menu means we may collapse the cmdk dependency once
  Base UI's primitives prove out in our spike.
- **TypeScript / styling ergonomics.** Render props expose component state to JSX
  directly, instead of Radix's `data-[state=open]:` CSS-attribute selector pattern.
  For terminal-dense UI with heavy state-conditional styling (severity row hover,
  expandable details, badge variants), render props produce more readable code.
- **Smaller, self-contained.** Single package vs Radix's many `@radix-ui/*`
  packages. Dialog ships ~6.4 KB gzipped vs Radix's ~9.2 KB. `node_modules`
  footprint smaller, lockfile cleaner.
- **Production users de-risk the choice.** Cloudflare's Kumo component library,
  cal.com creators' coss ui, Fragments design system platform. At Base UI 1.5.0
  the API is stable.

### The one API change to internalize

Radix composes via `asChild`; Base UI uses a `render` prop. Functionally
equivalent but the patterns differ:

```tsx
// Radix
<Tooltip.Trigger asChild>
  <button>Click</button>
</Tooltip.Trigger>

// Base UI
<Tooltip.Trigger render={<button>Click</button>} />
// or
<Tooltip.Trigger render={(props) => <button {...props}>Click</button>} />
```

shadcn abstracts most of this — component-internal usage looks the same to the
app code. The pattern matters when we extend or fork a shadcn component.

### Why not "raw Base UI + Tailwind" without shadcn

shadcn is essentially "primitive + Tailwind + sensible defaults you can edit."
Skipping shadcn means re-deriving those defaults from scratch. There is no
ownership difference (shadcn writes into your repo; you own the code either way).
shadcn-on-Base-UI gives us recipe-completeness *and* the better primitive layer
in one move.

### Why not `@primer/react`

Aesthetic mismatch. Primer renders github.com — padded application UI, dashboard
layout patterns, modal density. DESIGN.md explicitly rejects "dashboard built by
designers." We would import Primer for the palette and then override almost every
component's spacing, radius, and density. Net effect: more work than starting from
shadcn, plus a vendor dependency we don't need.

Primer is the right answer for a tool whose visual identity *is* "looks like
GitHub." Ripley's visual identity is "looks like a terminal that uses GitHub's
colors" — different problem.

### Why not cmdk for the command palette

cmdk exists primarily because Radix `Select` does not support search. shadcn ships
cmdk to fill that specific gap. Under our Base UI primitive choice, the gap is
already filled by Base UI's `Combobox` (native filter, multi-select, search,
grouped items, keyboard nav). Carrying cmdk would buy us:

- Fuzzy scoring (~100 lines of application-layer code on top of `match-sorter`)
- Recency tracking (`localStorage` + sort key — trivial)
- A purpose-built cmd-K UX (we get this by composing Base UI Combobox correctly)

In exchange we'd accept:

- A second composition vocabulary alongside Base UI in the most user-visible
  keyboard surface of the app
- A separate keyboard model the command palette doesn't share with anything else
- A port-it-someday item that compounds with every command we add

**Decision:** drop cmdk. Build the command palette as a Base UI `Combobox` with
application-layer scoring (`match-sorter`) and recency (`localStorage`). One
primitive vocabulary across the app. The "save a week of work" argument is the
exact local optimization this stack decision is meant to avoid.

### Stack lock-ins this creates

- **Node toolchain in dev environment.** pnpm (locked), Vite (Tauri default), TS.
- **No Material-UI, no Chakra, no Mantine.** Those are app-shell aesthetics
  that fight DESIGN.md the same way Primer does. shadcn's headless+Tailwind
  model is structurally compatible with terminal density; opinionated libraries
  are not.
- **No cmdk, no Headless UI, no Ariakit, no Reka UI.** One primitive layer —
  Base UI — for everything that takes user interaction. Mixing primitives is
  exactly the kind of architectural debt this section exists to prevent.
- **Tailwind v4.** Token system via `@theme`, not `tailwind.config.ts`. DESIGN.md
  tokens live in a single `theme.css` imported at app entry.

**Not recommended:**
- GPUI: pre-1.0 risk + macOS-mature/others-rough disqualifies it under Constraint 1.
- Status quo iced: structural ceilings on tray, menu bar, and automation are
  permanent — every native-feel item in `DESIGN_ISSUES.md` ships compromised.
- SwiftUI (any variant, including CodexBar's NSStatusItem hybrid): violates
  Constraint 1. Macheted from candidate list, retained as architectural reference.

## Canonical 2026 versions (locked)

All versions verified against official sources May 2026. Adopt at these majors;
patch-level drift is expected and acceptable. Cargo/pnpm lockfiles are the
source of truth for exact pins.

| Layer | Package | Version | Notes |
|---|---|---|---|
| Desktop framework | `tauri` + `@tauri-apps/cli` + `@tauri-apps/api` | **2.11** | v2 schema, capabilities ACL in `src-tauri/capabilities/*.json` |
| Runtime | `react` + `react-dom` | **19.2** | stable GA; `createRoot` + StrictMode; ignore Server Components (SPA) |
| Build tool | `vite` + `@vitejs/plugin-react` | **8** + **6** | Rolldown integration; Oxc replaces Babel |
| Language | `typescript` | **6.0** | TS 7 (Go-native port) still beta — defer |
| Styling | `tailwindcss` + `@tailwindcss/vite` | **4** | `@theme` block in CSS, no `tailwind.config.ts` |
| CSS helpers | `clsx` + `tailwind-merge` v3 + `class-variance-authority` | latest | one `lib/cn.ts` re-exporting `cn`, `cva`, `cx` |
| Components | `@base-ui/react` (via shadcn) | **~1.5** | shadcn CLI v4: `pnpm dlx shadcn@latest init` + `--base base-ui`; style `base-vega` |
| Icons | `lucide-react` | latest | tree-shaken; ~1kb/icon |
| State (client) | `zustand` | **5.0.x** | slice pattern at >2 slices; middleware at combined-store level only |
| State (IPC) | `@tanstack/react-query` | **v5** | wraps `tauri-specta` commands as "server state" |
| Tables | `@tanstack/react-table` + `@tanstack/react-virtual` | **8.x** + **3.x** | v9 still alpha — defer |
| Fuzzy search | `match-sorter` | latest | command palette + table filters |
| Animation | `motion` (import `motion/react`) | **12.x** | Framer Motion rebrand; defer until needed |
| Path aliases | `vite-tsconfig-paths` | latest | single source of truth (tsconfig `paths`) |
| Type-safe IPC | `tauri-specta` (Rust) + `specta` v2 | **v2** | codegen TS types from `#[tauri::command]` — adopt day one |

## Tooling stack (locked)

Greenfield 2026 canonical, biased toward security/supply-chain posture (audit
breadth > raw speed; minimize Node deps; favor single-binary tools).

| Tool | Choice | Rationale |
|---|---|---|
| Package manager | **pnpm** | Strict node_modules layout catches phantom deps; required for a supply-chain tool. npm/yarn forbidden. |
| Lint | **ESLint 9 flat config** (`eslint.config.js`) + `typescript-eslint` v8 + `eslint-plugin-react`, `react-hooks`, `jsx-a11y` | Biome 2.4 still missing `react-hooks/exhaustive-deps` and security plugin breadth — disqualifying for a supply-chain tool. Revisit when Biome closes the gap. |
| Format | **Prettier 3** | Paired with ESLint via `eslint-config-prettier`. |
| Unit tests | **Vitest 2.x** + `@testing-library/react` 16+ + `@vitest/coverage-v8` | Vite-aligned default; native ESM; React 19 ready. |
| E2E tests | **WebdriverIO 9 + `tauri-driver`** | Tauri 2's officially documented path. Playwright lacks WKWebView support and is not Tauri-supported. Linux + Windows CI only; macOS gap covered by Vitest + Peekaboo screenshots. |
| Browser regression | **Playwright 1.x** + `@axe-core/playwright` + `playwright-lighthouse` | Drives Vite-served React (not the Tauri shell), so WKWebView limitation does not apply. Complements WebdriverIO: provides a11y (axe), Lighthouse scoring (a11y ≥95, perf ≥90, best-practices ≥95), visual diffs, computed-style DESIGN.md token assertions, perf budgets, and bundle-size ceilings on all three OS runners. Specs at `apps/desktop/tests/browser/`; invoked via `just test:browser`. Production-build coverage runs against `vite preview` on `:4173`. |
| Interactive verification | **Chrome DevTools MCP** (agent-only) | Agent-driven Chromium for PR self-review. Canonical loop: `new_page` → `navigate_page` → `take_snapshot` → `take_screenshot` → `list_console_messages` → `list_network_requests` → `lighthouse_audit` → `close_page`. Not a runtime dep; not part of CI. Documented at `apps/desktop/docs/mcp-verification.md`. |
| Git hooks | **Lefthook 1.x** (`lefthook.yml` at root) | Single Go binary, parallel execution, language-agnostic. No Node post-install side effect — important when Rust-only contributors lack Node. |
| Pre-commit staging | **Lefthook's `{staged_files}` glob** | Drop `lint-staged` — Lefthook handles staged filtering natively. |
| Versioning (Rust) | **`release-plz`** + Conventional Commits | Workspace-aware; drives `Cargo.toml` bumps, CHANGELOG, git tags, GitHub releases. Single source of truth for desktop app + CLI. |
| Versioning (JS) | **Changesets** (only if `packages/*` ever publishes JS) | Coexists cleanly with `release-plz` in polyglot monorepo. Defer until needed. |
| Task runner | **`just`** (`justfile` at root) | Rust-ecosystem-canonical; small Rust binary; clean arg syntax; right for mixed Rust+JS monorepo. |
| Tool version pinning | **`mise`** (`.mise.toml` at root) | Pins node, pnpm, just, lefthook, tauri-cli versions. Supply-chain hygiene. |
| JS dep audit | **`pnpm audit`** + `osv-scanner` in CI | Gate JS deps the same way `cargo deny` gates Rust deps. |

## Software patterns (locked)

### Type-safe IPC end-to-end via `tauri-specta`

Rust side: annotate `#[tauri::command]` handlers + DTOs with `#[derive(specta::Type)]`.
A `build.rs` step emits `apps/desktop/src/lib/bindings.ts` with typed `commands`
and `events`. Frontend imports `commands.scan()` and calls it as a typed function —
no hand-written `invoke<T>("scan")` calls.

**Adopt day one (M24).** Retrofitting later means rewriting every IPC call site.
Generated bindings are **committed**, not `.gitignore`d — same principle as
`Cargo.lock` (visible diff is the feature).

### Two-store state split

- **TanStack Query v5** owns everything that crosses the Tauri boundary
  (alerts, scan results, audit reports, guard log, monitor events). Treats invokes
  as cacheable async resources; query keys include scan parameters; `invalidateQueries`
  on Tauri event receipt.
- **Zustand 5** owns pure client UI state (selected view, expanded sections,
  modal open/closed, command palette query, recency list). Slice pattern,
  `persist` middleware for command-palette recency, `devtools` middleware in dev.

Do not mirror server state into Zustand. A `useEffect` per Tauri event channel
calls `queryClient.invalidateQueries(...)` or `queryClient.setQueryData(...)` on
event receipt.

### Routing

SPA, no router for v1. `App.tsx` switches on Zustand's `currentView` (same model
as the iced app). Add `@tanstack/react-router` only when deep-linking or
window-level URLs are needed.

### TS config split (Vite canonical)

- `apps/desktop/tsconfig.json` — references only
- `apps/desktop/tsconfig.app.json` — `target: ES2024`, `module: ESNext`, `moduleResolution: bundler`, `jsx: react-jsx`, `strict`, `noUncheckedIndexedAccess`, `verbatimModuleSyntax`, `erasableSyntaxOnly`, `isolatedModules`, `allowImportingTsExtensions`, `noEmit`
- `apps/desktop/tsconfig.node.json` — `vite.config.ts` + tooling

### Cargo workspace inheritance

Root `Cargo.toml` uses `resolver = "3"` (Rust 1.84+) and `[workspace.package]` for
shared metadata. Member crates declare `version.workspace = true`,
`license.workspace = true`, etc. `apps/desktop/src-tauri` is listed explicitly
in `members =` (not via glob — keeps non-Tauri future apps safe).

### Accepted constraint: macOS e2e gap

WKWebView doesn't expose WebDriver the way WebView2/WebKitGTK do. `tauri-driver`
runs reliably on Linux + Windows; macOS e2e is **covered by**:
1. Vitest component coverage (primary)
2. Rust integration tests for IPC bridge + commands
3. Peekaboo screenshot-based verification for tray UX

This is documented as a known constraint, not a future-fix item.

## Repo layout (locked)

Cargo workspace at root is load-bearing; Tauri lives inside `apps/desktop/` as a
workspace member. `apps/` (plural) is future-proof for a second product (mobile,
hosted) without renaming. `packages/` scaffolded with `.gitkeep` only — populated
when concrete consumers exist.

```
ripley/
├── Cargo.toml                      # workspace; resolver = "3"
│                                   # members = ["crates/*", "apps/desktop/src-tauri"]
├── Cargo.lock
├── rust-toolchain.toml
├── deny.toml
├── release-plz.toml                # NEW: Rust release automation
├── .mise.toml                      # NEW: pins node, pnpm, just, lefthook, tauri-cli
├── package.json                    # NEW: private root; scripts only (proxy to `just`)
├── pnpm-workspace.yaml             # NEW: packages: ["apps/*", "packages/*"]
├── pnpm-lock.yaml                  # NEW
├── lefthook.yml                    # NEW: git hooks (single Go binary)
├── justfile                        # NEW: top-level task runner
├── .gitignore                      # extended: node_modules/, dist/, *.tsbuildinfo
├── README.md / ARCHITECTURE.md / ... (docs at root, unchanged)
├── STACK_DECISION.md               # this file
│
├── crates/                         # PURE RUST (unchanged from Phase 1-5)
│   ├── ripley-core/
│   ├── ripley-ipc/
│   └── ripley-guard/
│   # ripley-app/ DELETED at end of M28 (iced retired)
│
├── apps/                           # END-USER PRODUCTS
│   └── desktop/
│       ├── package.json            # React 19, shadcn, Tailwind v4, Zustand, Query, etc.
│       ├── tsconfig.json / .app.json / .node.json
│       ├── vite.config.ts
│       ├── components.json         # shadcn: style "base-vega", primitive "base-ui"
│       ├── eslint.config.js        # flat config (ESLint 9)
│       ├── .prettierrc
│       ├── wdio.conf.ts            # WebdriverIO e2e (Tauri shell; Linux + Windows CI)
│       ├── playwright.config.ts    # Playwright (Vite-served React; all 3 OS runners)
│       ├── index.html              # Vite entry
│       │
│       ├── src/                    # React/TS frontend
│       │   ├── main.tsx            # createRoot + StrictMode
│       │   ├── App.tsx             # QueryClientProvider + view switch
│       │   ├── routes/             # one file per top-level view
│       │   │   ├── Alerts.tsx
│       │   │   ├── GuardLog.tsx
│       │   │   ├── DeepScan.tsx
│       │   │   ├── Monitor.tsx
│       │   │   ├── Audit.tsx
│       │   │   ├── Posture.tsx
│       │   │   └── Settings.tsx
│       │   ├── components/
│       │   │   ├── ui/             # shadcn-generated, owned in repo
│       │   │   └── ripley/         # app-specific (AlertCard, SeverityBadge, ...)
│       │   ├── lib/
│       │   │   ├── bindings.ts     # tauri-specta GENERATED — committed, do not edit
│       │   │   ├── ipc.ts          # event subscription helpers
│       │   │   ├── query.ts        # QueryClient + invalidation helpers
│       │   │   ├── cn.ts           # cn/cva/cx re-export
│       │   │   └── fuzzy.ts        # match-sorter helpers
│       │   ├── store/              # Zustand slices
│       │   │   ├── ui.ts
│       │   │   ├── command-palette.ts
│       │   │   └── index.ts        # combined w/ persist + devtools
│       │   ├── hooks/              # useQuery wrappers per resource
│       │   ├── styles/
│       │   │   └── theme.css       # @import "tailwindcss"; @theme { DESIGN.md tokens }
│       │   └── env.d.ts
│       │
│       ├── src-tauri/              # Tauri Rust backend (workspace member)
│       │   ├── Cargo.toml          # name = "ripley-desktop"; path deps to ../../../crates/*
│       │   ├── build.rs            # tauri-build + tauri-specta codegen
│       │   ├── tauri.conf.json     # v2 schema
│       │   ├── icons/              # tray + bundle icons
│       │   ├── capabilities/       # JSON ACL files (Tauri 2 idiom)
│       │   └── src/
│       │       ├── main.rs         # tauri::Builder
│       │       ├── lib.rs
│       │       ├── commands/       # #[tauri::command] handlers, one file per domain
│       │       ├── ipc_bridge.rs   # UDS subscriber → tauri::Emitter
│       │       ├── tray.rs         # TrayIconBuilder
│       │       └── prewarm.rs      # hidden window pre-warm
│       │
│       ├── tests/
│       │   ├── e2e/                # WebdriverIO (Tauri shell; Linux + Windows CI)
│       │   ├── peekaboo/           # macOS screenshot loop (tray/dialog visuals)
│       │   └── browser/            # Playwright (Vite-served React; all 3 OSes)
│       │       ├── smoke.spec.ts
│       │       ├── views/          # per-view: axe + Lighthouse + token + visual
│       │       ├── a11y/
│       │       ├── lighthouse/
│       │       ├── perf/           # mount-to-interactive, bundle-size budgets
│       │       ├── visual/
│       │       └── __snapshots__/  # baselines per OS, committed
│       └── docs/
│           └── mcp-verification.md # Chrome DevTools MCP agent loop
│
├── packages/                       # FUTURE shared JS (scaffolded, empty)
│   └── .gitkeep
│
└── tests/                          # workspace Rust fixtures (existing)
    └── fixtures/
```

**Cargo workspace member entry** (root `Cargo.toml`):

```toml
[workspace]
resolver = "3"
members = ["crates/ripley-core", "crates/ripley-ipc", "crates/ripley-guard",
           "apps/desktop/src-tauri"]
```

After M28 (iced retirement), `crates/ripley-app` is removed from `members`.

## Implementation phases (no spikes — build in the order that surfaces risk)

We deliberately skip pre-decision spikes. The decision-flipping risks were narrow
(only guard-dialog latency), the platform is mature, and "validate then build" too
often becomes theater. Instead, the work is structured into phases ordered so that
the same risks the spikes would have measured surface as forward progress. Each
phase gates on completion, not on a calendar.

| Stage | PLAN.md milestone | Scope | Risk it surfaces | Stage verification signal |
|---|---|---|---|---|
| **Stage 1 — Scaffold & shell** | **M24** | Tauri scaffold + tray icon + hidden pre-warm window + Cmd/Ctrl+Shift+R opens it. `tauri-specta` codegen wired day one. `ripley-core` linked via typed `tauri::command`. shadcn init with `--base base-ui`. | "Can Tauri be a tray-only app on macOS?" Falsifies `set_activation_policy(.Accessory)` if it's broken. | Playwright smoke + axe baseline + Lighthouse baseline recorded against the empty shell; `ripley/no-raw-hex` ESLint rule fires on a deliberate violation; Chrome DevTools MCP can drive `localhost:5173`. |
| **Stage 2 — Guard-dialog critical path** | **M25** | Simulated `npm install` interception → IPC → pre-warmed window shows → measured latency on real hardware. Tune until <500ms cold, <50ms warm. | The one genuine unknown. Real number on real hardware. | Native cold-path measurement on macOS is the contract; Playwright warm-mount spec (`performance.mark` round-trip in headless Chromium, <50ms) is the continuous regression signal at PR time, and axe scan of the dialog is clean of `serious`/`critical`. |
| **Stage 3 — Cross-platform parity** | **M26** | Linux + Windows VM builds. Tray on GNOME + KDE + Hyprland + Win 11. Document degraded configs. Tauri bundler outputs `.dmg`/`.app`, `.msi`, AppImage/`.deb`/`.rpm`. WebdriverIO e2e on Linux + Windows. | Linux tray reality. Windows code signing. WebDriver macOS gap. | WebdriverIO covers the Tauri shell on Linux + Windows; Playwright runs on **all three** OS runners against Vite-served React — on macOS, Playwright is the only browser-level signal and is what compensates for the WKWebView WebDriver gap. Lighthouse baseline JSON recorded per OS. |
| **Stage 4 — View migration** | **M27** | Port views in DESIGN_ISSUES.md priority order using shadcn/Base UI + Tailwind v4 with DESIGN.md tokens. Real `AlertCard`, `SeverityBadge`, `DataTable` (TanStack Table + Virtual via shadcn recipe). Command palette via Base UI `Combobox` + `match-sorter` (Cmd/Ctrl+K). | Density + palette compose cleanly. shadcn DataTable wraps TanStack as expected. Terminal-adjacent aesthetic actually achievable. | Every migrated view ships a Playwright spec (`tests/browser/views/<view>.spec.ts`) asserting: axe-clean (zero `serious`/`critical`); Lighthouse a11y ≥95, perf ≥90, best-practices ≥95; `getComputedStyle` matches DESIGN.md tokens; keyboard reachability per `KEYMAP`; visual diff baseline ≤0.1% pixel delta. `ripley/no-raw-hex` clean across `apps/desktop/src/**`. |
| **Stage 5 — Release ops & retire iced** | **M28** | macOS notarization + Windows signing + Tauri updater (Ed25519). `release-plz` Rust automation. Delete `crates/ripley-app`. Remove iced/tray-icon/muda/cargo-bundle deps. Per-platform install docs. | Release pipeline is the last hidden cost. | Playwright suite re-runs against the production build (`vite preview` on `:4173`) — verifies minified bundle behavior, not just dev-server output. Main JS chunk ≤250KB gzipped enforced as a regression gate. Lighthouse + axe budgets hold on the shipped artifact. |

If anything in Stages 1-2 fails irrecoverably, we re-decide then — not after Stage 4.
These early stages are deliberately the same things the spikes would have measured.

**Verification harness placement.** Playwright + axe + Lighthouse are CI-resident
and graduate from baseline (Stage 1) → regression gate (Stage 2) → multi-OS coverage
(Stage 3) → per-view contract (Stage 4) → production-artifact gate (Stage 5). WebdriverIO
covers the Tauri shell on Linux + Windows; Playwright covers the React app on all three
OSes. Chrome DevTools MCP is **agent-only, dev-loop only**: it is the implementation
verification surface (PR self-review for any view touching DESIGN.md tokens, `KEYMAP`,
or IPC bindings) and never gates a build. The canonical loop and per-view checklists
live at `apps/desktop/docs/mcp-verification.md`. Per-milestone gate lists are in PLAN.md.

Stages gate on completion, not on a calendar — Stage N+1 starts when Stage N is
done, not on a date. PLAN.md M24-M28 expands each row to the same task-list
granularity as M22/M23. ("Stage" is used here for sub-phases of Phase 6 to avoid
collision with the top-level Phase 1-5 already shipped.)

## Tactical questions deferred to implementation

- Guard-dialog latency budget — measured in milestone 2, tuned in place. The
  hard <500ms target stays; we adjust pre-warm strategy until we hit it.
- Linux distro priorities for QA — Ubuntu LTS + Fedora as primary, document
  Arch/Hyprland setup, defer wider distro testing.
- React 19's strict-mode + Tauri IPC double-fire behavior — handle via standard
  effect cleanup patterns; not a decision, a coding discipline.
- Tauri sidecar pattern for `ripley-script-shell` invocation — design at the
  time we wire the guard interception, not now.

## Reference implementation: CodexBar (steipete)

CodexBar is the closest analog to Ripley by author profile, scope, and form factor —
tray-resident macOS app, multi-provider data plane, dense UI, 13k stars, actively
maintained, sold/distributed under MIT. Reviewing it against our research changes the
weighting on two of the three candidates.

### What CodexBar actually is

- **Language:** 100% Swift. `Package.swift` (SwiftPM), `swift-tools-version: 6.2`.
- **Platforms:** `.macOS(.v14)`. Sonoma+. No Linux/Windows app — but a `CodexBarCLI`
  Swift target with Linux test target (`TestsLinux`) ships CLI tarballs for both.
- **Concurrency:** Swift 6 `StrictConcurrency` enabled on every target.
- **Distribution:** Homebrew cask + GitHub Releases + AUR for the CLI. Sparkle 2.9+
  for in-app updates, signed appcast, notarized.
- **Dependencies (8 total):** Sparkle, sindresorhus/KeyboardShortcuts, Vortex (confetti),
  apple/swift-crypto, apple/swift-log, apple/swift-syntax (for their own macros),
  steipete/Commander (CLI parsing), steipete/SweetCookieKit. No web tech, no FFI, no
  cross-language glue.
- **Targets (multi-binary SwiftPM):**
  - `CodexBarCore` — cross-platform business logic, Linux-buildable
  - `CodexBar` — macOS app executable (the menu bar app)
  - `CodexBarCLI` — cross-platform Swift CLI
  - `CodexBarWidget` — macOS widget extension
  - `CodexBarClaudeWatchdog` — separate macOS executable for out-of-process work
  - `CodexBarMacros` + `CodexBarMacroSupport` — Swift macros + the shim target

### How CodexBar handles the menu bar (the part we care about most)

This is the **key finding that updates our research**:

**CodexBar does not use SwiftUI `MenuBarExtra`.** It uses AppKit
`NSStatusBar` / `NSStatusItem` / `NSMenu` directly via a `StatusItemController`, with
SwiftUI views hosted *inside* the menu/popover as content.

Evidence:
- `Sources/CodexBar/StatusItemController.swift`: `final class StatusItemController:
  NSObject, NSMenuDelegate` — owns `NSStatusBar.statusItem`, `var statusItems:
  [UsageProvider: NSStatusItem]` for per-provider tray icons.
- 30+ `StatusItemController+*.swift` extension files — this is the load-bearing
  surface of the app. It is AppKit, not MenuBarExtra.
- `CodexbarApp.swift` is a SwiftUI `App` only as a host — its `body` is two scenes:
  a hidden `WindowGroup("CodexBarLifecycleKeepalive")` and a `Settings { ... }`.
  The actual UI is bootstrapped via `@NSApplicationDelegateAdaptor(AppDelegate.self)`.
- `HiddenWindowView.swift` is a 1×1 invisible, off-screen, mouse-ignoring window whose
  sole purpose is keeping SwiftUI's scene lifecycle alive so the Settings scene tabs
  render. Comment in the source: *"keep SwiftUI's lifecycle alive so `Settings` scene
  shows the native toolbar tabs even though the UI is AppKit-based."*

This is a **hybrid AppKit/SwiftUI architecture**, not the pure-SwiftUI path our
research recommended. Implications:

1. The "MenuBarExtra has known quirks → budget FluidMenuBarExtra/MenuBarExtraAccess
   workarounds" risk in the SwiftUI research is **resolved by going around the
   problem entirely** — the production-grade pattern is to skip MenuBarExtra and
   use NSStatusItem directly.
2. The pattern is straightforward and well-trodden (this is how every menu-bar app
   pre-macOS 13 did it; MenuBarExtra is the *newer*, less-mature path).
3. SwiftUI is still used heavily — for menu popover content, Settings, Widget — but
   it is layered on top of AppKit shell, not driving it.

### Other patterns worth stealing

- **Hidden lifecycle window** — directly applicable if we ever need SwiftUI scenes
  to coexist with AppKit-driven UI.
- **Separate watchdog binary** (`CodexBarClaudeWatchdog`) — pattern matches our
  `ripley-script-shell.rs`: an out-of-process binary launched by the parent for
  isolation. This shape is the same whether the parent is Swift or Rust.
- **Multi-target SwiftPM with one cross-platform "Core" target** — directly mirrors
  our `ripley-core` (cross-platform) / `ripley-app` (desktop) / `ripley-guard` (CLI)
  split. The architectural shape transfers cleanly.
- **`@NSApplicationDelegateAdaptor`** — official bridge from SwiftUI App into
  AppKit lifecycle. This is the supported escape hatch for tray apps.
- **Sparkle for self-update** with signed appcast — battle-tested. Tauri's bundled
  updater is the alternative; for a security tool the Sparkle Ed25519 signing model
  is arguably stronger.

### How this affects our decision

The CodexBar data point is **specifically about a SwiftUI/AppKit hybrid on macOS-only**.
Under Constraint 1 (cross-platform GUI v1), that architecture is not a candidate. So
CodexBar's value to us is architectural reference, not stack selection:

- **What we can copy regardless of stack choice:** multi-target build layout
  (cross-platform core + per-platform shells), separate watchdog binary pattern
  (`CodexBarClaudeWatchdog` ↔ `ripley-script-shell`), Ed25519-signed updater approach.
- **What we cannot copy:** the entire UI shell. SwiftUI is macOS/iOS-only.
  CodexBar's answer for Linux is "ship a CLI, not a GUI." That is the exact trade
  Constraint 1 rejects.

CodexBar also confirms a useful general principle: when a framework's newer
high-level abstraction has known quirks (SwiftUI `MenuBarExtra`), production apps
drop a level and use the older, well-trodden APIs (AppKit `NSStatusItem`). The
analog for Tauri is to be ready to drop down to `tao` / `wry` directly if Tauri's
high-level tray API hits limits — same escape valve, different stack.

The decision under Constraint 1 is unchanged by CodexBar: **Tauri 2 first, Slint
contingent on AccessKit**.

## Status

Decision is final pending implementation. Tactical questions deferred to
implementation (see section above) are handled as code, not as further decision
work.
