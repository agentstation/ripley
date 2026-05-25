import { useEffect, useRef } from "react";

import { Button } from "@/components/ui/button";
import { RuleMatchList } from "@/components/ripley/RuleMatchList";
import { SeverityBadge } from "@/components/ripley/SeverityBadge";
import { commands, type GuardEvent } from "@/lib/bindings";
import { useGuardStore } from "@/store/guard";

type Props = {
  event: GuardEvent;
};

export function GuardDialog({ event }: Props) {
  const respond = useGuardStore((s) => s.respond);
  const inFlight = useGuardStore((s) => s.inFlight);
  const trustRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    trustRef.current?.focus();
    void commands.reportVisible(event.id);
  }, [event.id]);

  return (
    <section
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="guard-title"
      aria-describedby="guard-script"
      className="w-full max-w-xl rounded-lg border border-border bg-surface-elevated p-6 shadow-xl"
    >
      <header className="mb-4 flex items-start justify-between gap-4">
        <div>
          <h1 id="guard-title" className="text-lg font-semibold text-foreground">
            About to run an install script
          </h1>
          <p className="text-sm text-text-secondary">
            <span className="font-mono">{event.package}</span>
            <span className="text-text-muted"> @ </span>
            <span className="font-mono">{event.version}</span>
          </p>
        </div>
        <SeverityBadge severity={event.risk_level} />
      </header>

      <div className="mb-4">
        <h2 className="mb-1 text-xs font-medium uppercase tracking-wide text-text-muted">
          Matched rules
        </h2>
        <RuleMatchList rules={event.matched_rules} />
      </div>

      <div className="mb-6">
        <h2 className="mb-1 text-xs font-medium uppercase tracking-wide text-text-muted">Script</h2>
        <pre
          id="guard-script"
          className="max-h-48 overflow-auto rounded border border-border-subtle bg-bg p-3 text-xs text-text-secondary"
        >
          {event.script}
        </pre>
      </div>

      <footer className="flex items-center justify-end gap-2">
        <Button
          variant="ghost"
          disabled={inFlight}
          onClick={() => respond("Allow")}
          data-testid="allow-once"
        >
          Allow once
        </Button>
        <Button
          variant="destructive"
          disabled={inFlight}
          onClick={() => respond("Block")}
          data-testid="block"
        >
          Block
        </Button>
        <Button
          ref={trustRef}
          variant="default"
          disabled={inFlight}
          onClick={() => respond("Trust")}
          data-testid="trust"
        >
          Allow & trust
        </Button>
      </footer>
    </section>
  );
}
