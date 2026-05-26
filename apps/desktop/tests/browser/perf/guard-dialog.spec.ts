import { test, expect } from "@playwright/test";

const WARM_BUDGET_MS = 50;

test("guard dialog mounts within the warm budget", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Ripley" })).toBeVisible();

  const elapsed: number = await page.evaluate(async () => {
    const win = window as unknown as {
      __ripleyGuardStore?: { setState: (s: Record<string, unknown>) => void };
    };
    if (!win.__ripleyGuardStore) throw new Error("test handle missing");

    performance.mark("guard-set");
    win.__ripleyGuardStore.setState({
      currentEvent: {
        id: "perf-1",
        package: "perf-pkg",
        version: "1.0.0",
        script: "echo perf",
        risk_level: "high",
        matched_rules: ["PERF"],
      },
    });

    await new Promise<void>((resolve) => {
      const start = performance.now();
      const tick = () => {
        if (document.querySelector('[role="alertdialog"]')) {
          performance.mark("guard-visible");
          resolve();
          return;
        }
        if (performance.now() - start > 1000) {
          resolve();
          return;
        }
        requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    });

    performance.measure("guard-mount", "guard-set", "guard-visible");
    const entry = performance.getEntriesByName("guard-mount")[0];
    return entry?.duration ?? Number.POSITIVE_INFINITY;
  });

  expect(
    elapsed,
    `guard dialog mount took ${elapsed}ms (warm budget ${WARM_BUDGET_MS}ms)`,
  ).toBeLessThan(WARM_BUDGET_MS);
});
