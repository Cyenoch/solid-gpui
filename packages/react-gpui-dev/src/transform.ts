import { transformAsync } from "@babel/core";
import refreshBabel from "react-refresh/babel";
import jsxBabel from "@babel/plugin-transform-react-jsx";
import stableFamilyBabel from "./refresh-family-plugin.js";
import * as RefreshRuntimeModule from "react-refresh/runtime";

interface RefreshRuntime {
  injectIntoGlobalHook(global: typeof globalThis): void;
  register(type: unknown, id: string): void;
  createSignatureFunctionForTransform(...args: unknown[]): (...args: unknown[]) => unknown;
  performReactRefresh(): unknown;
}

interface FamilyRecord {
  current: (props: Record<string, unknown>) => unknown;
  wrapper: (props: Record<string, unknown>) => unknown;
  signature: string;
}
const runtime = (RefreshRuntimeModule.default ?? RefreshRuntimeModule) as unknown as RefreshRuntime;

const familyRecords = new Map<string, FamilyRecord>();
let installed = false;

declare global {
  var __reactGpuiFamily: (
    id: string,
    implementation: (props: Record<string, unknown>) => unknown,
    signature: string,
  ) => (props: Record<string, unknown>) => unknown;
}

function loaderFor(path: string): "js" | "jsx" | "ts" | "tsx" {
  if (path.endsWith(".tsx")) return "tsx";
  if (path.endsWith(".ts")) return "ts";
  if (path.endsWith(".jsx")) return "jsx";
  return "js";
}

export async function transformRefreshSource(source: string, path: string): Promise<string> {
  const refresh = await transformAsync(source, {
    filename: path,
    sourceType: "module",
    configFile: false,
    babelrc: false,
    parserOpts: { plugins: ["typescript", "jsx", "topLevelAwait"] },
    plugins: [
      [refreshBabel, { skipEnvCheck: true }],
      [jsxBabel, { runtime: "classic" }],
    ],
    sourceMaps: false,
  });
  if (refresh?.code === undefined) {
    throw new Error(`React Refresh transform produced no code for ${path}`);
  }
  const transformed = await transformAsync(refresh.code, {
    filename: path,
    sourceType: "module",
    configFile: false,
    babelrc: false,
    plugins: [stableFamilyBabel],
    sourceMaps: "inline",
  });
  if (transformed?.code === undefined) {
    throw new Error(`Stable family transform produced no code for ${path}`);
  }
  return transformed.code;
}

/** Install React Refresh hooks and transform only the requested source tree. */
export async function installFastRefreshTransform(sourceRoot = process.cwd()): Promise<void> {
  if (installed) return;
  runtime.injectIntoGlobalHook(globalThis);
  globalThis.$RefreshReg$ = (type: unknown, id: string) => runtime.register(type, id);
  globalThis.__reactGpuiFamily = (id, implementation, signature) => {
    const existing = familyRecords.get(id);
    if (existing !== undefined && existing.signature === signature) {
      existing.current = implementation;
      runtime.register(existing.wrapper, id);
      return existing.wrapper;
    }
    const record: FamilyRecord = {
      current: implementation,
      signature,
      wrapper: (props) => record.current(props),
    };
    Object.defineProperty(record.wrapper, "__reactGpuiStableFamily", { value: true });
    familyRecords.set(id, record);
    runtime.register(record.wrapper, id);
    return record.wrapper;
  };
  globalThis.$RefreshSig$ = runtime.createSignatureFunctionForTransform;
  globalThis.RefreshRuntime = runtime;

  await Bun.plugin({
    name: "react-gpui-fast-refresh",
    setup(build) {
      const normalizedRoot = sourceRoot.replace(/\/+$/, "");
      const escapedRoot = normalizedRoot.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const sourceFilter = new RegExp(`^${escapedRoot}/(?!node_modules/).*(?:[cm]?[jt]sx?)$`);
      build.onLoad({ filter: sourceFilter }, async ({ path }) => {
        const source = await Bun.file(path).text();
        const code = await transformRefreshSource(source, path);
        return { contents: code, loader: loaderFor(path) };
      });
    },
  });
  installed = true;
}

export function performReactRefresh(): void {
  runtime.performReactRefresh();
}
