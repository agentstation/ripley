# Peekaboo screenshot loop (macOS)

Repeatable screenshot capture for visual regression on macOS, where the
WKWebView WebDriver gap blocks WebdriverIO. Peekaboo's AX/screen-capture
loop substitutes.

## Setup

```
brew install steipete/tap/peekaboo
peekaboo permissions check    # screen capture + AX
```

Grant Screen Recording and Accessibility in System Settings → Privacy &
Security if any check fails.

## Scripts

| Script      | What it captures                       |
| ----------- | -------------------------------------- |
| `tray.sh`   | Tray icon in the menu bar              |
| `dialog.sh` | Guard dialog rendered by `guard-bench` |
| `home.sh`   | Default React shell (no event)         |

Run from the repo root:

```
just dev            # in another terminal — keep Ripley running
./apps/desktop/tests/peekaboo/dialog.sh
```

Output PNGs land in `apps/desktop/tests/peekaboo/snapshots/`. The directory
is `.gitignore`'d; review snapshots manually or commit a baseline if you're
intentionally changing the visual.

## CI

These scripts are **not** run in CI. They are a developer-loop tool. CI's
macOS visual signal comes from Playwright + Lighthouse against Vite-served
React (see `apps/desktop/tests/browser/`), which works on macOS because it
drives Chromium, not WKWebView.
