import { EmptyState } from "@/components/ripley/EmptyState";
import { ErrorPane } from "@/components/ripley/ErrorPane";
import { LoadingSkeleton } from "@/components/ripley/LoadingSkeleton";
import { SeverityBadge } from "@/components/ripley/SeverityBadge";
import { TimestampCell } from "@/components/ripley/TimestampCell";
import { useMonitorEvents } from "@/hooks/useReports";

export function Monitor() {
  const query = useMonitorEvents();

  if (query.isLoading) {
    return (
      <section data-testid="monitor-view" aria-busy="true" className="p-4">
        <LoadingSkeleton rows={6} />
      </section>
    );
  }
  if (query.isError) {
    return (
      <section data-testid="monitor-view" className="p-4">
        <ErrorPane
          title="Monitor unavailable"
          message={query.error instanceof Error ? query.error.message : String(query.error)}
        />
      </section>
    );
  }
  const events = query.data ?? [];
  if (events.length === 0) {
    return (
      <section data-testid="monitor-view" className="p-4">
        <EmptyState
          title="No monitor events"
          description="When the monitor daemon is running, recent activity appears here."
        />
      </section>
    );
  }
  return (
    <section
      data-testid="monitor-view"
      aria-label="Recent monitor events"
      className="flex flex-col gap-2 p-4"
    >
      {events.map((event, idx) => (
        <article
          key={`${event.timestamp}-${event.process}-${idx}`}
          data-testid="monitor-event"
          className="rounded-lg border border-border-subtle bg-surface p-3 text-sm"
        >
          <header className="flex items-center gap-2">
            <SeverityBadge severity={event.severity || "low"} />
            <span className="font-mono text-xs text-text-secondary">{event.event_type}</span>
            <span className="ml-auto text-xs text-text-muted">
              <TimestampCell iso={event.timestamp} />
            </span>
          </header>
          <p className="mt-1 text-text-primary">
            <span className="font-mono">{event.process}</span>
            {event.pid ? <span className="text-text-muted"> (pid {event.pid})</span> : null}
            {event.reason ? <span className="text-text-secondary"> — {event.reason}</span> : null}
          </p>
          {event.detail ? <p className="text-text-secondary">{event.detail}</p> : null}
        </article>
      ))}
    </section>
  );
}
