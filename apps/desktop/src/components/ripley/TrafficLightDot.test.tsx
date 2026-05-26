import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { TrafficLightDot } from "./TrafficLightDot";

describe("TrafficLightDot", () => {
  it("normalises unknown to yellow", () => {
    render(<TrafficLightDot status="weird" />);
    expect(screen.getByLabelText("yellow")).toBeInTheDocument();
  });

  it("uses provided label", () => {
    render(<TrafficLightDot status="red" label="Critical issue" />);
    expect(screen.getByLabelText("Critical issue")).toBeInTheDocument();
  });
});
