import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

const RECENCY_LIMIT = 5;

type CommandPaletteState = {
  open: boolean;
  recent: string[];
  setOpen: (open: boolean) => void;
  toggle: () => void;
  recordUse: (commandId: string) => void;
  clearRecent: () => void;
};

export const useCommandPaletteStore = create<CommandPaletteState>()(
  persist(
    (set, get) => ({
      open: false,
      recent: [],
      setOpen: (open) => set({ open }),
      toggle: () => set({ open: !get().open }),
      recordUse: (commandId) => {
        const previous = get().recent.filter((id) => id !== commandId);
        set({ recent: [commandId, ...previous].slice(0, RECENCY_LIMIT) });
      },
      clearRecent: () => set({ recent: [] }),
    }),
    {
      name: "ripley.command-palette",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ recent: state.recent }),
    },
  ),
);
