import { test } from "@playwright/test";
import { playAudit } from "playwright-lighthouse";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "..", "__snapshots__");
const SNAPSHOT = path.join(SNAPSHOT_DIR, "lighthouse-baseline.json");

test("lighthouse baseline (shell only, no regression gates yet)", async ({ page }) => {
  test.skip(
    !process.env.LIGHTHOUSE,
    "set LIGHTHOUSE=1 to record baseline (requires chrome --remote-debugging-port=9222) — M27 wires CI gates",
  );
  await page.goto("/");
  const result = await playAudit({
    page,
    port: 9222,
    thresholds: { accessibility: 0, performance: 0, "best-practices": 0, seo: 0 },
    reports: { formats: { json: true }, name: "lighthouse-baseline" },
  });

  await mkdir(SNAPSHOT_DIR, { recursive: true });
  await writeFile(
    SNAPSHOT,
    JSON.stringify(
      {
        capturedAt: new Date().toISOString(),
        accessibility: result.lhr.categories.accessibility?.score,
        performance: result.lhr.categories.performance?.score,
        bestPractices: result.lhr.categories["best-practices"]?.score,
        seo: result.lhr.categories.seo?.score,
      },
      null,
      2,
    ),
    "utf8",
  );
});
