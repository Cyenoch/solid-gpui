import { expect, test } from "bun:test";
import { once } from "node:events";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { PassThrough } from "node:stream";
import { StdioTransport, type ByteInput, type ByteOutput } from "../src/stdio";

// These probes exercise real cross-process lifetime: the renderer runs as its own
// Bun process, and the test ends the host that owns the renderer's stdio pipes.
// Pipe EOF, an orphan renderer, and process exit are platform behaviour, so no
// fake clock can stand in for them; the waits below poll for the observed event
// instead of guessing a duration.
const repoRoot = resolve(import.meta.dirname, "../../..");
const coreEntry = join(repoRoot, "packages/solid-gpui/src/index.ts");
const runtimeEntry = join(repoRoot, "packages/solid-gpui/src/runtime.ts");
const stdioEntry = join(repoRoot, "packages/solid-gpui/src/stdio.ts");

/** Renderer script: a real application over the host pipe, with a polling timer. */
function rendererSource(events: string, extraSetup: string): string {
  return `import { appendFileSync } from "node:fs";
import { mountApplication, Text } from ${JSON.stringify(coreEntry)};
import { createComponent } from ${JSON.stringify(runtimeEntry)};
import { StdioTransport } from ${JSON.stringify(stdioEntry)};

const record = (line) => appendFileSync(${JSON.stringify(events)}, line + "\\n");
process.on("exit", (code) => record("exit:" + code));
const transport = new StdioTransport();
transport.onTermination((error) => record("terminated:" + error.message.split("\\n")[0]));
record("start:" + process.pid);
mountApplication({
  transport: () => transport,
  setup() {
    setInterval(() => record("polling"), 50);
    const service = Bun.spawn([process.execPath, "-e", "setInterval(() => {}, 1000)"], {
      detached: true,
      stdin: "ignore",
      stdout: "ignore",
      stderr: "ignore",
    });
    record("service:" + service.pid);
${extraSetup}
    return { render: () => createComponent(Text, { children: "host lifetime probe" }) };
  },
});
record("mounted");
`;
}

/** Host script: owns the renderer's stdin/stdout pipes and then idles. */
function hostSource(renderer: string): string {
  return `const child = Bun.spawn([process.execPath, "--conditions=browser", ${JSON.stringify(renderer)}], {
  stdin: "pipe",
  stdout: "pipe",
  stderr: "inherit",
});
void (async () => {
  for await (const chunk of child.stdout) void chunk;
})();
setInterval(() => {}, 1000);
`;
}

async function readLog(path: string): Promise<string[]> {
  try {
    return (await readFile(path, "utf8")).split("\n").filter((line) => line.length > 0);
  } catch {
    return [];
  }
}

function isAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function pidOf(lines: string[], prefix: string): number {
  const line = lines.find((entry) => entry.startsWith(prefix));
  if (line === undefined) throw new Error(`missing ${prefix} in probe log: ${lines.join(", ")}`);
  const pid = Number(line.slice(prefix.length));
  if (!Number.isInteger(pid) || pid <= 0) throw new Error(`invalid pid in probe log: ${line}`);
  return pid;
}

async function startHost(extraSetup: string) {
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-stdio-"));
  const events = join(directory, "events.log");
  const renderer = join(directory, "renderer.js");
  const host = join(directory, "host.js");
  await writeFile(renderer, rendererSource(events, extraSetup));
  await writeFile(host, hostSource(renderer));
  const process_ = Bun.spawn([process.execPath, host], { stdin: "ignore", stdout: "pipe", stderr: "pipe" });
  let diagnostics = "";
  void (async () => {
    for await (const chunk of process_.stderr) diagnostics += new TextDecoder().decode(chunk);
  })();
  const waitFor = async (condition: () => Promise<boolean>, describe: string): Promise<void> => {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
      if (await condition()) return;
      await Bun.sleep(20);
    }
    const lines = await readLog(events);
    throw new Error(`timed out waiting for ${describe}\nprobe log:\n${lines.join("\n")}\nhost stderr:\n${diagnostics}`);
  };
  const stop = async (): Promise<void> => {
    const lines = await readLog(events);
    for (const prefix of ["start:", "service:"]) {
      const line = lines.find((entry) => entry.startsWith(prefix));
      if (line === undefined) continue;
      try {
        process.kill(pidOf(lines, prefix), "SIGKILL");
      } catch {
        // Already gone.
      }
    }
    process_.kill();
    await process_.exited;
    await rm(directory, { recursive: true, force: true });
  };
  return { host: process_, log: () => readLog(events), waitFor, stop };
}

