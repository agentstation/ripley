import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

import { EcosystemIcon } from "./EcosystemIcon";
import { SeverityBadge } from "./SeverityBadge";
import { TimestampCell } from "./TimestampCell";
import { WeightBar } from "./WeightBar";

type Props = {
  ecosystem: string;
  packageName: string;
  version?: string;
  severity: string;
  weight?: number;
  observedAt?: string;
  summary?: string;
  actions?: ReactNode;
  onActivate?: () => void;
  className?: string;
};

export function AlertCard({
  ecosystem,
  packageName,
  version,
  severity,
  weight,
  observedAt,
  summary,
  actions,
  onActivate,
  className,
}: Props) {
  const interactive = Boolean(onActivate);
  return (
    // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- role + tabIndex set when onActivate is provided
    <article
      data-testid="alert-card"
      data-package={packageName}
      tabIndex={interactive ? 0 : undefined}
      role={interactive ? "button" : undefined}
      onClick={onActivate}
      onKeyDown={(event) => {
        if (!onActivate) return;
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onActivate();
        }
      }}
      className={cn(
        "flex flex-col gap-2 rounded-lg border border-border-subtle bg-surface p-4 text-sm",
        interactive &&
          "cursor-pointer transition-colors hover:bg-surface-hover focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        className,
      )}
    >
      <header className="flex items-center gap-2">
        <EcosystemIcon ecosystem={ecosystem} />
        <span className="font-mono text-sm text-text-primary">{packageName}</span>
        {version ? <span className="font-mono text-xs text-text-secondary">@{version}</span> : null}
        <SeverityBadge severity={severity} className="ml-auto" />
      </header>
      {summary ? <p className="text-text-secondary">{summary}</p> : null}
      {typeof weight === "number" ? <WeightBar value={weight} /> : null}
      <footer className="flex items-center justify-between gap-2 text-xs text-text-muted">
        {observedAt ? <TimestampCell iso={observedAt} /> : <span />}
        {actions ? <div className="flex gap-2">{actions}</div> : null}
      </footer>
    </article>
  );
}
