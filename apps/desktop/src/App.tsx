import { useQuery } from "@tanstack/react-query";

import { commands } from "./lib/bindings";

export default function App() {
  const { data } = useQuery({
    queryKey: ["ping"],
    queryFn: () => commands.ping(),
  });

  return (
    <main className="min-h-screen bg-background text-foreground font-sans flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-2xl font-semibold">Ripley</h1>
        <p className="text-muted-foreground">Supply chain defense, in progress.</p>
        <p className="text-sm mt-4">IPC: {data ?? "…"}</p>
      </div>
    </main>
  );
}
