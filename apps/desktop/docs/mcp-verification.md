# Chrome DevTools MCP — agent-only verification recipe

This document is the canonical loop the desktop agent uses for **PR
self-review** of any view in `apps/desktop/src/` that touches:

- DESIGN.md tokens (color, typography, spacing, severity palette),
- the `KEYMAP` in `apps/desktop/src/lib/keymap.ts` (tab order, shortcut
  ownership), or
- `tauri-specta` IPC bindings in `apps/desktop/src/lib/bindings.ts`
  (changed argument shape, error surface, latency budget).

Chrome DevTools MCP is **agent-only**. It is not a CI dependency. It is
not invoked by `just test:browser`. It is not run on the user's machine
on demand. The MCP runs against the Vite dev server (`pnpm -F desktop
dev`) or the production preview (`pnpm -F desktop preview`); it never
exercises the Tauri WKWebView shell — that is the WebdriverIO suite's
job (Linux + Windows only).

The intended audience: the agent reviewing its own PR before requesting
merge. Findings are filed in the PR description with screenshots
attached and a short transcript of the relevant MCP calls.

---

## 1. The 8-step loop

Each view-touching PR runs this loop on **at least one affected route**:

1. **`new_page`** — open a fresh Chromium with no cached state. Set
   viewport to `1280 × 800` to match the default Playwright config.
2. **`navigate_page` → `http://localhost:5173/#/<route>`** — drive the
   view via hash so React Query priming and tray-driven navigation
   stay decoupled.
3. **`take_snapshot`** — capture the accessibility tree. Verify:
   - the route's stable `data-testid` (`alerts-view`, `guard-log-view`,
     etc.) is present at the section level;
   - severity-token elements expose `data-severity` and `role` where
     DESIGN.md requires (e.g. `role="status"` on traffic-light dots);
   - every `KEYMAP` shortcut is exposed via `aria-keyshortcuts` on the
     owning element.
4. **`take_screenshot`** — full-page PNG. Diff visually against the
   committed baseline in
   `tests/browser/visual/views.spec.ts-snapshots/<route>-chromium-darwin.png`.
   Note any intentional drift in the PR description.
5. **`list_console_messages`** — must be empty for `error` and `warning`
   severity. Tauri-API warnings (`__TAURI_INTERNALS__` missing) are
   acceptable when running against Vite preview because the guard-event
   listener is gated.
6. **`list_network_requests`** — every IPC call routes through the Tauri
   IPC channel; in Vite preview there should be zero requests to
   `tauri://localhost/_/cmd/...`. Static asset requests should fingerprint
   identically to the previous build (hashes match `dist/assets/`).
7. **`lighthouse_audit`** — assert a11y ≥95, perf ≥90, best-practices
   ≥95. Compare against the baseline in
   `tests/browser/__snapshots__/lighthouse-baseline.json`. A drop of
   more than 2 points from baseline requires explicit justification in
   the PR description; smaller drifts are acceptable noise.
8. **`close_page`** — release Chromium. The MCP loop is stateless across
   PRs by design.

A complete loop produces, at minimum:

- one screenshot per affected route attached to the PR,
- one accessibility-tree excerpt for the changed component,
- the Lighthouse score deltas vs. baseline.

---

## 2. When to run the loop

Run the loop **before requesting merge** on any PR whose diff matches:

```
apps/desktop/src/components/**
apps/desktop/src/routes/**
apps/desktop/src/styles/theme.css
apps/desktop/src/lib/keymap.ts
apps/desktop/src/lib/bindings.ts
```

Do **not** run it on:

- documentation-only PRs,
- Rust-only PRs (`crates/**` without an accompanying `bindings.ts`
  regeneration),
- test-only PRs that add specs but no production code,
- dependency bumps where the lockfile diff is the entire change.

If a PR touches multiple views, run the loop **per touched route**, not
once for the union. Each view has independent severity-token, KEYMAP,
and Lighthouse exposure; one route passing does not vouch for another.

---

## 3. Per-view checklist

These are the view-specific assertions that complement the generic
8-step loop. The agent ticks each box in the PR description.

### Alerts (`#/alerts`)

- Virtualized scroll: scroll to row 5000 of the seeded fixture
  (`tests/fixtures/alerts-10k.json`), confirm rendered DOM stays under
  100 row elements (DataTable windowing).
