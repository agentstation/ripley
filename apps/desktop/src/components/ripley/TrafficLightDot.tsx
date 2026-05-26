import { cn } from "@/lib/cn";

type TrafficLightLevel = "green" | "yellow" | "red";

const TONES: Record<TrafficLightLevel, string> = {
  green: "bg-severity-clean text-severity-clean-bg",
  yellow: "bg-severity-medium text-severity-medium-bg",
  red: "bg-severity-critical text-severity-critical-bg",
};

function normalise(value: string): TrafficLightLevel {
  const v = value.toLowerCase();
  if (v === "red" || v === "yellow" || v === "green") return v;
  return "yellow";
}

type Props = {
  status: string;
  label?: string;
  className?: string;
};

export function TrafficLightDot({ status, label, className }: Props) {
  const level = normalise(status);
  return (
    <span
      data-status={level}
      aria-label={label ?? level}
      className={cn(
        "inline-flex items-center gap-1.5 text-xs uppercase tracking-wide font-medium",
        className,
      )}
    >
      <span
        aria-hidden="true"
        className={cn("inline-block h-2.5 w-2.5 rounded-full", TONES[level])}
      />
      {label ?? level}
    </span>
  );
}
