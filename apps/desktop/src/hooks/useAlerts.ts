import { useQuery } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";

export function alertsQueryKey(projectPath: string) {
  return ["alerts", projectPath] as const;
}

export function useAlerts(projectPath: string) {
  return useQuery({
    queryKey: alertsQueryKey(projectPath),
    queryFn: async () => {
      const result = await commands.listAlerts(projectPath);
      if (result.status === "error") {
        throw new Error(result.error);
      }
      return result.data;
    },
  });
}
