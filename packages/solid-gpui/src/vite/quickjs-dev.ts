#!/usr/bin/env bun
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { createServer as createViteServer } from "vite";
import { createServer, type Socket } from "node:net";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { buildApplication } from "./build";

/** Tooling stays outside QuickJS; development and release use one build policy. */
export async function startQuickJsDev(entry: string, host: string): Promise<number> {
  entry = resolve(entry);
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-dev-"));
  const outfile = join(directory, "application.js");
  const vite = await createViteServer({
    configFile: false,
    appType: "custom",
    optimizeDeps: { noDiscovery: true, include: [] },
    server: {
      middlewareMode: true,
      hmr: false,
      ws: false,
      watch: { ignored: ["**/target/**", "**/.git/**", "**/.scratch/**"] },
    },
  });
  const maps = new Map<number, TraceMap>();
  let sent = 0;
  let rebuildTask: Promise<void> | undefined;
  let socket: Socket | undefined;
  let connected: (() => void) | undefined;
  const connection = new Promise<void>((resolve) => {
    connected = resolve;
  });
  const server = createServer((peer) => {
    if (socket) {
      peer.destroy();
      return;
    }
    socket = peer;
    let tail = "";
    peer.on("data", (data) => {
      tail += data.toString();
      if (tail.length > 8192) {
        peer.destroy(new Error("reload diagnostic exceeds budget"));
        return;
      }
      for (;;) {
        const end = tail.indexOf("\n");
        if (end < 0) break;
        const message = tail
          .slice(0, end)
          .replace(/([^ ()]+)\?generation=(\d+):(\d+):(\d+)/g, (location, _file, generation, line, column) => {
            const map = maps.get(Number(generation));
            if (!map) return location;
            const original = originalPositionFor(map, { line: Number(line), column: Math.max(0, Number(column) - 1) });
            return original.source && original.line !== null
              ? `${original.source}:${original.line}:${(original.column ?? 0) + 1}`
              : location;
          });
        console.error(`solid-gpui: ${message}`);
        tail = tail.slice(end + 1);
      }
    });
    peer.on("error", (error) => console.error(`solid-gpui: reload connection failed: ${error.message}`));
    connected?.();
  });
  let requested = 0;
  let building = false;
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let dependencies = new Set<string>();
  let buildFailed = false;
  const rebuild = async () => {
    if (building || stopped) return;
    building = true;
    try {
      let completed: number;
      do {
        completed = requested;
        try {
          buildFailed = false;
          dependencies = new Set(await buildApplication({ entry, outfile, runtime: "quickjs" }));
          vite.watcher.add([...dependencies, ...new Set([...dependencies].map(dirname))]);
          if (completed !== requested || stopped) continue;
          const bytes = new Uint8Array(await Bun.file(outfile).arrayBuffer());
          if (bytes.length > 32 * 1024 * 1024) throw new Error("reload bundle exceeds 32 MiB");
          const frame = Buffer.allocUnsafe(4 + bytes.length);
          frame.writeUInt32LE(bytes.length);
          frame.set(bytes, 4);
          await connection;
          if (stopped || completed !== requested) continue;
          const source = new TextDecoder().decode(bytes);
          const encodedMap = source.match(
            /sourceMappingURL=data:application\/json(?:;charset=utf-8)?;base64,([^\s]+)/,
          )?.[1];
          sent++;
          if (encodedMap) maps.set(sent, new TraceMap(JSON.parse(Buffer.from(encodedMap, "base64").toString())));
          for (const generation of maps.keys()) if (generation < sent - 2) maps.delete(generation);
          await new Promise<void>((resolve, reject) =>
            socket!.write(frame, (error) => (error ? reject(error) : resolve())),
          );
        } catch (error) {
          buildFailed = true;
          console.error(`solid-gpui: build rejected: ${String(error)}`);
        }
      } while (completed !== requested && !stopped);
    } finally {
      building = false;
    }
  };
  vite.watcher.on("all", (_event, path) => {
    path = resolve(path);
    if (/\.(rs|bop)$/.test(path) || /(?:vite\.config\.|Cargo\.(?:toml|lock)$)/.test(path)) {
      console.error("solid-gpui: native contract or configuration changed; restart the development command");
      return;
    }
    if (
      !dependencies.has(path) &&
      !(
        buildFailed &&
        /\.[cm]?[jt]sx?$/.test(path) &&
        [...dependencies].some((dependency) => path.startsWith(dirname(dependency) + "/"))
      )
    )
      return;
    requested++;
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (child && !building) rebuildTask = rebuild();
    }, 40);
  });
  let child: ReturnType<typeof Bun.spawn> | undefined;
  const stop = () => {
    stopped = true;
    clearTimeout(timer);
    connected?.();
    child?.kill();
    socket?.destroy();
  };
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);
  try {
    dependencies = new Set(await buildApplication({ entry, outfile, runtime: "quickjs" }));
    vite.watcher.add([...dependencies, ...new Set([...dependencies].map(dirname))]);
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject);
      server.listen(0, "127.0.0.1", resolve);
    });
    const address = server.address();
    if (!address || typeof address === "string") throw new Error("development server has no TCP endpoint");
    child = Bun.spawn([host, "--runtime", "quickjs-dev", outfile, `127.0.0.1:${address.port}`], {
      stdin: "inherit",
      stdout: "inherit",
      stderr: "inherit",
    });
    if (requested) rebuildTask = rebuild();
    return await child.exited;
  } finally {
    stop();
    server.close();
    await rebuildTask;
    await vite.close();
    await rm(directory, { recursive: true, force: true });
    process.off("SIGINT", stop);
    process.off("SIGTERM", stop);
  }
}
if (import.meta.main) {
  const [entry, host, ...extra] = process.argv.slice(2);
  if (!entry || !host || extra.length) throw new Error("Usage: solid-gpui-quickjs-dev <entry.tsx> <native-host>");
  process.exitCode = await startQuickJsDev(entry, host);
}
