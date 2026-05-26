import { cn } from "@/lib/cn";

type Props = {
  rows?: number;
  className?: string;
};

export function LoadingSkeleton({ rows = 3, className }: Props) {
  return (
    <div
      data-testid="loading-skeleton"
      role="status"
      aria-busy="true"
      aria-label="Loading"
      className={cn("flex flex-col gap-2", className)}
    >
      {Array.from({ length: rows }).map((_, i) => (
        <div
          key={i}
          className="h-4 w-full animate-pulse rounded bg-muted"
          data-testid="skeleton-row"
        />
      ))}
    </div>
  );
}
