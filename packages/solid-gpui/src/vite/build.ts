#!/usr/bin/env bun
import { isBuiltin } from "node:module";
import { basename, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { transformJsx } from "./transform";

export interface ApplicationBuildOptions {
  readonly entry: string;
  readonly outfile: string;
  readonly runtime: "bun" | "quickjs";
  readonly sourcemap?: "inline" | "none";
}

/** Bundle the same universal JSX for either runtime, including Solid's client graph. */
export async function buildApplication(options: ApplicationBuildOptions): Promise<void> {
  if (options.runtime !== "bun" && options.runtime !== "quickjs") {
    throw new TypeError("Application runtime must be bun or quickjs");
  }
  const entry = resolve(options.entry);
  const outfile = resolve(options.outfile);
  if (entry === outfile) throw new Error("The output must differ from the application entry");
  const embedded = options.runtime === "quickjs";
  const result = await Bun.build({
    throw: false,
    entrypoints: [embedded ? "solid-gpui:application" : entry],
    target: embedded ? "browser" : "bun",
    format: "esm",
    conditions: ["browser"],
    // Bun also uses NODE_ENV when choosing conditional package exports. A
    // production bundle must not retain Solid's development owner wrappers.
    define: { "process.env.NODE_ENV": '"production"' },
    packages: "bundle",
    splitting: false,
    sourcemap: options.sourcemap ?? "inline",
    naming: basename(outfile),
    ...(embedded ? { allowUnresolved: [] } : {}),
    plugins: [
      {
        name: "solid-gpui",
        setup(builder) {
          if (embedded) {
            builder.onResolve({ filter: /^solid-gpui:application$/ }, () => ({
              path: "application",
              namespace: "solid-gpui",
            }));
            // A separate entry initializes the explicit QuickJS platform before
            // application dependencies, without shifting authored source maps.
            builder.onLoad({ filter: /.*/, namespace: "solid-gpui" }, () => ({
              contents: `import ${JSON.stringify(fileURLToPath(new URL("./quickjs-platform.ts", import.meta.url)))};\nimport ${JSON.stringify(entry)};`,
              loader: "js",
            }));
            // Bun's browser target can stub Node builtins. A successful QuickJS
            // build must instead expose unsupported host services at build time.
            builder.onResolve({ filter: /.*/ }, ({ path }) => {
              if (isBuiltin(path) || path === "bun" || path.startsWith("bun:") || path.startsWith("node:")) {
                throw new Error(
                  `QuickJS cannot import ${JSON.stringify(path)}; expose host services through native commands`,
                );
              }
            });
          }
          builder.onLoad({ filter: /\.[jt]sx$/ }, async ({ path }) => {
            const { code, map } = transformJsx(await Bun.file(path).text(), path);
            return {
              contents: `${code}\n//# sourceMappingURL=data:application/json;base64,${Buffer.from(map).toString("base64")}`,
              loader: "js",
            };
          });
        },
      },
    ],
  });
  if (!result.success) {
    throw new AggregateError(result.logs, `Application bundle failed:\n${result.logs.map(String).join("\n")}`);
  }
  if (result.outputs.length !== 1 || result.outputs[0]!.kind !== "entry-point") {
    throw new Error("Application entries must bundle into one JavaScript module; load native assets through host APIs");
  }
  await Bun.write(outfile, result.outputs[0]!);
}

if (import.meta.main) {
  const { values, positionals } = parseArgs({
    args: Bun.argv.slice(2),
    options: { runtime: { type: "string" }, help: { type: "boolean", short: "h" } },
    allowPositionals: true,
    strict: true,
  });
  const usage = "Usage: solid-gpui-build --runtime <bun|quickjs> <entry.tsx> <output.js>";
  if (values.help) console.log(usage);
  else {
    if ((values.runtime !== "bun" && values.runtime !== "quickjs") || positionals.length !== 2) {
      throw new Error(usage);
    }
    await buildApplication({
      runtime: values.runtime,
      entry: positionals[0]!,
      outfile: positionals[1]!,
    });
  }
}
