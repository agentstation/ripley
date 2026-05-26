import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { commands, type SettingsDto } from "@/lib/bindings";

import { Settings } from "./Settings";

function wrapper(children: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

const baseSettings: SettingsDto = {
  poll_interval_secs: 300,
  launch_at_login: false,
  monitor_enabled: false,
  monitor_watch_processes: true,
  monitor_watch_persistence: true,
  monitor_watch_lockfiles: true,
  guard_mode: "strict",
  guard_timeout_secs: 30,
  guard_sandbox: false,
  posture_strict: false,
  posture_require_lockfile: false,
};

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("Settings", () => {
  it("loads settings and submits an updated payload", async () => {
    vi.spyOn(commands, "readSettings").mockResolvedValue({ status: "ok", data: baseSettings });
    const write = vi
      .spyOn(commands, "writeSettings")
      .mockResolvedValue({ status: "ok", data: { ...baseSettings, guard_mode: "audit" } });

    render(wrapper(<Settings />));

    await waitFor(() => {
      expect(screen.getByTestId("settings-form")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("guard-mode-audit"));
    fireEvent.change(screen.getByTestId("poll-interval"), { target: { value: "600" } });
    fireEvent.click(screen.getByTestId("settings-save"));

    await waitFor(() => {
      expect(write).toHaveBeenCalledTimes(1);
    });
    const payload = write.mock.calls[0]?.[0];
    expect(payload?.guard_mode).toBe("audit");
    expect(payload?.poll_interval_secs).toBe(600);
  });

  it("renders an error pane on load failure", async () => {
    vi.spyOn(commands, "readSettings").mockResolvedValue({
      status: "error",
      error: "config unreadable",
    });
    render(wrapper(<Settings />));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
  });

  it("toggles checkbox controls", async () => {
    vi.spyOn(commands, "readSettings").mockResolvedValue({ status: "ok", data: baseSettings });
    const write = vi
      .spyOn(commands, "writeSettings")
      .mockResolvedValue({ status: "ok", data: { ...baseSettings, guard_sandbox: true } });
    render(wrapper(<Settings />));
    await waitFor(() => screen.getByTestId("settings-form"));
    fireEvent.click(screen.getByTestId("guard-sandbox"));
    fireEvent.click(screen.getByTestId("settings-save"));
    await waitFor(() => {
      expect(write).toHaveBeenCalled();
    });
    expect(write.mock.calls[0]?.[0]?.guard_sandbox).toBe(true);
  });
});
