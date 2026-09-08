import * as fs from "node:fs/promises";
import { resolve } from "node:path";
import { Generator, getConfig, type Config } from "@tanstack/router-generator";
import { parseAst } from "@tanstack/router-utils";

export interface FileRouterOptions extends Partial<
  Pick<
    Config,
    | "routesDirectory"
    | "generatedRouteTree"
    | "routeFilePrefix"
    | "routeFileIgnorePrefix"
    | "routeFileIgnorePattern"
    | "quoteStyle"
    | "semicolons"
  >
> {
  /** Project root; defaults to the current directory, or Vite's resolved root. */
  root?: string;
  indexToken?: string;
  routeToken?: string;
}

const nativeModule = "@solid-gpui/router";
const solidModule = "@tanstack/solid-router";

// TanStack's transformer currently fixes imports to its framework package. Adapt
// module specifiers at its filesystem boundary, keeping user code native on disk.
function mapModule(source: string, filename: string, from: string, to: string): string {
  if (!source.includes(from)) return source;
  const ast = parseAst({ code: source, filename });
  const edits: { start: number; end: number }[] = [];
  for (const statement of ast.program.body) {
    const literal =
      statement.type === "ImportDeclaration" ||
      statement.type === "ExportNamedDeclaration" ||
      statement.type === "ExportAllDeclaration"
        ? statement.source
        : statement.type === "TSModuleDeclaration"
          ? statement.id
          : undefined;
    if (literal?.type === "StringLiteral" && literal.value === from && literal.start != null && literal.end != null) {
      edits.push({ start: literal.start + 1, end: literal.end - 1 });
    }
  }
  for (const edit of edits.reverse()) source = source.slice(0, edit.start) + to + source.slice(edit.end);
  return source;
}

/** @internal Shared generation session for Vite's initial scan and file events. */
export function createRouteGenerator(options: FileRouterOptions = {}): Generator {
  const root = resolve(options.root ?? process.cwd());
  const { root: _root, ...config } = options;
  const generator = new Generator({
    root,
    config: getConfig(
      {
        ...config,
        target: "solid",
        autoCodeSplitting: false,
        disableTypes: false,
        quoteStyle: options.quoteStyle ?? "double",
        semicolons: options.semicolons ?? true,
        tmpDir: resolve(root, "node_modules/.cache/solid-gpui-router"),
      },
      root,
    ),
    fs: {
      stat: async (path) => {
        const stat = await fs.stat(path, { bigint: true });
        return { mtimeMs: stat.mtimeMs, mode: Number(stat.mode), uid: Number(stat.uid), gid: Number(stat.gid) };
      },
      readFile: async (path) => {
        let handle;
        try {
          handle = await fs.open(path, "r");
          const stat = await handle.stat({ bigint: true });
          const source = await handle.readFile("utf8");
          return { stat, fileContent: mapModule(source, path, nativeModule, solidModule) };
        } catch (error) {
          if (error && typeof error === "object" && "code" in error && error.code === "ENOENT")
            return "file-not-existing";
          throw error;
        } finally {
          await handle?.close();
        }
      },
      writeFile: (path, source) => fs.writeFile(path, mapModule(source, path, solidModule, nativeModule)),
      rename: fs.rename,
      chmod: fs.chmod,
      chown: fs.chown,
    },
  });
  generator.targetTemplate.rootRoute.template = () =>
    "%%tsrImports%%\n%%tsrExportStart%%{ component: Outlet }%%tsrExportEnd%%\n";
  generator.targetTemplate.route.template = () => "%%tsrImports%%\n%%tsrExportStart%%{}%%tsrExportEnd%%\n";
  generator.targetTemplate.lazyRoute.template = () => "%%tsrImports%%\n%%tsrExportStart%%{}%%tsrExportEnd%%\n";
  return generator;
}
