import { useMemo } from "react";

export type CommandGroup = "navigate" | "action" | "view";

export type PaletteCommand = {
  id: string;
  label: string;
  hint?: string;
  group: CommandGroup;
  keywords?: string[];
  run: () => void;
};

export type CommandRegistryOptions = {
  navigate?: (target: string) => void;
};

export function useCommands(options: CommandRegistryOptions = {}): PaletteCommand[] {
  const { navigate } = options;
  return useMemo(() => {
    const go = (target: string) => () => navigate?.(target);
    const commands: PaletteCommand[] = [
      {
        id: "nav.alerts",
        label: "Go to Alerts",
        hint: "View vulnerability matches",
        group: "navigate",
        keywords: ["vulnerabilities", "advisories", "cve"],
        run: go("alerts"),
      },
      {
        id: "nav.guard-log",
        label: "Go to Guard log",
        hint: "Audit script-shell decisions",
        group: "navigate",
        keywords: ["history", "decisions", "scripts"],
        run: go("guard-log"),
      },
      {
        id: "nav.monitor",
        label: "Go to Monitor",
        hint: "Live process activity",
        group: "navigate",
        keywords: ["events", "process", "daemon"],
        run: go("monitor"),
      },
      {
        id: "nav.deep-scan",
        label: "Go to Deep scan",
        hint: "Forensic findings",
        group: "navigate",
        keywords: ["forensic", "ioc", "persistence", "credentials"],
        run: go("deep-scan"),
      },
      {
        id: "nav.audit",
        label: "Go to Audit",
        hint: "Machine security posture",
        group: "navigate",
        keywords: ["machine", "filevault", "firewall"],
        run: go("audit"),
      },
      {
        id: "nav.posture",
        label: "Go to Posture",
        hint: "Package manager hardening",
        group: "navigate",
        keywords: ["harden", "lockfile", "npm", "cargo"],
        run: go("posture"),
      },
      {
        id: "nav.settings",
        label: "Go to Settings",
        hint: "Configure Ripley",
        group: "navigate",
        keywords: ["preferences", "config"],
        run: go("settings"),
      },
    ];
    return commands;
  }, [navigate]);
}
