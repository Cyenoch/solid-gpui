import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { API, SymbolFlags } from "typescript/unstable/async";

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
    name: "router-generator",
    dts: "packages/solid-gpui-router/dist/generator.d.ts",
    runtime: "packages/solid-gpui-router/dist/generator.js",
    output: "fixtures/api-surface.router-generator.txt",
  },
  {
    name: "router-vite",
    dts: "packages/solid-gpui-router/dist/vite.d.ts",
    runtime: "packages/solid-gpui-router/dist/vite.js",
    output: "fixtures/api-surface.router-vite.txt",
  },
  {
    name: "shiki",
    dts: "packages/solid-gpui-shiki/dist/index.d.ts",
    runtime: "packages/solid-gpui-shiki/dist/index.js",
    output: "fixtures/api-surface.shiki.txt",
  },
  {
    name: "shiki-bun",
    dts: "packages/solid-gpui-shiki/dist/bun.d.ts",
    runtime: "packages/solid-gpui-shiki/dist/bun.js",
    output: "fixtures/api-surface.shiki-bun.txt",
  },
  {
    name: "core",
    dts: "packages/solid-gpui/dist/index.d.ts",
    runtime: "packages/solid-gpui/dist/index.js",
    output: "fixtures/api-surface.core.txt",
  },
  {
    name: "router",
    dts: "packages/solid-gpui-router/dist/index.d.ts",
    runtime: "packages/solid-gpui-router/dist/index.js",
    output: "fixtures/api-surface.router.txt",
  },
];

export async function declarationSurface(dtsPath: string): Promise<SurfaceExport[]> {
  const absolutePath = resolve(dtsPath);
  const configPath = join(dirname(absolutePath), ".api-surface.tsconfig.json");
  const config = JSON.stringify({
    compilerOptions: {
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "Bundler",
      jsx: "preserve",
      skipLibCheck: true,
      noEmit: true,
    },
    files: [absolutePath],
  });
  const api = new API({
    fs: {
      fileExists: (path) => (path === configPath ? true : undefined),
      readFile: (path) => (path === configPath ? config : undefined),
    },
  });
  try {
    const snapshot = await api.updateSnapshot({ openProjects: [configPath] });
    const project = snapshot.getProject(configPath);
    if (!project) throw new Error(`missing declaration project: ${absolutePath}`);
    const source = await project.program.getSourceFile(absolutePath);
    if (!source) throw new Error(`missing declaration entry: ${absolutePath}`);
    const checker = project.checker;
    const moduleSymbol = await checker.getSymbolAtLocation(source);
    if (!moduleSymbol) throw new Error(`declaration entry has no module symbol: ${absolutePath}`);
    const symbols = await checker.getExportsOfModule(moduleSymbol);
    const exports = await Promise.all(
      symbols.map(async (symbol) => {
        const resolved = symbol.flags & SymbolFlags.Alias ? await checker.getAliasedSymbol(symbol) : symbol;
        const hasValue = (resolved.flags & SymbolFlags.Value) !== 0;
        const hasType = (resolved.flags & SymbolFlags.Type) !== 0;
        const kind: SurfaceKind = hasValue && hasType ? "both" : hasValue ? "value" : "type";
        return { name: symbol.name, kind };
      }),
    );
    return exports.sort((left, right) => left.name.localeCompare(right.name));
  } finally {
    await api.close();
  }
}

async function runtimeNames(path: string): Promise<string[]> {
  const module = (await import(pathToFileURL(resolve(path)).href)) as Record<string, unknown>;
  return Object.keys(module).sort((left, right) => left.localeCompare(right));
}

export function renderSnapshot(exports: readonly SurfaceExport[]): string {
  return ["# api-surface-v1", "# name\tkind", ...exports.map(({ name, kind }) => `${name}\t${kind}`), ""].join("\n");
}

async function generate(spec: PackageSpec): Promise<void> {
  const declarations = await declarationSurface(spec.dts);
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
