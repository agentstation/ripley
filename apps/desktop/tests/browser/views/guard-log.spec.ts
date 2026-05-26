import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";
import { SEVERITY_COLORS_RGB } from "../helpers/tokens";

test.describe("guard log view", () => {
  test("renders without console errors and is axe-clean", async ({ page, consoleErrors }) => {
    await gotoRoute(page, "guard-log");
    await expect(page.getByTestId("guard-log-view")).toBeVisible();
    await expect(page.getByTestId("guard-log-total")).toContainText("2 entries");
    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "guard-log-view");
  });

  test("filter buttons are reachable via Tab", async ({ page }) => {
    await gotoRoute(page, "guard-log");
    await expect(page.getByTestId("filter-all")).toBeVisible();
    await page.locator("body").press("Tab");
    const reached = await page.evaluate(() => document.activeElement?.getAttribute("data-testid"));
    expect(reached).toMatch(/^filter-/);
  });

  test("severity badge for high risk uses the high token color", async ({ page }) => {
    await gotoRoute(page, "guard-log");
    await expect(page.getByTestId("guard-log-view")).toBeVisible();
    const highColor = await page
      .locator('[data-severity="high"]')
      .first()
      .evaluate((el) => getComputedStyle(el).color);
    expect(highColor).toBe(SEVERITY_COLORS_RGB.high);
  });
});
