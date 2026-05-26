import { spawn, spawnSync, type ChildProcessWithoutNullStreams } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Options } from "@wdio/types";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const isWindows = os.platform() === "win32";

const workspaceRoot = path.resolve(__dirname, "..", "..");

const application = path.resolve(
  workspaceRoot,
  "target",
  "release",
  isWindows ? "ripley-desktop.exe" : "ripley-desktop",
);

const tauriDriverBin = path.resolve(
  os.homedir(),
  ".cargo",
  "bin",
  isWindows ? "tauri-driver.exe" : "tauri-driver",
);

let tauriDriver: ChildProcessWithoutNullStreams | undefined;

export const config: Options.Testrunner = {
  runner: "local",
  tsConfigPath: path.resolve(__dirname, "tests", "e2e", "tsconfig.json"),
  specs: [path.resolve(__dirname, "tests", "e2e", "**", "*.spec.ts")],
  exclude: [],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application },
      browserName: "wry",
    } as WebdriverIO.Capabilities,
  ],
  logLevel: "info",
  hostname: "127.0.0.1",
  port: 4444,
  bail: 0,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 120_000,
  connectionRetryCount: 3,
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: { ui: "bdd", timeout: 60_000 },

  onPrepare() {
    if (process.env.RIPLEY_SKIP_BUILD) return;
    const result = spawnSync(
      "cargo",
      ["build", "-p", "ripley-desktop", "--release"],
      { cwd: __dirname, stdio: "inherit" },
    );
    if (result.status !== 0) {
      throw new Error(`cargo build failed with exit ${result.status}`);
    }
  },

  beforeSession() {
    const args: string[] = [];
    if (process.env.RIPLEY_TAURI_DRIVER_DEBUG) {
      args.push("--debug");
    }
    if (process.env.RIPLEY_NATIVE_DRIVER) {
      args.push("--native-driver", process.env.RIPLEY_NATIVE_DRIVER);
    }
    tauriDriver = spawn(tauriDriverBin, args, {
      stdio: ["ignore", process.stdout, process.stderr],
    });
  },

  afterSession() {
    tauriDriver?.kill();
    tauriDriver = undefined;
  },
};
