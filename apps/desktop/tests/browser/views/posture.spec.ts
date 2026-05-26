import { test, expect, expectAxeClean } from "../helpers/spec";
import { gotoRoute } from "../helpers/prime";

test.describe("posture view", () => {
  test("renders package manager findings without console errors and is axe-clean", async ({
    page,
    consoleErrors,
  }) => {
    await gotoRoute(page, "posture");
    await expect(page.getByTestId("posture-view")).toBeVisible();
    await expect(page.getByTestId("posture-category").first()).toBeVisible();
    await expect(page.getByTestId("posture-finding").first()).toBeVisible();
    await expect(page.getByTestId("detected-pms")).toContainText("npm");
    expect(consoleErrors, consoleErrors.join("\n")).toEqual([]);
    await expectAxeClean(page, "posture-view");
  });
});