test("a closed host connection ends a renderer that application timers keep alive", async () => {
  const probe = await startHost("");
  try {
    await probe.waitFor(async () => (await probe.log()).includes("mounted"), "the renderer to mount");
    await probe.waitFor(async () => (await probe.log()).includes("polling"), "the application timer to run");
    const lines = await probe.log();
    const renderer = pidOf(lines, "start:");
    const service = pidOf(lines, "service:");
    expect(isAlive(renderer)).toBe(true);

    // The host owns the connection; killing it closes the renderer's stdin pipe
    // while the application timer keeps Bun's event loop busy.
    process.kill(probe.host.pid, "SIGKILL");

    await probe.waitFor(async () => (await probe.log()).includes("exit:0"), "the renderer to exit");
    expect((await probe.log()).find((line) => line.startsWith("terminated:"))).toMatch(
      /^terminated:StdioTransport input (ended|closed)/,
    );
    await probe.waitFor(async () => !isAlive(renderer), "the renderer process to end");
    // Detached application services are separate processes; the connection owns
    // only the renderer, so they keep running.
    expect(isAlive(service)).toBe(true);
  } finally {
    await probe.stop();
  }
});

test("intentional transport disposal keeps the renderer running", async () => {
  const probe = await startHost(`    setTimeout(() => {
      transport.dispose();
      record("disposed");
    }, 200);`);
  try {
    await probe.waitFor(
      async () => (await probe.log()).includes("disposed"),
      "the application to dispose its transport",
    );
    const renderer = pidOf(await probe.log(), "start:");
    const disposed = (await probe.log()).indexOf("disposed");
    await probe.waitFor(
      async () => (await probe.log()).slice(disposed).includes("polling"),
      "the application to keep running after disposal",
    );
    expect((await probe.log()).some((line) => line.startsWith("exit:"))).toBe(false);
    expect(isAlive(renderer)).toBe(true);
  } finally {
    await probe.stop();
  }
});

// Node streams satisfy the transport's byte-stream surface; their overloaded
// event signatures need one structural assertion each.
const asInput = (stream: PassThrough): ByteInput => stream as unknown as ByteInput;
const asOutput = (stream: PassThrough): ByteOutput => stream as unknown as ByteOutput;

test("connections over supplied streams outlive the host end", async () => {
  const exits: number[] = [];
  const input = new PassThrough();
  new StdioTransport({
    input: asInput(input),
    output: asOutput(new PassThrough()),
    exit: (code) => exits.push(code),
  });
  input.end();
  await once(input, "end");
  expect(exits).toEqual([]);
});

test("a host-owned connection exits 0 on a closed pipe and 1 on a read failure", async () => {
  const eofExits: number[] = [];
  const eofInput = new PassThrough();
  new StdioTransport({
    input: asInput(eofInput),
    output: asOutput(new PassThrough()),
    exitOnHostClose: true,
    exit: (code) => eofExits.push(code),
  });
  eofInput.end();
  await once(eofInput, "end");
  expect(eofExits).toEqual([0]);

  const failureExits: number[] = [];
  const failureInput = new PassThrough();
  new StdioTransport({
    input: asInput(failureInput),
    output: asOutput(new PassThrough()),
    exitOnHostClose: true,
    exit: (code) => failureExits.push(code),
  });
  failureInput.destroy(new Error("broken pipe"));
  await once(failureInput, "error");
  expect(failureExits).toEqual([1]);
});
