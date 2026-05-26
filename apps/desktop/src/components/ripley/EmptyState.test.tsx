import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  it("renders title", () => {
    render(<EmptyState title="No alerts" />);
    expect(screen.getByRole("heading", { name: "No alerts" })).toBeInTheDocument();
  });

  it("renders optional description and action", () => {
    render(
      <EmptyState
        title="Empty"
        description="Run a scan to populate this view."
        action={<button type="button">Scan</button>}
      />,
    );
    expect(screen.getByText(/Run a scan/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Scan" })).toBeInTheDocument();
  });

  it("uses DESIGN.md surface tokens", () => {
    render(<EmptyState title="Empty" />);
    const card = screen.getByTestId("empty-state");
    expect(card.className).toContain("bg-surface");
    expect(card.className).toContain("border-border-subtle");
  });
});
