import { EmptyState } from "@/components/ripley/EmptyState";
import { ErrorPane } from "@/components/ripley/ErrorPane";
import { LoadingSkeleton } from "@/components/ripley/LoadingSkeleton";
import { TrafficLightDot } from "@/components/ripley/TrafficLightDot";
import { useHardenReport } from "@/hooks/useReports";

type Props = {
  projectPath: string;
};

export function Posture({ projectPath }: Props) {
  const query = useHardenReport(projectPath);

  if (query.isLoading) {
    return (
      <section data-testid="posture-view" aria-busy="true" className="p-4">
        <LoadingSkeleton rows={5} />
      </section>
    );
  }
  if (query.isError) {
    return (
      <section data-testid="posture-view" className="p-4">
        <ErrorPane
          title="Failed to compute posture"
          message={query.error instanceof Error ? query.error.message : String(query.error)}
        />
      </section>
    );
  }
  const report = query.data;
  if (!report || report.detected_pms.length === 0) {
    return (
      <section data-testid="posture-view" className="p-4">
        <EmptyState
          title="No package managers detected"
          description="Ripley looks for package-lock.json, pnpm-lock.yaml, yarn.lock, Cargo.lock and friends."
        />
      </section>
    );
  }
  return (
    <section
      data-testid="posture-view"
      aria-label="Package manager hardening"
      className="flex flex-col gap-4 p-4"
    >
      <header data-testid="detected-pms" className="flex flex-wrap gap-2 text-xs">
        {report.detected_pms.map((pm) => (
          <span
            key={pm.lockfile_path}
            className="rounded-md border border-border-subtle bg-surface px-2 py-0.5 font-mono"
          >
            {pm.name}
            {pm.version ? ` v${pm.version}` : ""}
          </span>
        ))}
      </header>
      {report.categories.map((cat) => (
        <article
          key={cat.category}
          data-testid="posture-category"
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
                data-testid="posture-finding"
                className="flex items-start gap-3"
              >
                <TrafficLightDot status={finding.status} label="" />
                <div className="flex-1">
                  <p className="font-medium text-text-primary">
                    {finding.name}
                    {finding.pm ? <span className="text-text-muted"> [{finding.pm}]</span> : null}
                  </p>
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
