#!/usr/bin/env node
import { spawn } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import os from "node:os";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const APP_ROOT = path.resolve(__dirname, "..");
const SNAPSHOT_DIR = path.join(APP_ROOT, "tests", "browser", "__snapshots__");
const SNAPSHOT = path.join(SNAPSHOT_DIR, "lighthouse-baseline.json");
const PORT = 4173;
const URL = `http://localhost:${PORT}`;

if (!existsSync(path.join(APP_ROOT, "dist", "index.html"))) {
  console.error(`dist/ missing — run pnpm -F desktop build:vite first`);
  process.exit(1);
}

const isWin = process.platform === "win32";
const pnpmCmd = isWin ? "pnpm.cmd" : "pnpm";

const preview = spawn(
  pnpmCmd,
  ["exec", "vite", "preview", "--port", String(PORT), "--strictPort"],
  { cwd: APP_ROOT, stdio: ["ignore", "inherit", "inherit"], shell: isWin },
);

const shutdown = () => {
  if (!preview.killed) preview.kill("SIGTERM");
};
process.on("exit", shutdown);
process.on("SIGINT", () => {
  shutdown();
  process.exit(130);
});

async function waitForServer(url, timeoutMs = 15_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // not ready yet
    }
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error(`vite preview did not respond at ${url} within ${timeoutMs}ms`);
}

await waitForServer(URL);

const tmpReport = path.join(os.tmpdir(), `lighthouse-${process.platform}-${Date.now()}.json`);
const chromeFlags = ["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"];

const lh = spawn(
  pnpmCmd,
  [
    "exec",
    "lighthouse",
    URL,
    "--output=json",
    `--output-path=${tmpReport}`,
    "--quiet",
    `--chrome-flags=${chromeFlags.join(" ")}`,
    "--preset=desktop",
    "--only-categories=accessibility,performance,best-practices",
  ],
  { cwd: APP_ROOT, stdio: ["ignore", "inherit", "inherit"], shell: isWin },
);

const lhExit = await new Promise((resolve) => lh.on("exit", resolve));
shutdown();

let lhr;
try {
  lhr = JSON.parse(await readFile(tmpReport, "utf8"));
} catch (err) {
  console.error(`lighthouse exited with ${lhExit}; no usable report at ${tmpReport}: ${err.message}`);
  process.exit(lhExit || 1);
}

if (lhExit !== 0) {
  console.warn(
    `lighthouse exited with ${lhExit} but produced a valid report (likely chrome-launcher tmp-cleanup race on Windows); continuing.`,
  );
}
const entry = {
  capturedAt: lhr.fetchTime,
  lighthouseVersion: lhr.lighthouseVersion,
  accessibility: lhr.categories.accessibility?.score ?? null,
  performance: lhr.categories.performance?.score ?? null,
  bestPractices: lhr.categories["best-practices"]?.score ?? null,
};

await mkdir(SNAPSHOT_DIR, { recursive: true });
const existing = existsSync(SNAPSHOT)
  ? JSON.parse(await readFile(SNAPSHOT, "utf8"))
  : {
      schemaVersion: 1,
      thresholds: { accessibility: 0.95, performance: 0.9, bestPractices: 0.95 },
      byOs: {},
    };
existing.byOs ??= {};
existing.byOs[process.platform] = entry;
await writeFile(SNAPSHOT, JSON.stringify(existing, null, 2) + "\n", "utf8");

console.log(`Recorded baseline for ${process.platform}:`, entry);

const t = existing.thresholds ?? { accessibility: 0.95, performance: 0.9, bestPractices: 0.95 };
const failures = [];
if ((entry.accessibility ?? 0) < t.accessibility) {
  failures.push(`accessibility ${entry.accessibility} < ${t.accessibility}`);
}
if ((entry.performance ?? 0) < t.performance) {
  failures.push(`performance ${entry.performance} < ${t.performance}`);
}
if ((entry.bestPractices ?? 0) < t.bestPractices) {
  failures.push(`bestPractices ${entry.bestPractices} < ${t.bestPractices}`);
}
if (failures.length) {
  console.error("Baseline below threshold:", failures.join("; "));
  process.exit(1);
}
