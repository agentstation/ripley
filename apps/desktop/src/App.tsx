import { useSyncExternalStore } from "react";

import { CommandPalette } from "./components/ripley/CommandPalette";
import { GuardDialog } from "./routes/GuardDialog";
import { Alerts } from "./routes/Alerts";
import { Audit } from "./routes/Audit";
import { DeepScan } from "./routes/DeepScan";
import { GuardLog } from "./routes/GuardLog";
import { Monitor } from "./routes/Monitor";
import { Posture } from "./routes/Posture";
import { Settings } from "./routes/Settings";
import { useCommandPaletteShortcut } from "./hooks/useCommandPaletteShortcut";
import { useCommands } from "./hooks/useCommands";
import { useGuardEvent } from "./hooks/useGuardEvent";
import { useGuardStore } from "./store/guard";

const ROUTES = [
  "alerts",
  "guard-log",
  "monitor",
  "deep-scan",
  "audit",
  "posture",
  "settings",
] as const;

type Route = (typeof ROUTES)[number];

const DEFAULT_ROUTE: Route = "alerts";
const DEFAULT_PROJECT_PATH = ".";

const ROUTE_SET = new Set<string>(ROUTES);

function readRoute(): Route {
  const raw = window.location.hash.replace(/^#\/?/, "");
  return ROUTE_SET.has(raw) ? (raw as Route) : DEFAULT_ROUTE;
}

function subscribeHash(callback: () => void) {
  window.addEventListener("hashchange", callback);
  return () => window.removeEventListener("hashchange", callback);
}

export function navigateTo(target: string) {
  window.location.hash = `/${target}`;
}

export default function App() {
  useGuardEvent();
  useCommandPaletteShortcut();
  const currentEvent = useGuardStore((s) => s.currentEvent);
  const route = useSyncExternalStore<Route>(subscribeHash, readRoute, () => DEFAULT_ROUTE);
  const commands = useCommands({ navigate: navigateTo });

  return (
    <main
      data-testid="app-root"
      data-route={route}
      className="min-h-screen bg-background text-foreground font-sans"
    >
      <header className="flex items-center gap-3 border-b border-border-subtle bg-surface px-4 py-2">
        <h1 className="text-sm font-semibold tracking-wide text-text-primary">Ripley</h1>
        <span data-testid="route-label" className="text-xs uppercase text-text-muted">
          {route}
        </span>
      </header>
      {currentEvent ? (
        <GuardDialog event={currentEvent} />
      ) : (
        <>
          <RouteView route={route} />
          <CommandPalette commands={commands} />
        </>
      )}
    </main>
  );
}

function RouteView({ route }: { route: Route }) {
  switch (route) {
    case "alerts":
      return <Alerts projectPath={DEFAULT_PROJECT_PATH} />;
    case "guard-log":
      return <GuardLog />;
    case "monitor":
      return <Monitor />;
    case "deep-scan":
      return <DeepScan projectPath={DEFAULT_PROJECT_PATH} />;
    case "audit":
      return <Audit />;
    case "posture":
      return <Posture projectPath={DEFAULT_PROJECT_PATH} />;
    case "settings":
      return <Settings />;
  }
}
