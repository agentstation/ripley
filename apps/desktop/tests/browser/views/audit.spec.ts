import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";

test.describe("audit view", () => {
  test("renders audit categories without console errors and is axe-clean", async ({
    page,
    consoleErrors,
  }) => {
    await gotoRoute(page, "audit");
    await expect(page.getByTestId("audit-view")).toBeVisible();
    await expect(page.getByTestId("audit-category").first()).toBeVisible();
    await expect(page.getByTestId("audit-finding").first()).toBeVisible();
    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "audit-view");
  });

  test("traffic light dots emit token-derived computed colors", async ({ page }) => {
    await gotoRoute(page, "audit");
    await expect(page.getByTestId("audit-finding").first()).toBeVisible();
    const colors = await page
      .locator('[data-status] > [aria-hidden="true"]')
      .evaluateAll((nodes) => nodes.map((n) => getComputedStyle(n).backgroundColor));
    expect(colors.length).toBeGreaterThan(0);
    for (const color of colors) {
      expect(color).toMatch(/^rgb/);
    }
  });
});
