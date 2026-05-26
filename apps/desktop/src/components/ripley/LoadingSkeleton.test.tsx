import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { LoadingSkeleton } from "./LoadingSkeleton";

describe("LoadingSkeleton", () => {
  it("renders the requested number of rows", () => {
    render(<LoadingSkeleton rows={5} />);
    expect(screen.getAllByTestId("skeleton-row")).toHaveLength(5);
  });

  it("defaults to 3 rows", () => {
    render(<LoadingSkeleton />);
    expect(screen.getAllByTestId("skeleton-row")).toHaveLength(3);
  });

  it("marks itself busy for assistive tech", () => {
    render(<LoadingSkeleton />);
    const status = screen.getByRole("status");
    expect(status).toHaveAttribute("aria-busy", "true");
    expect(status).toHaveAttribute("aria-label", "Loading");
  });
});
