import { expect, test } from "bun:test";
import { mkdtemp, mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { createServer } from "vite";
import { generateRoutes } from "../src/generator";
import { solidGpuiRouter } from "../src/vite";

const packageRoot = resolve(import.meta.dirname, "..");
const repoRoot = resolve(packageRoot, "../..");
const runtime = resolve(packageRoot, "src/index.ts");
const route = (id: string, options = "{}") =>
  `import { createFileRoute } from "@solid-gpui/router";\nexport const Route = createFileRoute(${JSON.stringify(id)})(${options});\n`;

async function fixture() {
  const root = await mkdtemp(resolve(packageRoot, ".file-routing-"));
  await mkdir(resolve(root, "src/routes"), { recursive: true });
  await writeFile(
    resolve(root, "src/routes/__root.ts"),
    `import { createRootRouteWithContext } from "@solid-gpui/router";
export const Route = createRootRouteWithContext<{ user: string }>()({});`,
  );
  return root;
}

test("generated native routes preserve nesting, loader types, lazy loading and window isolation", async () => {
  const root = await fixture();
  try {
    const files = {
      "index.ts": route("/"),
      "_app.ts": route("/_app"),
      "_app.posts.ts": route("/_app/posts"),
      "_app.posts.index.ts": route("/_app/posts/"),
      "_app.posts.$postId.ts":
        route(
          "/stale",
          `{
        validateSearch: (search: Record<string, unknown>) => ({ page: Number(search.page ?? 1) }),
        loaderDeps: ({ search }) => ({ page: search.page }),
        loader: ({ params, context, deps }) => ({ title: context.user + ":" + params.postId, page: deps.page }),
      }`,
        ) + `export const example = "@tanstack/solid-router";\nexport const nativeExample = "@solid-gpui/router";\n`,
      "_app.posts.$postId.lazy.ts": `import { createLazyFileRoute } from "@solid-gpui/router";
export const Route = createLazyFileRoute("/_app/posts/$postId")({ component: () => null });`,
      "files.$.ts": route("/files/$"),
      "-ignored.ts": "export const ignored = true;",
    };
    for (const [name, source] of Object.entries(files)) await writeFile(resolve(root, "src/routes", name), source);
    await generateRoutes({ root });
    const generated = await readFile(resolve(root, "src/routeTree.gen.ts"), "utf8");
    expect(generated).toContain('declare module "@solid-gpui/router"');
    expect(generated).not.toContain("@tanstack/solid-router");
    expect(generated).not.toContain("ignored");
    const updatedRoute = await readFile(resolve(root, "src/routes/_app.posts.$postId.ts"), "utf8");
    expect(updatedRoute).toContain('createFileRoute("/_app/posts/$postId")');
    expect(updatedRoute).toContain('example = "@tanstack/solid-router"');
    expect(updatedRoute).toContain('nativeExample = "@solid-gpui/router"');
    await generateRoutes({ root });
    expect(await readFile(resolve(root, "src/routes/_app.posts.$postId.ts"), "utf8")).toBe(updatedRoute);
    expect(await readFile(resolve(root, "src/routeTree.gen.ts"), "utf8")).toBe(generated);

    await writeFile(
      resolve(root, "src/consumer.ts"),
      `
import { createFileRoute, createRouter, useMatch, useNavigate, Link } from "@solid-gpui/router";
import { routeTree } from "./routeTree.gen";
import { Route as postRoute } from "./routes/_app.posts.$postId";
import { Route as lazyRoute } from "./routes/_app.posts.$postId.lazy";
const router = createRouter({ routeTree, context: { user: "Ada" }, initialEntries: ["/posts/42?page=3"] });
declare module "@solid-gpui/router" { interface Register { router: typeof router } }
function typeContracts() {
  const match = useMatch({ from: "/_app/posts/$postId" });
  const navigate = useNavigate();
  navigate({ to: "/posts/$postId", params: { postId: "42" }, search: { page: 3 } });
  Link({ to: "/posts/$postId", params: { postId: "42" }, search: { page: 3 }, children: "Open" });
  // @ts-expect-error Native links require the generated dynamic params.
  Link({ to: "/posts/$postId", search: { page: 3 }, children: "Open" });
  // @ts-expect-error Hook navigation requires the generated dynamic params.
  navigate({ to: "/posts/$postId", search: { page: 3 } });
  const title: string = match().loaderData!.title;
  const page: number = match().search.page;
  const id: string = match().params.postId;
  // @ts-expect-error Unknown files cannot be registered.
  createFileRoute("/missing")({});
  // @ts-expect-error Dynamic destinations require params.
  router.navigate({ to: "/posts/$postId", search: { page: 3 } });
  // @ts-expect-error Params retain their inferred type.
  const invalidParam: number = match().params.postId;
  // @ts-expect-error Loader results retain their inferred type.
  const invalidData: number = match().loaderData!.title;
  return { title, page, id, invalidParam, invalidData };
}
await router.load();
const ids = router.state.matches.map(match => match.routeId);
if (ids.join(",") !== "__root__,/_app,/_app/posts,/_app/posts/$postId") throw new Error("Wrong route nesting: " + ids);
const data = router.state.matches.at(-1)?.loaderData;
if (JSON.stringify(data) !== JSON.stringify({ title: "Ada:42", page: 3 })) throw new Error("Wrong loader data");
if (postRoute.options.component !== lazyRoute.options.component) throw new Error("Lazy component not loaded");
const second = createRouter({ routeTree, context: { user: "Grace" } });
await second.load();
await router.navigate({ to: "/files/$", params: { _splat: "one/two" } });
if (router.state.matches.find(match => match.routeId === "/files/$")?.params._splat !== "one/two") throw new Error("Splat not matched");
if (second.state.location.pathname !== "/") throw new Error("Shared window history");
`,
    );
    await writeFile(
      resolve(root, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "Bundler",
          strict: true,
          skipLibCheck: true,
          noEmit: true,
          paths: { "@solid-gpui/router": [runtime] },
        },
        include: ["src/**/*.ts"],
      }),
    );
    const checker = Bun.spawn([resolve(repoRoot, "node_modules/.bin/tsc"), "-p", resolve(root, "tsconfig.json")], {
      stdout: "pipe",
      stderr: "pipe",
    });
    const output = (await new Response(checker.stdout).text()) + (await new Response(checker.stderr).text());
    expect({ exitCode: await checker.exited, output }).toEqual({ exitCode: 0, output: "" });
    const bundle = await Bun.build({
      entrypoints: [resolve(root, "src/consumer.ts")],
      outdir: resolve(root, "dist"),
      target: "bun",
      conditions: ["browser"],
      plugins: [
        {
          name: "router-source",
          setup(build) {
            build.onResolve({ filter: /^@solid-gpui\/router$/ }, () => ({ path: runtime }));
          },
        },
      ],
    });
    expect(bundle.success).toBe(true);
    const child = Bun.spawn(["bun", "--conditions=browser", resolve(root, "dist/consumer.js")], {
      stdout: "pipe",
      stderr: "pipe",
    });
    const errors = await new Response(child.stderr).text();
    expect({ exitCode: await child.exited, errors }).toEqual({ exitCode: 0, errors: "" });
    await writeFile(resolve(root, "src/routes/index.ts"), "export const Route = createFileRoute(");
    await expect(generateRoutes({ root })).rejects.toThrow();
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}, 15_000);

