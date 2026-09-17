import { dlopen } from "bun:ffi";
import { fork, spawnSync } from "node:child_process";
import cluster from "node:cluster";
import { readdirSync } from "node:fs";
import { Module } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";

function expectRejected(label: string, code: string, operation: () => unknown): void {
  try {
    operation();
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === code) return;
    throw new Error(`${label} did not reject with ${code}`, { cause: error });
  }
  throw new Error(`${label} unexpectedly succeeded`);
}

export function checkNativeExtraction(resourcePath: string): void {
  // Reach the native loader's bundled-file branch, not an OS-only path miss.
  const graphPath = process.platform === "win32" ? resourcePath.replaceAll("\\", "/") : resourcePath;
  const before = readdirSync(tmpdir())
    .filter((name) => name.startsWith(".bun-"))
    .sort()
    .join("\0");
  // A harmless text resource detects extraction even without a loadable DLL.
  expectRejected("FFI graph extraction", "ERR_DLOPEN_FAILED", () =>
    dlopen(graphPath, { missing: { args: [], returns: "void" } }),
  );
  expectRejected("N-API graph extraction", "ERR_DLOPEN_FAILED", () =>
    process.dlopen(new Module("embedded-native-check"), graphPath),
  );
  if (
    readdirSync(tmpdir())
      .filter((name) => name.startsWith(".bun-"))
      .sort()
      .join("\0") !== before
  ) {
    throw new Error("A native loader extracted an embedded resource");
  }
}

export function checkChildInterpreter(): void {
  expectRejected("fork with the host executable", "ERR_INVALID_ARG_VALUE", () => fork(import.meta.filename));
  cluster.setupPrimary({ exec: import.meta.filename });
  expectRejected("cluster with the host executable", "ERR_INVALID_ARG_VALUE", () => cluster.fork());

  const windows = process.platform === "win32";
  const executable = windows ? join(process.env.SystemRoot!, "System32", "cmd.exe") : "/usr/bin/printf";
  const args = windows ? ["/d", "/c", "echo embedded-child-ok"] : ["embedded-child-ok"];
  const child = spawnSync(executable, args, { encoding: "utf8", windowsHide: true });
  if (child.error) throw child.error;
  if (child.status !== 0 || child.stdout.trim() !== "embedded-child-ok") {
    throw new Error(`External child failed: ${child.status}: ${child.stderr}`);
  }
}
