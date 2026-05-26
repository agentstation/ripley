import { spawn } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { browser, expect } from "@wdio/globals";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const isWindows = os.platform() === "win32";
const workspaceRoot = path.resolve(__dirname, "..", "..", "..", "..");
const guardBenchBin = path.resolve(
  workspaceRoot,
  "target",
  "release",
  isWindows ? "guard-bench.exe" : "guard-bench",
);

function fireGuardPrompt(): Promise<number> {
  return new Promise((resolve, reject) => {
    const proc = spawn(guardBenchBin, ["1"], {
      stdio: ["ignore", "inherit", "inherit"],
    });
    proc.on("exit", (code) => resolve(code ?? -1));
    proc.on("error", reject);
  });
}

async function waitForGuardListener() {
  const root = await browser.$('[data-testid="app-root"]');
  await root.waitForExist({ timeout: 30_000 });
  // Tauri event subscriptions register asynchronously inside useEffect; if
  // guard-bench fires before the listener attaches, the event is dropped.
  // Give the microtask + WebView IPC plumbing a beat to settle.
  await browser.pause(500);
}

// Windows skip: guard-bench + the desktop UDS bridge are #[cfg(unix)] —
// no Windows transport (named pipes) exists yet. The Windows guard-dialog
// path lands with M27 when ripley-ipc grows a named-pipe transport. The
// smoke spec still covers the Tauri shell + WebView2 on Windows.
const describeGuardDialog = isWindows ? describe.skip : describe;

describeGuardDialog("guard dialog (e2e)", () => {
  it("renders the dialog and the Allow path closes it", async () => {
    await waitForGuardListener();
    const inFlight = fireGuardPrompt();

    const trust = await browser.$('[data-testid="trust"]');
    await trust.waitForDisplayed({ timeout: 30_000 });
    await trust.click();

    const exitCode = await inFlight;
    expect(exitCode).toBe(0);
  });

  it("renders the dialog and the Block path closes it", async () => {
    await waitForGuardListener();
    const inFlight = fireGuardPrompt();

    const block = await browser.$('[data-testid="block"]');
    await block.waitForDisplayed({ timeout: 30_000 });
    await block.click();

    const exitCode = await inFlight;
    expect(exitCode).toBe(0);
  });
});
