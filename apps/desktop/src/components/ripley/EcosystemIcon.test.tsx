import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { EcosystemIcon } from "./EcosystemIcon";

describe("EcosystemIcon", () => {
  it("renders npm label", () => {
    render(<EcosystemIcon ecosystem="npm" />);
    expect(screen.getByTestId("ecosystem-icon")).toHaveTextContent("npm");
  });

  it("normalises case", () => {
    render(<EcosystemIcon ecosystem="PyPI" />);
    expect(screen.getByTestId("ecosystem-icon")).toHaveAttribute("data-ecosystem", "pypi");
  });

  it("falls back to unknown for unsupported ecosystems", () => {
    render(<EcosystemIcon ecosystem="cocoapods" />);
    const icon = screen.getByTestId("ecosystem-icon");
    expect(icon).toHaveAttribute("data-ecosystem", "unknown");
    expect(icon).toHaveTextContent("?");
  });

  it("uses severity tokens", () => {
    render(<EcosystemIcon ecosystem="npm" />);
    expect(screen.getByTestId("ecosystem-icon").className).toContain("bg-severity-critical-bg");
  });
});