test("Vite generates before resolving modules and tracks route additions, renames and deletions", async () => {
  const root = await fixture();
  let server: Awaited<ReturnType<typeof createServer>> | undefined;
  try {
    let ready: Promise<void> = Promise.resolve();
    server = await createServer({
      configFile: false,
      root,
      plugins: [
        solidGpuiRouter(),
        {
          name: "watcher-ready",
          configureServer(server) {
            ready = new Promise((done) => server.watcher.once("ready", done));
          },
        },
      ],
      server: { middlewareMode: true, ws: false },
      logLevel: "silent",
    });
    const tree = resolve(root, "src/routeTree.gen.ts");
    expect(await readFile(tree, "utf8")).toContain("rootRouteImport");
    await server.transformRequest("/src/routeTree.gen.ts");
    // Let Vite finish its initial watcher scan before creating a route.
    await ready;
    const first = resolve(root, "src/routes/first.ts");
    const second = resolve(root, "src/routes/second.ts");
    await writeFile(first, "");
    await waitFor(async () => (await readFile(tree, "utf8")).includes('"/first"'));
    expect(await readFile(first, "utf8")).toContain("@solid-gpui/router");
    await rename(first, second);
    await waitFor(async () => {
      const source = await readFile(tree, "utf8");
      return source.includes('"/second"') && !source.includes('"/first"');
    });
    expect(await readFile(second, "utf8")).toContain('createFileRoute("/second")');
    await rm(second);
    await waitFor(async () => !(await readFile(tree, "utf8")).includes('"/second"'));
  } finally {
    await server?.close();
    await rm(root, { recursive: true, force: true });
  }
}, 15_000);

async function waitFor(condition: () => Promise<boolean>) {
  const deadline = Date.now() + 5_000;
  while (!(await condition())) {
    if (Date.now() > deadline) throw new Error("Route generation did not settle");
    await Bun.sleep(25);
  }
}
