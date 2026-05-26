import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { SeverityBadge, normaliseSeverity } from "./SeverityBadge";

describe("normaliseSeverity", () => {
  it("maps common aliases", () => {
    expect(normaliseSeverity("Critical")).toBe("critical");
    expect(normaliseSeverity("HIGH")).toBe("high");
    expect(normaliseSeverity("moderate")).toBe("medium");
    expect(normaliseSeverity("info")).toBe("low");
    expect(normaliseSeverity("none")).toBe("clean");
  });

  it("falls back to medium for unknown input", () => {
    expect(normaliseSeverity("weird")).toBe("medium");
  });
});

describe("SeverityBadge", () => {
  it("renders the normalised level", () => {
    render(<SeverityBadge severity="moderate" />);
    const badge = screen.getByText("medium");
    expect(badge).toHaveAttribute("data-severity", "medium");
    expect(badge.className).toContain("bg-severity-medium-bg");
  });

  it("uppercases visually via tailwind class", () => {
    render(<SeverityBadge severity="critical" />);
    expect(screen.getByText("critical").className).toContain("uppercase");
  });
});
