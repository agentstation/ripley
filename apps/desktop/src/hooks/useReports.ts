import { useQuery } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";

export function useAuditReport() {
  return useQuery({
    queryKey: ["audit"],
    queryFn: async () => {
      const result = await commands.runAuditReport();
      if (result.status === "error") throw new Error(result.error);
      return result.data;
    },
  });
}

export function useHardenReport(projectPath: string) {
  return useQuery({
    queryKey: ["harden", projectPath],
    queryFn: async () => {
      const result = await commands.runHardenReport(projectPath);
      if (result.status === "error") throw new Error(result.error);
      return result.data;
    },
  });
}

export function useDeepScan(projectPath: string) {
  return useQuery({
    queryKey: ["deep-scan", projectPath],
    queryFn: async () => {
      const result = await commands.runDeepScan(projectPath);
      if (result.status === "error") throw new Error(result.error);
      return result.data;
    },
  });
}

export function useMonitorEvents(limit = 100) {
  return useQuery({
    queryKey: ["monitor", limit],
    queryFn: async () => {
      const result = await commands.listMonitorEvents(limit);
      if (result.status === "error") throw new Error(result.error);
      return result.data;
    },
  });
}
