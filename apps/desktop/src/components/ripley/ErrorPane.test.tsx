import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { ErrorPane } from "./ErrorPane";

describe("ErrorPane", () => {
  it("renders as an alert with title", () => {
    render(<ErrorPane title="Scan failed" />);
    const alert = screen.getByRole("alert");
    expect(alert).toBeInTheDocument();
    expect(screen.getByText("Scan failed")).toBeInTheDocument();
  });

  it("renders optional message and action", () => {
    render(
      <ErrorPane
        title="Failed"
        message="ENOENT: package.json"
        action={<button type="button">Retry</button>}
      />,
    );
    expect(screen.getByText(/ENOENT/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
  });

  it("uses severity-critical tokens", () => {
    render(<ErrorPane title="x" />);
    const pane = screen.getByTestId("error-pane");
    expect(pane.className).toContain("bg-severity-critical-bg");
    expect(pane.querySelector("h3")?.className).toContain("text-severity-critical");
  });
});
