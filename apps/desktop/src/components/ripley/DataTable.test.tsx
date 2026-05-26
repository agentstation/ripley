import { describe, expect, it } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import type { ColumnDef } from "@tanstack/react-table";

import { DataTable } from "./DataTable";
import { EmptyState } from "./EmptyState";

type Row = { id: number; name: string; weight: number };

const COLUMNS: ColumnDef<Row, unknown>[] = [
  { id: "name", accessorKey: "name", header: "Name" },
  { id: "weight", accessorKey: "weight", header: "Weight" },
];

function rows(n: number): Row[] {
  return Array.from({ length: n }, (_, i) => ({
    id: i,
    name: `pkg-${String(i).padStart(3, "0")}`,
    weight: (n - i) / n,
  }));
}

describe("DataTable", () => {
  it("renders rows below the virtualization threshold without virtualizing", () => {
    render(<DataTable data={rows(10)} columns={COLUMNS} ariaLabel="t" />);
    const scroll = screen.getByTestId("data-table-scroll");
    expect(scroll).toHaveAttribute("data-virtualized", "false");
    expect(screen.getAllByTestId("data-table-row")).toHaveLength(10);
  });

  it("virtualizes above the threshold", () => {
    const scroll = render(
      <DataTable data={rows(500)} columns={COLUMNS} ariaLabel="t" virtualizeFrom={200} />,
    ).getByTestId("data-table-scroll");
    expect(scroll).toHaveAttribute("data-virtualized", "true");
    expect(screen.queryAllByTestId("data-table-row").length).toBeLessThan(500);
  });

  it("toggles sort direction via header button", () => {
    render(<DataTable data={rows(5)} columns={COLUMNS} ariaLabel="t" />);
    const sortBtn = screen.getByTestId("sort-name");
    fireEvent.click(sortBtn);
    const header = sortBtn.closest("th");
    expect(header).toHaveAttribute("aria-sort", "ascending");
    fireEvent.click(sortBtn);
    expect(header).toHaveAttribute("aria-sort", "descending");
  });

  it("renders empty state when data is empty", () => {
    render(
      <DataTable
        data={[]}
        columns={COLUMNS}
        ariaLabel="t"
        emptyState={<EmptyState title="no rows" />}
      />,
    );
    expect(screen.getByTestId("empty-state")).toBeInTheDocument();
  });

  it("filters rows when globalFilter is set", () => {
    render(<DataTable data={rows(20)} columns={COLUMNS} ariaLabel="t" globalFilter="pkg-005" />);
    const visible = screen.getAllByTestId("data-table-row");
    expect(visible).toHaveLength(1);
    expect(within(visible[0]!).getByText("pkg-005")).toBeInTheDocument();
  });
});
