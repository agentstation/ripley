import { useEffect } from "react";

import { events } from "@/lib/bindings";
import { useGuardStore } from "@/store/guard";

export function useGuardEvent() {
  const setEvent = useGuardStore((s) => s.setEvent);

  useEffect(() => {
    const unlistenPromise = events.guardEvent.listen((e) => {
      setEvent(e.payload);
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [setEvent]);
}
