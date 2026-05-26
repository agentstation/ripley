import type { Page } from "@playwright/test";

import type { RouteId } from "./routes";

type StubBuilder = () => unknown;

const STUB_BUILDERS: Record<RouteId, StubBuilder | null> = {
  alerts: () => [
    {
      id: "alert-1",
      advisory_id: "GHSA-test-1",
      ecosystem: "npm",
      package: "left-pad",
      version: "1.0.0",
      severity: "critical",
      summary: "Critical advisory in left-pad",
      references: ["https://example.com/advisory"],
      project_path: ".",
    },
    {
      id: "alert-2",
      advisory_id: "GHSA-test-2",
      ecosystem: "cargo",
      package: "serde",
      version: "1.2.3",
      severity: "high",
      summary: "High advisory in serde",
      references: [],
      project_path: ".",
    },
    {
      id: "alert-3",
      advisory_id: "GHSA-test-3",
      ecosystem: "pypi",
      package: "requests",
      version: "2.0.0",
      severity: "medium",
      summary: "Medium advisory in requests",
      references: [],
      project_path: ".",
    },
  ],
  "guard-log": () => ({
    total: 2,
    entries: [
      {
        timestamp: "2026-05-26T12:00:00Z",
        package: "scripty",
        version: "1.0.0",
        script: "preinstall",
        action: "blocked",
        risk_level: "high",
        matched_rules: ["NET001"],
        source: "npm",
        user_decision: false,
      },
      {
        timestamp: "2026-05-26T12:05:00Z",
        package: "benign",
        version: "2.0.0",
        script: "postinstall",
        action: "allowed",
        risk_level: "low",
        matched_rules: [],
        source: "npm",
        user_decision: true,
      },
    ],
  }),
  monitor: () => [
    {
      timestamp: "2026-05-26T12:00:00Z",
      event_type: "process.spawn",
      severity: "high",
      process: "/usr/bin/curl",
      pid: 1234,
      reason: "outbound network",
      detail: "curl https://example.com",
      action: "logged",
    },
    {
      timestamp: "2026-05-26T12:01:00Z",
      event_type: "fs.write",
      severity: "low",
      process: "/usr/bin/node",
      pid: 1235,
      reason: "lockfile updated",
      detail: "package-lock.json",
      action: "logged",
    },
  ],
  "deep-scan": () => ({
    project_path: ".",
    ioc_count: 1,
    persistence_count: 0,
    credential_count: 0,
    dead_man_switch_warning: null,
    findings: [
      {
        kind: "ioc",
        path: "node_modules/evil/index.js",
        description: "Matched IOC profile",
        severity: "critical",
        profile_id: "ioc.test",
        rotation_command: null,
      },
    ],
  }),
  audit: () => ({
    timestamp: "2026-05-26T12:00:00Z",
    categories: [
      {
        category: "Disk encryption",
        overall: "ok",
        findings: [{ name: "FileVault", status: "ok", detail: "Enabled.", fix_command: null }],
      },
      {
        category: "Firewall",
        overall: "warn",
        findings: [
          {
            name: "Application firewall",
            status: "warn",
            detail: "Not enforcing stealth mode.",
            fix_command:
              "sudo defaults write /Library/Preferences/com.apple.alf stealthenabled -bool true",
          },
        ],
      },
    ],
  }),
  posture: () => ({
    timestamp: "2026-05-26T12:00:00Z",
    detected_pms: [
      { name: "npm", version: "10.0.0", lockfile_path: "package-lock.json", binary: true },
    ],
    categories: [
      {
        category: "Lockfile discipline",
        overall: "ok",
        findings: [
          {
            name: "Lockfile present",
            status: "ok",
            detail: "package-lock.json found",
            fix_command: null,
            pm: "npm",
          },
        ],
      },
    ],
  }),
  settings: () => ({
    poll_interval_secs: 600,
    launch_at_login: false,
    monitor_enabled: true,
    monitor_watch_processes: true,
    monitor_watch_persistence: true,
    monitor_watch_lockfiles: true,
    guard_mode: "audit",
    guard_timeout_secs: 60,
    guard_sandbox: false,
    posture_strict: false,
    posture_require_lockfile: false,
  }),
};

const QUERY_KEYS: Record<RouteId, unknown[] | null> = {
  alerts: ["alerts", "."],
  "guard-log": ["guard-log", 0, 50],
  monitor: ["monitor", 100],
  "deep-scan": ["deep-scan", "."],
  audit: ["audit"],
  posture: ["harden", "."],
  settings: ["settings"],
};

export async function primeRoute(page: Page, route: RouteId): Promise<void> {
  const builder = STUB_BUILDERS[route];
  const key = QUERY_KEYS[route];
  if (!builder || !key) return;
  const data = builder();
  await page.waitForFunction(
    () =>
      (window as unknown as { __ripleyQueryClient?: unknown }).__ripleyQueryClient !== undefined,
  );
  await page.evaluate(
    ({ key, data }) => {
      const client = (
        window as unknown as {
          __ripleyQueryClient: { setQueryData: (k: unknown[], v: unknown) => void };
        }
      ).__ripleyQueryClient;
      client.setQueryData(key, data);
    },
    { key, data },
  );
}

export async function gotoRoute(page: Page, route: RouteId): Promise<void> {
  await page.goto("/");
  await primeRoute(page, route);
  await page.evaluate((id) => {
    window.location.hash = `/${id}`;
  }, route);
}
