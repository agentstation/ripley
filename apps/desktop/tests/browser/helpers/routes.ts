export type RouteId =
  | "alerts"
  | "guard-log"
  | "monitor"
  | "deep-scan"
  | "audit"
  | "posture"
  | "settings";

export type RouteDef = {
  id: RouteId;
  hash: string;
  testId: string;
  label: string;
};

export const ROUTES: readonly RouteDef[] = [
  { id: "alerts", hash: "#/alerts", testId: "alerts-view", label: "Alerts" },
  { id: "guard-log", hash: "#/guard-log", testId: "guard-log-view", label: "Guard log" },
  { id: "monitor", hash: "#/monitor", testId: "monitor-view", label: "Monitor" },
  { id: "deep-scan", hash: "#/deep-scan", testId: "deep-scan-view", label: "Deep scan" },
  { id: "audit", hash: "#/audit", testId: "audit-view", label: "Audit" },
  { id: "posture", hash: "#/posture", testId: "posture-view", label: "Posture" },
  { id: "settings", hash: "#/settings", testId: "settings-view", label: "Settings" },
] as const;
