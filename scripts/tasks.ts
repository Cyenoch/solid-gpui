#!/usr/bin/env bun

import { mkdtemp, mkdir, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Command } from "commander";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const corePackageDir = join(repoRoot, "packages/solid-gpui");
const vitePackageDir = join(repoRoot, "packages/solid-gpui-vite");
const routerPackageDir = join(repoRoot, "packages/solid-gpui-router");
const shikiPackageDir = join(repoRoot, "packages/solid-gpui-shiki");
const websiteNativeEntry = join(repoRoot, "examples/website/dist-native/main.native.js");
const websiteDir = join(repoRoot, "examples/website");

type RunOptions = {
  readonly cwd?: string;
  readonly env?: Record<string, string | undefined>;
};

class ProcessFailure extends Error {
  constructor(
    readonly command: readonly string[],
    readonly exitCode: number,
  ) {
    super(`command failed with exit code ${exitCode}: ${formatCommand(command)}`);
  }
}

function formatCommand(command: readonly string[]): string {
  return command.map((part) => (/^[A-Za-z0-9_./:=@+-]+$/.test(part) ? part : JSON.stringify(part))).join(" ");
}

async function run(command: readonly string[], options: RunOptions = {}): Promise<void> {
  console.error(`\n$ ${formatCommand(command)}`);
  const child = Bun.spawn([...command], {
    cwd: options.cwd ?? repoRoot,
    env: { ...process.env, ...options.env },
    stdio: ["inherit", "inherit", "inherit"],
  });
  const forwardInterrupt = (): void => child.kill("SIGINT");
  const forwardTermination = (): void => child.kill("SIGTERM");
  process.once("SIGINT", forwardInterrupt);
  process.once("SIGTERM", forwardTermination);
  try {
    const exitCode = await child.exited;
    if (exitCode !== 0 || child.signalCode !== null) {
      const signalExitCode = child.signalCode === "SIGINT" ? 130 : child.signalCode === "SIGTERM" ? 143 : 1;
      throw new ProcessFailure(command, exitCode === 0 ? signalExitCode : exitCode);
    }
  } finally {
    process.off("SIGINT", forwardInterrupt);
    process.off("SIGTERM", forwardTermination);
  }
}

function requireTool(name: string, installHint: string): void {
  if (Bun.which(name) === null) throw new Error(`${name} is required; ${installHint}`);
}

async function runBash(script: string, args: readonly string[] = []): Promise<void> {
  requireTool("bash", "install Bash to run this platform task");
  await run(["bash", script, ...args]);
}

function requireMacOS(task: string): void {
  if (process.platform !== "darwin") throw new Error(`${task} requires macOS`);
}

class Tasks {
  private installPromise: Promise<void> | undefined;
  private packageBuildPromise: Promise<void> | undefined;
  private corePackageBuildPromise: Promise<void> | undefined;
  private routerPackageBuildPromise: Promise<void> | undefined;
  private shikiPackageBuildPromise: Promise<void> | undefined;
  private nativeCodegenPromise: Promise<void> | undefined;

  install(): Promise<void> {
    this.installPromise ??= run(["bun", "install", "--frozen-lockfile"]);
    return this.installPromise;
  }

  packageBuild(): Promise<void> {
    this.packageBuildPromise ??= this.buildPackages();
    return this.packageBuildPromise;
  }

  private async buildPackages(): Promise<void> {
    await this.install();
    await this.nativeCodegen();
    await this.buildJavaScriptPackages();
  }

  private async buildJavaScriptPackages(): Promise<void> {
    await this.corePackageBuild();
    await this.buildVitePackage();
    await this.routerPackageBuild();
    await this.shikiPackageBuild();
  }

  private corePackageBuild(): Promise<void> {
    this.corePackageBuildPromise ??= this.buildCorePackage();
    return this.corePackageBuildPromise;
  }

  private async buildCorePackage(): Promise<void> {
    await this.install();
    await this.buildPackage({
      directory: corePackageDir,
      entrypoints: [
        "./src/index.ts",
        "./src/runtime.ts",
        "./src/native.ts",
        "./src/components.ts",
        "./src/motion.ts",
        "./src/stdio.ts",
        "./src/embedded.ts",
        "./src/web.ts",
      ],
      conditions: ["browser"],
      external: ["@solid-gpui/core/native", "@solid-gpui/core/components", "bebop", "solid-js", "solid-js/*"],
    });
  }

