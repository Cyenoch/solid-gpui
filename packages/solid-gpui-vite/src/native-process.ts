import { spawn } from "node:child_process";

export interface NativeCommandOptions {
  /**
   * Report Cargo's status and diagnostic output while it is produced. Without this the caller
   * receives the full stderr buffer on completion, which hides multi-minute compiles.
   */
  readonly onStderrLine?: (line: string) => void;
}

/** Cargo owns rustc and build-script children; cancelling only Cargo leaks those processes. */
export function runNativeCommand(
  command: string,
  args: string[],
  cwd: string,
  signal?: AbortSignal,
  options: NativeCommandOptions = {},
): Promise<{ stdout: string; stderr: string }> {
  signal?.throwIfAborted();
  const { promise, resolve, reject } = Promise.withResolvers<{ stdout: string; stderr: string }>();
  const child = spawn(command, args, {
    cwd,
    detached: process.platform !== "win32",
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  let pending = "";
  let bytes = 0;
  let failure: Error | undefined;
  let stopping = false;
  const stop = () => {
    if (stopping || !child.pid) return;
    stopping = true;
    if (process.platform === "win32") {
      const killer = spawn("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      killer.on("error", reject);
    } else {
      try {
        process.kill(-child.pid, "SIGTERM");
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== "ESRCH") reject(error);
      }
    }
  };
  signal?.addEventListener("abort", stop, { once: true });
  const collect = (chunk: string, destination: "stdout" | "stderr") => {
    bytes += Buffer.byteLength(chunk);
    if (bytes > 32 * 1024 * 1024) {
      failure ??= new Error(`${command} output exceeded 32 MiB`);
      stop();
      return;
    }
    if (destination === "stdout") {
      stdout += chunk;
      return;
    }
    stderr += chunk;
    pending += chunk;
    for (let end = pending.indexOf("\n"); end >= 0; end = pending.indexOf("\n")) {
      options.onStderrLine?.(pending.slice(0, end));
      pending = pending.slice(end + 1);
    }
  };
  child.stdout.setEncoding("utf8").on("data", (chunk: string) => collect(chunk, "stdout"));
  child.stderr.setEncoding("utf8").on("data", (chunk: string) => collect(chunk, "stderr"));
  child.on("error", (error) => {
    failure = error;
  });
  child.on("close", (code, termination) => {
    signal?.removeEventListener("abort", stop);
    if (pending) options.onStderrLine?.(pending);
    if (signal?.aborted) reject(signal.reason);
    else if (failure) reject(failure);
    else if (code !== 0) reject(new Error(`${command} failed (${termination ?? code}):\n${stderr}`));
    else resolve({ stdout, stderr });
  });
  return promise;
}
