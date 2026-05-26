import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { commands } from "@/lib/bindings";

import { Audit } from "./Audit";

function wrapper(children: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("Audit", () => {
  it("renders categories with traffic-light overall status", async () => {
    vi.spyOn(commands, "runAuditReport").mockResolvedValue({
      status: "ok",
      data: {
        timestamp: "2026-05-26T00:00:00Z",
        categories: [
          {
            category: "Machine Security",
            overall: "yellow",
            findings: [
              {
                name: "Disk encryption",
                status: "green",
                detail: "FileVault is enabled",
                fix_command: null,
              },
              {
                name: "Firewall",
                status: "yellow",
                detail: "Stealth mode off",
                fix_command: "sudo socketfilterfw --setstealthmode on",
              },
            ],
          },
        ],
      },
    });
    render(wrapper(<Audit />));
    await waitFor(() => {
      expect(screen.getByText("Machine Security")).toBeInTheDocument();
    });
    expect(screen.getByText("Disk encryption")).toBeInTheDocument();
    expect(screen.getAllByTestId("audit-finding")).toHaveLength(2);
  });

  it("renders empty state for an empty report", async () => {
    vi.spyOn(commands, "runAuditReport").mockResolvedValue({
      status: "ok",
      data: { categories: [], timestamp: "2026-05-26T00:00:00Z" },
    });
    render(wrapper(<Audit />));
    await waitFor(() => {
      expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    });
  });

  it("renders an error pane on failure", async () => {
    vi.spyOn(commands, "runAuditReport").mockResolvedValue({
      status: "error",
      error: "subprocess failed",
    });
    render(wrapper(<Audit />));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
  });
});
