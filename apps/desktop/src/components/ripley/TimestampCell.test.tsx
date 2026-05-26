import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { TimestampCell } from "./TimestampCell";

const NOW = new Date("2026-05-26T17:00:00Z");
const now = () => NOW;

describe("TimestampCell", () => {
  it("formats recent past as relative", () => {
    render(<TimestampCell iso="2026-05-26T16:30:00Z" now={now} />);
    expect(screen.getByTestId("timestamp")).toHaveTextContent(/30 minutes ago/);
  });

  it("formats days in the past", () => {
    render(<TimestampCell iso="2026-05-23T17:00:00Z" now={now} />);
    expect(screen.getByTestId("timestamp")).toHaveTextContent(/3 days ago/);
  });

  it("formats sub-second deltas as just now", () => {
    render(<TimestampCell iso="2026-05-26T17:00:00.500Z" now={now} />);
    expect(screen.getByTestId("timestamp")).toHaveTextContent(/just now/);
  });

  it("renders an invalid marker for unparseable input", () => {
    render(<TimestampCell iso="not-a-date" />);
    const ts = screen.getByTestId("timestamp");
    expect(ts).toHaveAttribute("data-invalid", "true");
    expect(ts).toHaveTextContent("invalid");
  });

  it("exposes ISO via title for hover", () => {
    render(<TimestampCell iso="2026-05-26T16:30:00Z" now={now} />);
    expect(screen.getByTestId("timestamp")).toHaveAttribute("title", "2026-05-26T16:30:00.000Z");
  });
});
