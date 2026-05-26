import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

export type KeyValueRow = {
  key: string;
  value: ReactNode;
  mono?: boolean;
};

type Props = {
  rows: KeyValueRow[];
  className?: string;
  ariaLabel?: string;
};

export function KeyValueGrid({ rows, className, ariaLabel }: Props) {
  return (
    <dl
      data-testid="key-value-grid"
      aria-label={ariaLabel}
      className={cn(
        "grid grid-cols-[minmax(8rem,max-content)_1fr] gap-x-4 gap-y-2 rounded-lg border border-border-subtle bg-surface px-4 py-3 text-sm",
        className,
      )}
    >
      {rows.map((row) => (
        <div key={row.key} className="contents" data-testid="key-value-row">
          <dt className="text-text-secondary">{row.key}</dt>
          <dd className={cn("text-text-primary", row.mono && "font-mono text-xs tabular-nums")}>
            {row.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}
