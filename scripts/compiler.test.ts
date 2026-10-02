import { expect, test } from "bun:test";
import { mkdtemp, rm } from "node:fs/promises";
import { join, resolve } from "node:path";
import remapping from "@jridgewell/remapping";
import { compile } from "@solid-gpui/vite/compiler";
import { solidGpui } from "@solid-gpui/vite";

const repo = resolve(import.meta.dir, "..");

test("compiler-only TSX diagnostics map through both transforms to authored code", () => {
  const filename = "/application/Example.tsx";
  const source = `import type { Unused } from "./absent";
class Fields { declare erased: Unused; retained?: string; }
const View = () => null;
const element = <View />;
throw new Error("source position");
`;
  const result = compile(source, filename);
  const lines = result.code.split("\n");
  const line = lines.findIndex((value) => value.includes('new Error("source position")'));
  const column = lines[line]!.indexOf("new Error");
  const location = remapping(
    [{ version: 3, names: [], sources: [filename], mappings: [[[0, 0, line, column]]] }, result.map],
    () => null,
    { decodedMappings: true },
  );
  expect(location.mappings).toEqual([[[0, 0, 4, 6]]]);
  expect(location.sources).toEqual([filename]);
  expect(location.sourcesContent).toEqual([source]);
});

test("Vite uses exactly the public compiler for JSX and TSX, including source maps", () => {
  const plugin = solidGpui({ target: "web" });
  if (typeof plugin.transform !== "function") throw new Error("Vite transform hook is missing");
  for (const filename of ["/application/View.jsx", "/application/View.tsx"]) {
    const source = `const View = () => null; export const content = <View />;`;
    expect(Reflect.apply(plugin.transform, {}, [source, `${filename}?v=1`])).toEqual(compile(source, filename));
  }
  expect(() => compile("const value = <View>;", "/application/Broken.tsx")).toThrow();
  expect(() => compile("const value = 1;", "/application/View.ts")).toThrow(".jsx or .tsx");
});

test("compiler-only output runs reactive control flow and the public ref ABI", async () => {
  const directory = await mkdtemp(join(repo, "packages/solid-gpui-vite/fixtures/.compiler-test-"));
  try {
    for (const fixture of ["compiler-app.tsx", "jsx-refs.tsx"]) {
      const filename = join(repo, "packages/solid-gpui-vite/fixtures", fixture);
      const result = compile(await Bun.file(filename).text(), filename);
      if (fixture === "compiler-app.tsx") expect(result.code).toContain('from "node:path"');
      const output = join(directory, fixture.replace(/\.tsx$/, ".js"));
      await Bun.write(output, result.code);
      const child = Bun.spawn(["bun", "--conditions=browser", "--conditions=solid-gpui-source", output], {
        stdout: "pipe",
        stderr: "pipe",
      });
      const [status, stderr] = await Promise.all([child.exited, new Response(child.stderr).text()]);
      expect({ status, stderr }).toEqual({ status: 0, stderr: "" });
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