  private async buildVitePackage(): Promise<void> {
    await rm(join(vitePackageDir, "dist"), { recursive: true, force: true });
    const { build } = await import("vite");
    await build({
      configFile: false,
      root: vitePackageDir,
      build: {
        ssr: true,
        target: "esnext",
        outDir: "dist",
        rolldownOptions: {
          input: ["index", "dev", "runner", "quickjs-platform"].map((name) => join(vitePackageDir, `src/${name}.ts`)),
          external: (id) => !id.startsWith(".") && !id.startsWith("/") && !/^[A-Za-z]:[\\/]/.test(id),
          output: {
            preserveModules: true,
            preserveModulesRoot: join(vitePackageDir, "src"),
            entryFileNames: "[name].js",
          },
        },
      },
    });
    await run(["bunx", "tsc", "--project", "tsconfig.build.json"], { cwd: vitePackageDir });
  }

  private routerPackageBuild(): Promise<void> {
    this.routerPackageBuildPromise ??= this.buildPackage({
      directory: routerPackageDir,
      entrypoints: ["./src/index.ts", "./src/generator.ts", "./src/generator-process.ts", "./src/vite.ts"],
      conditions: [],
      external: [
        "@solid-gpui/core",
        "@solid-gpui/core/*",
        "@tanstack/router-core",
        "@tanstack/history",
        "@tanstack/router-generator",
        "@tanstack/router-utils",
        "vite",
        "solid-js",
        "solid-js/*",
      ],
    });
    return this.routerPackageBuildPromise;
  }

  private shikiPackageBuild(): Promise<void> {
    this.shikiPackageBuildPromise ??= this.buildPackage({
      directory: shikiPackageDir,
      entrypoints: ["./src/index.ts", "./src/bun.ts", "./src/worker.ts"],
      conditions: ["browser"],
      external: ["@solid-gpui/core", "@solid-gpui/core/*", "shiki", "shiki/*", "solid-js", "solid-js/*"],
    });
    return this.shikiPackageBuildPromise;
  }

  nativeCodegen(): Promise<void> {
    this.nativeCodegenPromise ??= run(["bun", "scripts/native-codegen.ts"]);
    return this.nativeCodegenPromise;
  }

  async nativeCodegenCheck(): Promise<void> {
    await run(["bun", "scripts/native-codegen.ts", "--check"]);
  }

  async embeddedCheck(): Promise<void> {
    requireMacOS("embedded-check");
    await run([
      "rustfmt",
      "--check",
      "--edition",
      "2024",
      "--config",
      "skip_children=true",
      "crates/solid-gpui-bun-sys/embedded/runtime.rs",
      "crates/solid-gpui-bun-sys/embedded/lifecycle.rs",
    ]);
    await run(
      [
        "cargo",
        "clippy",
        "--locked",
        "-p",
        "solid-gpui",
        "--all-targets",
        "--features",
        "embedded-bun",
        "--",
        "-D",
        "warnings",
      ],
      { env: { SOLID_GPUI_BUN_CHECK_ONLY: "1" } },
    );
  }

  async embeddedTest(): Promise<void> {
    requireMacOS("embedded-test");
    await this.packageBuild();
    await run(
      [
        "cargo",
        "test",
        "--locked",
        "-p",
        "solid-gpui",
        "--features",
        "embedded-bun",
        "--test",
        "host_embedded_counter",
      ],
      { env: { SOLID_GPUI_BUN_CHECK_ONLY: undefined } },
    );
  }

  private async buildPackage(options: {
    readonly directory: string;
    readonly entrypoints: readonly string[];
    readonly conditions: readonly string[];
    readonly external: readonly string[];
  }): Promise<void> {
    await this.install();
    await rm(join(options.directory, "dist"), { recursive: true, force: true });
    const { build } = await import("vite");
    await build({
      configFile: false,
      root: options.directory,
      ssr: { noExternal: true, resolve: { conditions: ["bun", ...options.conditions] } },
      build: {
        ssr: true,
        target: "esnext",
        outDir: "dist",
        minify: false,
        rolldownOptions: {
          input: Object.fromEntries(
            options.entrypoints.map((entry) => [
              entry.split("/").at(-1)!.replace(/\.ts$/, ""),
              resolve(options.directory, entry),
            ]),
          ),
          external: (id) =>
            id.startsWith("bun:") ||
            id === "bun" ||
            options.external.some((pattern) =>
              pattern.endsWith("/*") ? id.startsWith(pattern.slice(0, -1)) : id === pattern,
            ),
          output: { format: "esm", entryFileNames: "[name].js", chunkFileNames: "[name]-[hash].js" },
        },
      },
    });
    await run(["bunx", "tsc", "--project", "tsconfig.build.json"], { cwd: options.directory });
  }

