import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";
import { SEVERITY_COLORS_RGB } from "../helpers/tokens";

test.describe("deep scan view", () => {
  test("renders forensic findings without console errors and is axe-clean", async ({
    page,
    consoleErrors,
  }) => {
    await gotoRoute(page, "deep-scan");
    await expect(page.getByTestId("deep-scan-view")).toBeVisible();
    await expect(page.getByTestId("deep-scan-finding")).toHaveCount(1);
    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "deep-scan-view");
  });

  test("critical finding renders with the critical token color", async ({ page }) => {
    await gotoRoute(page, "deep-scan");
    await expect(page.getByTestId("deep-scan-finding")).toHaveCount(1);
    const color = await page
      .locator('[data-severity="critical"]')
      .first()
      .evaluate((el) => getComputedStyle(el).color);
    expect(color).toBe(SEVERITY_COLORS_RGB.critical);
  });
});
