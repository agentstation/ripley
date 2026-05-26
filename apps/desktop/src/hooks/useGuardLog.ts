import { useQuery } from "@tanstack/react-query";

import { commands, type GuardLogPage } from "@/lib/bindings";

export function guardLogQueryKey(offset: number, limit: number) {
  return ["guard-log", offset, limit] as const;
}

export function useGuardLog(offset: number, limit: number) {
  return useQuery<GuardLogPage>({
    queryKey: guardLogQueryKey(offset, limit),
    queryFn: async () => {
      const result = await commands.listGuardLog(offset, limit);
      if (result.status === "error") {
        throw new Error(result.error);
      }
      return result.data;
    },
  });
}
