import { expect, test } from "bun:test";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { createLogger, createServer, type ViteDevServer } from "vite";
import { solidGpui } from "../packages/solid-gpui-vite/src/index.ts";

const repo = resolve(import.meta.dir, "..");

test("an unavailable external executable leaves Vite watching without an unhandled startup promise", async () => {
  const root = await mkdtemp(join(repo, ".scratch/dev-session-test-"));
  const logger = createLogger("silent");
  let diagnostic = "";
  logger.error = (message) => {
    diagnostic += message;
  };
  const server = await createServer({
    root,
    configFile: false,
    customLogger: logger,
    server: { port: 0 },
    plugins: [solidGpui({ entry: "app.js", host: { command: join(root, "missing-host") } })],
  });
  try {
    await server.listen();
    expect(diagnostic).toContain("Watching for changes");
  } finally {
    await server.close();
    await rm(root, { recursive: true, force: true });
  }
});

for (const runtime of ["bun", "quickjs"] as const) {
  test(`${runtime} dev survives failed builds and sessions, rebuilding local Rust dependencies with matching bindings`, async () => {
    const directory = await mkdtemp(join(repo, ".scratch/dev-session-test-"));
    const root = join(directory, "application");
    const native = join(root, "native");
    const contract = join(directory, "contract");
    const source = join(contract, "src/lib.rs");
    const entry = join(root, "app.js");
    const observations = join(root, "observations.log");
    const host = join(root, "host.ts");
    let server: ViteDevServer | undefined;
    let diagnostics = "";
    const logger = createLogger("silent");
    logger.error =
      logger.info =
      logger.warn =
        (message) => {
          diagnostics += message + "\n";
        };
    const read = () => readFile(observations, "utf8");
    const until = async (predicate: () => boolean | Promise<boolean>) => {
      const deadline = Date.now() + 15000;
      while (!(await predicate())) {
        if (Date.now() > deadline) throw new Error(`Session recovery timed out:\n${diagnostics}\n${await read()}`);
        await Bun.sleep(25);
      }
    };
    const alive = (pid: number) => {
      try {
        process.kill(pid, 0);
        return true;
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
        throw error;
      }
    };
    const rust = (version: number) => `pub const VERSION: u32 = ${version};\n`;
    const app = (label: string) => `import { answer } from '#native';
      import { answer as componentAnswer } from '@solid-gpui/core/components';
      if (answer !== componentAnswer) throw new Error('mixed generated bindings');
      globalThis.record(answer, '${label}');\n`;
    try {
      await mkdir(join(native, "src"), { recursive: true });
      await mkdir(join(contract, "src"), { recursive: true });
      await mkdir(join(root, ".cargo"));
      await writeFile(join(root, ".cargo/config.toml"), '[build]\ntarget-dir="native/build-cache"\n');
      await writeFile(observations, "");
      await writeFile(join(root, "package.json"), '{"type":"module"}');
      await writeFile(
        join(contract, "Cargo.toml"),
        '[package]\nname="session-contract"\nversion="0.0.0"\nedition="2024"\n',
      );
      await writeFile(
        join(native, "Cargo.toml"),
        '[package]\nname="session-host"\nversion="0.0.0"\nedition="2024"\n[dependencies]\nsession-contract={path="../../contract"}\n[workspace]\n',
      );
      await writeFile(source, "pub const VERSION: u32 = ;");
      await writeFile(
        join(native, "src/main.rs"),
        `fn main() {
        if std::env::args().nth(1).as_deref() == Some("--export-native") {
          println!("export const answer = {};", session_contract::VERSION); return;
        }
        let mut command = std::process::Command::new(${JSON.stringify(process.execPath)});
        command.arg(${JSON.stringify(host)}).args(std::env::args().skip(1))
          .env("SESSION_VERSION", session_contract::VERSION.to_string());
        #[cfg(unix)] { use std::os::unix::process::CommandExt; panic!("{}", command.exec()); }
        #[cfg(not(unix))] { std::process::exit(command.status().unwrap().code().unwrap_or(1)); }
      }`,
      );
      const preload = join(root, "record.ts");
      await writeFile(
        preload,
        `import { appendFileSync } from 'node:fs';
        globalThis.record = (version, label) => {
          if (version !== Number(process.env.SESSION_VERSION)) throw new Error('mixed native contract');
          appendFileSync(${JSON.stringify(observations)}, 'render:' + version + ':' + label + '\\n');
          if (label === 'crash') process.exit(7);
          if (label === 'close') process.exit(0);
        };`,
      );
      await writeFile(
        host,
        `import { appendFileSync, readFileSync } from 'node:fs';
        import { connect } from 'node:net';
        import ${JSON.stringify(preload)};
        const log = (value) => appendFileSync(${JSON.stringify(observations)}, value + '\\n');
        log('pid:' + process.pid);
        log('host:' + process.env.SESSION_VERSION);
        if (process.env.SOLID_GPUI_VITE_RUNNER) {
          const runner = JSON.parse(process.env.SOLID_GPUI_VITE_RUNNER);
          runner.splice(1, 0, '--preload', ${JSON.stringify(preload)});
          const child = Bun.spawn(runner, { stdin: 'pipe', stdout: 'inherit', stderr: 'ignore' });
          log('pid:' + child.pid);
          process.exitCode = await child.exited;
        } else {
          const [, , , , initial, endpoint] = process.argv;
          const [hostname, port] = endpoint.split(':');
          let pending = Buffer.alloc(0);
          let imports = Promise.resolve();
          const socket = connect({ host: hostname, port: Number(port) });
          socket.on('close', () => process.exit(0));
          socket.on('data', (bytes) => {
            pending = Buffer.concat([pending, bytes]);
            while (pending.length >= 4 && pending.length >= 4 + pending.readUInt32LE(0)) {
              const length = pending.readUInt32LE(0);
              const source = pending.subarray(4, 4 + length).toString('base64');
              pending = pending.subarray(4 + length);
              imports = imports.then(() => import('data:text/javascript;base64,' + source));
            }
          });
          await import(initial);
        }`,
      );
      await writeFile(entry, "export const broken = ;");
      const config = join(root, "vite.config.ts");
      await writeFile(
        config,
        `import { solidGpui } from ${JSON.stringify(join(repo, "packages/solid-gpui-vite/src/index.ts"))};
        export default { plugins: [solidGpui({ entry: 'app.js', runtime: '${runtime}',
          native: { manifestPath: 'native/Cargo.toml', bin: 'session-host' } })] };`,
      );
      server = await createServer({
        root,
        configFile: config,
        customLogger: logger,
        server: { port: 0 },
        clearScreen: false,
      });
      await server.listen();
      expect(diagnostics).toContain("Watching for changes");
      expect(await read()).toBe("");

      await writeFile(source, rust(1));
      await until(() => diagnostics.includes("app.js"));
      await writeFile(entry, app("initial"));
      await until(async () => (await read()).includes("render:1:initial"));
      const oldPids = [...(await read()).matchAll(/pid:(\d+)/g)].map((match) => Number(match[1]));

      await writeFile(source, rust(2));
      await until(async () => (await read()).includes("render:2:initial"));
      await until(() => oldPids.every((pid) => !alive(pid)));
      await Bun.sleep(150);
      expect((await read()).match(/render:2:initial/g)).toHaveLength(1);
      expect(await readFile(join(root, ".generated/native.ts"), "utf8")).toContain("answer = 2");

      const failures = () => diagnostics.match(/Watching for changes/g)?.length ?? 0;
      const before = failures();
      await writeFile(entry, app("crash"));
      await until(() => failures() > before);
      const starts = (await read()).match(/host:/g)?.length;
      await Bun.sleep(250);
      expect((await read()).match(/host:/g)?.length).toBe(starts);
      await writeFile(entry, app("recovered"));
      await until(async () => (await read()).includes("render:2:recovered"));
      expect(diagnostics).not.toContain("mixed native contract");
      await writeFile(entry, app("close"));
      await until(() => diagnostics.includes("application closed. Watching for changes"));
      await writeFile(entry, app("reopened"));
      await until(async () => (await read()).includes("render:2:reopened"));
      if (runtime === "bun") {
        await writeFile(
          join(native, "build.rs"),
          `use std::io::Write;
          fn main() {
            let mut log = std::fs::OpenOptions::new().append(true).open(${JSON.stringify(observations)}).unwrap();
            writeln!(log, "build-pid:{}", std::process::id()).unwrap();
            println!("cargo:rerun-if-changed={}", ${JSON.stringify(source)});
            std::fs::write(std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("generated.rs"), "// Generated build input\n").unwrap();
            if !std::fs::read_to_string(${JSON.stringify(source)}).unwrap().contains("= 3;") {
              std::thread::sleep(std::time::Duration::from_secs(30));
            }
          }`,
        );
        await until(async () => (await read()).includes("build-pid:"));
        const obsoleteBuild = Number((await read()).match(/build-pid:(\d+)/)![1]);
        await writeFile(source, rust(3));
        await until(async () => (await read()).includes("render:3:reopened"));
        expect(alive(obsoleteBuild)).toBe(false);
        const builds = (await read()).match(/build-pid:/g)!.length;
        await writeFile(source, rust(4));
        await until(async () => (await read()).match(/build-pid:/g)!.length > builds);
      }
      const closeStarted = Date.now();
      await server.close();
      expect(Date.now() - closeStarted).toBeLessThan(5000);
      const pids = [...(await read()).matchAll(/pid:(\d+)/g)].map((match) => Number(match[1]));
      await until(() => pids.every((pid) => !alive(pid)));
    } finally {
      await server?.close();
      // Also contain failed cancellation assertions so the fixture cannot leave a compiler behind.
      for (const match of (await read()).matchAll(/(?:build-)?pid:(\d+)/g)) {
        const pid = Number(match[1]);
        if (alive(pid)) process.kill(pid, "SIGKILL");
      }
      await rm(directory, { recursive: true, force: true });
    }
  }, 60000);
}
