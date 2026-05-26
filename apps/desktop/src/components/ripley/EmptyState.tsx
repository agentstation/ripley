import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

type Props = {
  title: string;
  description?: string;
  icon?: ReactNode;
  action?: ReactNode;
  className?: string;
};

export function EmptyState({ title, description, icon, action, className }: Props) {
  return (
    <div
      role="status"
      data-testid="empty-state"
      className={cn(
        "flex flex-col items-center justify-center gap-3 rounded-lg border border-border-subtle bg-surface px-6 py-12 text-center",
        className,
      )}
    >
      {icon ? <div className="text-text-muted">{icon}</div> : null}
      <h3 className="text-base font-semibold text-text-primary">{title}</h3>
      {description ? <p className="text-sm text-text-secondary">{description}</p> : null}
      {action ? <div className="pt-2">{action}</div> : null}
    </div>
  );
}
