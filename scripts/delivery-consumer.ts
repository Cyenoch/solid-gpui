#!/usr/bin/env bun
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { parseArgs } from "node:util";

const repo = resolve(import.meta.dirname, "..");
async function run(command: string[], cwd: string): Promise<void> {
  console.error(`$ ${command.join(" ")}`);
  const child = Bun.spawn(command, { cwd, env: process.env, stdio: ["ignore", "inherit", "inherit"] });
  if ((await child.exited) !== 0) throw new Error(`Consumer command failed: ${command[0]}`);
}

const { values } = parseArgs({
  options: {
    manifest: { type: "string" },
    mode: { type: "string" },
    "stock-only": { type: "boolean" },
    "native-only": { type: "boolean" },
    keep: { type: "boolean" },
  },
  strict: true,
});
if (typeof values.manifest !== "string") throw new Error("Pass --manifest <delivery.json or manifest-path.txt>");
const manifestPath = resolve(
  values.manifest.endsWith(".txt") ? (await readFile(values.manifest, "utf8")).trim() : values.manifest,
);
const temporary = await mkdtemp(
  join(process.env.SOLID_GPUI_CONSUMER_TEMP ?? tmpdir(), "solid-gpui-delivery-consumer-"),
);
const mode = typeof values.mode === "string" ? values.mode : "production";
try {
  const cli = join(repo, "packages/solid-gpui-vite/dist/cli.js");
  const applications = [
    { name: "stock-quickjs", runtime: "quickjs", native: false },
    { name: "stock-bun", runtime: "bun", native: false },
    { name: "rust-quickjs", runtime: "quickjs", native: true },
    { name: "rust-bun", runtime: "bun", native: true },
  ].filter(({ native }) => (native ? !values["stock-only"] : !values["native-only"]));
  for (const { name, runtime, native } of applications) {
    const root = join(temporary, name);
    await run(
      [
        process.execPath,
        cli,
        "create",
        root,
        "--runtime",
        runtime,
        "--manifest",
        manifestPath,
        ...(native ? ["--native"] : []),
      ],
      repo,
    );
    await run([process.execPath, "install"], root);
    const consumerCli = join(root, "node_modules/@solid-gpui/vite/dist/cli.js");
    if (!native) await run([process.execPath, consumerCli, "host", "install", "--manifest", manifestPath], root);
    await run([process.execPath, consumerCli, "prepare", "--mode", mode], root);
    await run([process.execPath, "run", "typecheck"], root);
    await run([process.execPath, consumerCli, "test", "src/counter.test.tsx"], root);
    await run([process.execPath, "run", "build", "--mode", mode], root);
    await run([process.execPath, consumerCli, "package", "--name", name, "--mode", mode], root);
    // The extracted app check is a real VM/protocol check, not a physical display assertion.
    if (native) {
      const artifacts = JSON.parse(await readFile(join(root, ".solid-gpui/artifacts.json"), "utf8"));
      const binary = artifacts.native.executable;
      if (process.platform === "darwin") {
        const preview = Bun.spawn([process.execPath, consumerCli, "preview", "--mode", mode], {
          cwd: root,
          env: process.env,
          stdout: "pipe",
          stderr: "pipe",
        });
        const timer = setTimeout(() => preview.kill("SIGTERM"), 2500);
        const started = Date.now();
        const [stderr] = await Promise.all([new Response(preview.stderr).text(), preview.exited]);
        clearTimeout(timer);
        if (Date.now() - started < 2000) throw new Error(`Production preview exited early: ${stderr}`);
      }
      await writeFile(
        join(root, "src/service.tsx"),
        `import { mountApplication, Text } from "@solid-gpui/core";\nimport { ${runtime === "quickjs" ? "EmbeddedTransport" : "StdioTransport"} } from "@solid-gpui/core/${runtime === "quickjs" ? "embedded" : "stdio"}";\nimport { createSignal, onMount } from "@solid-gpui/core/runtime";\nimport { useNative } from "#native";\nmountApplication({ transport: () => new ${runtime === "quickjs" ? "EmbeddedTransport" : "StdioTransport"}(), setup: () => ({ render: () => {\n  const native = useNative(); const [message, setMessage] = createSignal("pending");\n  onMount(async () => setMessage(await native.greeting()));\n  return <Text>{message()}</Text>;\n} }) });\n`,
      );
      await writeFile(
        join(root, "vite.service.config.ts"),
        (await readFile(join(root, "vite.config.ts"), "utf8")).replace(
          'entry: "src/app.tsx"',
          'entry: "src/service.tsx"',
        ),
      );
      await run([process.execPath, "run", "build", "--mode", mode, "--config", "vite.service.config.ts"], root);
      const serviceArtifacts = JSON.parse(await readFile(join(root, ".solid-gpui/artifacts.json"), "utf8"));
      await run([binary, "--check-app", runtime, serviceArtifacts.bundle, "Hello from Rust"], root);
    }
  }
  console.log(`Clean consumer delivery verified outside the SDK: ${temporary}`);
} finally {
  if (!values.keep) await rm(temporary, { recursive: true, force: true });
}
