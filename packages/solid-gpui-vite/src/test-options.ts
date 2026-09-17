/** Options accepted by {@link runTests}. */
export interface RunTestsOptions {
  /** Project root holding the application's Vite config; defaults to the current working directory. */
  readonly root?: string;
  /** Vite config file passed to Vite; omitted lets Vite discover it inside `root`. */
  readonly configFile?: string;
  /** Arguments forwarded verbatim to `bun test`, such as file filters and flags. */
  readonly args?: readonly string[];
}

/** What the parent-side Vite pipeline is started with. */
export interface TestPipelineOptions {
  readonly root: string;
  readonly configFile?: string;
  /** Node-style resolution conditions applied to Solid and to the spawned test process. */
  readonly conditions: readonly string[];
}

/** Environment handoff from the runner to the preloaded loader. */
export const TEST_ENV = {
  root: "SOLID_GPUI_TEST_ROOT",
  endpoint: "SOLID_GPUI_TEST_ENDPOINT",
  token: "SOLID_GPUI_TEST_TOKEN",
} as const;

export interface TestLoadRequest {
  /** Vite URL or absolute path, exactly as it reached Bun's loader. */
  readonly specifier: string;
}

/** `code` is absent when the module is not Vite-owned and Bun should load it itself. */
export interface TestLoadResponse {
  readonly code?: string;
}

/** Solid's client build is the only one a native renderer can observe. */
export const CLIENT_CONDITION = "browser";

const CONDITION_FLAG = /^--conditions=(.+)$/;

/** Conditions the caller already asked for, plus the client condition Solid needs. */
export function collectConditions(execArgv: readonly string[]): string[] {
  const conditions = [CLIENT_CONDITION];
  for (const argument of execArgv) {
    const match = CONDITION_FLAG.exec(argument);
    if (!match) continue;
    for (const listed of match[1]!.split(",")) {
      const condition = listed.trim();
      if (condition.length > 0 && !conditions.includes(condition)) conditions.push(condition);
    }
  }
  return conditions;
}
