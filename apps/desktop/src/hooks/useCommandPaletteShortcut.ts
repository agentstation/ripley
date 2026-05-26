import { useEffect } from "react";

import { useCommandPaletteStore } from "@/store/command-palette";

export function useCommandPaletteShortcut() {
  const toggle = useCommandPaletteStore((s) => s.toggle);
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        toggle();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggle]);
}
