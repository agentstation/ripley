import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

type Props = {
  title: string;
  message?: string;
  action?: ReactNode;
  className?: string;
};

export function ErrorPane({ title, message, action, className }: Props) {
  return (
    <div
      role="alert"
      data-testid="error-pane"
      className={cn(
        "flex flex-col items-start gap-2 rounded-lg border border-severity-critical-bg bg-severity-critical-bg px-4 py-3 text-sm",
        className,
      )}
    >
      <h3 className="font-semibold text-severity-critical">{title}</h3>
      {message ? <p className="text-text-secondary">{message}</p> : null}
      {action ? <div className="pt-1">{action}</div> : null}
    </div>
  );
}
