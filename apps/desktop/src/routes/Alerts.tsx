import { useMemo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { AlertCard } from "@/components/ripley/AlertCard";
import { EmptyState } from "@/components/ripley/EmptyState";
import { ErrorPane } from "@/components/ripley/ErrorPane";
import { LoadingSkeleton } from "@/components/ripley/LoadingSkeleton";
import { useAlerts } from "@/hooks/useAlerts";
import type { AlertSummary } from "@/lib/bindings";

type Props = {
  projectPath: string;
  onSelect?: (alert: AlertSummary) => void;
  virtualizeFrom?: number;
  cardHeight?: number;
};

const DEFAULT_VIRTUALIZE_FROM = 30;
const DEFAULT_CARD_HEIGHT = 140;

export function Alerts({
  projectPath,
  onSelect,
  virtualizeFrom = DEFAULT_VIRTUALIZE_FROM,
  cardHeight = DEFAULT_CARD_HEIGHT,
}: Props) {
  const query = useAlerts(projectPath);
  const items = useMemo(() => query.data ?? [], [query.data]);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const shouldVirtualize = items.length >= virtualizeFrom;

  // eslint-disable-next-line react-hooks/incompatible-library -- TanStack Virtual returns non-memoizable functions; this is its documented usage.
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => cardHeight,
    overscan: 4,
    enabled: shouldVirtualize,
  });

  if (query.isLoading) {
    return (
      <section data-testid="alerts-view" aria-busy="true" className="space-y-3 p-4">
        <LoadingSkeleton rows={4} />
      </section>
    );
  }

  if (query.isError) {
    return (
      <section data-testid="alerts-view" className="p-4">
        <ErrorPane
          title="Failed to load alerts"
          message={query.error instanceof Error ? query.error.message : String(query.error)}
        />
      </section>
    );
  }

  if (items.length === 0) {
    return (
      <section data-testid="alerts-view" className="p-4">
        <EmptyState
          title="No active alerts"
          description="Ripley has not matched any advisories against the packages installed in this project."
        />
      </section>
    );
  }

  const renderCard = (alert: AlertSummary) => (
    <AlertCard
      ecosystem={alert.ecosystem}
      packageName={alert.package}
      version={alert.version}
      severity={alert.severity}
      summary={alert.summary}
      onActivate={onSelect ? () => onSelect(alert) : undefined}
    />
  );

  return (
    <section
      data-testid="alerts-view"
      aria-label="Active advisory alerts"
      className="h-full"
      data-virtualized={shouldVirtualize ? "true" : "false"}
    >
      <div
        ref={scrollRef}
        data-testid="alerts-scroll"
        className="h-full max-h-[calc(100vh-3rem)] overflow-auto p-4"
      >
        {shouldVirtualize ? (
          <div style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}>
            {virtualizer.getVirtualItems().map((row) => {
              const alert = items[row.index]!;
              return (
                <div
                  key={alert.id}
                  data-testid="alerts-row"
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    transform: `translateY(${row.start}px)`,
                    paddingBottom: 12,
                  }}
                >
                  {renderCard(alert)}
                </div>
              );
            })}
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {items.map((alert) => (
              <div key={alert.id} data-testid="alerts-row">
                {renderCard(alert)}
              </div>
            ))}
          </div>
        )}
      </div>
    </section>
  );
}
