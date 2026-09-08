import { expect, test } from "bun:test";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import remapping from "@jridgewell/remapping";
import { FrameDecoder } from "../packages/solid-gpui/src/protocol";
import { Envelope, type Snapshot } from "../packages/solid-gpui/src/protocol/generated/protocol";
import { transformJsx } from "../packages/solid-gpui-vite/src/transform";

const repo = resolve(import.meta.dir, "..");

test("TSX diagnostics map through JSX and TypeScript lowering to the authored location", () => {
  const filename = join(repo, "Example.tsx");
  const source = `import type { Unused } from "./absent";
class Fields { declare erased: Unused; retained?: string; }
const View = () => null;
const element = <View />;
throw new Error("source position");
`;
  const result = transformJsx(source, filename);
  const lines = result.code.split("\n");
  const line = lines.findIndex((value) => value.includes('new Error("source position")'));
  const column = lines[line]!.indexOf("new Error");
  // Map one generated expression through the complete map, as a debugger would.
  const location = remapping(
    [{ version: 3, names: [], sources: [filename], mappings: [[[0, 0, line, column]]] }, result.map],
    () => null,
    { decodedMappings: true },
  );
  expect(location.mappings).toEqual([[[0, 0, 4, 6]]]);
  expect(location.sources).toEqual([filename]);
  expect(location.sourcesContent).toEqual([source]);
});

