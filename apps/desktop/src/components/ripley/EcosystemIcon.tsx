import { cn } from "@/lib/cn";

export type Ecosystem =
  | "npm"
  | "pypi"
  | "cargo"
  | "rubygems"
  | "go"
  | "maven"
  | "nuget"
  | "composer"
  | "unknown";

const LABELS: Record<Ecosystem, string> = {
  npm: "npm",
  pypi: "py",
  cargo: "rs",
  rubygems: "rb",
  go: "go",
  maven: "mvn",
  nuget: "nu",
  composer: "ph",
  unknown: "?",
};

const TONES: Record<Ecosystem, string> = {
  npm: "bg-severity-critical-bg text-severity-critical",
  pypi: "bg-severity-low-bg text-severity-low",
  cargo: "bg-severity-high-bg text-severity-high",
  rubygems: "bg-severity-critical-bg text-severity-critical",
  go: "bg-severity-low-bg text-severity-low",
  maven: "bg-severity-high-bg text-severity-high",
  nuget: "bg-severity-low-bg text-severity-low",
  composer: "bg-severity-medium-bg text-severity-medium",
  unknown: "bg-muted text-text-secondary",
};

function normalise(raw: string): Ecosystem {
  const key = raw.toLowerCase();
  if (key in LABELS) {
    return key as Ecosystem;
  }
  return "unknown";
}

type Props = {
  ecosystem: string;
  className?: string;
};

export function EcosystemIcon({ ecosystem, className }: Props) {
  const eco = normalise(ecosystem);
  return (
    <span
      data-testid="ecosystem-icon"
      data-ecosystem={eco}
      aria-label={`${ecosystem} ecosystem`}
      className={cn(
        "inline-flex h-5 w-8 items-center justify-center rounded font-mono text-[10px] font-bold uppercase",
        TONES[eco],
        className,
      )}
    >
      {LABELS[eco]}
    </span>
  );
}
