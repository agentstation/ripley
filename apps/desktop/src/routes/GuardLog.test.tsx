import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { commands, type GuardLogEntry, type GuardLogPage } from "@/lib/bindings";

import { GuardLog } from "./GuardLog";

function wrapper(children: ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function entry(overrides: Partial<GuardLogEntry> = {}): GuardLogEntry {
  return {
    timestamp: "2026-05-26T00:00:00Z",
    package: "left-pad",
    version: "1.3.0",
    script: "postinstall",
    action: "allowed",
    risk_level: "low",
    matched_rules: [],
    source: "script-shell",
    user_decision: true,
    ...overrides,
  };
}

function page(entries: GuardLogEntry[], total?: number): GuardLogPage {
  return { entries, total: total ?? entries.length };
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("GuardLog", () => {
  it("renders an empty state when the log is empty", async () => {
    vi.spyOn(commands, "listGuardLog").mockResolvedValue({
      status: "ok",
      data: page([], 0),
    });
    render(wrapper(<GuardLog />));
    await waitFor(() => {
      expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    });
  });

  it("renders rows with severity + timestamp + matched rules", async () => {
    vi.spyOn(commands, "listGuardLog").mockResolvedValue({
      status: "ok",
      data: page([entry({ action: "blocked", risk_level: "critical", matched_rules: ["NET001"] })]),
    });
    render(wrapper(<GuardLog />));
    await waitFor(() => {
      expect(screen.getByText("left-pad")).toBeInTheDocument();
    });
    expect(screen.getByText("critical")).toBeInTheDocument();
    expect(screen.getByText("NET001")).toBeInTheDocument();
    // The cell-level decision text is rendered uppercase via tailwind, raw text is lowercase
    const blockedCells = screen.getAllByText("blocked");
    expect(blockedCells.length).toBeGreaterThan(0);
    // ISO timestamp is rendered inside a <time> with dateTime attr
    expect(screen.getByText((_, el) => el?.tagName === "TIME").getAttribute("datetime")).toMatch(
      /^2026-05-26T00:00:00/,
    );
  });

  it("filters by decision when a chip is selected", async () => {
    vi.spyOn(commands, "listGuardLog").mockResolvedValue({
      status: "ok",
      data: page([
        entry({ package: "a", action: "blocked" }),
        entry({ package: "b", action: "allowed" }),
      ]),
    });
    render(wrapper(<GuardLog />));
    await waitFor(() => {
      expect(screen.getByText("a")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByTestId("filter-blocked"));
    await waitFor(() => {
      expect(screen.queryByText("b")).not.toBeInTheDocument();
    });
    expect(screen.getByText("a")).toBeInTheDocument();
  });

  it("paginates forward and back", async () => {
    const spy = vi
      .spyOn(commands, "listGuardLog")
      .mockResolvedValueOnce({ status: "ok", data: page([entry({ package: "first" })], 200) })
      .mockResolvedValueOnce({ status: "ok", data: page([entry({ package: "second" })], 200) })
      .mockResolvedValueOnce({ status: "ok", data: page([entry({ package: "first" })], 200) });

    render(wrapper(<GuardLog />));
    await waitFor(() => {
      expect(screen.getByText("first")).toBeInTheDocument();
    });
    expect(spy).toHaveBeenLastCalledWith(0, 50);

    fireEvent.click(screen.getByTestId("guard-log-next"));
    await waitFor(() => {
      expect(screen.getByText("second")).toBeInTheDocument();
    });
    expect(spy).toHaveBeenLastCalledWith(50, 50);

    fireEvent.click(screen.getByTestId("guard-log-prev"));
    await waitFor(() => {
      expect(screen.getByText("first")).toBeInTheDocument();
    });
    expect(spy).toHaveBeenLastCalledWith(0, 50);
  });

  it("renders an error pane when the command fails", async () => {
    vi.spyOn(commands, "listGuardLog").mockResolvedValue({
      status: "error",
      error: "log file unreadable",
    });
    render(wrapper(<GuardLog />));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
  });
});
