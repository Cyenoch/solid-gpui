import { expect, test } from "bun:test";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { FrameDecoder } from "../packages/solid-gpui/src/protocol";
import { Envelope, type Snapshot } from "../packages/solid-gpui/src/protocol/generated/protocol";

const repo = resolve(import.meta.dir, "..");

test("Vite on Bun replaces TSX dependencies, recovers errors, preserves state and binary stdio", async () => {
  await mkdir(join(repo, ".scratch"), { recursive: true });
  const directory = await mkdtemp(join(repo, ".scratch/hot-reload-test-"));
  const dependency = join(directory, "view.tsx");
  const entry = join(directory, "app.tsx");
  const view = (label: string) => `import { Text } from '@solid-gpui/core';
export function Demo(props: { generation: number }) { return <Text>${label}:{props.generation}</Text>; }
`;
  await writeFile(dependency, view("v1"));
  await writeFile(
    entry,
    `import { mountApplication } from '@solid-gpui/core';
import { onCleanup } from '@solid-gpui/core/runtime';
import { Demo } from './view';
mountApplication<number>({ hotKey: import.meta.url, setup(previous = 0) {
  const generation = previous + 1;
  onCleanup(() => console.error('disposed-generation:' + generation));
  return { render: () => <Demo generation={generation} />, captureState: () => generation };
}});
`,
  );
  const config = join(directory, "vite.config.ts");
  await writeFile(
    config,
    `import { solidGpui } from '${repo}/packages/solid-gpui/src/vite/index.ts';
export default { plugins: [solidGpui({entry: ${JSON.stringify(entry)}})], resolve: { alias: [
{find: '@solid-gpui/core/runtime', replacement: '${repo}/packages/solid-gpui/src/runtime.ts'},
{find: '@solid-gpui/core', replacement: '${repo}/packages/solid-gpui/src/index.ts'}
] } };`,
  );
  const child = Bun.spawn(
    ["bun", "run", "--conditions=browser", join(repo, "packages/solid-gpui/src/vite/dev.ts"), entry, config],
    {
      cwd: repo,
      env: process.env,
      stdin: "pipe",
      stdout: "pipe",
      stderr: "pipe",
    },
  );
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
  try {
    await until(() => snapshots.length === 1);
    expect(text()).toBe("v1:1");
    await writeFile(dependency, view("v2"));
    await until(() => snapshots.length === 2);
    expect(text()).toBe("v2:2");
    expect(diagnostics).toContain("disposed-generation:1");
    await writeFile(dependency, "export const syntax = ;");
    await until(() => /error/i.test(diagnostics));
    expect(snapshots).toHaveLength(2);
    await writeFile(dependency, "export function Demo() { throw new Error('hot-render-failure'); }");
    await until(() => diagnostics.includes("hot-render-failure"));
    expect(snapshots).toHaveLength(2);
    await writeFile(dependency, view("v3"));
    await until(() => snapshots.length === 3);
    expect(text()).toBe("v3:3");
    expect(snapshots.map((snapshot) => [snapshot.surfaceId, snapshot.epoch, snapshot.baseRevision])).toEqual([
      [1, 1, 0],
      [1, 2, 0],
      [1, 3, 0],
    ]);
    expect(child.exitCode).toBeNull();
    expect(outputFailure).toBeUndefined();
  } finally {
    child.kill();
    await child.exited;
    await Promise.all([stdout, stderr]);
    await rm(directory, { recursive: true, force: true });
  }
}, 45000);
