import * as ts from "typescript";
import { describe, expect, it } from "bun:test";
import * as dev from "./index";

type SurfaceKind = "value" | "type" | "both";
type SurfaceExport = { readonly name: string; readonly kind: SurfaceKind };

function sourceSurface(path: string): SurfaceExport[] {
  const program = ts.createProgram([path], {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    jsx: ts.JsxEmit.ReactJSX,
    skipLibCheck: true,
    noEmit: true,
  });
  const source = program.getSourceFile(path);
  if (source === undefined) throw new Error(`missing source entry ${path}`);
  const checker = program.getTypeChecker();
  const moduleSymbol = checker.getSymbolAtLocation(source);
  if (moduleSymbol === undefined) throw new Error(`missing module symbol ${path}`);
  return checker
    .getExportsOfModule(moduleSymbol)
    .map((symbol) => {
      const resolved = symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
      const hasValue = (resolved.flags & ts.SymbolFlags.Value) !== 0;
      const hasType = (resolved.flags & ts.SymbolFlags.Type) !== 0;
      const kind: SurfaceKind = hasValue && hasType ? "both" : hasValue ? "value" : "type";
      return { name: symbol.name, kind };
    })
    .sort((left, right) => left.name.localeCompare(right.name));
}

async function fixtureSurface(): Promise<SurfaceExport[]> {
  const lines = (await Bun.file(`${import.meta.dir}/../../../fixtures/api-surface.dev.txt`).text())
    .split(/\r?\n/)
    .filter((line) => line.length > 0 && !line.startsWith("#"));
  return lines.map((line) => {
    const [name, kind] = line.split("\t");
    return { name, kind: kind as SurfaceKind };
  });
}

describe("dev public API surface", () => {
  it("locks value exports and source type/value names without locking structures", async () => {
    const expected = await fixtureSurface();
    const actual = sourceSurface(`${import.meta.dir}/index.ts`);
    expect(actual, "API surface changed; run make api-surface-generate after review").toEqual(expected);
    expect(
      Object.keys(dev).sort((left, right) => left.localeCompare(right)),
      "runtime value exports changed; run make api-surface-generate after review",
    ).toEqual(expected.filter((item) => item.kind !== "type").map((item) => item.name));
  });
});
