import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { RuleMatchList } from "./RuleMatchList";

describe("RuleMatchList", () => {
  it("renders an empty-state when no rules matched", () => {
    render(<RuleMatchList rules={[]} />);
    expect(screen.getByTestId("no-rules")).toBeInTheDocument();
  });

  it("renders each rule as a list item", () => {
    render(<RuleMatchList rules={["postinstall-script", "obfuscated-eval"]} />);
    const items = screen.getAllByRole("listitem");
    expect(items.map((el) => el.textContent)).toEqual(["postinstall-script", "obfuscated-eval"]);
  });

  it("labels the list for assistive tech", () => {
    render(<RuleMatchList rules={["a"]} />);
    expect(screen.getByLabelText("Matched detection rules")).toBeInTheDocument();
  });
});