  async protocolCodegen(): Promise<void> {
    await this.install();
    await run(["bunx", "bebopc", "--config", "packages/solid-gpui/src/protocol/bebop.json", "build"], {
      env: { TERM: "xterm-256color", COLUMNS: "120", LINES: "40" },
    });
    await run(["bun", "scripts/normalize-bebop-rust.ts"]);
    await run(["rustfmt", "--edition", "2024", "crates/solid-gpui/src/protocol/generated/protocol.rs"]);
    await run(["bun", "scripts/protocol-schema-meta.ts"]);
    await run([
      "rustfmt",
      "--edition",
      "2024",
      "crates/solid-gpui/src/protocol/generated/schema_facts.rs",
      "crates/solid-gpui/src/protocol/generated/schema_guard.rs",
    ]);
    await run([
      "bunx",
      "oxfmt",
      "--write",
      "packages/solid-gpui/src/protocol/generated/protocol.ts",
      "packages/solid-gpui/src/protocol/generated/schema-meta.ts",
      "packages/solid-gpui/src/protocol/generated/schema-facts.ts",
    ]);
  }

  async protocolCodegenCheck(): Promise<void> {
    await this.protocolCodegen();
    await run([
      "git",
      "diff",
      "--exit-code",
      "--",
      "packages/solid-gpui/src/protocol/generated/protocol.ts",
      "packages/solid-gpui/src/protocol/generated/schema-meta.ts",
      "packages/solid-gpui/src/protocol/generated/schema-facts.ts",
      "crates/solid-gpui/src/protocol/generated/protocol.rs",
      "crates/solid-gpui/src/protocol/generated/schema-meta.json",
      "crates/solid-gpui/src/protocol/generated/schema_facts.rs",
      "crates/solid-gpui/src/protocol/generated/schema_guard.rs",
      "packages/solid-gpui/src/protocol/schema-lock.json",
    ]);
  }

