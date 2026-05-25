import { test, expect } from "@playwright/test";

test("app loads with no console errors and no failed requests", async ({ page }) => {
  const consoleErrors: string[] = [];
  const failedRequests: string[] = [];

  page.on("console", (msg) => {
    if (msg.type() === "error") consoleErrors.push(msg.text());
  });

  page.on("requestfailed", (req) => {
    failedRequests.push(`${req.method()} ${req.url()} -> ${req.failure()?.errorText ?? "unknown"}`);
  });

  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Ripley" })).toBeVisible();

  expect(consoleErrors, `unexpected console errors: ${consoleErrors.join("\n")}`).toEqual([]);
  expect(failedRequests, `unexpected failed requests: ${failedRequests.join("\n")}`).toEqual([]);
});
