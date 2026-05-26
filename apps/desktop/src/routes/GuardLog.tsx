import { useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";

import { DataTable } from "@/components/ripley/DataTable";
import { EmptyState } from "@/components/ripley/EmptyState";
import { ErrorPane } from "@/components/ripley/ErrorPane";
import { LoadingSkeleton } from "@/components/ripley/LoadingSkeleton";
import { SeverityBadge } from "@/components/ripley/SeverityBadge";
import { TimestampCell } from "@/components/ripley/TimestampCell";
import { Button } from "@/components/ui/button";
import { useGuardLog } from "@/hooks/useGuardLog";
import { cn } from "@/lib/cn";
import type { GuardLogEntry } from "@/lib/bindings";

type Decision = "all" | "allowed" | "blocked" | "trusted";

const DECISIONS: Decision[] = ["all", "allowed", "blocked", "trusted"];
const PAGE_SIZE = 50;

const COLUMNS: ColumnDef<GuardLogEntry, unknown>[] = [
  {
    id: "timestamp",
    accessorKey: "timestamp",
    header: "Time",
    cell: ({ row }) => <TimestampCell iso={row.original.timestamp} />,
    size: 160,
  },
  {
    id: "package",
    accessorKey: "package",
    header: "Package",
    cell: ({ row }) => (
      <span className="font-mono text-xs">
        {row.original.package || <span className="text-text-muted">unknown</span>}
        {row.original.version ? (
          <span className="text-text-muted"> @ {row.original.version}</span>
        ) : null}
      </span>
    ),
  },
  {
    id: "action",
    accessorKey: "action",
    header: "Decision",
    cell: ({ row }) => {
      const action = row.original.action;
      const tone =
        action === "blocked"
          ? "text-severity-critical"
          : action === "allowed"
            ? "text-severity-clean"
            : "text-text-secondary";
      return <span className={cn("uppercase text-xs font-semibold", tone)}>{action || "—"}</span>;
    },
    size: 110,
  },
  {
    id: "risk_level",
    accessorKey: "risk_level",
    header: "Risk",
    cell: ({ row }) => <SeverityBadge severity={row.original.risk_level || "low"} />,
    size: 110,
  },
  {
    id: "rules",
    accessorKey: "matched_rules",
    header: "Matched rules",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-text-secondary">
        {row.original.matched_rules.length ? row.original.matched_rules.join(", ") : "—"}
      </span>
    ),
  },
];

export function GuardLog() {
  const [offset, setOffset] = useState(0);
  const [decision, setDecision] = useState<Decision>("all");
  const query = useGuardLog(offset, PAGE_SIZE);

  const filtered = useMemo(() => {
    const entries = query.data?.entries ?? [];
    if (decision === "all") return entries;
    return entries.filter((e) => e.action === decision);
  }, [query.data, decision]);

  const total = query.data?.total ?? 0;
  const hasNext = offset + PAGE_SIZE < total;
  const hasPrev = offset > 0;

  if (query.isLoading) {
    return (
      <section data-testid="guard-log-view" aria-busy="true" className="p-4">
        <LoadingSkeleton rows={6} />
      </section>
    );
  }

  if (query.isError) {
    return (
      <section data-testid="guard-log-view" className="p-4">
        <ErrorPane
          title="Failed to read guard log"
          message={query.error instanceof Error ? query.error.message : String(query.error)}
        />
      </section>
    );
  }

  return (
    <section
      data-testid="guard-log-view"
      aria-label="Guard log"
      className="flex h-full flex-col gap-3 p-4"
    >
      <header className="flex items-center justify-between gap-2">
        <div
          role="radiogroup"
          aria-label="Filter by decision"
          className="flex gap-1"
          data-testid="guard-log-filter"
        >
          {DECISIONS.map((d) => (
            <button
              key={d}
              type="button"
              role="radio"
              aria-checked={decision === d}
              data-testid={`filter-${d}`}
              onClick={() => setDecision(d)}
              className={cn(
                "rounded-md border border-border-subtle px-3 py-1 text-xs uppercase tracking-wide",
                decision === d
                  ? "bg-surface-elevated text-text-primary"
                  : "bg-surface text-text-secondary hover:text-text-primary",
              )}
            >
              {d}
            </button>
          ))}
        </div>
        <span className="text-xs text-text-muted" data-testid="guard-log-total">
          {total} entries
        </span>
      </header>

      {filtered.length === 0 ? (
        <EmptyState
          title={decision === "all" ? "No guard entries yet" : `No ${decision} entries`}
          description="Decisions taken on lifecycle scripts will appear here."
        />
      ) : (
        <DataTable
          data={filtered}
          columns={COLUMNS}
          ariaLabel="Guard log entries"
          virtualizeFrom={200}
        />
      )}

      <footer className="flex items-center justify-end gap-2">
        <Button
          variant="ghost"
          size="sm"
          disabled={!hasPrev}
          onClick={() => setOffset((o) => Math.max(0, o - PAGE_SIZE))}
          data-testid="guard-log-prev"
        >
          Previous
        </Button>
        <span className="text-xs text-text-muted" data-testid="guard-log-page">
          {offset + 1}–{Math.min(offset + PAGE_SIZE, total)} of {total}
        </span>
        <Button
          variant="ghost"
          size="sm"
          disabled={!hasNext}
          onClick={() => setOffset((o) => o + PAGE_SIZE)}
          data-testid="guard-log-next"
        >
          Next
        </Button>
      </footer>
    </section>
  );
}
