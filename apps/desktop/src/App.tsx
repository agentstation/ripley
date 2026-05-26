import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";

import { GuardDialog } from "./routes/GuardDialog";
import { commands } from "./lib/bindings";
import { useGuardEvent } from "./hooks/useGuardEvent";
import { useGuardStore } from "./store/guard";

export default function App() {
  useGuardEvent();
  const currentEvent = useGuardStore((s) => s.currentEvent);

  useEffect(() => {
    if (import.meta.env.DEV) {
      (window as unknown as { __ripleyGuardStore?: typeof useGuardStore }).__ripleyGuardStore =
        useGuardStore;
    }
  }, []);

  return (
    <main
      data-testid="app-root"
      className="min-h-screen bg-background text-foreground font-sans flex items-center justify-center p-6"
    >
      {currentEvent ? <GuardDialog event={currentEvent} /> : <HomeView />}
    </main>
  );
}

function HomeView() {
  const { data } = useQuery({
    queryKey: ["ping"],
    queryFn: () => commands.ping(),
  });

  return (
    <div className="text-center">
      <h1 className="text-2xl font-semibold">Ripley</h1>
      <p className="text-muted-foreground">Supply chain defense, in progress.</p>
      <p className="text-sm mt-4">IPC: {data ?? "…"}</p>
    </div>
  );
}
