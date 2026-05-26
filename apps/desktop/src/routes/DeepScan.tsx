import { EmptyState } from "@/components/ripley/EmptyState";
import { ErrorPane } from "@/components/ripley/ErrorPane";
import { KeyValueGrid } from "@/components/ripley/KeyValueGrid";
import { LoadingSkeleton } from "@/components/ripley/LoadingSkeleton";
import { SeverityBadge } from "@/components/ripley/SeverityBadge";
import { useDeepScan } from "@/hooks/useReports";

type Props = {
  projectPath: string;
};

export function DeepScan({ projectPath }: Props) {
  const query = useDeepScan(projectPath);

  if (query.isLoading) {
    return (
      <section data-testid="deep-scan-view" aria-busy="true" className="p-4">
        <LoadingSkeleton rows={4} />
      </section>
    );
  }
  if (query.isError) {
    return (
      <section data-testid="deep-scan-view" className="p-4">
        <ErrorPane
          title="Deep scan failed"
          message={query.error instanceof Error ? query.error.message : String(query.error)}
        />
      </section>
    );
  }
  const report = query.data;
  if (!report) {
    return (
      <section data-testid="deep-scan-view" className="p-4">
        <EmptyState title="No deep scan available" />
      </section>
    );
  }

  return (
    <section
      data-testid="deep-scan-view"
      aria-label="Forensic deep scan"
      className="flex flex-col gap-4 p-4"
    >
      <KeyValueGrid
        ariaLabel="Forensic summary"
        rows={[
          { key: "Project", value: report.project_path, mono: true },
          { key: "IOC findings", value: String(report.ioc_count) },
          { key: "Persistence findings", value: String(report.persistence_count) },
          { key: "Credential findings", value: String(report.credential_count) },
          ...(report.dead_man_switch_warning
            ? [{ key: "Dead-man switch", value: report.dead_man_switch_warning }]
            : []),
        ]}
      />
      {report.findings.length === 0 ? (
        <EmptyState
          title="No forensic findings"
          description="Nothing flagged by the bundled IOC profiles."
        />
      ) : (
        <ul data-testid="deep-scan-findings" className="flex flex-col gap-2 text-sm">
          {report.findings.map((finding) => (
            <li
              key={`${finding.kind}-${finding.path}-${finding.description}`}
              data-testid="deep-scan-finding"
              className="rounded-lg border border-border-subtle bg-surface p-3"
            >
              <header className="flex items-center justify-between">
                <span className="text-xs uppercase tracking-wide text-text-muted">
                  {finding.kind}
                </span>
                <SeverityBadge severity={finding.severity} />
              </header>
              <p className="mt-1 font-mono text-xs text-text-secondary">{finding.path}</p>
              <p className="mt-1 text-text-primary">{finding.description}</p>
              {finding.rotation_command ? (
                <code className="mt-1 inline-block rounded bg-bg px-2 py-0.5 font-mono text-xs text-text-secondary">
                  {finding.rotation_command}
                </code>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
