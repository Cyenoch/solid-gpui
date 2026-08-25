import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import * as ts from "../packages/react-gpui/node_modules/typescript/lib/typescript.js";

export type SurfaceKind = "value" | "type" | "both";
export interface SurfaceExport {
  readonly name: string;
  readonly kind: SurfaceKind;
}

type PackageSpec = {
  readonly name: string;
  readonly dts: string;
  readonly runtime: string;
  readonly output: string;
};

const specs: readonly PackageSpec[] = [
  {
    name: "core",
    dts: "packages/react-gpui/dist/index.d.ts",
    runtime: "packages/react-gpui/dist/index.js",
    output: "fixtures/api-surface.core.txt",
  },
  {
    name: "dev",
    dts: "packages/react-gpui-dev/dist/index.d.ts",
    runtime: "packages/react-gpui-dev/dist/index.js",
    output: "fixtures/api-surface.dev.txt",
  },
];

function compilerOptions(): ts.CompilerOptions {
  return {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    jsx: ts.JsxEmit.ReactJSX,
    skipLibCheck: true,
    noEmit: true,
  };
}

export function declarationSurface(dtsPath: string): SurfaceExport[] {
  const absolutePath = resolve(dtsPath);
  const program = ts.createProgram([absolutePath], compilerOptions());
  const source = program.getSourceFile(absolutePath);
  if (source === undefined) throw new Error(`missing declaration entry: ${absolutePath}`);
  const checker = program.getTypeChecker();
  const moduleSymbol = checker.getSymbolAtLocation(source);
  if (moduleSymbol === undefined) throw new Error(`declaration entry has no module symbol: ${absolutePath}`);
  const symbols = checker.getExportsOfModule(moduleSymbol);
  return symbols
    .map((symbol) => {
      const resolved = symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
      const hasValue = (resolved.flags & ts.SymbolFlags.Value) !== 0;
      const hasType = (resolved.flags & ts.SymbolFlags.Type) !== 0;
      const kind: SurfaceKind = hasValue && hasType ? "both" : hasValue ? "value" : "type";
      return { name: symbol.name, kind };
    })
    .sort((left, right) => left.name.localeCompare(right.name));
}

async function runtimeNames(path: string): Promise<string[]> {
  const module = (await import(pathToFileURL(resolve(path)).href)) as Record<string, unknown>;
  return Object.keys(module).sort((left, right) => left.localeCompare(right));
}

export function renderSnapshot(exports: readonly SurfaceExport[]): string {
  return ["# api-surface-v1", "# name\tkind", ...exports.map(({ name, kind }) => `${name}\t${kind}`), ""].join("\n");
}

async function generate(spec: PackageSpec): Promise<void> {
  const declarations = declarationSurface(spec.dts);
  const values = await runtimeNames(spec.runtime);
  const declared = new Map(declarations.map((item) => [item.name, item.kind]));
  for (const name of values) {
    const kind = declared.get(name);
    if (kind === undefined || kind === "type") {
      throw new Error(`${spec.name}: runtime export ${name} is missing a value declaration`);
    }
  }
  await mkdir("fixtures", { recursive: true });
  await writeFile(spec.output, renderSnapshot(declarations), "utf8");
  const bytes = await readFile(spec.output, "utf8");
  console.log(`${spec.name}: ${declarations.length} exports -> ${spec.output} (${bytes.length} bytes)`);
}

if (import.meta.main) {
  for (const spec of specs) await generate(spec);
}