- Severity sort: clicking the severity column header reorders rows by
  `critical → high → medium → low → clean`. The `aria-sort` attribute
  flips to `descending`.
- Empty state: with `__ripleyQueryClient.setQueryData(["alerts"], [])`,
  the view renders the `EmptyState` with title "No alerts" and no CTA
  (per M27.7 deferral of V1.6).

### Guard log (`#/guard-log`)

- Pagination: `nextPage`/`prevPage` buttons advance the
  `?page=` query parameter and refetch via the bound command.
- Filter chips: clicking "blocked" filters to `decision = blocked` only;
  `data-testid="guard-log-page"` updates to show the new total.
- Empty state: with an empty result set, the page indicator reads
  "0 of 0" and the action buttons render disabled.

### Monitor (`#/monitor`)

- Real-time append: invoke `__ripleyQueryClient.setQueryData(["monitor"],
prev => [...prev, newEvent])` and confirm the new row renders
  prepended without scroll loss.
- Severity coloring: `data-severity="high"` rows compute color
  `rgb(240, 136, 62)` (the `severity-high` token).

### Deep scan (`#/deep-scan`)

- Critical finding banner uses `severity-critical-strong` for its
  destructive-action button (per M25.6 token split), not
  `severity-critical`.
- Forensic detail panel renders code snippets in `font-mono` and
  `text-text-secondary` per the body-mono spec.

### Audit (`#/audit`)

- TrafficLightDot per category: `[data-status="green"]` dots compute to
  `rgb(63, 185, 80)`, `yellow` to `rgb(210, 153, 34)`, `red` to
  `rgb(255, 123, 114)`.
- Category articles use `<article>` semantics with `aria-labelledby`
  pointing at the category heading.

### Posture (`#/posture`)

- Detected package managers render in a single list with one row per
  manager and an icon from the `EcosystemIcon` set.
- Missing-manifest warnings render in `severity-medium-bg` panels with
  `severity-medium` body text.

### Settings (`#/settings`)

- The route testid `settings-view` is present in all four render
  states (loading, error, empty, loaded) — verify by hitting the route
  with a primed cache and again with the query rejected.
- Saving the form invokes the `save_settings` command with the
  serialized DTO; the IPC payload type matches `bindings.ts`
  `SettingsDto` exactly (no extra keys, no `undefined` smuggled
  through).

### Command palette (Cmd+K from any route)

- Subsequence ranking: typing `set` ranks `nav.settings` first, ahead
  of `audit.settings-export` and any other label containing `s`, `e`,
  `t` in order.
- Recency persistence: invoking a command, closing the palette,
  reloading the page, and reopening the palette places the
  most-recently-used command first when the input is empty. The
  persisted store lives in `localStorage` under
  `ripley.command-palette`.
- ARIA: the input has `role="combobox"`,
  `aria-expanded="true"`, `aria-controls` pointing at the listbox, and
  `aria-activedescendant` pointing at the active option's id. The
  options have `role="option"` and **no focusable descendants** (the
  combobox-with-listbox pattern; see DESIGN_NOTES.md M27.8).

---

## 4. Filing findings

In the PR description, under a `## MCP verification` section:

1. List the routes the loop ran against.
2. Paste the Lighthouse score table (a11y / perf / best-practices /
   delta from baseline).
3. Attach the per-route screenshots inline.
4. List the per-view checklist items above with `[x]` or `[ ]`.
5. For any `[ ]`, link to the follow-on issue or include the
   justification.

The MCP transcript itself does not get committed. The PR description is
the durable artifact.

---

## 5. Anti-patterns

- **Do not** invoke MCP in CI. The MCP is interactive and lives in the
  agent's environment, not the GitHub Actions runner.
- **Do not** treat MCP as a substitute for `just test:browser`. The
  Playwright suite is the merge gate; MCP is the agent's pre-flight.
- **Do not** treat MCP as a substitute for the native macOS cold-path
  latency measurement on real Apple Silicon hardware (M26.4). MCP runs
  against Chromium, not WKWebView.
- **Do not** rerun the loop on every commit. Once per PR, before
  requesting merge, is the cadence. If the diff fundamentally changes
  after the first run, rerun once more.
