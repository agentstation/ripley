import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";
import { SEVERITY_COLORS_RGB } from "../helpers/tokens";

test.describe("alerts view", () => {
  test("renders without console errors and is axe-clean", async ({ page, consoleErrors }) => {
    await gotoRoute(page, "alerts");
    await expect(page.getByTestId("alerts-view")).toBeVisible();
    await expect(page.getByTestId("alerts-row").first()).toBeVisible();
    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "alerts-view");
  });

  test("severity badges render with DESIGN.md token colors", async ({ page }) => {
    await gotoRoute(page, "alerts");
    await expect(page.getByTestId("alerts-row").first()).toBeVisible();

    const criticalColor = await page
      .locator('[data-severity="critical"]')
      .first()
      .evaluate((el) => getComputedStyle(el).color);
    expect(criticalColor).toBe(SEVERITY_COLORS_RGB.critical);

    const highColor = await page
      .locator('[data-severity="high"]')
      .first()
      .evaluate((el) => getComputedStyle(el).color);
    expect(highColor).toBe(SEVERITY_COLORS_RGB.high);
  });
});
