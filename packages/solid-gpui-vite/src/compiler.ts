import remapping from "@jridgewell/remapping";
import { transform } from "@solidjs/compiler";
import { transformSync } from "oxc-transform";

/** JavaScript module and a JSON source map pointing to the authored JSX/TSX. */
export interface CompileResult {
  readonly code: string;
  readonly map: string;
}

/**
 * Compile one .jsx or .tsx module to the Solid GPUI universal runtime.
 * Does not bundle imports, resolve native bindings, or start a host.
 * Throws for unsupported filenames or invalid source.
 */
export function compile(source: string, filename: string): CompileResult {
  if (!/\.[jt]sx$/.test(filename)) {
    throw new Error(`solid-gpui: compiler requires a .jsx or .tsx filename: ${filename}`);
  }
  const jsx = transform(source, {
    filename,
    generate: "universal",
    moduleName: "@solid-gpui/core/runtime",
    // Control-flow components belong to the application's Solid runtime. The
    // compiler's Solid 2 auto-import defaults are not part of our host ABI.
    builtIns: [],
    sourceMap: true,
  });
  if (!jsx.map) throw new Error(`solid-gpui: JSX compiler produced no source map: ${filename}`);
  if (!filename.endsWith(".tsx")) return { code: jsx.code, map: jsx.map };

  // The Solid compiler preserves TypeScript. Erase explicit types while keeping
  // runtime imports and JavaScript class fields, then map back through both passes.
  const result = transformSync(filename, jsx.code, {
    jsx: "preserve",
    typescript: { onlyRemoveTypeImports: true },
    sourcemap: true,
  });
  if (result.errors.length) {
    throw new Error(
      `solid-gpui: TypeScript transform failed: ${filename}\n${result.errors.map((error) => error.codeframe ?? error.message).join("\n")}`,
    );
  }
  if (!result.map) throw new Error(`solid-gpui: TypeScript compiler produced no source map: ${filename}`);
  return { code: result.code, map: remapping([JSON.stringify(result.map), jsx.map], () => null).toString() };
}
