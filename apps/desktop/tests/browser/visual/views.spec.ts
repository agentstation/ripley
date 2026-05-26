import { test, expect } from "@playwright/test";

import { ROUTES } from "../helpers/routes";
import { gotoRoute } from "../helpers/prime";

test.describe("visual diffs", () => {
  test.skip(
    process.platform !== "darwin",
    "baselines are darwin-only; cross-OS font/AA rendering makes per-OS baselines high-maintenance noise. See DESIGN_NOTES.md M27.8.",
  );
  for (const route of ROUTES) {
    test(`${route.id} matches snapshot`, async ({ page }) => {
      await gotoRoute(page, route.id);
      await expect(page.getByTestId(route.testId)).toBeVisible();
      await page.evaluate(() => document.fonts.ready);
      await expect(page).toHaveScreenshot(`${route.id}.png`, {
        fullPage: true,
        maxDiffPixelRatio: 0.001,
      });
    });
  }
});
