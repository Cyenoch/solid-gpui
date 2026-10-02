import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { chmod, copyFile, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { NativeHostOptions } from "./environment.ts";
import { sdkPackage } from "./source.ts";

/** Candidate package identity is computed over exact shipped JavaScript sources. */
export async function deliverySourceDigest(directory: string): Promise<string> {
  return sourceDigest(directory);
}

function sourceDigest(directory: string): string {
  const hash = createHash("sha256");
  function visit(relative: string): void {
    for (const entry of readdirSync(join(directory, relative), { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name),
    )) {
      const path = `${relative}/${entry.name}`;
      if (entry.isDirectory()) visit(path);
      else if (entry.isFile())
        hash
          .update(path)
          .update("\0")
          .update(readFileSync(join(directory, path)))
          .update("\0");
      else throw new Error("SDK source identity cannot include symbolic links");
    }
  }
  visit("src");
  return hash.digest("hex");
}

/** Delivery uses the installed tool's exact version, never a template branch or registry tag. */
export function deliveryVersion(): string {
  const manifest = JSON.parse(readFileSync(resolve(import.meta.dirname, "../package.json"), "utf8"));
  if (!/^\d+\.\d+\.\d+$/.test(manifest.version)) throw new Error("Delivery requires an exact SDK release version");
  return manifest.version;
}

export function stockTarget(): string {
  const targets: Record<string, string> = {
    "darwin-arm64": "aarch64-apple-darwin",
    "darwin-x64": "x86_64-apple-darwin",
    "linux-x64": "x86_64-unknown-linux-gnu",
    "win32-x64": "x86_64-pc-windows-msvc",
  };
  const target = targets[`${process.platform}-${process.arch}`];
  if (!target)
    throw new Error(
      `No stock host artifact for ${process.platform}/${process.arch}; select an explicit custom Rust host`,
    );
  return target;
}

export interface DeliveryFile {
  readonly file: string;
  readonly sha256: string;
  readonly bytes: number;
}

/** Generated together from one release checkout by scripts/delivery-release.ts. */
export interface DeliveryManifest {
  readonly format: 1;
  readonly version: string;
  readonly revision: string;
  readonly target: string;
  readonly protocolVersion: number;
  readonly schemaDigest: string;
  readonly runtimes: readonly ["bun", "quickjs"];
  readonly host: DeliveryFile;
  readonly bindings: DeliveryFile;
  readonly source: DeliveryFile;
  readonly packages: { readonly core: DeliveryFile; readonly vite: DeliveryFile };
  readonly javascript: { readonly core: string; readonly vite: string };
}

export async function deliveryHash(path: string): Promise<string> {
  return createHash("sha256")
    .update(await readFile(path))
    .digest("hex");
}

function assertManifest(value: unknown, target: string): asserts value is DeliveryManifest {
  if (!value || typeof value !== "object") throw new Error("Invalid delivery manifest");
  const manifest = value as DeliveryManifest;
  if (
    manifest.format !== 1 ||
    manifest.version !== deliveryVersion() ||
    manifest.target !== target ||
    !/^[a-f0-9]{40}$/.test(manifest.revision) ||
    !Number.isInteger(manifest.protocolVersion) ||
    !/^[a-f0-9]{64}$/.test(manifest.schemaDigest) ||
    JSON.stringify(manifest.runtimes) !== '["bun","quickjs"]'
  ) {
    throw new Error("Delivery manifest does not match this exact SDK release, target or runtime contract");
  }
  if (
    !manifest.javascript ||
    !/^[a-f0-9]{64}$/.test(manifest.javascript.core) ||
    !/^[a-f0-9]{64}$/.test(manifest.javascript.vite)
  )
    throw new Error("Delivery manifest has no exact JavaScript package identity");
  for (const file of [
    manifest.host,
    manifest.bindings,
    manifest.source,
    manifest.packages?.core,
    manifest.packages?.vite,
  ]) {
    if (
      !file ||
      !/^[A-Za-z0-9._-]+$/.test(file.file) ||
      !/^[a-f0-9]{64}$/.test(file.sha256) ||
      !Number.isSafeInteger(file.bytes) ||
      file.bytes <= 0 ||
      file.bytes > 512 * 1024 * 1024
    ) {
      throw new Error("Invalid or oversized delivery artifact");
    }
  }
}

export async function deliveryRun(
  command: readonly string[],
  cwd: string,
  env: NodeJS.ProcessEnv = process.env,
): Promise<string> {
  const child = Bun.spawn([...command], { cwd, env, stdin: "ignore", stdout: "pipe", stderr: "pipe" });
  const timer = setTimeout(() => child.kill(), 30_000);
  try {
    const [stdout, stderr, code] = await Promise.all([
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
      child.exited,
    ]);
    if (code !== 0) throw new Error(`${command[0]} failed (${code}): ${stderr.trim()}`);
    return stdout;
  } finally {
    clearTimeout(timer);
  }
}

async function acquire(url: URL, destination: string, expected?: DeliveryFile): Promise<void> {
  if (url.protocol === "file:") await copyFile(fileURLToPath(url), destination);
  else {
    if (url.protocol !== "https:") throw new Error("Delivery downloads require HTTPS or an explicit local file");
    const response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
    if (!response.ok)
      throw new Error(
        `Exact release artifact unavailable: ${url} (${response.status}); no source fallback is attempted`,
      );
    if (!response.body) throw new Error(`Empty release response: ${url}`);
    const chunks: Uint8Array[] = [];
    let size = 0;
    for await (const chunk of response.body) {
      size += chunk.byteLength;
      if (size > (expected?.bytes ?? 64 * 1024)) throw new Error("Release download exceeds its declared byte budget");
      chunks.push(chunk);
    }
    await writeFile(destination, Buffer.concat(chunks));
  }
  const bytes = await readFile(destination);
  if (bytes.length > (expected?.bytes ?? 64 * 1024))
    throw new Error("Release artifact exceeds its declared byte budget");
  if (expected && (bytes.length !== expected.bytes || (await deliveryHash(destination)) !== expected.sha256)) {
    throw new Error(`Release artifact checksum or size mismatch: ${expected.file}`);
  }
}

export interface InstallStockHostOptions {
  readonly root?: string;
  /** Local manifest path or HTTPS URL for an unpublished/offline candidate. */
  readonly manifest?: string;
}

export async function readDeliveryManifest(
  options: InstallStockHostOptions,
  directory: string,
): Promise<{ manifest: DeliveryManifest; url: URL }> {
  const target = stockTarget();
  if (!options.manifest && deliveryVersion() === "0.5.2")
    throw new Error(
      "Standalone delivery is an unpublished 0.5.2 candidate; pass --manifest for locally paired package/host artifacts. Published 0.5.2 does not provide these APIs",
    );
  const url = options.manifest
    ? /^https:\/\//.test(options.manifest)
      ? new URL(options.manifest)
      : pathToFileURL(resolve(options.manifest))
    : new URL(
        `https://github.com/Cyenoch/solid-gpui/releases/download/v${deliveryVersion()}/solid-gpui-delivery-${deliveryVersion()}-${target}.json`,
      );
  const path = join(directory, "delivery.json");
  await acquire(url, path);
  const manifest: unknown = JSON.parse(await readFile(path, "utf8"));
  assertManifest(manifest, target);
  return { manifest, url };
}

/** Explicit acquisition only. Neither npm installation nor Vite builds download/build a host. */
export async function installStockHost(options: InstallStockHostOptions = {}): Promise<string> {
  const root = resolve(options.root ?? process.cwd());
  const parent = join(root, ".solid-gpui/hosts");
  await mkdir(parent, { recursive: true });
  const stage = await mkdtemp(join(parent, ".download-"));
  try {
    const { manifest, url } = await readDeliveryManifest(options, stage);
    const binary = join(stage, "solid-gpui-host" + (process.platform === "win32" ? ".exe" : ""));
    await acquire(new URL(manifest.host.file, url), binary, manifest.host);
    await acquire(new URL(manifest.bindings.file, url), join(stage, "native.ts"), manifest.bindings);
    if (process.platform !== "win32") await chmod(binary, 0o755);
    await verifyStockHost(stage);
    const destination = join(parent, `${manifest.version}-${manifest.target}`);
    if (existsSync(destination)) {
      await verifyStockHost(destination);
      if (
        (await deliveryHash(join(destination, "delivery.json"))) !== (await deliveryHash(join(stage, "delivery.json")))
      ) {
        throw new Error(
          "An installed host already has another build identity; remove that specific host directory before replacing it",
        );
      }
    } else await rename(stage, destination);
    return stockHost(root).command;
  } finally {
    await rm(stage, { recursive: true, force: true });
  }
}

/** Pure selection; the standard project plan owns export, bindings and bundle paths. */
export function stockHost(root = process.cwd()): NativeHostOptions {
  const directory = join(resolve(root), ".solid-gpui/hosts", `${deliveryVersion()}-${stockTarget()}`);
  if (!existsSync(join(directory, "delivery.json")))
    throw new Error("Stock host is missing; run solid-gpui host install explicitly before prepare, develop or build");
  const manifest: unknown = JSON.parse(readFileSync(join(directory, "delivery.json"), "utf8"));
  assertManifest(manifest, stockTarget());
  const core = sdkPackage("@solid-gpui/core", root);
  if (!core || core.version !== manifest.version)
    throw new Error("Stock host requires the exact paired @solid-gpui/core version");
  const lock = JSON.parse(readFileSync(join(core.directory, "src/protocol/schema-lock.json"), "utf8"));
  if (lock.protocolVersion !== manifest.protocolVersion || lock.sha256 !== manifest.schemaDigest)
    throw new Error("Stock host schema/build mismatch with the installed SDK");
  const vite = sdkPackage("@solid-gpui/vite", root);
  if (
    !vite ||
    sourceDigest(core.directory) !== manifest.javascript.core ||
    sourceDigest(vite.directory) !== manifest.javascript.vite
  )
    throw new Error(
      "Stock host requires the exact paired JavaScript candidate packages, not another build with the same version",
    );
  const command = join(directory, "solid-gpui-host" + (process.platform === "win32" ? ".exe" : ""));
  for (const [path, expected] of [
    [command, manifest.host.sha256],
    [join(directory, "native.ts"), manifest.bindings.sha256],
  ]) {
    if (createHash("sha256").update(readFileSync(path!)).digest("hex") !== expected)
      throw new Error("Installed stock host checksum mismatch");
  }
  return { command };
}

export async function verifyStockHost(directory: string): Promise<void> {
  const manifest: unknown = JSON.parse(await readFile(join(directory, "delivery.json"), "utf8"));
  assertManifest(manifest, stockTarget());
  const binary = join(directory, "solid-gpui-host" + (process.platform === "win32" ? ".exe" : ""));
  if (
    (await deliveryHash(binary)) !== manifest.host.sha256 ||
    (await deliveryHash(join(directory, "native.ts"))) !== manifest.bindings.sha256
  ) {
    throw new Error("Installed stock host or bindings checksum mismatch");
  }
  const version = (await deliveryRun([binary, "--version"], directory)).trim();
  if (version !== `solid-gpui-host ${manifest.version} protocol=v${manifest.protocolVersion}`)
    throw new Error("Stock host release/protocol mismatch");
  const bindings = await deliveryRun([binary, "--export-native"], directory);
  if (createHash("sha256").update(bindings).digest("hex") !== manifest.bindings.sha256)
    throw new Error("Stock host catalog/build mismatch");
}

export interface ScaffoldOptions {
  readonly directory: string;
  readonly runtime?: "bun" | "quickjs";
  readonly native?: boolean;
  readonly manifest?: string;
}

/** A versioned template in this package. Custom Rust explicitly acquires the paired source archive. */
export async function scaffoldApplication(options: ScaffoldOptions): Promise<void> {
  const root = resolve(options.directory);
  if (existsSync(root) && (await readdir(root)).length > 0) throw new Error("Scaffold destination must be empty");
  const runtime = options.runtime ?? "quickjs";
  if (runtime !== "bun" && runtime !== "quickjs") throw new Error("Scaffold runtime must be bun or quickjs");
  const version = deliveryVersion();
  if (version === "0.5.2" && !options.manifest)
    throw new Error(
      "This unpublished scaffold requires --manifest with locally packed candidate packages; published 0.5.2 has no delivery APIs",
    );
  const name = basename(root)
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, "-");
  if (!/^[a-z][a-z0-9-]*$/.test(name)) throw new Error("Application directory must start with a letter");
  await mkdir(join(root, "src"), { recursive: true });
  const write = async (file: string, text: string) => {
    await mkdir(dirname(join(root, file)), { recursive: true });
    await writeFile(join(root, file), text);
  };
  let coreDependency = version;
  let viteDependency = version;
  if (options.manifest) {
    const stage = await mkdtemp(join(root, ".packages-"));
    try {
      const { manifest, url } = await readDeliveryManifest({ manifest: options.manifest }, stage);
      const destination = join(root, ".solid-gpui/packages");
      await mkdir(destination, { recursive: true });
      for (const file of [manifest.packages.core, manifest.packages.vite])
        await acquire(new URL(file.file, url), join(destination, file.file), file);
      coreDependency = `file:./.solid-gpui/packages/${manifest.packages.core.file}`;
      viteDependency = `file:./.solid-gpui/packages/${manifest.packages.vite.file}`;
      await copyFile(join(stage, "delivery.json"), join(root, ".solid-gpui/candidate.json"));
    } finally {
      await rm(stage, { recursive: true, force: true });
    }
  }
  if (options.native) {
    const stage = await mkdtemp(join(root, ".sdk-"));
    try {
      const { manifest, url } = await readDeliveryManifest({ manifest: options.manifest }, stage);
      const source = join(stage, "sdk.tar.gz");
      await acquire(new URL(manifest.source.file, url), source, manifest.source);
      const entries = (await deliveryRun(["tar", "-tzf", source], root)).trim().split("\n");
      if (entries.some((path) => !path.startsWith("sdk/") || path.split("/").includes("..") || path.includes("\\")))
        throw new Error("Unsafe SDK archive path");
      const listing = await deliveryRun(["tar", "-tvzf", source], root);
      if (listing.split("\n").some((line) => line && !/^[d-]/.test(line)))
        throw new Error("SDK archive must contain only directories and regular files");
      await deliveryRun(["tar", "-xzf", source, "-C", stage], root);
      const paired = JSON.parse(await readFile(join(stage, "sdk/delivery-source.json"), "utf8"));
      if (
        paired.version !== version ||
        paired.revision !== manifest.revision ||
        paired.schemaDigest !== manifest.schemaDigest
      )
        throw new Error("SDK source build identity mismatch");
      await mkdir(join(root, ".solid-gpui"), { recursive: true });
      await rename(join(stage, "sdk"), join(root, ".solid-gpui/sdk"));
      await write(
        "Cargo.toml",
        `[workspace]\nresolver = "2"\nmembers = ["native"]\n\n${await readFile(join(root, ".solid-gpui/sdk/consumer-cargo.toml"), "utf8")}`,
      );
      await write("rust-toolchain.toml", await readFile(join(root, ".solid-gpui/sdk/rust-toolchain.toml"), "utf8"));
      await write(
        "native/Cargo.toml",
        `[package]\nname = "${name}"\nversion = "0.1.0"\nedition = "2024"\n\n[dependencies]\nsolid-gpui = { path = "../.solid-gpui/sdk/crates/solid-gpui", features = ["gpui-component", "quickjs"] }\n`,
      );
      await write(
        "native/src/main.rs",
        `use solid_gpui::native_module;\n\n#[native_module(name = "app")]\nmod app {\n    #[command]\n    pub fn greeting() -> String { "Hello from Rust".into() }\n}\n\nfn main() {\n    solid_gpui::host::run(app::native_module);\n}\n`,
      );
    } finally {
      await rm(stage, { recursive: true, force: true });
    }
  }
  await write(
    "package.json",
    JSON.stringify(
      {
        name,
        private: true,
        type: "module",
        scripts: {
          ...(!options.native
            ? {
                "host:install": `solid-gpui host install${options.manifest ? ` --manifest ${JSON.stringify(options.manifest)}` : ""}`,
              }
            : {}),
          generate: "solid-gpui prepare",
          "check:generated": "solid-gpui prepare --check",
          doctor: "solid-gpui doctor",
          typecheck: "tsc --noEmit && tsc --noEmit -p tsconfig.test.json",
          dev: "bun --bun vite",
          test: "solid-gpui test",
          build: "bun --bun vite build",
          preview: "solid-gpui preview",
          package: "solid-gpui package --name " + name,
        },
        dependencies: { "@solid-gpui/core": coreDependency, "solid-js": "1.9.15" },
        devDependencies: {
          "@solid-gpui/vite": viteDependency,
          vite: "8.2.2",
          typescript: "7.0.2",
          "bun-types": "1.4.2",
        },
      },
      null,
      2,
    ) + "\n",
  );
  await write(
    "vite.config.ts",
    `import { defineConfig } from "vite";\nimport { solidGpui } from "@solid-gpui/vite";\n${options.native ? "" : 'import { stockHost } from "@solid-gpui/vite/delivery";\n'}\nexport default defineConfig(({ mode }) => ({\n  plugins: [solidGpui({ entry: "src/app.tsx", runtime: "${runtime}", ${options.native ? `native: { manifestPath: "native/Cargo.toml", bin: "${name}", locked: false, profile: mode === "production" ? "release" : "dev" }` : "host: stockHost(import.meta.dirname)"} })],\n}));\n`,
  );
  await write(
    "tsconfig.json",
    JSON.stringify(
      {
        extends: "./.solid-gpui/tsconfig.json",
        compilerOptions: { strict: true, skipLibCheck: true },
        include: ["src", "vite.config.ts"],
        exclude: ["src/**/*.test.tsx"],
      },
      null,
      2,
    ) + "\n",
  );
  await write(
    "tsconfig.test.json",
    JSON.stringify(
      {
        extends: "./tsconfig.json",
        compilerOptions: { types: ["bun-types"], lib: ["ES2024"] },
        include: ["src/**/*.test.tsx"],
        exclude: [],
      },
      null,
      2,
    ) + "\n",
  );
  await write(".gitignore", "node_modules/\n.solid-gpui/\n.generated/\ndist/\npackages/\ntarget/\n");
  await write(
    "LICENSE",
    "Copyright (c) application contributors\n\nSelect your application's distribution license before publishing.\n",
  );
  await write(
    "THIRD-PARTY-NOTICES.md",
    "# Application dependency notices\n\nSolid GPUI and SolidJS are MIT licensed. Complete the notices and license texts for your application's resolved native and JavaScript dependencies before redistribution.\n",
  );
  await write(
    "src/counter.tsx",
    `import { Pressable, Text, TextInput, View } from "@solid-gpui/core";\nimport { createSignal } from "@solid-gpui/core/runtime";\n\nexport function Counter() {\n  const [count, setCount] = createSignal(0);\n  const [text, setText] = createSignal("Hello 🙂");\n  return <View style={{ padding: 24, gap: 12 }}>\n    <Text>Count: {count()}</Text>\n    <Pressable accessibilityLabel="Increment" onPress={() => setCount(count() + 1)}><Text>Increment</Text></Pressable>\n    <TextInput value={text()} onChangeText={setText} />\n    <Text>{text()}</Text>\n  </View>;\n}\n`,
  );
  await write(
    "src/counter.test.tsx",
    `import { expect, test } from "bun:test";\nimport { createRoot } from "@solid-gpui/core";\nimport { TestHost } from "@solid-gpui/core/testing";\nimport { Counter } from "./counter.tsx";\n\ntest("counter and Unicode edits publish reactive content", () => {\n  const host = new TestHost();\n  const root = createRoot(host.transport, { surfaceId: 1 });\n  try {\n    root.render(() => <Counter />);\n    host.dispatch(host.surface(1)!.nodes.find(node => node.kind === "Pressable")!, { type: "press" });\n    expect(host.surface(1)!.nodes.map(node => node.text ?? "").join("")).toContain("Count: 1");\n    host.dispatch(host.surface(1)!.nodes.find(node => node.kind === "TextInput")!, { type: "input", text: "新值🙂", selectionStart: 4, selectionEnd: 4 });\n    expect(host.surface(1)!.nodes.some(node => node.text === "新值🙂")).toBe(true);\n  } finally { root.unmount(); }\n});\n`,
  );
  await write(
    "src/app.tsx",
    `import { mountApplication } from "@solid-gpui/core";\nimport { ${runtime === "quickjs" ? "EmbeddedTransport" : "StdioTransport"} } from "@solid-gpui/core/${runtime === "quickjs" ? "embedded" : "stdio"}";\nimport { Counter } from "./counter.tsx";\n${options.native ? 'import { useNative } from "#native";\nimport { onMount } from "@solid-gpui/core/runtime";\n' : ""}\nmountApplication({\n  transport: () => new ${runtime === "quickjs" ? "EmbeddedTransport" : "StdioTransport"}(),\n  setup: () => ({ render: () => { ${options.native ? "const native = useNative(); onMount(async () => console.log(await native.greeting())); " : ""}return <Counter />; } }),\n});\n`,
  );
  await write(
    "README.md",
    `# ${name}\n\nSDK ${version}; runtime ${runtime}; ${options.native ? "application-owned Rust services" : "stock host and TypeScript composition"}.\n\nRun \`bun install\`, ${options.native ? "" : "then `bun run host:install`, "}then \`bun run generate\`, \`bun run typecheck\`, \`bun run dev\`. Production: \`bun run build\`, \`bun run preview\`, \`bun run package\`. Commit bun.lock and ${options.native ? "Cargo.lock after first resolution; set native.locked to true afterward. The paired SDK source lives in .solid-gpui/sdk and should be preserved or reacquired from the exact manifest. " : "the exact delivery manifest for offline acquisition. "}Installation has no native build hooks. ${runtime === "bun" ? "The portable package requires Bun 1.4.2+ on the destination PATH; embedded Bun remains a separate experimental packager." : "The portable package runs QuickJS inside its native host and requires no Bun on the destination."}\n\nPackaging verifies an extracted archive and its native contract headlessly; qualify native display/input and complete license notices and signing before distribution.\n`,
  );
}
