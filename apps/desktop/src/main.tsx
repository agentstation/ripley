import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClientProvider, type QueryClient } from "@tanstack/react-query";

import App, { navigateTo } from "./App";
import { createQueryClient } from "./lib/query";
import { useGuardStore } from "./store/guard";
import "./styles/theme.css";

const queryClient = createQueryClient();

if (import.meta.env.DEV) {
  const handles = window as unknown as {
    __ripleyQueryClient?: QueryClient;
    __ripleyGuardStore?: typeof useGuardStore;
    __ripleyNavigate?: (target: string) => void;
  };
  handles.__ripleyQueryClient = queryClient;
  handles.__ripleyGuardStore = useGuardStore;
  handles.__ripleyNavigate = navigateTo;
}

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Root element #root not found in index.html");
}

createRoot(rootElement).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </StrictMode>,
);
