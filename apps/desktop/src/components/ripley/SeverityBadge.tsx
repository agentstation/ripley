import { cn } from "@/lib/cn";

export type SeverityLevel = "critical" | "high" | "medium" | "low" | "clean";

const STYLES: Record<SeverityLevel, string> = {
  critical: "bg-severity-critical-bg text-severity-critical",
  high: "bg-severity-high-bg text-severity-high",
  medium: "bg-severity-medium-bg text-severity-medium",
  low: "bg-severity-low-bg text-severity-low",
  clean: "bg-severity-clean-bg text-severity-clean",
};

const WEIGHTS: Record<string, SeverityLevel> = {
  critical: "critical",
  high: "high",
  medium: "medium",
  moderate: "medium",
  low: "low",
  info: "low",
  none: "clean",
  clean: "clean",
};

export function normaliseSeverity(raw: string): SeverityLevel {
  return WEIGHTS[raw.toLowerCase()] ?? "medium";
}

type Props = {
  severity: string;
  className?: string;
};

export function SeverityBadge({ severity, className }: Props) {
  const level = normaliseSeverity(severity);
  return (
    <span
      data-severity={level}
      className={cn(
        "inline-flex items-center rounded-md px-2 py-0.5 text-xs font-semibold uppercase tracking-wide",
        STYLES[level],
        className,
      )}
    >
      {level}
    </span>
  );
}
