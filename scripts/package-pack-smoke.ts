#!/usr/bin/env bun

import { mkdtemp, mkdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const corePackageDir = join(repoRoot, "packages/solid-gpui");
const vitePackageDir = join(repoRoot, "packages/solid-gpui-vite");
const routerPackageDir = join(repoRoot, "packages/solid-gpui-router");
const shikiPackageDir = join(repoRoot, "packages/solid-gpui-shiki");

const shikiSource = `import { createRoot, MemoryTransport } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { CodeBlock } from "@solid-gpui/shiki";
import { createBunHighlighter } from "@solid-gpui/shiki/bun";
const highlighter = await createBunHighlighter({ languages: ["typescript"], themes: ["github-dark"] });
const request = { code: 'const value = "你好😀";\\r\\n', language: "typescript", theme: "github-dark" };
const transport = new MemoryTransport();
const root = createRoot(transport);
try {
  const result = await highlighter.highlight(request);
  if (result.runs.map(run => run.text).join("") !== request.code) throw new Error("Packed worker changed the source");
  root.render(() => createComponent(CodeBlock, { highlighter, ...request }));
  await highlighter.highlight(request);
  await Promise.resolve();
  if (transport.submitted.length < 2) throw new Error("Packed CodeBlock emitted no highlighted Patch");
} finally { root.unmount(); highlighter.dispose(); }
`;

async function run(command: readonly string[], cwd: string, env: Record<string, string> = {}): Promise<void> {
  console.error(`\n$ ${command.join(" ")}`);
  const child = Bun.spawn([...command], {
    cwd,
    env: { ...process.env, ...env },
    stdio: ["inherit", "inherit", "inherit"],
  });
  const exitCode = await child.exited;
  if (exitCode !== 0 || child.signalCode !== null) {
    throw new Error(`package consumer command failed with exit code ${exitCode}: ${command.join(" ")}`);
  }
}

const coreRuntimeSource = `import { MemoryTransport, Text, View, createRoot } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";
/** @type {(() => void) | undefined} */
let increment;
function App() {
  const [count, setCount] = createSignal(0);
  increment = () => setCount((value) => value + 1);
  return createComponent(View, {
    children: createComponent(Text, { children: () => "Count: " + count() }),
  });
}

const transport = new MemoryTransport();
const root = createRoot(transport);
function submittedFrameCount() {
  return transport.submitted.length;
}
root.render(() => createComponent(App, {}));
if (submittedFrameCount() !== 1) throw new Error("packed core emitted no Snapshot");
increment?.();
await Promise.resolve();
if (submittedFrameCount() !== 2) throw new Error("packed core emitted no signal Patch");
root.unmount();
`;

const viewSource = `/** @jsxImportSource @solid-gpui/core */
import { Text, View } from "@solid-gpui/core";

export const tree = (
  <View>
    <Text>Hello</Text>
  </View>
);
`;

const nativeSource = `import { createRoot, MemoryTransport } from "@solid-gpui/core";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";
import { Button, Progress } from "@solid-gpui/core/components";
import { encodeJson, decodeJson } from "@solid-gpui/core/native";
import { solidGpui } from "@solid-gpui/vite";
import { startDev } from "@solid-gpui/vite/dev";

if (typeof StdioTransport !== "function" || typeof EmbeddedTransport !== "function")
  throw new Error("packed runtime transports are missing");
const plugin = solidGpui({ entry: "app.tsx", native: { manifestPath: "native-host/Cargo.toml" } });
if (plugin.name !== "solid-gpui" || typeof startDev !== "function")
  throw new Error("packed Vite entrypoints are missing");
if (typeof plugin.config !== "function") throw new Error("packed Vite config hook is missing");
await Reflect.apply(plugin.config, {}, [{ root: process.cwd() }, { command: "build", mode: "production" }]);
if (await Bun.file(".generated/native.ts").text() !== "export const answer = 42;\\n")
  throw new Error("packed native binding formatter failed");
if (decodeJson(encodeJson("native")) !== "native") throw new Error("packed native codec failed");
const transport = new MemoryTransport();
const root = createRoot(transport);
let update: (() => void) | undefined;
root.render(() => {
  const [label, setLabel] = createSignal("before");
  update = () => setLabel("after");
  return [Button({ get label() { return label(); } }), Progress({})];
});
update?.();
await Promise.resolve();
if (transport.submitted.length !== 2) throw new Error("packed native components lost their owning root or reactive props");
root.unmount();
`;

const routerSource = `import { MemoryTransport, Text, createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import {
  RouterProvider,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  useLocation,
} from "@solid-gpui/router";

interface AppState {
  readonly loadedBy: string[];
}
interface WindowRouterContext {
  readonly app: AppState;
  readonly windowId: string;
}

const app: AppState = { loadedBy: [] };
const rootRoute = createRootRouteWithContext<WindowRouterContext>()({
  component: () => {
    const location = useLocation();
    return createComponent(Text, { children: () => location().pathname });
  },
  loader: ({ context }) => {
    context.app.loadedBy.push(context.windowId);
  },
});
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
});
const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "settings",
});
const routeTree = rootRoute.addChildren([indexRoute, settingsRoute]);
const firstRouter = createRouter({ routeTree, context: { app, windowId: "first" } });
const secondRouter = createRouter({ routeTree, context: { app, windowId: "second" } });
const firstTransport = new MemoryTransport();
const secondTransport = new MemoryTransport();
const firstRoot = createRoot(firstTransport);
const secondRoot = createRoot(secondTransport);
firstRoot.render(() => createComponent(RouterProvider, { router: firstRouter }));
secondRoot.render(() => createComponent(RouterProvider, { router: secondRouter }));
await Promise.all([firstRouter.load(), secondRouter.load()]);
await Promise.resolve();
if (!app.loadedBy.includes("first") || !app.loadedBy.includes("second")) {
  throw new Error("packed router did not share application state");
}
if (firstRouter.options.context.app !== app || secondRouter.options.context.app !== app) {
  throw new Error("packed router copied application state");
}
const firstFrameCount = firstTransport.submitted.length;
const secondFrameCount = secondTransport.submitted.length;
if (firstFrameCount === 0 || secondFrameCount === 0) throw new Error("packed router emitted no host Snapshot");
await firstRouter.navigate({ to: "/settings" });
await Promise.resolve();
if (firstRouter.latestLocation.pathname !== "/settings" || secondRouter.latestLocation.pathname !== "/") {
  throw new Error("packed router shared window navigation state");
}
if (firstTransport.submitted.length <= firstFrameCount) throw new Error("packed router emitted no navigation Patch");
if (secondTransport.submitted.length !== secondFrameCount) throw new Error("packed router updated the inactive window");
firstRoot.unmount();
secondRoot.unmount();
`;

const packageSpecs = [
  {
    name: "core",
    directory: corePackageDir,
    archiveName: "solid-gpui-core.tgz",
    requiredEntries: [
      "package/package.json",
      "package/dist/index.js",
      "package/dist/index.d.ts",
      "package/dist/runtime.js",
      "package/dist/runtime.d.ts",
      "package/dist/stdio.js",
      "package/dist/stdio.d.ts",
      "package/dist/embedded.js",
      "package/dist/embedded.d.ts",
      "package/dist/jsx-runtime.d.ts",
      "package/dist/native.js",
      "package/dist/native.d.ts",
      "package/dist/components.js",
      "package/dist/components.d.ts",
    ],
  },
  {
    name: "vite",
    directory: vitePackageDir,
    archiveName: "solid-gpui-vite.tgz",
    requiredEntries: [
      "package/package.json",
      "package/LICENSE",
      "package/README.md",
      "package/dist/index.js",
      "package/dist/index.d.ts",
      "package/dist/dev.d.ts",
      "package/dist/dev.js",
      "package/dist/transform.js",
      "package/dist/environment.js",
      "package/dist/runner.js",
      "package/dist/quickjs-build.js",
      "package/dist/quickjs-dev.js",
      "package/dist/quickjs-platform.js",
      "package/dist/quickjs-abort.js",
      "package/dist/quickjs-headers.js",
      "package/dist/native-export.js",
    ],
  },
  {
    name: "router",
    directory: routerPackageDir,
    archiveName: "solid-gpui-router.tgz",
    requiredEntries: [
      "package/package.json",
      "package/LICENSE",
      "package/dist/index.js",
      "package/dist/index.d.ts",
      "package/dist/generator.js",
      "package/dist/generator.d.ts",
      "package/dist/generator-process.js",
      "package/dist/generator-engine.d.ts",
      "package/dist/vite.js",
      "package/dist/vite.d.ts",
    ],
  },
  {
    name: "shiki",
    directory: shikiPackageDir,
    archiveName: "solid-gpui-shiki.tgz",
    requiredEntries: [
      "package/package.json",
      "package/LICENSE",
      "package/README.md",
      "package/dist/index.js",
      "package/dist/index.d.ts",
      "package/dist/bun.js",
      "package/dist/bun.d.ts",
      "package/dist/worker.js",
    ],
  },
] as const;

const temporaryDir = await mkdtemp(join(tmpdir(), "solid-gpui-pack-smoke-"));
const archivePaths = Object.fromEntries(
  packageSpecs.map((spec) => [spec.name, join(temporaryDir, spec.archiveName)]),
) as Record<(typeof packageSpecs)[number]["name"], string>;
const consumerDir = join(temporaryDir, "consumer");

try {
  for (const spec of packageSpecs) {
    const archivePath = archivePaths[spec.name];
    await run(["bun", "pm", "pack", "--dry-run", "--quiet"], spec.directory);
    await run(["bun", "pm", "pack", "--filename", archivePath, "--quiet"], spec.directory);
    const archive = new Bun.Archive(await Bun.file(archivePath).bytes());
    const files = await archive.files();
    for (const entry of spec.requiredEntries) {
      if (!files.has(entry)) throw new Error(`missing ${entry} in packed ${spec.name}`);
    }
    const packageJson = files.get("package/package.json");
    if (packageJson === undefined) throw new Error(`missing package manifest in packed ${spec.name}`);
    const manifest = JSON.parse(await packageJson.text()) as {
      readonly dependencies?: Record<string, string>;
      readonly peerDependencies?: Record<string, string>;
    };
    if (spec.name === "core") {
      if (manifest.dependencies?.["solid-js"] !== undefined) throw new Error("packed core bundles a Solid dependency");
      if (manifest.peerDependencies?.["solid-js"] === undefined)
        throw new Error("packed core has no Solid peer dependency");
    } else if (spec.name === "router") {
      if (manifest.dependencies?.["@tanstack/history"] === undefined)
        throw new Error("packed router has no history dependency");
      if (manifest.dependencies?.["@tanstack/router-core"] === undefined)
        throw new Error("packed router has no router-core dependency");
      if (manifest.peerDependencies?.["@solid-gpui/core"] === undefined)
        throw new Error("packed router has no core peer dependency");
      if (manifest.peerDependencies?.["solid-js"] === undefined)
        throw new Error("packed router has no Solid peer dependency");
    } else if (spec.name === "vite") {
      if (!manifest.peerDependencies?.["@solid-gpui/core"] || !manifest.peerDependencies?.vite)
        throw new Error("packed Vite package is missing its peers");
    } else {
      if (manifest.dependencies?.shiki !== "4.4.2") throw new Error("packed Shiki dependency is not pinned");
      if (!manifest.peerDependencies?.["@solid-gpui/core"] || !manifest.peerDependencies?.["solid-js"])
        throw new Error("packed Shiki package is missing its peers");
    }
  }

  await mkdir(consumerDir, { recursive: true });
  await Bun.write(
    join(consumerDir, "package.json"),
    `${JSON.stringify(
      {
        name: "solid-gpui-pack-consumer",
        private: true,
        type: "module",
        dependencies: {
          "@solid-gpui/core": "file:" + archivePaths.core,
          "@solid-gpui/router": "file:" + archivePaths.router,
          "@solid-gpui/shiki": "file:" + archivePaths.shiki,
          "@solid-gpui/vite": "file:" + archivePaths.vite,
          "solid-js": "1.9.15",
        },
        overrides: {
          "@solid-gpui/core": "file:" + archivePaths.core,
        },
        devDependencies: { "bun-types": "1.4.2", typescript: "7.0.2", vite: "8.2.2" },
      },
      null,
      2,
    )}\n`,
  );
  await Promise.all([
    Bun.write(join(consumerDir, "core-runtime.js"), coreRuntimeSource),
    Bun.write(join(consumerDir, "view.tsx"), viewSource),
    Bun.write(join(consumerDir, "router.ts"), routerSource),
    Bun.write(
      join(consumerDir, "generate-routes.ts"),
      `import { generateRoutes } from "@solid-gpui/router/generator";
await generateRoutes({ root: import.meta.dirname });
`,
    ),
    Bun.write(
      join(consumerDir, "src/routes/__root.ts"),
      `import { createRootRoute } from "@solid-gpui/router";
export const Route = createRootRoute({});
`,
    ),
    Bun.write(
      join(consumerDir, "src/routes/index.ts"),
      `import { createFileRoute } from "@solid-gpui/router";
export const Route = createFileRoute("/")({ loader: () => ({ title: "Packed file route" }) });
`,
    ),
    Bun.write(
      join(consumerDir, "file-router.ts"),
      `import { createRouter } from "@solid-gpui/router";
import { routeTree } from "./src/routeTree.gen";
const router = createRouter({ routeTree });
await router.load();
if (router.state.matches.at(-1)?.loaderData?.title !== "Packed file route") throw new Error("Packed file routes failed");
`,
    ),
    Bun.write(join(consumerDir, "shiki.ts"), shikiSource),
    Bun.write(join(consumerDir, "native.ts"), nativeSource),
    Bun.write(
      join(consumerDir, "native-host/Cargo.toml"),
      '[package]\nname = "packed-native-host"\nversion = "0.0.0"\nedition = "2024"\n[workspace]\n',
    ),
    Bun.write(
      join(consumerDir, "native-host/src/main.rs"),
      'fn main() { assert_eq!(std::env::args().nth(1).as_deref(), Some("--export-native")); println!("export const answer=42;"); }',
    ),
  ]);

  await run(["bun", "install", "--no-progress"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "core-runtime.js"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "router.ts"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "generate-routes.ts"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "file-router.ts"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "shiki.ts"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "native.ts"], consumerDir);
  await Bun.write(
    join(consumerDir, "vite.config.ts"),
    `import { solidGpui } from '@solid-gpui/vite';
export default { plugins: [solidGpui({ entry: process.env.SOLID_GPUI_ENTRY,
runtime: process.env.SOLID_GPUI_RUNTIME, host: false })], build: { emptyOutDir: false,
rolldownOptions: { output: { entryFileNames: process.env.SOLID_GPUI_OUTPUT } } } };`,
  );
  for (const [runtime, entry, output] of [
    ["bun", "view.tsx", "bun-app.js"],
    ["quickjs", "view.tsx", "quickjs-app.js"],
    ["bun", "router.ts", "bundled-router.js"],
    ["bun", "shiki.ts", "bundled-shiki.js"],
  ]) {
    await run(["bun", "--bun", "vite", "build"], consumerDir, {
      SOLID_GPUI_RUNTIME: runtime!,
      SOLID_GPUI_ENTRY: entry!,
      SOLID_GPUI_OUTPUT: output!,
    });
  }
  await run(["bun", "dist/bundled-router.js"], consumerDir);
  await run(["bun", "dist/bundled-shiki.js"], consumerDir);
  const typecheck = [
    "bunx",
    "--no-install",
    "tsc",
    "--noEmit",
    "--strict",
    "--allowJs",
    "--checkJs",
    "--skipLibCheck",
    "--target",
    "ES2022",
    "--module",
    "ESNext",
    "--moduleResolution",
    "Bundler",
    "--types",
    "bun-types",
  ] as const;
  await run([...typecheck, "core-runtime.js"], consumerDir);
  await run([...typecheck, "router.ts"], consumerDir);
  await run([...typecheck, "shiki.ts"], consumerDir);
  await run([...typecheck, "native.ts"], consumerDir);
  await run([...typecheck, "--jsx", "preserve", "core-runtime.js", "view.tsx"], consumerDir);
  console.log("core, Vite, router and Shiki package tarball consumer smoke passed");
} finally {
  await rm(temporaryDir, { recursive: true, force: true });
}
