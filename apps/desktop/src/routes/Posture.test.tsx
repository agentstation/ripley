import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { commands } from "@/lib/bindings";

import { Posture } from "./Posture";

function wrapper(children: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("Posture", () => {
  it("renders detected PMs and categories", async () => {
    vi.spyOn(commands, "runHardenReport").mockResolvedValue({
      status: "ok",
      data: {
        timestamp: "2026-05-26T00:00:00Z",
        detected_pms: [
          { name: "npm", version: "10.0.0", lockfile_path: "/p/package-lock.json", binary: false },
        ],
        categories: [
          {
            category: "Dependency Pinning",
            overall: "red",
            findings: [
              {
                name: "Lockfile up to date",
                status: "red",
                detail: "Caret ranges allow unpinned upgrades",
                fix_command: "npm install --save-exact",
                pm: "npm",
              },
            ],
          },
        ],
      },
    });
    render(wrapper(<Posture projectPath="/p" />));
    await waitFor(() => {
      expect(screen.getByText("Dependency Pinning")).toBeInTheDocument();
    });
    expect(screen.getByText(/npm v10.0.0/)).toBeInTheDocument();
    expect(screen.getByText(/--save-exact/)).toBeInTheDocument();
  });

  it("renders empty state when no PMs detected", async () => {
    vi.spyOn(commands, "runHardenReport").mockResolvedValue({
      status: "ok",
      data: {
        timestamp: "2026-05-26T00:00:00Z",
        detected_pms: [],
        categories: [],
      },
    });
    render(wrapper(<Posture projectPath="/p" />));
    await waitFor(() => {
      expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    });
  });

  it("renders an error pane on failure", async () => {
    vi.spyOn(commands, "runHardenReport").mockResolvedValue({
      status: "error",
      error: "harden failed",
    });
    render(wrapper(<Posture projectPath="/p" />));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
  });
});
