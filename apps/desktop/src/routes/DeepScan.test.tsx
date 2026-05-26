import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";

import { commands } from "@/lib/bindings";

import { DeepScan } from "./DeepScan";

function wrapper(children: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("DeepScan", () => {
  it("renders summary stats and findings", async () => {
    vi.spyOn(commands, "runDeepScan").mockResolvedValue({
      status: "ok",
      data: {
        project_path: "/p",
        ioc_count: 1,
        persistence_count: 0,
        credential_count: 1,
        findings: [
          {
            kind: "ioc",
            path: "/p/.env",
            description: "Hardcoded API key",
            severity: "high",
            profile_id: "ioc-keys",
            rotation_command: null,
          },
          {
            kind: "credential",
            path: "/p/aws.json",
            description: "Exposed AWS key",
            severity: "critical",
            profile_id: "cred-aws",
            rotation_command: "aws iam create-access-key",
          },
        ],
        dead_man_switch_warning: "Stale credentials older than 90 days",
      },
    });
    render(wrapper(<DeepScan projectPath="/p" />));
    await waitFor(() => {
      expect(screen.getAllByTestId("deep-scan-finding")).toHaveLength(2);
    });
    expect(screen.getByText("Hardcoded API key")).toBeInTheDocument();
    expect(screen.getByText(/aws iam create-access-key/)).toBeInTheDocument();
    expect(screen.getByText(/Stale credentials/)).toBeInTheDocument();
  });

  it("renders empty findings list", async () => {
    vi.spyOn(commands, "runDeepScan").mockResolvedValue({
      status: "ok",
      data: {
        project_path: "/p",
        ioc_count: 0,
        persistence_count: 0,
        credential_count: 0,
        findings: [],
        dead_man_switch_warning: null,
      },
    });
    render(wrapper(<DeepScan projectPath="/p" />));
    await waitFor(() => {
      expect(screen.getByText("No forensic findings")).toBeInTheDocument();
    });
  });

  it("renders an error pane on failure", async () => {
    vi.spyOn(commands, "runDeepScan").mockResolvedValue({
      status: "error",
      error: "deep scan failed",
    });
    render(wrapper(<DeepScan projectPath="/p" />));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
  });
});
