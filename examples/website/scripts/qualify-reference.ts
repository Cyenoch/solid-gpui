import { resolve } from "node:path";
import { createRunnableDevEnvironment, createServer, isRunnableDevEnvironment } from "vite";

const [binary, mode = "deterministic", output = ".scratch/reference-acceptance"] = Bun.argv.slice(2);
if (
  !process.execArgv.some(
    (argument) => argument.startsWith("--conditions=") && argument.slice(13).split(",").includes("browser"),
  )
)
  throw new Error(
    "Native qualification requires Bun's client Solid runtime: bun --conditions=browser scripts/qualify-reference.ts ...",
  );
if (!binary || !["deterministic", "gpu"].includes(mode))
  throw new Error("Usage: qualify-reference.ts <website-acceptance binary> [deterministic|gpu] [artifact directory]");
const root = resolve(import.meta.dirname, "..");
const server = await createServer({
  root,
  configFile: resolve(root, "vite.reference.config.ts"),
  ssr: {
    noExternal: [/^@solid-gpui\//, /^solid-js(?:\/|$)/],
    resolve: { conditions: ["solid-gpui-source", "browser", "bun"], externalConditions: ["browser", "bun"] },
  },
  server: { middlewareMode: true, hmr: false, ws: false },
  environments: {
    ssr: {
      dev: { createEnvironment: createRunnableDevEnvironment },
      resolve: {
        conditions: ["solid-gpui-source", "browser", "bun"],
        external: [],
        noExternal: [/^@solid-gpui\//, /^solid-js(?:\/|$)/],
      },
    },
  },
  logLevel: "error",
});
const deadline = setTimeout(() => {
  console.error("Reference native qualification exceeded five minutes");
  process.exit(1);
}, 300_000);
try {
  const environment = server.environments.ssr;
  if (!environment || !isRunnableDevEnvironment(environment))
    throw new Error("Missing native qualification module environment");
  const fixture = await environment.runner.import(resolve(root, "tests/reference-studio.driver.ts"));
  const report = await fixture.qualifyReferenceStudio(resolve(binary), mode, resolve(output));
  await Bun.write(resolve(output, `reference-${mode}.json`), JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));
} finally {
  clearTimeout(deadline);
  await server.close();
}
