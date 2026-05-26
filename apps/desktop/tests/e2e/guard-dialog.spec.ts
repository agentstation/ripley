import { spawn } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { browser, expect } from "@wdio/globals";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const isWindows = os.platform() === "win32";
const guardBenchBin = path.resolve(
  __dirname,
  "..",
  "..",
  "src-tauri",
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

describe("guard dialog (e2e)", () => {
  it("renders the dialog and the Allow path closes it", async () => {
    const inFlight = fireGuardPrompt();

    const trust = await browser.$('[data-testid="trust"]');
    await trust.waitForDisplayed({ timeout: 10_000 });
    await trust.click();

    const exitCode = await inFlight;
    expect(exitCode).toBe(0);
  });

  it("renders the dialog and the Block path closes it", async () => {
    const inFlight = fireGuardPrompt();

    const block = await browser.$('[data-testid="block"]');
    await block.waitForDisplayed({ timeout: 10_000 });
    await block.click();

    const exitCode = await inFlight;
    expect(exitCode).toBe(0);
  });
});
