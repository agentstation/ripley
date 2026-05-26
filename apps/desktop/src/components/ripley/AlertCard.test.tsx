import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

import { AlertCard } from "./AlertCard";

describe("AlertCard", () => {
  it("renders package + ecosystem + severity", () => {
    render(
      <AlertCard
        ecosystem="npm"
        packageName="left-pad"
        version="1.3.0"
        severity="high"
        summary="Suspicious postinstall hook"
      />,
    );
    expect(screen.getByText("left-pad")).toBeInTheDocument();
    expect(screen.getByText("@1.3.0")).toBeInTheDocument();
    expect(screen.getByText(/HIGH/i)).toBeInTheDocument();
    expect(screen.getByText(/postinstall/)).toBeInTheDocument();
  });

  it("is non-interactive by default", () => {
    render(<AlertCard ecosystem="npm" packageName="x" severity="low" />);
    const card = screen.getByTestId("alert-card");
    expect(card).not.toHaveAttribute("role", "button");
    expect(card).not.toHaveAttribute("tabIndex");
  });

  it("invokes onActivate on click, Enter, and Space when provided", () => {
    const handler = vi.fn();
    render(<AlertCard ecosystem="npm" packageName="x" severity="low" onActivate={handler} />);
    const card = screen.getByRole("button");
    fireEvent.click(card);
    fireEvent.keyDown(card, { key: "Enter" });
    fireEvent.keyDown(card, { key: " " });
    expect(handler).toHaveBeenCalledTimes(3);
  });

  it("renders weight bar when provided", () => {
    render(<AlertCard ecosystem="npm" packageName="x" severity="medium" weight={0.6} />);
    expect(screen.getByRole("meter")).toBeInTheDocument();
  });

  it("renders observedAt as a relative timestamp", () => {
    render(
      <AlertCard
        ecosystem="npm"
        packageName="x"
        severity="medium"
        observedAt="2026-05-26T17:00:00Z"
      />,
    );
    expect(screen.getByTestId("timestamp")).toBeInTheDocument();
  });
});
