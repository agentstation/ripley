import { useMemo, useRef, useState } from "react";
import {
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type ColumnSizingState,
  type SortingState,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";

import { cn } from "@/lib/cn";

type Props<TData> = {
  data: TData[];
  columns: ColumnDef<TData, unknown>[];
  globalFilter?: string;
  ariaLabel: string;
  rowHeight?: number;
  virtualizeFrom?: number;
  className?: string;
  emptyState?: React.ReactNode;
};

export function DataTable<TData>({
  data,
  columns,
  globalFilter,
  ariaLabel,
  rowHeight = 36,
  virtualizeFrom = 200,
  className,
  emptyState,
}: Props<TData>) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnSizing, setColumnSizing] = useState<ColumnSizingState>({});

  // eslint-disable-next-line react-hooks/incompatible-library -- TanStack Table returns non-memoizable functions; this is its documented usage.
  const table = useReactTable({
    data,
    columns,
    state: { sorting, columnSizing, globalFilter },
    onSortingChange: setSorting,
    onColumnSizingChange: setColumnSizing,
    columnResizeMode: "onChange",
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  });

  const rows = table.getRowModel().rows;
  const shouldVirtualize = rows.length > virtualizeFrom;
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => rowHeight,
    overscan: 10,
    enabled: shouldVirtualize,
  });

  const virtualRows = shouldVirtualize ? virtualizer.getVirtualItems() : [];
  const totalSize = shouldVirtualize ? virtualizer.getTotalSize() : 0;

  const headerGroups = useMemo(() => table.getHeaderGroups(), [table]);

  if (rows.length === 0 && emptyState) {
    return <>{emptyState}</>;
  }

  return (
    <div
      ref={scrollRef}
      data-testid="data-table-scroll"
      data-virtualized={shouldVirtualize ? "true" : "false"}
      className={cn("relative overflow-auto rounded-lg border border-border-subtle", className)}
    >
      <table
        aria-label={ariaLabel}
        className="w-full border-collapse text-left text-sm"
        style={{ tableLayout: "fixed" }}
      >
        <thead className="sticky top-0 z-10 bg-surface">
          {headerGroups.map((group) => (
            <tr key={group.id}>
              {group.headers.map((header) => {
                const sortDir = header.column.getIsSorted();
                const canSort = header.column.getCanSort();
                return (
                  <th
                    key={header.id}
                    scope="col"
                    aria-sort={
                      sortDir === "asc"
                        ? "ascending"
                        : sortDir === "desc"
                          ? "descending"
                          : canSort
                            ? "none"
                            : undefined
                    }
                    style={{ width: header.getSize() }}
                    className="border-b border-border-subtle px-3 py-2 font-semibold text-text-secondary"
                  >
                    <div className="flex items-center gap-1">
                      {canSort ? (
                        <button
                          type="button"
                          data-testid={`sort-${header.column.id}`}
                          onClick={header.column.getToggleSortingHandler()}
                          className="inline-flex items-center gap-1 hover:text-text-primary"
                        >
                          {flexRender(header.column.columnDef.header, header.getContext())}
                          <span aria-hidden="true" className="text-text-muted">
                            {sortDir === "asc" ? "▲" : sortDir === "desc" ? "▼" : "↕"}
                          </span>
                        </button>
                      ) : (
                        flexRender(header.column.columnDef.header, header.getContext())
                      )}
                    </div>
                    {header.column.getCanResize() ? (
                      <span
                        role="separator"
                        aria-orientation="vertical"
                        aria-hidden="true"
                        onMouseDown={header.getResizeHandler()}
                        onTouchStart={header.getResizeHandler()}
                        data-testid={`resize-${header.column.id}`}
                        className="absolute right-0 top-0 h-full w-1 cursor-col-resize select-none touch-none bg-transparent hover:bg-accent"
                      />
                    ) : null}
                  </th>
                );
              })}
            </tr>
          ))}
        </thead>
        <tbody
          style={{ height: shouldVirtualize ? `${totalSize}px` : undefined, position: "relative" }}
        >
          {shouldVirtualize
            ? virtualRows.map((vRow) => {
                const row = rows[vRow.index];
                if (!row) return null;
                return (
                  <tr
                    key={row.id}
                    data-testid="data-table-row"
                    data-index={vRow.index}
                    className="absolute left-0 w-full hover:bg-surface-hover"
                    style={{
                      transform: `translateY(${vRow.start}px)`,
                      height: `${vRow.size}px`,
                    }}
                  >
                    {row.getVisibleCells().map((cell) => (
                      <td
                        key={cell.id}
                        style={{ width: cell.column.getSize() }}
                        className="truncate border-b border-border-subtle px-3 py-2 text-text-primary"
                      >
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </td>
                    ))}
                  </tr>
                );
              })
            : rows.map((row, index) => (
                <tr
                  key={row.id}
                  data-testid="data-table-row"
                  data-index={index}
                  className="hover:bg-surface-hover"
                >
                  {row.getVisibleCells().map((cell) => (
                    <td
                      key={cell.id}
                      style={{ width: cell.column.getSize() }}
                      className="truncate border-b border-border-subtle px-3 py-2 text-text-primary"
                    >
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </td>
                  ))}
                </tr>
              ))}
        </tbody>
      </table>
    </div>
  );
}
