import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { commands } from "@/lib/bindings";

import { Monitor } from "./Monitor";

function wrapper(children: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("Monitor", () => {
  it("renders monitor events", async () => {
    vi.spyOn(commands, "listMonitorEvents").mockResolvedValue({
      status: "ok",
      data: [
        {
          timestamp: "2026-05-26T00:00:00Z",
          event_type: "process_alert",
          severity: "high",
          process: "node",
          pid: 1234,
          reason: "new_connection",
          detail: "evil.com:443",
          action: "observed",
        },
        {
          timestamp: "2026-05-26T00:01:00Z",
          event_type: "process_alert",
          severity: "low",
          process: "python",
          pid: null,
          reason: "spawn",
          detail: "",
          action: "observed",
        },
      ],
    });
    render(wrapper(<Monitor />));
    await waitFor(() => {
      expect(screen.getAllByTestId("monitor-event")).toHaveLength(2);
    });
    expect(screen.getByText("node")).toBeInTheDocument();
    expect(screen.getByText(/pid 1234/)).toBeInTheDocument();
    expect(screen.getByText(/evil.com:443/)).toBeInTheDocument();
  });

  it("renders empty state when no events", async () => {
    vi.spyOn(commands, "listMonitorEvents").mockResolvedValue({
      status: "ok",
      data: [],
    });
    render(wrapper(<Monitor />));
    await waitFor(() => {
      expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    });
  });

  it("renders an error pane on failure", async () => {
    vi.spyOn(commands, "listMonitorEvents").mockResolvedValue({
      status: "error",
      error: "monitor read failed",
    });
    render(wrapper(<Monitor />));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
  });
});
