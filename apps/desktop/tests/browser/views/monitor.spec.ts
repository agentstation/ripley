import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";
import { SEVERITY_COLORS_RGB } from "../helpers/tokens";

test.describe("monitor view", () => {
  test("renders monitor events without console errors and is axe-clean", async ({
    page,
    consoleErrors,
  }) => {
    await gotoRoute(page, "monitor");
    await expect(page.getByTestId("monitor-view")).toBeVisible();
    await expect(page.getByTestId("monitor-event").first()).toBeVisible();
    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "monitor-view");
  });

  test("high-severity event uses the high token color", async ({ page }) => {
    await gotoRoute(page, "monitor");
    await expect(page.getByTestId("monitor-event").first()).toBeVisible();
    const color = await page
      .locator('[data-severity="high"]')
      .first()
      .evaluate((el) => getComputedStyle(el).color);
    expect(color).toBe(SEVERITY_COLORS_RGB.high);
  });
});
