import remapping from "@jridgewell/remapping";
import { transform } from "@solidjs/compiler";
import { transformSync } from "oxc-transform";

/** Compile Solid's universal JSX once for both Vite and Bun. */
export function transformJsx(source: string, filename: string): { code: string; map: string } {
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
