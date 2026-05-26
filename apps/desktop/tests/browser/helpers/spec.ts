import { expect, test as base, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

type Fixtures = {
  consoleErrors: string[];
};

export const test = base.extend<Fixtures>({
  consoleErrors: async ({ page }, use) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });
    page.on("pageerror", (err) => {
      errors.push(err.message);
    });
    await use(errors);
  },
});

export { expect };

export async function expectAxeClean(page: Page, testId: string): Promise<void> {
  const results = await new AxeBuilder({ page })
    .include(`[data-testid="${testId}"]`)
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
    .analyze();

  const blocking = results.violations.filter(
    (v) => v.impact === "serious" || v.impact === "critical",
  );

  expect(blocking, JSON.stringify(blocking, null, 2)).toEqual([]);
}

export async function expectTabReachesElements(page: Page, count: number): Promise<void> {
  await page.evaluate(() => {
    const focusable = document.body;
    focusable.focus();
  });
  const reached = new Set<string>();
  for (let i = 0; i < count + 5 && reached.size < count; i++) {
    await page.keyboard.press("Tab");
    const id = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) return null;
      const tag = el.tagName.toLowerCase();
      const testId = el.getAttribute("data-testid") ?? "";
      return `${tag}#${testId}@${i}`;
    });
    if (id) reached.add(id);
  }
  expect(
    reached.size,
    `Tab traversal reached ${reached.size} unique elements (expected ≥${count})`,
  ).toBeGreaterThanOrEqual(count);
}
