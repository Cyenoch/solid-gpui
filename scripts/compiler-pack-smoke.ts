#!/usr/bin/env bun

import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const repo = resolve(import.meta.dir, "..");
const directory = await mkdtemp(join(tmpdir(), "solid-gpui-compiler-consumer-"));
async function run(args: string[], cwd = directory): Promise<void> {
  const child = Bun.spawn(args, { cwd, stdout: "inherit", stderr: "inherit" });
  if ((await child.exited) !== 0) throw new Error(`Compiler consumer failed: ${args.join(" ")}`);
}

try {
  for (const [name, path] of [
    ["vite", "solid-gpui-vite"],
    ["core", "solid-gpui"],
  ]) {
    await run(
      ["bun", "pm", "pack", "--filename", join(directory, `${name}.tgz`), "--quiet"],
      join(repo, "packages", path!),
    );
  }
  const archive = new Bun.Archive(await Bun.file(join(directory, "vite.tgz")).bytes());
  const files = await archive.files();
  const manifest = JSON.parse(await files.get("package/package.json")!.text());
  for (const target of Object.values(manifest.exports["./compiler"])) {
    if (typeof target !== "string" || !files.has(`package/${target.replace(/^\.\//, "")}`))
      throw new Error(`Packed compiler export is missing: ${target}`);
  }
  await Bun.write(
    join(directory, "package.json"),
    JSON.stringify({
      private: true,
      type: "module",
      dependencies: {
        "@solid-gpui/vite": "file:./vite.tgz",
        "@solid-gpui/core": "file:./core.tgz",
        "solid-js": "1.9.15",
      },
      devDependencies: { "bun-types": "1.4.2" },
    }),
  );
  await run(["bun", "install", "--no-progress"]);
  if (await Bun.file(join(directory, "node_modules/vite/package.json")).exists())
    throw new Error("Compiler-only installation unexpectedly requires Vite");
  for (const name of ["compiler-app", "jsx-refs"]) {
    await Bun.write(
      join(directory, `${name}.txt`),
      Bun.file(join(repo, "packages/solid-gpui-vite/fixtures", `${name}.tsx`)),
    );
  }
  await Bun.write(
    join(directory, "compiler-smoke.ts"),
    `import { compile, type CompileResult } from "@solid-gpui/vite/compiler";
for (const name of ["compiler-app", "jsx-refs"]) {
  const filename = name + ".tsx";
  const source = await Bun.file(name + ".txt").text();
  const result: CompileResult = compile(source, filename);
  const map = JSON.parse(result.map);
  if (map.sources[0] !== filename || map.sourcesContent[0] !== source || !map.mappings)
    throw new Error("Packed compiler lost authored source maps");
  await Bun.write(name + ".js", result.code);
  await import("./" + name + ".js");
}
`,
  );
  await run(["bun", "--conditions=browser", "compiler-smoke.ts"]);
  await run(["bun", "--conditions=browser", "--conditions=solid-gpui-source", "compiler-smoke.ts"]);
  await Bun.write(
    join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "Bundler",
        types: ["bun-types"],
      },
      include: ["compiler-smoke.ts"],
    }),
  );
  // Use installed consumer declarations; only the TypeScript executable comes from the workspace.
  await run([join(repo, "node_modules/.bin/tsc"), "--project", "tsconfig.json"]);
  console.log("Packed compiler exports, declarations, source resolution, reactivity and refs passed without Vite");
} finally {
  await rm(directory, { recursive: true, force: true });
}
