import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { mkdtemp, mkdir, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { resolveConfig } from "vite";
import {
  doctor,
  type DoctorCheck,
  type DoctorCheckId,
  type DoctorReport,
} from "../packages/solid-gpui-vite/src/doctor";
import { pluginSourceMappings, sdkSource, solidGpuiSource } from "../packages/solid-gpui-vite/src/source";

const repo = resolve(import.meta.dir, "..");
const packages = [
  ["@solid-gpui/core", "solid-gpui"],
  ["@solid-gpui/vite", "solid-gpui-vite"],
  ["@solid-gpui/router", "solid-gpui-router"],
  ["@solid-gpui/shiki", "solid-gpui-shiki"],
] as const;
const PLATFORM_PATCH: Partial<Record<NodeJS.Platform, string>> = {
  darwin: "gpui-pre-macos",
  win32: "gpui-pre-windows",
  linux: "gpui-pre-linux",
};

/** A consumer layout: the SDK packages installed under node_modules, nothing else. */
async function createConsumer(prefix: string, dependencies: readonly string[]): Promise<string> {
  const directory = await mkdtemp(join(tmpdir(), prefix));
  const scope = join(directory, "node_modules/@solid-gpui");
  await mkdir(scope, { recursive: true });
  for (const [name, folder] of packages) {
    await symlink(join(repo, "packages", folder), join(scope, name.slice("@solid-gpui/".length)), "dir");
  }
  await writeFile(
    join(directory, "package.json"),
    JSON.stringify(
      {
        name: "source-doctor-fixture",
        private: true,
        type: "module",
        devDependencies: Object.fromEntries(dependencies.map((name) => [name, "0.3.0"])),
      },
      null,
      2,
    ),
  );
  return directory;
}

function checkOf(report: DoctorReport, id: DoctorCheckId): DoctorCheck {
  const entry = report.checks.find((candidate) => candidate.id === id);
  if (entry === undefined)
    throw new Error(`no ${id} check in ${report.checks.map((candidate) => candidate.id).join(", ")}`);
  return entry;
}

/** The SDK's own `[patch.crates-io]` block, with paths re-rooted at the checkout. */
async function sdkPatches(): Promise<string> {
  const manifest = Bun.TOML.parse(await Bun.file(join(repo, "Cargo.toml")).text()) as {
    patch: { "crates-io": Record<string, { path: string }> };
  };
  return Object.entries(manifest.patch["crates-io"])
    .map(([name, spec]) => `${name} = { path = "${resolve(repo, spec.path)}" }`)
    .join("\n");
}

test("source mappings come from the installed packages' own exports, never a hand-maintained table", async () => {
  if (process.platform === "win32") return;
  const directory = await createConsumer("solid-source-", ["@solid-gpui/core", "@solid-gpui/vite"]);
  try {
    const source = sdkSource({ root: directory });
    const entries = source.packages.flatMap((pkg) => pkg.entries);
    const specifiers = entries.map((entry) => entry.specifier);
    expect(specifiers).toContain("@solid-gpui/core");
    expect(specifiers).toContain("@solid-gpui/core/runtime");
    expect(specifiers).toContain("@solid-gpui/core/quickjs");
    expect(specifiers).toContain("@solid-gpui/router/vite");
    expect(specifiers).toContain("@solid-gpui/vite/test");
    expect(specifiers).toContain("@solid-gpui/shiki/worker");
    // The solidGpui plugin remaps this one to the selected host's generated bindings.
    expect(specifiers).not.toContain("@solid-gpui/core/components");
    for (const entry of entries) expect(existsSync(entry.source)).toBe(true);
    expect(source.paths["@solid-gpui/core/runtime"]).toBe(join(repo, "packages/solid-gpui/src/runtime.ts"));
    // Vite's alias matching treats `find` as a path prefix, so subpaths must come first.
    const aliasOrder = source.aliases.map((alias) => alias.find);
    expect(aliasOrder.indexOf("@solid-gpui/core/runtime")).toBeLessThan(aliasOrder.indexOf("@solid-gpui/core"));

    const plugin = solidGpuiSource({ root: directory });
    expect(pluginSourceMappings(plugin)).toEqual(source.paths);
    const resolved = await resolveConfig({ configFile: false, root: directory, plugins: [plugin] }, "build");
    expect(resolved.resolve.conditions).toContain("solid-gpui-source");
    expect(resolved.resolve.dedupe).toContain("solid-js");
    expect(resolved.optimizeDeps.exclude).toContain("@solid-gpui/core");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("doctor reports the missing platform patches and the QuickJS profile the host build needs", async () => {
  if (process.platform === "win32") return;
  const directory = await createConsumer("solid-doctor-", ["@solid-gpui/vite"]);
  const native = { manifestPath: "crates/host/Cargo.toml", output: ".generated/native.ts" };
  try {
    await mkdir(join(directory, "crates/host/src"), { recursive: true });
    await writeFile(join(directory, "crates/host/src/main.rs"), "fn main() {}\n");
    await writeFile(
      join(directory, "crates/host/Cargo.toml"),
      `[package]\nname = "host"\nversion = "0.0.0"\nedition = "2024"\n\n[dependencies]\nsolid-gpui = { path = "${join(repo, "crates/solid-gpui")}", features = ["quickjs"] }\n`,
    );
    await writeFile(join(directory, "Cargo.toml"), '[workspace]\nresolver = "2"\nmembers = ["crates/host"]\n');

    const broken = await doctor({ root: directory, config: { runtime: "quickjs", native } });
    expect(broken.status).toBe("fail");
    expect(checkOf(broken, "solid-instance").status).toBe("pass");
    const patches = checkOf(broken, "cargo-patches");
    expect(patches.status).toBe("fail");
    expect(patches.hint).toContain("[patch.crates-io]");
    const platformPatch = PLATFORM_PATCH[process.platform];
    if (platformPatch !== undefined) expect(patches.summary).toContain(platformPatch);
    const profiles = checkOf(broken, "cargo-profiles");
    expect(profiles.status).toBe("fail");
    expect(profiles.hint).toContain("[profile.dev.package.rquickjs-sys]");
    expect(profiles.hint).toContain("opt-level = 3");
    // Profiles do not inherit from another manifest, so the report must ask for them here.
    expect(profiles.hint).toContain("do not inherit");
    const host = checkOf(broken, "host-package");
    expect(host.status).toBe("fail");
    expect(host.hint).toContain("bun add @solid-gpui/core");

    await writeFile(
      join(directory, "Cargo.toml"),
      `[workspace]\nresolver = "2"\nmembers = ["crates/host"]\n\n[profile.dev.package.rquickjs-sys]\nopt-level = 3\n\n[profile.dev.package.gpui-pre]\nopt-level = 3\n\n[profile.dev.package.taffy]\nopt-level = 3\n\n[profile.dev.package.slotmap]\nopt-level = 3\n\n[patch.crates-io]\n${await sdkPatches()}\n`,
    );
    await writeFile(
      join(directory, "package.json"),
      JSON.stringify(
        {
          name: "source-doctor-fixture",
          private: true,
          type: "module",
          dependencies: { "@solid-gpui/core": "0.3.0", "@solid-gpui/vite": "0.3.0" },
        },
        null,
        2,
      ),
    );
    await mkdir(join(directory, ".generated"), { recursive: true });
    await writeFile(join(directory, ".generated/native.ts"), "export const answer = 42;\n");
    const fixed = await doctor({ root: directory, config: { runtime: "quickjs", native } });
    expect(checkOf(fixed, "cargo-patches").status).toBe("pass");
    expect(checkOf(fixed, "cargo-profiles").status).toBe("pass");
    expect(checkOf(fixed, "host-package").status).toBe("pass");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}, 60_000);

test("doctor catches source mappings that disagree with the runtime or with TypeScript", async () => {
  if (process.platform === "win32") return;
  const directory = await createConsumer("solid-doctor-source-", ["@solid-gpui/core", "@solid-gpui/vite"]);
  const configFile = join(directory, "vite.config.ts");
  const generated = join(directory, ".solid-gpui/tsconfig.json");
  try {
    await mkdir(join(directory, ".solid-gpui"), { recursive: true });
    await writeFile(generated, JSON.stringify({ compilerOptions: { customConditions: ["solid-gpui-source"] } }));
    await writeFile(configFile, "export default { plugins: [] };\n");
    const distRuntime = await doctor({ root: directory, config: { runtime: "bun" } });
    expect(distRuntime.mode).toBe("source");
    expect(checkOf(distRuntime, "source-mapping").status).toBe("fail");
    expect(checkOf(distRuntime, "source-mapping").hint).toContain("solidGpuiSource");

    await writeFile(
      configFile,
      'import { solidGpuiSource } from "@solid-gpui/vite/source";\nexport default { plugins: [solidGpuiSource()] };\n',
    );
    const agreed = await doctor({ root: directory, config: { runtime: "bun" } });
    expect(agreed.mode).toBe("source");
    expect(checkOf(agreed, "source-mapping").status).toBe("pass");

    await rm(generated);
    await writeFile(join(directory, "tsconfig.json"), JSON.stringify({ compilerOptions: { customConditions: [] } }));
    const sourceRuntime = await doctor({ root: directory, config: { runtime: "bun" } });
    expect(checkOf(sourceRuntime, "source-mapping").status).toBe("fail");
    expect(checkOf(sourceRuntime, "source-mapping").hint).toContain("customConditions");

    // The other source route: prepare writes the SDK specifier -> source file table.
    await writeFile(
      generated,
      JSON.stringify({
        compilerOptions: { paths: { "@solid-gpui/core": [join(repo, "packages/solid-gpui/src/index.ts")] } },
      }),
    );
    const viaPaths = await doctor({ root: directory, config: { runtime: "bun" } });
    expect(viaPaths.mode).toBe("source");
    expect(checkOf(viaPaths, "source-mapping").status).toBe("pass");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("a prepared dist consumer keeps host bindings without looking like a source consumer", async () => {
  if (process.platform === "win32") return;
  const directory = await createConsumer("solid-doctor-dist-", ["@solid-gpui/core", "@solid-gpui/vite"]);
  const bindings = join(directory, ".generated/native.ts");
  try {
    await mkdir(join(directory, ".solid-gpui"), { recursive: true });
    await mkdir(join(directory, ".generated"), { recursive: true });
    await mkdir(join(directory, "src"), { recursive: true });
    await writeFile(bindings, "export const answer = 42;\n");
    // Exactly what `solid-gpui prepare` writes when no source plugin is configured: the host
    // bindings are mapped, but nothing maps an SDK specifier to an SDK source file.
    await writeFile(
      join(directory, ".solid-gpui/tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            target: "ES2024",
            module: "preserve",
            moduleResolution: "bundler",
            jsx: "preserve",
            jsxImportSource: "@solid-gpui/core",
            noEmit: true,
            allowImportingTsExtensions: true,
            paths: { "#native": [bindings], "@solid-gpui/core/components": [bindings] },
          },
        },
        null,
        2,
      ),
    );
    await writeFile(
      join(directory, "tsconfig.json"),
      JSON.stringify(
        {
          extends: "./.solid-gpui/tsconfig.json",
          compilerOptions: { strict: true, skipLibCheck: true, types: [] },
          include: ["src"],
        },
        null,
        2,
      ),
    );
    await writeFile(
      join(directory, "src/main.ts"),
      'import { answer } from "#native";\nimport { answer as bound } from "@solid-gpui/core/components";\nimport { Text, View } from "@solid-gpui/core";\nimport { createComponent } from "@solid-gpui/core/runtime";\n\nexport const surface = [answer, bound, Text, View, createComponent] as const;\n',
    );
    await writeFile(
      join(directory, "vite.config.ts"),
      'import { solidGpui } from "@solid-gpui/vite";\n\nexport default { plugins: [solidGpui({ entry: "src/main.tsx" })] };\n',
    );

    const report = await doctor({ root: directory, config: { runtime: "bun" } });
    expect(report.mode).toBe("dist");
    expect(checkOf(report, "source-mapping").status).toBe("pass");

    const typecheck = Bun.spawnSync(["bunx", "tsc", "--noEmit", "--project", join(directory, "tsconfig.json")], {
      cwd: repo,
      stdout: "pipe",
      stderr: "pipe",
    });
    expect(`${typecheck.exitCode}: ${typecheck.stdout.toString().trim()}`).toBe("0: ");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("doctor refuses a cross-target export without an explicitly supplied host exporter", async () => {
  const directory = await createConsumer("solid-doctor-target-", ["@solid-gpui/core", "@solid-gpui/vite"]);
  try {
    await writeFile(join(directory, "Cargo.toml"), '[workspace]\nresolver = "2"\nmembers = []\n');
    const target = "x86_64-pc-windows-msvc";
    const cross = await doctor({
      root: directory,
      config: { runtime: "bun", native: { manifestPath: "Cargo.toml", output: ".generated/native.ts", target } },
    });
    const refused = checkOf(cross, "runtime-capabilities");
    expect(refused.status).toBe("fail");
    expect(refused.summary).toContain(target);
    expect(refused.hint).toContain("native.exporter");

    const exported = await doctor({
      root: directory,
      config: {
        runtime: "bun",
        native: { manifestPath: "Cargo.toml", output: ".generated/native.ts", target, exporter: { command: "true" } },
      },
    });
    expect(checkOf(exported, "runtime-capabilities").status).toBe("pass");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
