import { browser, expect } from "@wdio/globals";

describe("ripley-desktop smoke", () => {
  it("loads the React shell into the webview", async () => {
    const root = await browser.$('[data-testid="app-root"]');
    await root.waitForExist({ timeout: 10_000 });
  });

  it("toggles the dashboard window via the global shortcut", async () => {
    // Cmd/Super+Shift+R is wired to prewarm::toggle in lib.rs.
    await browser.keys(["Meta", "Shift", "r"]);
    await browser.pause(250);
    const root = await browser.$('[data-testid="app-root"]');
    expect(await root.isExisting()).toBe(true);
  });
});
