import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { Button } from "./button";

describe("Button", () => {
  it("renders children", () => {
    render(<Button>Scan now</Button>);
    expect(screen.getByRole("button", { name: "Scan now" })).toBeInTheDocument();
  });

  it("applies the default variant token classes", () => {
    render(<Button>Default</Button>);
    const button = screen.getByRole("button", { name: "Default" });
    expect(button.className).toContain("bg-primary");
    expect(button.className).toContain("text-primary-foreground");
  });

  it("applies the destructive variant", () => {
    render(<Button variant="destructive">Quarantine</Button>);
    const button = screen.getByRole("button", { name: "Quarantine" });
    expect(button.className).toContain("bg-destructive");
  });

  it("respects size prop", () => {
    render(<Button size="lg">Big</Button>);
    expect(screen.getByRole("button", { name: "Big" }).className).toContain("h-10");
  });

  it("toggles via data-theme on document.documentElement", () => {
    document.documentElement.setAttribute("data-theme", "light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    document.documentElement.removeAttribute("data-theme");
    expect(document.documentElement.getAttribute("data-theme")).toBeNull();
  });
});
