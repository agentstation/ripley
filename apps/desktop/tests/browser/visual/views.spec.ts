import { test, expect } from "@playwright/test";

import { ROUTES } from "../helpers/routes";
import { gotoRoute } from "../helpers/prime";

test.describe("visual diffs", () => {
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
