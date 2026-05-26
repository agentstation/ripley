import { EmptyState } from "@/components/ripley/EmptyState";
import { ErrorPane } from "@/components/ripley/ErrorPane";
import { LoadingSkeleton } from "@/components/ripley/LoadingSkeleton";
import { TrafficLightDot } from "@/components/ripley/TrafficLightDot";
import { useAuditReport } from "@/hooks/useReports";

export function Audit() {
  const query = useAuditReport();

  if (query.isLoading) {
    return (
      <section data-testid="audit-view" aria-busy="true" className="p-4">
        <LoadingSkeleton rows={5} />
      </section>
    );
  }
  if (query.isError) {
    return (
      <section data-testid="audit-view" className="p-4">
        <ErrorPane
          title="Audit failed"
          message={query.error instanceof Error ? query.error.message : String(query.error)}
        />
      </section>
    );
  }
  const categories = query.data?.categories ?? [];
  if (categories.length === 0) {
    return (
      <section data-testid="audit-view" className="p-4">
        <EmptyState title="No audit categories" />
      </section>
    );
  }
  return (
    <section data-testid="audit-view" aria-label="System audit" className="flex flex-col gap-4 p-4">
      {categories.map((cat) => (
        <article
          key={cat.category}
          data-testid="audit-category"
          className="rounded-lg border border-border-subtle bg-surface p-4"
        >
          <header className="mb-3 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-text-primary">{cat.category}</h2>
            <TrafficLightDot status={cat.overall} />
          </header>
          <ul className="flex flex-col gap-2 text-sm">
            {cat.findings.map((finding) => (
              <li
                key={`${cat.category}-${finding.name}`}
                data-testid="audit-finding"
                className="flex items-start gap-3"
              >
                <TrafficLightDot status={finding.status} label="" />
                <div className="flex-1">
                  <p className="font-medium text-text-primary">{finding.name}</p>
                  <p className="text-text-secondary">{finding.detail}</p>
                  {finding.fix_command ? (
                    <code className="mt-1 inline-block rounded bg-bg px-2 py-0.5 font-mono text-xs text-text-secondary">
                      {finding.fix_command}
                    </code>
                  ) : null}
                </div>
              </li>
            ))}
          </ul>
        </article>
      ))}
    </section>
  );
}
