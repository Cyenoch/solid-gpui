import { chmod, copyFile, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { tmpdir } from "node:os";
import { readNativeArtifacts, recordedHostExecutable, targetRunsOnHost } from "./artifacts.ts";
import { deliveryHash, deliveryRun } from "./delivery.ts";
import { assertRecordedConfiguration, loadProject, type ProjectOptions } from "./project.ts";

export interface PackageApplicationOptions extends ProjectOptions {
  readonly name: string;
  readonly output?: string;
  /** Additional application-owned runtime resources, preserving their basenames. */
  readonly assets?: readonly string[];
  /** License/notice files, defaulting to the application's LICENSE and THIRD-PARTY-NOTICES.md. */
  readonly licenses?: readonly string[];
}

export interface ApplicationPackage {
  readonly archive: string;
  readonly sha256: string;
  readonly runtime: "bun" | "quickjs";
  readonly verification: "extracted-runtime-contract";
}

async function copyTree(source: string, destination: string): Promise<void> {
  await mkdir(destination, { recursive: true });
  for (const entry of await readdir(source, { withFileTypes: true })) {
    if (entry.isDirectory()) await copyTree(join(source, entry.name), join(destination, entry.name));
    else if (entry.isFile()) await copyFile(join(source, entry.name), join(destination, entry.name));
    else throw new Error(`Application packages cannot contain symlinks or special files: ${join(source, entry.name)}`);
  }
}

async function inventory(root: string): Promise<string[]> {
  const result: string[] = [];
  for (const entry of await readdir(root, { withFileTypes: true })) {
    if (entry.isDirectory())
      result.push(...(await inventory(join(root, entry.name))).map((path) => `${entry.name}/${path}`));
    else if (entry.isFile()) result.push(entry.name);
    else throw new Error(`Unexpected archive entry: ${entry.name}`);
  }
  return result.sort();
}

/** Package only built artifacts; no Cargo/Vite build, download, signing or publication. */
export async function packageApplication(options: PackageApplicationOptions): Promise<ApplicationPackage> {
  if (!/^[a-z][a-z0-9-]*$/.test(options.name))
    throw new Error("Package name must contain lowercase letters, numbers and hyphens");
  const project = await loadProject(options);
  const artifacts = await readNativeArtifacts(project.root);
  if (!artifacts?.bundle) throw new Error("Build this application with vite build before packaging");
  assertRecordedConfiguration(project, artifacts);
  if (artifacts.runtime === "web") throw new Error("Native application packaging requires bun or quickjs");
  if (!targetRunsOnHost(artifacts.native?.target))
    throw new Error("Extracted application verification must run on the destination platform");
  if (artifacts.host?.args?.length)
    throw new Error("Portable application packaging requires a native executable, without interpreted host arguments");
  const executable = recordedHostExecutable(artifacts);
  if (!executable) throw new Error("No recorded native executable; run solid-gpui prepare and vite build");
  const output = resolve(project.root, options.output ?? "packages");
  if (output === artifacts.outDir || output.startsWith(artifacts.outDir + sep))
    throw new Error("Package output must be outside the Vite build output directory");
  // Copying the Vite output into a child of itself recursively would include packaging intermediates.
  const temporaryParent = join(project.root, ".solid-gpui");
  await mkdir(temporaryParent, { recursive: true });
  const temporary = await mkdtemp(join(temporaryParent, ".package-"));
  let extracted: string | undefined;
  try {
    const stage = join(temporary, options.name);
    await mkdir(join(stage, "licenses"), { recursive: true });
    const hostName = "host" + (process.platform === "win32" ? ".exe" : "");
    const host = join(stage, hostName);
    await copyFile(resolve(project.root, executable), host);
    if (process.platform !== "win32") await chmod(host, 0o755);
    const files = await inventory(artifacts.outDir);
    for (const file of files) {
      const destination = join(stage, "app", file);
      await mkdir(dirname(destination), { recursive: true });
      await copyFile(join(artifacts.outDir, file), destination);
    }
    const bundle = `app/${relative(artifacts.outDir, artifacts.bundle).replaceAll("\\", "/")}`;
    if (!/^[A-Za-z0-9_./-]+$/.test(bundle))
      throw new Error("Portable launcher bundle paths require letters, numbers, underscores, dots, slashes or hyphens");
    for (const file of options.licenses ?? ["LICENSE", "THIRD-PARTY-NOTICES.md"]) {
      await copyFile(resolve(project.root, file), join(stage, "licenses", basename(file)));
    }
    for (const asset of options.assets ?? []) {
      const source = resolve(project.root, asset);
      await copyTree(source, join(stage, "assets", basename(source)));
    }
    const args =
      artifacts.runtime === "quickjs"
        ? `--runtime quickjs "$here/${bundle}"`
        : `bun --conditions=browser "$here/${bundle}"`;
    await writeFile(
      join(stage, "launch"),
      `#!/bin/sh\nset -eu\nhere="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"\ncd "$here"\nexec "$here/${hostName}" ${args}\n`,
    );
    await chmod(join(stage, "launch"), 0o755);
    await copyFile(join(stage, "launch"), join(stage, "launch.command"));
    await chmod(join(stage, "launch.command"), 0o755);
    await writeFile(
      join(stage, "launch.cmd"),
      `@echo off\r\ncd /d "%~dp0"\r\n"%~dp0${hostName}" ${artifacts.runtime === "quickjs" ? `--runtime quickjs "%~dp0${bundle}"` : `bun --conditions=browser "%~dp0${bundle}"`}\r\n`,
    );
    const version = (await deliveryRun([host, "--version"], stage)).trim();
    const bindings = await deliveryRun([host, "--export-native"], stage);
    await writeFile(join(stage, "native.ts"), bindings);
    await writeFile(
      join(stage, "application.json"),
      JSON.stringify(
        { format: 1, name: options.name, runtime: artifacts.runtime, bundle, host: hostName, version },
        null,
        2,
      ) + "\n",
    );
    await writeFile(
      join(stage, "README.md"),
      `# ${options.name}\n\nRun launch.command on macOS, launch on Linux, or launch.cmd on Windows. ${artifacts.runtime === "quickjs" ? "QuickJS is included in the native host. Bun/Node is not required." : "Bun 1.4.2+ is required on PATH. This package preserves the application's Bun runtime choice."}\n\nThe extracted runtime, native catalog, and committed content were checked without the source tree. Native libraries, fonts and a working graphics session remain platform requirements. This portable archive is unsigned; display/input qualification and signing belong to the application.\n`,
    );
    if (process.platform === "darwin") {
      const libraries = await deliveryRun(["otool", "-L", host], stage);
      if (
        libraries
          .split("\n")
          .slice(1)
          .filter(Boolean)
          .some((line) => !/^\s*\/(System\/Library|usr\/lib)\//.test(line))
      )
        throw new Error("Host links unbundled non-system native libraries");
      await writeFile(join(stage, "NATIVE-DEPENDENCIES.txt"), libraries.replace(host, hostName));
    } else if (process.platform === "linux") {
      const libraries = await deliveryRun(["ldd", host], stage);
      if (libraries.includes("not found")) throw new Error("Host has missing native libraries");
      await writeFile(join(stage, "NATIVE-DEPENDENCIES.txt"), libraries);
    }
    const contents = await inventory(stage);
    const sums = await Promise.all(contents.map(async (file) => `${await deliveryHash(join(stage, file))}  ${file}`));
    await writeFile(join(stage, "SHA256SUMS"), sums.join("\n") + "\n");
    const archive = join(temporary, `${options.name}.tar.gz`);
    await deliveryRun(["tar", "-czf", archive, "-C", temporary, options.name], project.root);
    const checkParent = process.env.SOLID_GPUI_CONSUMER_TEMP ?? tmpdir();
    extracted = await mkdtemp(join(checkParent, "solid-gpui-consumer-"));
    await deliveryRun(["tar", "-xzf", archive, "-C", extracted], extracted);
    const extractedRoot = join(extracted, options.name);
    const actual = await inventory(extractedRoot);
    if (JSON.stringify(actual) !== JSON.stringify([...contents, "SHA256SUMS"].sort()))
      throw new Error("Extracted package inventory changed");
    for (const [index, file] of contents.entries()) {
      if (`${await deliveryHash(join(extractedRoot, file))}  ${file}` !== sums[index])
        throw new Error(`Extracted content changed: ${file}`);
    }
    const runtimeDirectory = join(extracted, "runtime");
    if (artifacts.runtime === "bun") {
      await mkdir(runtimeDirectory);
      const bun = join(runtimeDirectory, process.platform === "win32" ? "bun.exe" : "bun");
      await copyFile(process.execPath, bun);
      if (process.platform !== "win32") await chmod(bun, 0o755);
    }
    const env: NodeJS.ProcessEnv = {
      ...process.env,
      PATH: artifacts.runtime === "quickjs" ? "" : runtimeDirectory,
    };
    for (const key of Object.keys(env))
      if (key.startsWith("SOLID_GPUI_") || key === "NODE_PATH" || key === "BUN_OPTIONS") delete env[key];
    const extractedHost = join(extractedRoot, hostName);
    if ((await deliveryRun([extractedHost, "--version"], extracted, env)).trim() !== version)
      throw new Error("Extracted version changed");
    if ((await deliveryRun([extractedHost, "--export-native"], extracted, env)) !== bindings)
      throw new Error("Extracted native contract changed");
    await deliveryRun(
      [extractedHost, "--check-app", artifacts.runtime, join(extractedRoot, bundle)],
      extractedRoot,
      env,
    );
    await mkdir(output, { recursive: true });
    const destination = join(output, `${options.name}-${process.platform}-${process.arch}.tar.gz`);
    await rename(archive, destination);
    const sha256 = await deliveryHash(destination);
    await writeFile(destination + ".sha256", `${sha256}  ${basename(destination)}\n`);
    return { archive: destination, sha256, runtime: artifacts.runtime, verification: "extracted-runtime-contract" };
  } finally {
    await rm(temporary, { recursive: true, force: true });
    if (extracted) await rm(extracted, { recursive: true, force: true });
  }
}
