import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { WeightBar } from "./WeightBar";

describe("WeightBar", () => {
  it("clamps values above 1 to 100%", () => {
    render(<WeightBar value={1.6} />);
    const meter = screen.getByRole("meter");
    expect(meter).toHaveAttribute("aria-valuenow", "1");
    expect(screen.getByText("100%")).toBeInTheDocument();
  });

  it("clamps negative values to 0%", () => {
    render(<WeightBar value={-0.5} />);
    expect(screen.getByRole("meter")).toHaveAttribute("aria-valuenow", "0");
    expect(screen.getByText("00%")).toBeInTheDocument();
  });

  it("uses critical tone for high weights", () => {
    render(<WeightBar value={0.9} />);
    expect(screen.getByTestId("weight-fill").className).toContain("bg-severity-critical");
  });

  it("uses low tone for benign weights", () => {
    render(<WeightBar value={0.1} />);
    expect(screen.getByTestId("weight-fill").className).toContain("bg-severity-low");
  });

  it("uses high tone for ≥0.5 < 0.75", () => {
    render(<WeightBar value={0.6} />);
    expect(screen.getByTestId("weight-fill").className).toContain("bg-severity-high");
  });

  it("uses medium tone for ≥0.25 < 0.5", () => {
    render(<WeightBar value={0.4} />);
    expect(screen.getByTestId("weight-fill").className).toContain("bg-severity-medium");
  });

  it("uses provided aria-label", () => {
    render(<WeightBar value={0.5} label="install risk" />);
    expect(screen.getByRole("meter")).toHaveAttribute("aria-label", "install risk");
  });
});
