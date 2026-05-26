import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { KeyValueGrid } from "./KeyValueGrid";

describe("KeyValueGrid", () => {
  it("renders dt/dd pairs in order", () => {
    render(
      <KeyValueGrid
        rows={[
          { key: "Package", value: "left-pad" },
          { key: "Version", value: "1.3.0" },
        ]}
      />,
    );
    const dts = screen.getAllByRole("term");
    const dds = screen.getAllByRole("definition");
    expect(dts.map((el) => el.textContent)).toEqual(["Package", "Version"]);
    expect(dds.map((el) => el.textContent)).toEqual(["left-pad", "1.3.0"]);
  });

  it("applies mono styling to flagged rows", () => {
    render(<KeyValueGrid rows={[{ key: "Hash", value: "deadbeef", mono: true }]} />);
    expect(screen.getByRole("definition").className).toContain("font-mono");
  });

  it("forwards aria-label to the dl", () => {
    render(<KeyValueGrid rows={[]} ariaLabel="Forensic report" />);
    expect(screen.getByLabelText("Forensic report")).toBeInTheDocument();
  });
});
