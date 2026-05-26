import { useEffect } from "react";

import { events } from "@/lib/bindings";
import { useGuardStore } from "@/store/guard";

function hasTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function useGuardEvent() {
  const setEvent = useGuardStore((s) => s.setEvent);

  useEffect(() => {
    if (!hasTauriRuntime()) return;
    const unlistenPromise = events.guardEvent.listen((e) => {
      setEvent(e.payload);
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten()).catch(() => undefined);
    };
  }, [setEvent]);
}