  async protocolGoldenCheck(): Promise<void> {
    await this.install();
    const directory = await mkdtemp(join(tmpdir(), "solid-gpui-goldens-"));
    try {
      await run(["bun", "scripts/protocol-golden.ts", directory]);
      await run([
        "cargo",
        "run",
        "--locked",
        "-p",
        "solid-gpui",
        "--example",
        "protocol_golden",
        "--",
        join(directory, "rust_to_ts.hex"),
      ]);
      await run(["bun", "scripts/protocol-golden.ts", directory, "--verify"]);
      for (const name of ["ts_to_rust.hex", "rust_to_ts.hex", "invalid.hex", "frames.hex"]) {
        const [committed, generated] = await Promise.all([
          readFile(join(repoRoot, "fixtures/protocol", name)),
          readFile(join(directory, name)),
        ]);
        if (!committed.equals(generated)) throw new Error(`Protocol golden fixture is stale: ${name}`);
      }
      await run(["cargo", "test", "--locked", "-p", "solid-gpui", "--test", "protocol_golden"]);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  }

  async packageFormat(): Promise<void> {
    await this.install();
    await run([
      "bunx",
      "oxfmt",
      "--check",
      ".oxfmtrc.json",
      "scripts/**/*.ts",
      "fixtures/*.{ts,tsx,json}",
      "packages/solid-gpui/src/*.ts",
      "packages/solid-gpui/src/renderer/**/*.ts",
      "packages/solid-gpui/src/protocol/*.ts",
      "packages/solid-gpui/src/protocol/*.json",
      "packages/solid-gpui/src/protocol/generated/*.ts",
      "packages/solid-gpui/tests/**/*.{ts,tsx,js,json}",
      "examples/**/*.{ts,tsx,json}",
      "packages/solid-gpui-vite/src/**/*.{ts,js}",
      "packages/solid-gpui-router/src/**/*.ts",
      "packages/solid-gpui-router/tests/**/*.{ts,js,json}",
      "package.json",
      "tsconfig.tools.json",
      "packages/solid-gpui/*.json",
      "packages/solid-gpui-vite/*.json",
      "packages/solid-gpui-router/*.json",
      "packages/solid-gpui-shiki/src/**/*.ts",
      "packages/solid-gpui-shiki/tests/**/*.ts",
      "packages/solid-gpui-shiki/examples/**/*.ts",
      "packages/solid-gpui-shiki/*.json",
    ]);
  }

  async packageTypecheck(): Promise<void> {
    await this.packageBuild();
    await Promise.all([
      run(["bunx", "tsc", "--noEmit"], { cwd: corePackageDir }),
      run(["bunx", "tsc", "--noEmit"], { cwd: vitePackageDir }),
      run(["bunx", "tsc", "--noEmit"], { cwd: routerPackageDir }),
      run(["bunx", "tsc", "--noEmit"], { cwd: shikiPackageDir }),
      run(["bunx", "tsc", "--project", "tsconfig.tools.json"]),
      run(["bunx", "tsc", "--project", "fixtures/tsconfig.json"]),
    ]);
  }
  async packageTest(): Promise<void> {
    await this.packageBuild();
    await Promise.all([
      run(["bun", "test", "--conditions=browser", "--preload", join(repoRoot, "scripts/solid-jsx.ts")], {
        cwd: corePackageDir,
      }),
      run(["bun", "--conditions=browser", "test"], { cwd: routerPackageDir }),
      run(["bun", "--conditions=browser", "test"], { cwd: shikiPackageDir }),
      run([
        "bun",
        "test",
        "scripts/api-surface.test.ts",
        "scripts/application-build.test.ts",
        "scripts/hot-reload.test.ts",
        "scripts/native-export.test.ts",
        "scripts/dev-session.test.ts",
        "scripts/task-contract.test.ts",
        "scripts/release-prep.test.ts",
      ]),
    ]);
  }

  async packagePackSmoke(): Promise<void> {
    await this.packageBuild();
    await run(["bun", "scripts/package-pack-smoke.ts"]);
  }
  async websitePackage(outputPath?: string): Promise<void> {
    await this.packageBuild();
    await run(["bun", "scripts/website-package.ts", ...(outputPath === undefined ? [] : ["--out", outputPath])]);
  }
  async packagePack(outputPath: string): Promise<void> {
    await this.corePackageBuild();
    await this.packPackage(corePackageDir, outputPath);
  }

  async routerPackagePack(outputPath: string): Promise<void> {
    await this.corePackageBuild();
    await this.routerPackageBuild();
    await this.packPackage(routerPackageDir, outputPath);
  }

  async vitePackagePack(outputPath: string): Promise<void> {
    await this.install();
    await this.buildVitePackage();
    await this.packPackage(vitePackageDir, outputPath);
  }

  async shikiPackagePack(outputPath: string): Promise<void> {
    await this.corePackageBuild();
    await this.shikiPackageBuild();
    await this.packPackage(shikiPackageDir, outputPath);
  }

  private async packPackage(directory: string, outputPath: string): Promise<void> {
    const destination = resolve(repoRoot, outputPath);
    await mkdir(dirname(destination), { recursive: true });
    await run(["bun", "pm", "pack", "--filename", destination, "--quiet"], { cwd: directory });
  }

  async packageCI(): Promise<void> {
    await this.packageBuild();
    await Promise.all([
      this.packageFormat(),
      this.packageTypecheck(),
      this.packageTest(),
      this.packagePackSmoke(),
      this.nativeCodegenCheck(),
    ]);
  }
  async format(): Promise<void> {
    await Promise.all([this.rustFormat(), this.packageFormat()]);
  }

  async check(): Promise<void> {
    await Promise.all([this.protocolCodegenCheck(), this.nativeCodegenCheck()]);
    await this.packageBuild();
    await Promise.all([this.rustCompile(), this.packageTypecheck()]);
  }

  async rustFormat(): Promise<void> {
    await run(["cargo", "fmt", "--all", "--", "--check"]);
  }

  async rustCompile(): Promise<void> {
    await this.packageBuild();
    await run(["cargo", "check", "--workspace", "--locked"]);
    await run([
      "cargo",
      "clippy",
      "--workspace",
      "--all-targets",
      "--locked",
      "--features",
      "solid-gpui/quickjs",
      "--",
      "-D",
      "warnings",
    ]);
  }

  async rustTest(): Promise<void> {
    await this.packageBuild();
    await run(["cargo", "test", "--workspace", "--locked", "--features", "solid-gpui/quickjs"]);
    await run(["cargo", "test", "-p", "gpui-pre-reqwest-client", "--lib", "--locked"]);
  }

  async rustCheck(): Promise<void> {
    await this.rustCompile();
    await this.rustTest();
  }

  async test(): Promise<void> {
    await this.packageBuild();
    await Promise.all([this.rustTest(), this.packageTest()]);
  }

  async audit(): Promise<void> {
    await this.install();
    requireTool("cargo-deny", "install version 0.20.2");
    await Promise.all([run(["bun", "audit"]), run(["cargo", "deny", "--all-features", "check", "advisories"])]);

    const temporaryDir = await mkdtemp(join(tmpdir(), "solid-gpui-notices-check-"));
    const generatedNotices = join(temporaryDir, "THIRD-PARTY-NOTICES.md");
    try {
      await runBash("scripts/third-party-notices.sh", [generatedNotices]);
      const [committed, generated] = await Promise.all([
        readFile(join(repoRoot, "THIRD-PARTY-NOTICES.md")),
        readFile(generatedNotices),
      ]);
      if (!committed.equals(generated)) {
        throw new Error("THIRD-PARTY-NOTICES.md is out of date; run 'bun run task third-party-notices'");
      }
    } finally {
      await rm(temporaryDir, { recursive: true, force: true });
    }
  }

  async apiSurface(): Promise<void> {
    await this.packageBuild();
    await run(["bun", "scripts/api-surface.ts"]);
  }

  async thirdPartyNotices(): Promise<void> {
    await runBash("scripts/third-party-notices.sh", [join(repoRoot, "THIRD-PARTY-NOTICES.md")]);
  }

  async build(): Promise<void> {
    await this.packageBuild();
    await run(["cargo", "build", "--workspace", "--locked"]);
  }

  async ci(): Promise<void> {
    await this.rustFormat();
    await this.protocolCodegenCheck();
    await this.packageBuild();
    await this.protocolGoldenCheck();
    await this.rustCheck();
    await this.packageCI();
  }

  async websiteNative(profile = false): Promise<void> {
    await this.packageBuild();
    await run(["bun", "run", "build:native"], { cwd: websiteDir });
    await run([
      "cargo",
      "run",
      "-p",
      "website-host",
      ...(profile ? ["--features", "solid-gpui/frame-profile"] : []),
      "--",
      "bun",
      "run",
      "--conditions=browser",
      websiteNativeEntry,
    ]);
  }

  async websiteNativeDev(): Promise<void> {
    await this.buildJavaScriptPackages();
    await run(["bun", "--bun", "vite", "--config", join(websiteDir, "vite.native.config.ts")]);
  }

  async quickJsDev(): Promise<void> {
    await this.buildJavaScriptPackages();
    await run(["bun", "--bun", "vite", "--config", join(repoRoot, "fixtures/vite.config.ts")]);
  }

  async websiteNavigationCheck(): Promise<void> {
    await run(["bun", "--conditions=browser", "test", "examples/website/tests/runtime.test.ts"]);
  }

  async hostCandidateSmoke(): Promise<void> {
    requireMacOS("host-candidate-smoke");
    await this.packageBuild();
    await this.hostRelease("check");
    await runBash("scripts/host-candidate-smoke.sh");
  }

  async hostEmbeddedCandidateSmoke(): Promise<void> {
    requireMacOS("host-embedded-candidate-smoke");
    await this.packageBuild();
    await runBash("scripts/host-embedded-candidate-smoke.sh");
  }

  async hostRelease(mode: "bundle" | "check"): Promise<void> {
    requireMacOS("host-release");
    await runBash("scripts/host-release.sh", [mode]);
  }

  async releasePrep(version: string): Promise<void> {
    await runBash("scripts/release-prep.sh", [version]);
  }

  async soakSmoke(): Promise<void> {
    requireMacOS("soak-smoke");
    await this.packageBuild();
    await runBash("scripts/soak-smoke.sh");
  }

  async killResilience(): Promise<void> {
    requireMacOS("kill-resilience");
    await this.packageBuild();
    await runBash("scripts/kill-resilience.sh");
  }
}

const tasks = new Tasks();
const program = new Command();

program.name("task").description("Solid GPUI workspace task runner");

function addTask(name: string, description: string, action: (...args: readonly string[]) => Promise<void>): void {
  program
    .command(name)
    .description(description)
    .action(async (...args: string[]) => {
      try {
        // Commander appends options and the command object after the declared operands.
        await action(...args.slice(0, -2));
      } catch (error) {
        if (error instanceof ProcessFailure) {
          process.exit(error.exitCode);
        }
        throw error;
      }
    });
}

addTask("install", "Install workspace dependencies", () => tasks.install());
addTask("package-build", "Build TypeScript packages", () => tasks.packageBuild());
addTask("protocol-codegen", "Generate TypeScript and Rust protocol bindings", () => tasks.protocolCodegen());
addTask("protocol-codegen-check", "Verify protocol bindings match schema", () => tasks.protocolCodegenCheck());
addTask("protocol-golden-check", "Verify cross-language protocol fixtures", () => tasks.protocolGoldenCheck());
addTask("native-codegen", "Generate bindings from the SDK and website hosts", () => tasks.nativeCodegen());
addTask("native-codegen-check", "Verify bindings match the actual native hosts", () => tasks.nativeCodegenCheck());
addTask("embedded-check", "Check embedded Rust integration without building Bun", () => tasks.embeddedCheck());
addTask("embedded-test", "Build Bun and test the embedded VM lifecycle", () => tasks.embeddedTest());
addTask("package-format", "Format TypeScript packages", () => tasks.packageFormat());
addTask("package-typecheck", "Typecheck TypeScript packages", () => tasks.packageTypecheck());
addTask("api-surface", "Generate public API surface fixtures", () => tasks.apiSurface());
addTask("package-test", "Run TypeScript package tests", () => tasks.packageTest());
addTask("package-pack-smoke", "Smoke test packed TypeScript packages", () => tasks.packagePackSmoke());
addTask("package-pack <output>", "Pack core package", (output) => tasks.packagePack(output));
addTask("vite-package-pack <output>", "Pack Vite package", (output) => tasks.vitePackagePack(output));
addTask("router-package-pack <output>", "Pack router package", (output) => tasks.routerPackagePack(output));
addTask("shiki-package-pack <output>", "Pack Shiki package", (output) => tasks.shikiPackagePack(output));
addTask("package-ci", "Run TypeScript package CI suite", () => tasks.packageCI());
addTask("rust-format", "Check Rust formatting", () => tasks.rustFormat());
addTask("rust-compile", "Compile Rust workspace", () => tasks.rustCompile());
addTask("rust-test", "Run Rust tests", () => tasks.rustTest());
addTask("rust-check", "Run Rust checks", () => tasks.rustCheck());
addTask("format", "Format workspace code", () => tasks.format());
addTask("check", "Run all workspace checks", () => tasks.check());
addTask("test", "Run all workspace tests", () => tasks.test());
addTask("audit", "Run security and license audits", () => tasks.audit());
addTask("third-party-notices", "Generate THIRD-PARTY-NOTICES.md", () => tasks.thirdPartyNotices());
addTask("build", "Build all workspace packages and binaries", () => tasks.build());
addTask("ci", "Check generated contracts, formatting, types, and tests", () => tasks.ci());
addTask("website-native", "Build and run the native website", () => tasks.websiteNative());
addTask("quickjs-dev", "Run the counter with QuickJS application reload", () => tasks.quickJsDev());
addTask("website-native-dev", "Run the native website with in-window hot reload", () => tasks.websiteNativeDev());
addTask("website-package [output]", "Build and verify a standalone website package for this platform", (output) =>
  tasks.websitePackage(output),
);
addTask("website-native-profile", "Run the native website with native frame and input latency measurements", () =>
  tasks.websiteNative(true),
);
addTask("website-navigation-check", "Verify website previews and retained router navigation", () =>
  tasks.websiteNavigationCheck(),
);
addTask("host-candidate-smoke", "Build and smoke the extracted host candidate", () => tasks.hostCandidateSmoke());
addTask("host-embedded-candidate-smoke", "Run host embedded candidate smoke suite", () =>
  tasks.hostEmbeddedCandidateSmoke(),
);
addTask("host-release", "Build the host release bundle", () => tasks.hostRelease("bundle"));
addTask("host-release-check", "Build and verify the extracted host release bundle", () => tasks.hostRelease("check"));
addTask("release-prep <version>", "Prepare release manifests", (version) => tasks.releasePrep(version));
addTask("soak-smoke", "Run soak smoke test", () => tasks.soakSmoke());
addTask("kill-resilience", "Run kill resilience test", () => tasks.killResilience());

await program.parseAsync(process.argv);
