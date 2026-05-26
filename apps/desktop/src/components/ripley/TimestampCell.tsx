import { cn } from "@/lib/cn";

type Props = {
  iso: string;
  className?: string;
  now?: () => Date;
};

const UNITS: { unit: Intl.RelativeTimeFormatUnit; seconds: number }[] = [
  { unit: "year", seconds: 31_536_000 },
  { unit: "month", seconds: 2_592_000 },
  { unit: "week", seconds: 604_800 },
  { unit: "day", seconds: 86_400 },
  { unit: "hour", seconds: 3_600 },
  { unit: "minute", seconds: 60 },
  { unit: "second", seconds: 1 },
];

const RTF = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

function relativeFromSeconds(deltaSeconds: number): string {
  const abs = Math.abs(deltaSeconds);
  if (abs < 1) {
    return "just now";
  }
  for (const { unit, seconds } of UNITS) {
    if (abs >= seconds) {
      const value = Math.round(deltaSeconds / seconds);
      return RTF.format(value, unit);
    }
  }
  return "just now";
}

export function TimestampCell({ iso, className, now }: Props) {
  const parsed = new Date(iso);
  if (Number.isNaN(parsed.getTime())) {
    return (
      <span
        data-testid="timestamp"
        data-invalid="true"
        className={cn("font-mono text-xs text-text-muted", className)}
      >
        invalid
      </span>
    );
  }
  const reference = now ? now() : new Date();
  const deltaSeconds = (parsed.getTime() - reference.getTime()) / 1000;
  const relative = relativeFromSeconds(deltaSeconds);
  return (
    <time
      data-testid="timestamp"
      dateTime={parsed.toISOString()}
      title={parsed.toISOString()}
      className={cn("font-mono text-xs text-text-secondary tabular-nums", className)}
    >
      {relative}
    </time>
  );
}
