import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { commands, type SettingsDto } from "@/lib/bindings";

const QUERY_KEY = ["settings"] as const;

export function useSettings() {
  return useQuery({
    queryKey: QUERY_KEY,
    queryFn: async () => {
      const result = await commands.readSettings();
      if (result.status === "error") throw new Error(result.error);
      return result.data;
    },
  });
}

export function useSaveSettings() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: async (settings: SettingsDto) => {
      const result = await commands.writeSettings(settings);
      if (result.status === "error") throw new Error(result.error);
      return result.data;
    },
    onSuccess: (data) => {
      client.setQueryData(QUERY_KEY, data);
    },
  });
}
