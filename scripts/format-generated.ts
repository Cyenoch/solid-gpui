import { readFile } from "node:fs/promises";
import { format } from "prettier/standalone";
import * as babel from "prettier/plugins/babel";
import * as estree from "prettier/plugins/estree";
import * as typescript from "prettier/plugins/typescript";
import type { Options } from "prettier";

// Codegen is also imported by tests using Solid's browser condition. Explicit
// standalone plugins work there as well as in the normal Bun task process.
const config: Options = JSON.parse(await readFile(new URL("../.prettierrc.json", import.meta.url), "utf8"));

export function formatGenerated(source: string, parser: "json" | "typescript"): Promise<string> {
  return format(source, { ...config, parser, plugins: [babel, estree, typescript] });
}
