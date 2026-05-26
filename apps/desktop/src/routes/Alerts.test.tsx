import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { commands, type AlertSummary } from "@/lib/bindings";

import { Alerts } from "./Alerts";

function wrapper(children: ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function alert(idx: number, overrides: Partial<AlertSummary> = {}): AlertSummary {
  const id = String(idx).padStart(3, "0");
  return {
    id: `adv-${id}`,
    advisory_id: `GHSA-${id}`,
    ecosystem: "npm",
    package: `pkg-${id}`,
    version: "1.0.0",
    severity: "high",
    summary: `Vulnerability ${id}`,
    references: [],
    project_path: "/tmp/project",
    ...overrides,
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("Alerts", () => {
  it("renders an empty state when no alerts match", async () => {
    vi.spyOn(commands, "listAlerts").mockResolvedValue({ status: "ok", data: [] });
    render(wrapper(<Alerts projectPath="/tmp/p" />));
    await waitFor(() => {
      expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    });
  });

  it("renders a single alert as a card", async () => {
    vi.spyOn(commands, "listAlerts").mockResolvedValue({
      status: "ok",
      data: [alert(1, { package: "left-pad", severity: "critical" })],
    });
    render(wrapper(<Alerts projectPath="/tmp/p" />));
    await waitFor(() => {
      expect(screen.getByText("left-pad")).toBeInTheDocument();
    });
    expect(screen.getByTestId("alerts-view")).toHaveAttribute("data-virtualized", "false");
    expect(screen.getAllByTestId("alerts-row")).toHaveLength(1);
  });

  it("virtualizes once the threshold is exceeded", async () => {
    const data = Array.from({ length: 50 }, (_, i) => alert(i));
    vi.spyOn(commands, "listAlerts").mockResolvedValue({ status: "ok", data });
    render(wrapper(<Alerts projectPath="/tmp/p" virtualizeFrom={30} />));
    await waitFor(() => {
      expect(screen.getByTestId("alerts-view")).toHaveAttribute("data-virtualized", "true");
    });
    expect(screen.queryAllByTestId("alerts-row").length).toBeLessThan(50);
  });

  it("renders an error pane when the command fails", async () => {
    vi.spyOn(commands, "listAlerts").mockResolvedValue({
      status: "error",
      error: "advisory database unreadable",
    });
    render(wrapper(<Alerts projectPath="/tmp/p" />));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
    expect(screen.getByText(/advisory database unreadable/)).toBeInTheDocument();
  });
});
