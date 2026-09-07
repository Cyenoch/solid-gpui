#!/usr/bin/env bun

import { mkdtemp, mkdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const corePackageDir = join(repoRoot, "packages/solid-gpui");
const routerPackageDir = join(repoRoot, "packages/solid-gpui-router");

async function run(command: readonly string[], cwd: string): Promise<void> {
  console.error(`\n$ ${command.join(" ")}`);
  const child = Bun.spawn([...command], {
    cwd,
    env: process.env,
    stdio: ["inherit", "inherit", "inherit"],
  });
  const exitCode = await child.exited;
  if (exitCode !== 0 || child.signalCode !== null) {
    throw new Error(`package consumer command failed with exit code ${exitCode}: ${command.join(" ")}`);
  }
}

const coreRuntimeSource = `import { MemoryTransport, Text, View, createRoot } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";
import type { JSX } from "@solid-gpui/core/jsx-runtime";

const emptyElement: JSX.Element = null;
void emptyElement;
let increment: (() => void) | undefined;
function App() {
  const [count, setCount] = createSignal(0);
  increment = () => setCount((value) => value + 1);
  return createComponent(View, {
    children: createComponent(Text, { children: () => "Count: " + count() }),
  });
}

const transport = new MemoryTransport();
const root = createRoot(transport);
function submittedFrameCount(): number {
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
import { solidGpui } from "@solid-gpui/core/vite";
import { startDev } from "@solid-gpui/core/vite/dev";
import { buildApplication } from "@solid-gpui/core/vite/build";

if (typeof StdioTransport !== "function" || typeof EmbeddedTransport !== "function")
  throw new Error("packed runtime transports are missing");
const plugin = solidGpui({ entry: "app.tsx", native: { manifestPath: "native-host/Cargo.toml" } });
if (plugin.name !== "solid-gpui" || typeof startDev !== "function" || typeof buildApplication !== "function")
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
      "package/src/vite/index.ts",
      "package/src/vite/dev.ts",
      "package/src/vite/transform.ts",
      "package/src/vite/build.ts",
      "package/src/vite/quickjs-platform.ts",
      "package/src/vite/quickjs-platform-types.d.ts",
      "package/src/vite/quickjs-abort.ts",
      "package/src/vite/quickjs-headers.js",
      "package/src/vite/native-export.ts",
    ],
  },
  {
    name: "router",
    directory: routerPackageDir,
    archiveName: "solid-gpui-router.tgz",
    requiredEntries: ["package/package.json", "package/LICENSE", "package/dist/index.js", "package/dist/index.d.ts"],
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
    } else {
      if (manifest.dependencies?.["@tanstack/history"] === undefined)
        throw new Error("packed router has no history dependency");
      if (manifest.dependencies?.["@tanstack/router-core"] === undefined)
        throw new Error("packed router has no router-core dependency");
      if (manifest.peerDependencies?.["@solid-gpui/core"] === undefined)
        throw new Error("packed router has no core peer dependency");
      if (manifest.peerDependencies?.["solid-js"] === undefined)
        throw new Error("packed router has no Solid peer dependency");
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
    Bun.write(join(consumerDir, "core-runtime.ts"), coreRuntimeSource),
    Bun.write(join(consumerDir, "view.tsx"), viewSource),
    Bun.write(join(consumerDir, "router.ts"), routerSource),
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
  await run(["bun", "--conditions=browser", "run", "core-runtime.ts"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "router.ts"], consumerDir);
  await run(["bun", "--conditions=browser", "run", "native.ts"], consumerDir);
  for (const runtime of ["bun", "quickjs"]) {
    await run(
      ["bun", "node_modules/.bin/solid-gpui-build", "--runtime", runtime, "view.tsx", `${runtime}-app.js`],
      consumerDir,
    );
  }
  await run(
    ["bun", "node_modules/.bin/solid-gpui-build", "--runtime", "bun", "router.ts", "bundled-router.js"],
    consumerDir,
  );
  await run(["bun", "bundled-router.js"], consumerDir);
  const typecheck = [
    "bunx",
    "--no-install",
    "tsc",
    "--noEmit",
    "--strict",
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
  await run([...typecheck, "core-runtime.ts"], consumerDir);
  await run([...typecheck, "router.ts"], consumerDir);
  await run([...typecheck, "native.ts"], consumerDir);
  await run([...typecheck, "--jsx", "preserve", "core-runtime.ts", "view.tsx"], consumerDir);
  console.log("core and router package tarball consumer smoke passed");
} finally {
  await rm(temporaryDir, { recursive: true, force: true });
}
