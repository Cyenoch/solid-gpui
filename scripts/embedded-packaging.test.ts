import { expect, test } from "bun:test";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  EmbeddedPackagingError,
  assertAbortPanicStrategy,
  embeddedEntryPointIdentity,
  embeddedSourceRoot,
  embeddedWorkerIdentity,
  mergePatchTables,
  mergeProfileTables,
  packageEmbeddedApplication,
  resolveEmbeddedApplication,
  resolveEmbeddedTarget,
  runEmbeddedCommand,
  writeGeneratedApplication,
} from "../packages/solid-gpui-vite/src/embedded";

const repoRoot = resolve(import.meta.dir, "..");
const fixtureRoot = join(repoRoot, "fixtures/embedded-custom-host");

const application = {
  manifest: join(fixtureRoot, "Cargo.toml"),
  package: "embedded-custom-host",
  features: ["counter"],
  main: join(fixtureRoot, "host/embedded-main.rs"),
};

async function toml(path: string): Promise<Record<string, unknown>> {
  return Bun.TOML.parse(await readFile(path, "utf8")) as Record<string, unknown>;
}

/**
 * A stand-in for the pinned Bun checkout, which only exists after a packaging
 * run. It carries the profile shape the merge depends on: the panic strategy the
 * embedded link is built with, plus a package override.
 */
async function syntheticBunSource(directory: string): Promise<string> {
  const source = join(directory, "bun-source");
  await mkdir(join(source, "src/bun_bin"), { recursive: true });
  await writeFile(
    join(source, "Cargo.toml"),
    [
      "[workspace]",
      'resolver = "2"',
      "",
      "[profile.dev]",
      'panic = "abort"',
      "",
      "[profile.release]",
      'panic = "abort"',
      'lto = "fat"',
      "",
      "[profile.release.package.bun_react_compiler]",
      'opt-level = "s"',
      "",
    ].join("\n"),
  );
  await writeFile(join(source, "src/bun_bin/Cargo.toml"), '[package]\nname = "bun_bin"\nversion = "0.0.0"\n');
  return source;
}

