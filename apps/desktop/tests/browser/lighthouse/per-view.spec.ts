import { test, expect } from "@playwright/test";
import { playAudit } from "playwright-lighthouse";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { ROUTES } from "../helpers/routes";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT = path.join(__dirname, "..", "__snapshots__", "lighthouse-baseline.json");

type Thresholds = { accessibility: number; performance: number; bestPractices: number };

async function readThresholds(): Promise<Thresholds> {
  const raw = await readFile(SNAPSHOT, "utf8");
  const parsed = JSON.parse(raw) as { thresholds?: Partial<Thresholds> };
  return {
    accessibility: parsed.thresholds?.accessibility ?? 0.95,
    performance: parsed.thresholds?.performance ?? 0.9,
    bestPractices: parsed.thresholds?.bestPractices ?? 0.95,
  };
}

test.describe("lighthouse per-view", () => {
  test.skip(
    !process.env.LIGHTHOUSE,
    "set LIGHTHOUSE=1 to run per-view audits (requires chrome --remote-debugging-port=9222). CI captures via capture-lighthouse-baseline.mjs.",
  );

  for (const route of ROUTES) {
    test(`${route.id} meets baseline thresholds`, async ({ page }) => {
      const thresholds = await readThresholds();
      await page.goto(`/${route.hash}`);
      await expect(page.getByTestId(route.testId)).toBeVisible();

      const result = await playAudit({
        page,
        port: 9222,
        thresholds: {
          accessibility: Math.round(thresholds.accessibility * 100),
          performance: Math.round(thresholds.performance * 100),
          "best-practices": Math.round(thresholds.bestPractices * 100),
          seo: 0,
        },
        reports: { formats: { json: false } },
      });

      const a11y = result.lhr.categories.accessibility?.score ?? 0;
      const perf = result.lhr.categories.performance?.score ?? 0;
      const bp = result.lhr.categories["best-practices"]?.score ?? 0;

      expect(a11y, `${route.id} a11y ${a11y}`).toBeGreaterThanOrEqual(thresholds.accessibility);
      expect(perf, `${route.id} perf ${perf}`).toBeGreaterThanOrEqual(thresholds.performance);
      expect(bp, `${route.id} best-practices ${bp}`).toBeGreaterThanOrEqual(
        thresholds.bestPractices,
      );
    });
  }
});
