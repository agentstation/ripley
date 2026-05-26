import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";

test.describe("command palette", () => {
  test("opens via Cmd+K, lists commands, and is axe-clean", async ({ page, consoleErrors }) => {
    await gotoRoute(page, "alerts");
    await page.keyboard.press("Meta+K");
    const palette = page.getByTestId("command-palette");
    await expect(palette).toBeVisible();
    await expect(page.getByTestId("command-palette-item").first()).toBeVisible();

    const items = await page.getByTestId("command-palette-item").count();
    expect(items).toBeGreaterThanOrEqual(7);

    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "command-palette");
  });

  test("filters with fuzzy search and Enter navigates to the chosen route", async ({ page }) => {
    await gotoRoute(page, "alerts");
    await page.keyboard.press("Meta+K");
    await expect(page.getByTestId("command-palette")).toBeVisible();

    await page.getByTestId("command-palette-input").fill("set");
    const firstItem = page.getByTestId("command-palette-item").first();
    await expect(firstItem).toHaveAttribute("data-command-id", "nav.settings");

    await page.keyboard.press("Enter");
    await expect(page.getByTestId("settings-view")).toBeVisible();
    expect(await page.evaluate(() => window.location.hash)).toBe("#/settings");
  });

  test("Escape closes the palette", async ({ page }) => {
    await gotoRoute(page, "alerts");
    await page.keyboard.press("Meta+K");
    await expect(page.getByTestId("command-palette")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.getByTestId("command-palette")).not.toBeVisible();
  });
});