test("patch tables merge by declaring root and refuse two targets for one crate", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-embedded-patch-"));
  try {
    const other = join(directory, "other");
    await mkdir(other, { recursive: true });
    const sdkDocument = await toml(join(repoRoot, "Cargo.toml"));
    const applicationDocument = await toml(join(fixtureRoot, "Cargo.toml"));

    const merged = mergePatchTables([
      { path: join(repoRoot, "Cargo.toml"), document: sdkDocument },
      { path: join(fixtureRoot, "Cargo.toml"), document: applicationDocument },
    ]);
    // The fixture patches the same crates as the SDK, relative to its own
    // manifest: both resolve to the SDK's vendored checkout, so the merge keeps
    // one entry with an absolute path instead of failing or duplicating.
    const cratesIo = merged["crates-io"]!;
    expect(cratesIo["gpui-pre"]).toEqual({ path: join(repoRoot, "vendor/gpui") });
    expect(cratesIo["gpui-pre-macos"]).toEqual({ path: join(repoRoot, "vendor/gpui-macos") });
    expect(cratesIo["gpui-pre-windows"]).toEqual({ path: join(repoRoot, "vendor/gpui-windows") });

    const conflicting = {
      patch: { "crates-io": { "gpui-pre": { path: "elsewhere/gpui" } } },
    };
    expect(() =>
      mergePatchTables([
        { path: join(repoRoot, "Cargo.toml"), document: sdkDocument },
        { path: join(other, "Cargo.toml"), document: conflicting },
      ]),
    ).toThrow(/two different targets/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("profile merge layers application over SDK over pinned Bun and keeps abort", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-embedded-profile-"));
  try {
    const bunDocument = await toml(join(await syntheticBunSource(directory), "Cargo.toml"));
    const sdkDocument = await toml(join(repoRoot, "Cargo.toml"));
    const applicationDocument = await toml(join(fixtureRoot, "Cargo.toml"));
    const merged = mergeProfileTables([bunDocument, sdkDocument, applicationDocument]);

    // The pinned base and both package-level overrides survive; the application's
    // own `debug` wins over anything below it.
    expect(merged["dev"]!["panic"]).toBe("abort");
    expect(merged["dev"]!["debug"]).toBe(1);
    expect(merged["dev"]!["package"]).toEqual({
      "gpui-pre": { "opt-level": 3 },
      taffy: { "opt-level": 3 },
      slotmap: { "opt-level": 3 },
      "rquickjs-sys": { "opt-level": 3 },
    });
    expect(merged["release"]!["lto"]).toBe("thin");
    expect((merged["release"]!["package"] as Record<string, unknown>)["bun_react_compiler"]).toEqual({
      "opt-level": "s",
    });
    expect((merged["release"]!["package"] as Record<string, unknown>)["taffy"]).toEqual({ "opt-level": 3 });

    assertAbortPanicStrategy(merged);
    expect(merged["dev"]!["panic"]).toBe("abort");
    expect(() => assertAbortPanicStrategy({ release: { panic: "unwind" } })).toThrow(/panic = "abort"/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("an application package resolves inside its own workspace with its features", async () => {
  const resolved = await resolveEmbeddedApplication(application, repoRoot);
  expect(resolved.packageDir).toBe(join(fixtureRoot, "host"));
  expect(resolved.workspaceManifestPath).toBe(join(fixtureRoot, "Cargo.toml"));
  expect(resolved.features).toEqual(["counter"]);
  expect(resolved.main).toBe(application.main);

  await expect(resolveEmbeddedApplication({ ...application, features: ["missing-feature"] }, repoRoot)).rejects.toThrow(
    /does not declare feature "missing-feature"/,
  );
});

test("a wildcard member workspace resolves an inherited SDK dependency from its root", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-embedded-workspace-"));
  try {
    const host = join(directory, "crates/host");
    await mkdir(join(host, "src"), { recursive: true });
    await writeFile(
      join(directory, "Cargo.toml"),
      [
        "[workspace]",
        'resolver = "2"',
        'members = ["crates/*"]',
        "",
        "[workspace.dependencies]",
        // Declared relative to this manifest, which is the workspace root: the
        // member below inherits it, and Cargo resolves it against the root.
        `solid-gpui = { path = ${JSON.stringify(join(repoRoot, "crates/solid-gpui"))} }`,
        "",
      ].join("\n"),
    );
    await writeFile(
      join(host, "Cargo.toml"),
      [
        "[package]",
        'name = "embedded-wildcard-host"',
        'version = "0.0.0"',
        'edition = "2024"',
        "",
        "[features]",
        "counter = []",
        "",
        "[dependencies]",
        "solid-gpui = { workspace = true }",
        "",
      ].join("\n"),
    );
    await writeFile(join(host, "src/lib.rs"), "");
    await writeFile(join(host, "embedded-main.rs"), "fn main() {}\n");

    const resolved = await resolveEmbeddedApplication(
      {
        manifest: join(directory, "Cargo.toml"),
        package: "embedded-wildcard-host",
        features: ["counter"],
        main: join(host, "embedded-main.rs"),
      },
      repoRoot,
    );
    expect(resolved.packageDir).toBe(host);
    expect(resolved.workspaceManifestPath).toBe(join(directory, "Cargo.toml"));

    // The same package with a deliberately different SDK copy is refused.
    const other = join(directory, "other/solid-gpui");
    await mkdir(join(other, "src"), { recursive: true });
    await writeFile(
      join(other, "Cargo.toml"),
      ["[package]", 'name = "solid-gpui"', 'version = "0.0.0"', 'edition = "2024"', ""].join("\n"),
    );
    await writeFile(join(other, "src/lib.rs"), "");
    await writeFile(
      join(directory, "Cargo.toml"),
      [
        "[workspace]",
        'resolver = "2"',
        'members = ["crates/*"]',
        "",
        "[workspace.dependencies]",
        `solid-gpui = { path = ${JSON.stringify(other)} }`,
        "",
      ].join("\n"),
    );
    await expect(
      resolveEmbeddedApplication(
        {
          manifest: join(directory, "Cargo.toml"),
          package: "embedded-wildcard-host",
          main: join(host, "embedded-main.rs"),
        },
        repoRoot,
      ),
    ).rejects.toThrow(/not the SDK checkout under build/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("the generated application merges the owner's manifest and includes its main", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-embedded-generate-"));
  try {
    const bunSource = await syntheticBunSource(directory);
    const nativeManifest = join(directory, "embed-native.json");
    await writeFile(
      nativeManifest,
      JSON.stringify({ objects: ["/tmp/object.o"], archives: [], linkArgs: ["-framework", "AppKit"] }),
    );
    const graphSource = join(directory, "graph/bun-embedded-graph.rs");
    await mkdir(join(directory, "graph"), { recursive: true });
    await writeFile(graphSource, 'pub const BUN_EMBEDDED_ENTRY: &str = "/$bunfs/root/app.js";\n');

    const generated = await writeGeneratedApplication({
      sdkRoot: repoRoot,
      bunSource,
      directory: join(directory, "application"),
      nativeManifest,
      graphSource,
      application,
    });

    const document = await toml(generated.cargoManifest);
    const dependencies = document["dependencies"] as Record<string, Record<string, unknown>>;
    expect(dependencies["solid_gpui"]).toEqual({
      package: "solid-gpui",
      path: join(repoRoot, "crates/solid-gpui"),
      features: ["embedded-bun"],
    });
    expect(dependencies["bun_rust"]).toEqual({
      package: "bun_bin",
      path: join(bunSource, "src/bun_bin"),
      features: ["solid-gpui-embed"],
    });
    expect(dependencies["embedded-custom-host"]).toEqual({
      path: join(fixtureRoot, "host"),
      features: ["counter"],
    });
    expect((document["patch"] as Record<string, Record<string, unknown>>)["crates-io"]!["gpui-pre"]).toEqual({
      path: join(repoRoot, "vendor/gpui"),
    });
    expect((document["profile"] as Record<string, Record<string, unknown>>)["dev"]!["panic"]).toBe("abort");
    expect(generated.profiles).toEqual(["dev", "release"]);
    expect(generated.applicationPackageDir).toBe(join(fixtureRoot, "host"));

    const main = await readFile(generated.mainSource, "utf8");
    expect(main).toContain(`include!(${JSON.stringify(graphSource)})`);
    expect(main).toContain(`include!(${JSON.stringify(application.main)})`);
    expect(main).not.toContain("static-application.rs");
    expect(await readFile(generated.buildScript, "utf8")).toContain(
      'println!("cargo:rustc-link-arg={}", "-framework")',
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("the default entry keeps the SDK application and its two pinned dependencies", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-embedded-default-"));
  try {
    const bunSource = await syntheticBunSource(directory);
    const nativeManifest = join(directory, "embed-native.json");
    await writeFile(nativeManifest, JSON.stringify({ objects: [], archives: [], linkArgs: [] }));
    const generated = await writeGeneratedApplication({
      sdkRoot: repoRoot,
      bunSource,
      directory: join(directory, "application"),
      nativeManifest,
      graphSource: join(directory, "graph.rs"),
    });
    const document = await toml(generated.cargoManifest);
    expect(Object.keys(document["dependencies"] as Record<string, unknown>).sort()).toEqual(["bun_rust", "solid_gpui"]);
    const main = await readFile(generated.mainSource, "utf8");
    expect(main).toContain(join(repoRoot, "crates/solid-gpui-bun-sys/static-application.rs"));
    expect(generated.applicationPackageDir).toBeUndefined();

    // A custom `main` over the default dependency set is what the previous
    // CLI expressed as `--main` alone, and it keeps the same two dependencies.
    const custom = await writeGeneratedApplication({
      sdkRoot: repoRoot,
      bunSource,
      directory: join(directory, "custom"),
      nativeManifest,
      graphSource: join(directory, "graph.rs"),
      main: join(repoRoot, "fixtures/embedded-static-check.rs"),
    });
    const customMain = await readFile(custom.mainSource, "utf8");
    expect(customMain).toContain(join(repoRoot, "fixtures/embedded-static-check.rs"));
    expect(customMain).not.toContain("static-application.rs");
    expect(Object.keys((await toml(custom.cargoManifest))["dependencies"] as Record<string, unknown>).sort()).toEqual([
      "bun_rust",
      "solid_gpui",
    ]);
    await expect(
      writeGeneratedApplication({
        sdkRoot: repoRoot,
        bunSource,
        directory: join(directory, "conflict"),
        nativeManifest,
        graphSource: join(directory, "graph.rs"),
        application,
        main: application.main,
      }),
    ).rejects.toThrow(/through application\.main/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("an application cannot bring its own Bun graph or allocator", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-embedded-invariants-"));
  try {
    const host = join(directory, "host");
    await mkdir(join(host, "src"), { recursive: true });
    await writeFile(
      join(directory, "Cargo.toml"),
      ["[workspace]", 'resolver = "2"', 'members = ["host"]', ""].join("\n"),
    );
    const writeHost = async (extraDependency: string, library: string) => {
      await writeFile(
        join(host, "Cargo.toml"),
        [
          "[package]",
          'name = "embedded-invariant-host"',
          'version = "0.0.0"',
          'edition = "2024"',
          "",
          "[dependencies]",
          `solid-gpui = { path = ${JSON.stringify(join(repoRoot, "crates/solid-gpui"))} }`,
          extraDependency,
          "",
        ].join("\n"),
      );
      await writeFile(join(host, "src/lib.rs"), library);
    };
    await writeFile(join(host, "main.rs"), "fn main() {}\n");

    // The embedding library is refused even when it resolves: a real second Bun
    // graph usually comes from a registry or a different checkout, not a typo.
    const stub = join(directory, "bun-stub");
    await mkdir(join(stub, "src"), { recursive: true });
    await writeFile(
      join(stub, "Cargo.toml"),
      ["[package]", 'name = "bun_bin"', 'version = "0.0.0"', 'edition = "2024"', ""].join("\n"),
    );
    await writeFile(join(stub, "src/lib.rs"), "");
    await writeHost(`bun_bin = { path = ${JSON.stringify(stub)} }`, "");
    await expect(
      resolveEmbeddedApplication(
        { manifest: join(directory, "Cargo.toml"), package: "embedded-invariant-host", main: join(host, "main.rs") },
        repoRoot,
      ),
    ).rejects.toThrow(/must\s+not depend on the embedding library itself/);

    await writeHost("", "#[global_allocator]\nstatic ALLOCATOR: () = ();\n");
    await expect(
      resolveEmbeddedApplication(
        { manifest: join(directory, "Cargo.toml"), package: "embedded-invariant-host", main: join(host, "main.rs") },
        repoRoot,
      ),
    ).rejects.toThrow(/declares #\[global_allocator\]/);

    await writeHost("", "");
    await expect(
      resolveEmbeddedApplication(
        { manifest: join(directory, "Cargo.toml"), package: "absent-package", main: join(host, "main.rs") },
        repoRoot,
      ),
    ).rejects.toThrow(/is not a member of the workspace .*members: embedded-invariant-host/s);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("entry identities follow the serializer's key rules, not file names", () => {
  // The application entry point is keyed after the serializer's output file,
  // which the packager names after the entry itself; PE drops the `.exe` again.
  expect(embeddedEntryPointIdentity("bun-darwin-arm64", "app.js")).toBe("/$bunfs/root/app.js");
  expect(embeddedEntryPointIdentity("bun-windows-x64", "app.js")).toBe("B:/~BUN/root/app.js");

  // Additional entries are keyed relative to the source root the packager
  // passes, with the emitted `.js` extension. These keys were read back from
  // images produced by the serializer for exactly these input shapes.
  expect(embeddedSourceRoot(["/app/dist/app.js", "/app/fixtures/w.worker.ts"])).toBe("/app");
  expect(embeddedSourceRoot(["/app/dist/app.js", "/app/dist/worker.ts"])).toBe("/app/dist");
  expect(embeddedSourceRoot(["/app/dist/app.js"])).toBe("/app/dist");
  expect(embeddedWorkerIdentity("bun-darwin-arm64", "/app/dist/worker.ts", "/app/dist")).toBe("/$bunfs/root/worker.js");
  expect(embeddedWorkerIdentity("bun-darwin-arm64", "/app/fixtures/w.worker.ts", "/app")).toBe(
    "/$bunfs/root/fixtures/w.worker.js",
  );
  expect(embeddedWorkerIdentity("bun-windows-x64", "/app/dist/worker.mts", "/app/dist")).toBe("B:/~BUN/root/worker.js");
  expect(() => embeddedWorkerIdentity("bun-darwin-arm64", "/elsewhere/worker.ts", "/app")).toThrow(
    /not inside the serializer source root/,
  );
});

test("the target matrix is explicit and unsupported combinations fail before any build command", async () => {
  const linux = resolveEmbeddedTarget("x86_64-unknown-linux-gnu");
  expect(linux.qualification).toBe("prepare-only");
  expect(linux.graph).toBeUndefined();
  expect(resolveEmbeddedTarget("aarch64-apple-darwin").graph).toBe("bun-darwin-arm64");
  expect(() => resolveEmbeddedTarget("riscv64gc-unknown-linux-gnu")).toThrow(/aarch64-apple-darwin/);

  const commands: string[][] = [];
  await expect(
    packageEmbeddedApplication({
      sdkRoot: repoRoot,
      target: "x86_64-unknown-linux-gnu",
      profile: "release",
      entry: "/app/dist/app.js",
      output: "/app/build/app",
      bun: "/pinned/bun",
      onCommand: (command) => commands.push([...command]),
    }),
  ).rejects.toThrow(/no application graph transport/);
  expect(commands).toEqual([]);

  await expect(
    packageEmbeddedApplication({
      sdkRoot: repoRoot,
      target: "aarch64-apple-darwin",
      output: "/app/build/app",
    } as never),
  ).rejects.toThrow(/required/);
});

test("the embedded CLI reports usage errors without reaching for the SDK", async () => {
  const lines: string[] = [];
  const write = (line: string) => lines.push(line);

  expect(await runEmbeddedCommand(["--help"], { cwd: repoRoot, write })).toBe(0);
  expect(lines.join("\n")).toContain("--manifest <Cargo.toml>");

  lines.length = 0;
  expect(await runEmbeddedCommand(["--entry", "app.js"], { cwd: repoRoot, write })).toBe(2);
  expect(lines.join("\n")).toContain("--entry, --output and --bun are required");

  lines.length = 0;
  expect(
    await runEmbeddedCommand(["--manifest", "app/Cargo.toml", "--main", "app/main.rs"], { cwd: repoRoot, write }),
  ).toBe(2);
  expect(lines.join("\n")).toContain("--package <name> is required");

  lines.length = 0;
  expect(
    await runEmbeddedCommand(["--entry", "app.js", "--output", "build/app", "--bun", "bun", "--profile", "fast"], {
      cwd: repoRoot,
      write,
    }),
  ).toBe(2);
  expect(lines.join("\n")).toContain("--profile must be debug or release");

  expect(new EmbeddedPackagingError("x")).toBeInstanceOf(Error);
});
