import { chmod, copyFile, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { tmpdir } from "node:os";
import { createPublicKey, verify } from "node:crypto";
import { readNativeArtifacts, recordedHostExecutable, targetRunsOnHost } from "./artifacts.ts";
import { deliveryHash, deliveryRun } from "./delivery.ts";
import { assertRecordedConfiguration, loadProject, type ProjectOptions } from "./project.ts";
import { writePortableArchive } from "./portable-archive.ts";

export interface PackageApplicationOptions extends ProjectOptions {
  readonly name: string;
  readonly output?: string;
  /** Additional application-owned runtime resources, preserving their basenames. */
  readonly assets?: readonly string[];
  /** License/notice files, defaulting to the application's LICENSE and THIRD-PARTY-NOTICES.md. */
  readonly licenses?: readonly string[];
  /** macOS bundle metadata; defaults to the package name, dev.solidgpui.<name>, and 0.1.0. */
  readonly application?: { readonly id: string; readonly version: string };
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
    const bundleName = `${options.name}.app`;
    const payload = process.platform === "darwin" ? join(stage, bundleName, "Contents/Resources") : stage;
    await mkdir(join(payload, "licenses"), { recursive: true });
    const hostName = "host" + (process.platform === "win32" ? ".exe" : "");
    const hostRelative = process.platform === "darwin" ? `${bundleName}/Contents/MacOS/host` : hostName;
    const host = join(stage, hostRelative);
    await mkdir(dirname(host), { recursive: true });
    await copyFile(resolve(project.root, executable), host);
    if (process.platform !== "win32") await chmod(host, 0o755);
    const files = await inventory(artifacts.outDir);
    for (const file of files) {
      const destination = join(payload, "app", file);
      await mkdir(dirname(destination), { recursive: true });
      await copyFile(join(artifacts.outDir, file), destination);
    }
    const bundle = `app/${relative(artifacts.outDir, artifacts.bundle).replaceAll("\\", "/")}`;
    if (!/^[A-Za-z0-9_./-]+$/.test(bundle))
      throw new Error("Portable launcher bundle paths require letters, numbers, underscores, dots, slashes or hyphens");
    for (const file of options.licenses ?? ["LICENSE", "THIRD-PARTY-NOTICES.md"]) {
      await copyFile(resolve(project.root, file), join(payload, "licenses", basename(file)));
    }
    for (const asset of options.assets ?? []) {
      const source = resolve(project.root, asset);
      await copyTree(source, join(payload, "assets", basename(source)));
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
    if (process.platform === "darwin") {
      const application = options.application ?? { id: `dev.solidgpui.${options.name}`, version: "0.1.0" };
      if (!/^[A-Za-z0-9.-]+$/.test(application.id) || !/^\d+\.\d+\.\d+$/.test(application.version))
        throw new Error("macOS application identifier/version is invalid");
      const launcher = join(stage, bundleName, "Contents/MacOS", options.name);
      await writeFile(
        launcher,
        `#!/bin/sh\nset -eu\nhere="$(CDPATH= cd -- "$(dirname -- "$0")/../Resources" && pwd)"\ncd "$here"\nexec "$here/../MacOS/host" ${args}\n`,
      );
      await chmod(launcher, 0o755);
      await writeFile(
        join(stage, bundleName, "Contents/Info.plist"),
        `<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n<plist version="1.0"><dict><key>CFBundleExecutable</key><string>${options.name}</string><key>CFBundleIdentifier</key><string>${application.id}</string><key>CFBundleName</key><string>${options.name}</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>${application.version}</string><key>CFBundleVersion</key><string>${application.version}</string><key>NSHighResolutionCapable</key><true/></dict></plist>\n`,
      );
      await deliveryRun(["plutil", "-lint", join(stage, bundleName, "Contents/Info.plist")], stage);
      await writeFile(
        join(stage, "launch"),
        `#!/bin/sh\nset -eu\nhere="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"\nexec "$here/${bundleName}/Contents/MacOS/${options.name}"\n`,
      );
      await copyFile(join(stage, "launch"), join(stage, "launch.command"));
    }
    const version = (await deliveryRun([host, "--version"], stage)).trim();
    const bindings = await deliveryRun([host, "--export-native"], stage);
    await writeFile(join(payload, "native.ts"), bindings);
    await writeFile(
      join(payload, "application.json"),
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
    await writePortableArchive(stage, options.name, archive);
    const checkParent = process.env.SOLID_GPUI_CONSUMER_TEMP ?? tmpdir();
    extracted = await mkdtemp(join(checkParent, "solid-gpui-consumer-"));
    await new Bun.Archive(await Bun.file(archive).bytes()).extract(extracted);
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
    const extractedHost = join(extractedRoot, hostRelative);
    const extractedPayload =
      process.platform === "darwin" ? join(extractedRoot, bundleName, "Contents/Resources") : extractedRoot;
    if ((await deliveryRun([extractedHost, "--version"], extracted, env)).trim() !== version)
      throw new Error("Extracted version changed");
    if ((await deliveryRun([extractedHost, "--export-native"], extracted, env)) !== bindings)
      throw new Error("Extracted native contract changed");
    await deliveryRun(
      [extractedHost, "--check-app", artifacts.runtime, join(extractedPayload, bundle)],
      extractedPayload,
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

export interface SignedUpdatePackageOptions {
  /** Final, signed/notarized as required, self-contained macOS app bundle. */
  readonly bundle: string;
  readonly executable: string;
  readonly appId: string;
  readonly channel: string;
  readonly sequence: number;
  readonly version: string;
  readonly url: string;
  readonly output: string;
  /** Ed25519 SPKI public key, paired with an application-owned release signer. */
  readonly publicKey: string;
  readonly sign: (payload: Uint8Array) => Promise<Uint8Array>;
}

/** Produce the updater's exact additional USTAR/signature format without install or publication. */
export async function packageSignedUpdate(
  options: SignedUpdatePackageOptions,
): Promise<{ archive: string; feed: string; sha256: string }> {
  if (process.platform !== "darwin") throw new Error("Signed update application packaging currently requires macOS");
  if (!Number.isSafeInteger(options.sequence) || options.sequence <= 0 || !/^\d+\.\d+\.\d+$/.test(options.version))
    throw new Error("Signed update requires a positive monotonic sequence and explicit version");
  const bundle = resolve(options.bundle);
  if (
    !/^[A-Za-z0-9._-]+\.app$/.test(basename(bundle)) ||
    options.executable !== `Contents/MacOS/${basename(options.executable)}`
  )
    throw new Error("Update bundle/executable identity is invalid");
  const url = new URL(options.url);
  if (url.protocol !== "https:" || url.username || url.password || url.hash)
    throw new Error("Update artifact URL requires HTTPS without credentials or fragment");
  const files = await inventory(bundle);
  if (!files.includes(options.executable)) throw new Error("Update executable is absent");
  const info = await deliveryRun(
    ["plutil", "-convert", "json", "-o", "-", join(bundle, "Contents/Info.plist")],
    dirname(bundle),
  );
  const metadata = JSON.parse(info);
  if (
    metadata.CFBundleIdentifier !== options.appId ||
    metadata.CFBundleShortVersionString !== options.version ||
    metadata.CFBundleExecutable !== basename(options.executable)
  )
    throw new Error("Final app bundle does not match signed release identity");
  const output = resolve(options.output);
  await mkdir(output, { recursive: true });
  const stage = await mkdtemp(join(output, ".update-"));
  try {
    const extracted = join(stage, "extracted");
    await mkdir(extracted);
    const archive = join(stage, `${basename(bundle)}-${options.sequence}.tar`);
    await deliveryRun(
      ["tar", "--format=ustar", "--no-xattrs", "-cf", archive, "-C", dirname(bundle), basename(bundle)],
      dirname(bundle),
      { ...process.env, COPYFILE_DISABLE: "1" },
    );
    await deliveryRun(["tar", "-xf", archive, "-C", extracted], stage);
    const copied = join(extracted, basename(bundle));
    if (JSON.stringify(await inventory(copied)) !== JSON.stringify(files))
      throw new Error("Update archive inventory mismatch");
    for (const file of files)
      if ((await deliveryHash(join(bundle, file))) !== (await deliveryHash(join(copied, file))))
        throw new Error("Update archive changed final app content");
    const resources = join(copied, "Contents/Resources");
    const app = JSON.parse(await readFile(join(resources, "application.json"), "utf8"));
    if (app.runtime !== "quickjs") throw new Error("Signed macOS updates require a self-contained QuickJS package");
    await deliveryRun(
      [join(copied, "Contents/MacOS/host"), "--check-app", "quickjs", join(resources, app.bundle)],
      copied,
      { ...process.env, PATH: "" },
    );
    const sha256 = await deliveryHash(archive);
    const payload = Buffer.from(
      JSON.stringify({
        format: "solid-gpui-update-v1",
        appId: options.appId,
        channel: options.channel,
        platform: `macos-${process.arch === "arm64" ? "aarch64" : "x86_64"}`,
        sequence: options.sequence,
        version: options.version,
        archiveFormat: "app-tar-v1",
        url: url.href,
        archiveBytes: (await readFile(archive)).length,
        sha256,
        bundle: basename(bundle),
        executable: options.executable,
      }),
    );
    const signature = Buffer.from(await options.sign(payload));
    const key = createPublicKey(options.publicKey);
    if (key.asymmetricKeyType !== "ed25519" || signature.length !== 64 || !verify(null, payload, key, signature))
      throw new Error("Release signer did not produce a valid Ed25519 signature for the exact manifest");
    const destination = join(output, basename(archive));
    const feed = destination + ".json";
    await rename(archive, destination);
    await writeFile(
      feed,
      JSON.stringify({ payload: payload.toString("hex"), signature: signature.toString("hex") }) + "\n",
    );
    return { archive: destination, feed, sha256 };
  } finally {
    await rm(stage, { recursive: true, force: true });
  }
}
