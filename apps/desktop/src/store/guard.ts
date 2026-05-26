import { create } from "zustand";

import { commands, type GuardEvent, type Decision } from "@/lib/bindings";

type GuardState = {
  currentEvent: GuardEvent | null;
  inFlight: boolean;
  setEvent: (event: GuardEvent | null) => void;
  respond: (decision: Decision) => Promise<void>;
};

export const useGuardStore = create<GuardState>((set, get) => ({
  currentEvent: null,
  inFlight: false,
  setEvent: (event) => set({ currentEvent: event }),
  respond: async (decision) => {
    const event = get().currentEvent;
    if (!event) return;
    set({ inFlight: true });
    try {
      await commands.submitGuardDecision(event.id, decision);
      set({ currentEvent: null });
    } finally {
      set({ inFlight: false });
    }
  },
}));