test("Vite preserves TSX semantics, Bun APIs, HMR recovery, binary stdio and process ownership", async () => {
  await mkdir(join(repo, ".scratch"), { recursive: true });
  const directory = await mkdtemp(join(repo, ".scratch/hot-reload-test-"));
  const dependency = join(directory, "view.tsx");
  const entry = join(directory, "app.tsx");
  const view = (label: string) => `import { Text } from '@solid-gpui/core';
import { registration } from './registration';
import type { FieldType } from './type-only-module';
class Fields { declare erased: FieldType; retained?: string; }
if (globalThis.__jsxRegistration !== 'ready' || Object.keys(new Fields()).join(',') !== 'retained') {
  throw new Error('TSX must preserve runtime imports and class fields while erasing explicit types');
}
export function Demo(props: { generation: number }) { return <Text>${label}:{props.generation}</Text>; }
`;
  await writeFile(
    join(directory, "registration.ts"),
    `import { file } from 'bun'; import { Database } from 'bun:sqlite';
const db = new Database(':memory:');
if (db.query('select 42 as value').get().value !== 42 || !await file(import.meta.filename).exists()) throw new Error('Bun APIs unavailable');
db.close(); console.log('Bun APIs available');
globalThis.__jsxRegistration = 'ready'; export const registration = true;\n`,
  );
  await writeFile(dependency, view("v1"));
  await writeFile(
    entry,
    `import { mountApplication } from '@solid-gpui/core';
import { StdioTransport } from '@solid-gpui/core/stdio';
import { onCleanup } from '@solid-gpui/core/runtime';
import { Demo } from './view';
console.dir({ message: 'structured diagnostic' });
console.table([{ runtime: 'Bun' }]);
mountApplication<number>({ hotKey: import.meta.url, transport: () => new StdioTransport(), setup(previous = 0) {
  const generation = previous + 1;
  onCleanup(() => console.error('disposed-generation:' + generation));
  return { render: () => <Demo generation={generation} />, captureState: () => generation };
}});
`,
  );
  const config = join(directory, "vite.config.ts");
  const host = join(directory, "host.ts");
  await writeFile(
    host,
    `const child = Bun.spawn(JSON.parse(process.env.SOLID_GPUI_VITE_RUNNER!), {
    stdin: 'pipe', stdout: 'inherit', stderr: 'inherit'
  });
  console.error('owned-processes:' + process.pid + ',' + child.pid);
  process.exitCode = await child.exited;`,
  );
  await writeFile(
    config,
    `import { solidGpui } from '@solid-gpui/vite';
export default { logLevel: 'error', plugins: [solidGpui({entry: ${JSON.stringify(entry)}, host: { command: 'bun', args: [${JSON.stringify(host)}] }})], resolve: { alias: [
{find: '@solid-gpui/core/runtime', replacement: '${repo}/packages/solid-gpui/src/runtime.ts'},
{find: '@solid-gpui/core/stdio', replacement: '${repo}/packages/solid-gpui/src/stdio.ts'},
{find: '@solid-gpui/core', replacement: '${repo}/packages/solid-gpui/src/index.ts'}
] } };`,
  );
  const preloadEntry = join(directory, "preload.tsx");
  await writeFile(preloadEntry, "import { Demo } from './view'; console.log(typeof Demo);\n");
  await writeFile(
    join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        target: "ES2022",
        useDefineForClassFields: true,
        verbatimModuleSyntax: true,
        paths: {
          "@solid-gpui/core": [join(repo, "packages/solid-gpui/src/index.ts")],
          "@solid-gpui/core/runtime": [join(repo, "packages/solid-gpui/src/runtime.ts")],
          "@solid-gpui/core/stdio": [join(repo, "packages/solid-gpui/src/stdio.ts")],
        },
      },
    }),
  );
  const preload = Bun.spawn(
    ["bun", "--conditions=browser", "--preload", join(repo, "scripts/solid-jsx.ts"), preloadEntry],
    { cwd: repo, stdout: "pipe", stderr: "pipe" },
  );
  const [preloadCode, preloadOutput, preloadErrors] = await Promise.all([
    preload.exited,
    new Response(preload.stdout).text(),
    new Response(preload.stderr).text(),
  ]);
  const child = Bun.spawn(["bun", "--bun", "vite", "--config", config], {
    cwd: repo,
    env: process.env,
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  });
  const snapshots: Snapshot[] = [];
  let outputFailure: unknown;
  let diagnostics = "";
  const stdout = (async () => {
    const decoder = new FrameDecoder();
    for await (const chunk of child.stdout) {
      for (const bytes of decoder.push(chunk)) {
        const message = Envelope.decode(bytes).body;
        if (message?.tag === 1) snapshots.push(message.value);
      }
    }
  })().catch((error) => {
    outputFailure = error;
  });
  const stderr = (async () => {
    for await (const chunk of child.stderr) diagnostics = (diagnostics + new TextDecoder().decode(chunk)).slice(-64000);
  })();
  const until = async (predicate: () => boolean): Promise<void> => {
    const deadline = Date.now() + 10000;
    while (!predicate()) {
      if (outputFailure) throw outputFailure;
      if (child.exitCode !== null || Date.now() > deadline) throw new Error(`hot reload failed: ${diagnostics}`);
      await Bun.sleep(20);
    }
  };
  const text = () =>
    snapshots
      .at(-1)
      ?.nodes?.map((node) => node.text ?? "")
      .join("");
  const completedUpdates = () => diagnostics.match(/^hot updated:/gm)?.length ?? 0;
  const ownedProcesses = () =>
    [...diagnostics.matchAll(/owned-processes:(\d+),(\d+)/g)].map((match) => [Number(match[1]), Number(match[2])]);
  const isAlive = (pid: number) => {
    try {
      process.kill(pid, 0);
      return true;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
      throw error;
    }
  };
  try {
    expect(preloadCode, preloadErrors).toBe(0);
    expect(preloadOutput.trim()).toBe("Bun APIs available\nfunction");
    await until(() => snapshots.length === 1);
    expect(text()).toBe("v1:1");
    await writeFile(dependency, view("v2"));
    // A snapshot or error can arrive before the current HMR update completes.
    await until(() => snapshots.length === 2 && completedUpdates() === 1);
    expect(text()).toBe("v2:2");
    expect(diagnostics).toContain("disposed-generation:1");
    await writeFile(dependency, "export const syntax = ;");
    await until(() => /error/i.test(diagnostics) && completedUpdates() === 2);
    expect(snapshots).toHaveLength(2);
    await writeFile(dependency, "export function Demo() { throw new Error('hot-render-failure'); }");
    await until(() => diagnostics.includes("hot-render-failure") && completedUpdates() === 3);
    expect(snapshots).toHaveLength(2);
    await writeFile(dependency, view("v3"));
    await until(() => snapshots.length === 3 && completedUpdates() === 4);
    expect(text()).toBe("v3:3");
    expect(snapshots.map((snapshot) => [snapshot.surfaceId, snapshot.epoch, snapshot.baseRevision])).toEqual([
      [1, 1, 0],
      [1, 2, 0],
      [1, 3, 0],
    ]);
    expect(child.exitCode).toBeNull();
    expect(outputFailure).toBeUndefined();
    await writeFile(
      config,
      (await Bun.file(config).text()).replace("logLevel: 'error'", "clearScreen: false, logLevel: 'error'"),
    );
    await until(() => snapshots.length === 4 && ownedProcesses().length === 2);
    expect(text()).toBe("v3:1");
    expect(ownedProcesses()[0]!.map(isAlive)).toEqual([false, false]);
  } finally {
    child.kill();
    await child.exited;
    await Promise.all([stdout, stderr]);
    await rm(directory, { recursive: true, force: true });
  }
  expect(ownedProcesses()).toHaveLength(2);
  expect(ownedProcesses().flat().some(isAlive)).toBe(false);
}, 45000);
