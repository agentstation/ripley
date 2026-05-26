import { cn } from "@/lib/cn";

type Props = {
  value: number;
  label?: string;
  className?: string;
};

export function WeightBar({ value, label, className }: Props) {
  const clamped = Math.max(0, Math.min(1, value));
  const pct = Math.round(clamped * 100);
  const tone =
    clamped >= 0.75
      ? "bg-severity-critical"
      : clamped >= 0.5
        ? "bg-severity-high"
        : clamped >= 0.25
          ? "bg-severity-medium"
          : "bg-severity-low";

  return (
    <div
      data-testid="weight-bar"
      className={cn("flex items-center gap-2", className)}
      role="meter"
      aria-valuemin={0}
      aria-valuemax={1}
      aria-valuenow={clamped}
      aria-label={label ?? "Risk weight"}
    >
      <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-surface-active">
        <div
          data-testid="weight-fill"
          className={cn("h-full rounded-full transition-[width]", tone)}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="font-mono text-xs text-text-secondary tabular-nums">
        {pct.toString().padStart(2, "0")}%
      </span>
    </div>
  );
}
