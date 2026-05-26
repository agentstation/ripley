import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";

test.describe("settings view", () => {
  test("renders the form without console errors and is axe-clean", async ({
    page,
    consoleErrors,
  }) => {
    await gotoRoute(page, "settings");
    await expect(page.getByTestId("settings-view")).toBeVisible();
    await expect(page.getByTestId("settings-form")).toBeVisible();
    await expect(page.getByTestId("settings-save")).toBeVisible();
    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "settings-view");
  });

  test("guard mode radios are reachable and form fields participate in Tab order", async ({
    page,
  }) => {
    await gotoRoute(page, "settings");
    await expect(page.getByTestId("guard-mode-audit")).toBeVisible();

    await page.getByTestId("poll-interval").focus();
    const reached = new Set<string>();
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("Tab");
      const id = await page.evaluate(() => document.activeElement?.getAttribute("data-testid"));
      if (id) reached.add(id);
    }
    expect(reached).toContain("launch-at-login");
    expect(reached).toContain("settings-save");
  });
});
