import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, act } from "@testing-library/react";

import { useCommandPaletteStore } from "@/store/command-palette";

import { CommandPalette, orderCommands } from "./CommandPalette";
import type { PaletteCommand } from "@/hooks/useCommands";

const makeCommands = (runs: Record<string, () => void> = {}): PaletteCommand[] => [
  {
    id: "nav.alerts",
    label: "Go to Alerts",
    hint: "View vulnerability matches",
    group: "navigate",
    keywords: ["vulnerabilities", "cve"],
    run: runs["nav.alerts"] ?? (() => {}),
  },
  {
    id: "nav.guard-log",
    label: "Go to Guard log",
    hint: "Audit script-shell decisions",
    group: "navigate",
    keywords: ["history", "scripts"],
    run: runs["nav.guard-log"] ?? (() => {}),
  },
  {
    id: "nav.settings",
    label: "Go to Settings",
    hint: "Configure Ripley",
    group: "navigate",
    keywords: ["preferences"],
    run: runs["nav.settings"] ?? (() => {}),
  },
];

beforeEach(() => {
  localStorage.clear();
  useCommandPaletteStore.setState({ open: false, recent: [] });
});

describe("orderCommands", () => {
  it("returns commands ordered with recent first when no query", () => {
    const commands = makeCommands();
    const ordered = orderCommands(commands, ["nav.settings"], "");
    expect(ordered.map((c) => c.id)).toEqual(["nav.settings", "nav.alerts", "nav.guard-log"]);
  });

  it("fuzzy-matches across label, hint, and keywords", () => {
    const commands = makeCommands();
    expect(orderCommands(commands, [], "cve").map((c) => c.id)).toEqual(["nav.alerts"]);
    expect(orderCommands(commands, [], "script").map((c) => c.id)).toEqual(["nav.guard-log"]);
    expect(orderCommands(commands, [], "config").map((c) => c.id)).toEqual(["nav.settings"]);
  });
});

describe("CommandPalette", () => {
  it("filters by input and runs the active command on Enter", () => {
    const run = vi.fn();
    useCommandPaletteStore.setState({ open: true });
    render(<CommandPalette commands={makeCommands({ "nav.guard-log": run })} />);
    const input = screen.getByTestId("command-palette-input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "guard" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(run).toHaveBeenCalledTimes(1);
    expect(useCommandPaletteStore.getState().open).toBe(false);
    expect(useCommandPaletteStore.getState().recent[0]).toBe("nav.guard-log");
  });

  it("Esc closes the palette", () => {
    useCommandPaletteStore.setState({ open: true });
    render(<CommandPalette commands={makeCommands()} />);
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    });
    expect(useCommandPaletteStore.getState().open).toBe(false);
  });

  it("arrow keys move active selection", () => {
    useCommandPaletteStore.setState({ open: true });
    render(<CommandPalette commands={makeCommands()} />);
    const input = screen.getByTestId("command-palette-input");
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    const items = screen.getAllByTestId("command-palette-item");
    const third = items[2];
    expect(third).toBeDefined();
    expect(third?.getAttribute("data-active")).toBe("true");
  });

  it("renders an empty state for unmatched queries", () => {
    useCommandPaletteStore.setState({ open: true });
    render(<CommandPalette commands={makeCommands()} />);
    fireEvent.change(screen.getByTestId("command-palette-input"), {
      target: { value: "zzzzzz" },
    });
    expect(screen.getByTestId("command-palette-empty")).toBeInTheDocument();
  });

  it("persists recency across mounts", () => {
    useCommandPaletteStore.setState({ open: true });
    const { unmount } = render(<CommandPalette commands={makeCommands()} />);
    fireEvent.change(screen.getByTestId("command-palette-input"), {
      target: { value: "settings" },
    });
    fireEvent.keyDown(screen.getByTestId("command-palette-input"), { key: "Enter" });
    unmount();
    const persisted = localStorage.getItem("ripley.command-palette");
    expect(persisted).toContain("nav.settings");
  });
});
