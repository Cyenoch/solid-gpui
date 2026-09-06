import { expect, test } from "bun:test";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { declarationSurface } from "./api-surface";

test("native compiler resolves re-exports and merged type/value declarations", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-api-surface-"));
  try {
    await writeFile(
      join(directory, "types.d.ts"),
      `
      export interface Options { enabled: boolean }
      export declare class Widget { value: string }
      export declare function run(options: Options): Widget;
    `,
    );
    const entry = join(directory, "index.d.ts");
    await writeFile(
      entry,
      `
      export type { Options } from "./types";
      export { Widget as Control, run } from "./types";
    `,
    );
    expect(await declarationSurface(entry)).toEqual([
      { name: "Control", kind: "both" },
      { name: "Options", kind: "type" },
      { name: "run", kind: "value" },
    ]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
