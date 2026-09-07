#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { parseArgs } from "node:util";
import { buildApplication } from "../packages/solid-gpui/src/vite/build";

const root = resolve(import.meta.dirname, "..");
const executable = "solid-gpui-gallery";
const appName = "Solid GPUI Gallery";
const identifier = "io.github.cyenoch.solid-gpui-gallery";

async function run(command: string[], cwd = root, environment: Record<string, string> = {}): Promise<string> {
  console.error(`$ ${command.join(" ")}`);
  const child = Bun.spawn(command, {
    cwd,
    env: { ...process.env, ...environment },
    stdin: "ignore",
    stdout: "pipe",
    stderr: "inherit",
  });
  const [output, status] = await Promise.all([new Response(child.stdout).text(), child.exited]);
  if (status !== 0) throw new Error(`${command[0]} failed with exit code ${status}`);
  return output;
}

async function write(path: string, contents: string): Promise<void> {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, contents);
}

async function copy(source: string, destination: string): Promise<void> {
  await mkdir(dirname(destination), { recursive: true });
  await copyFile(source, destination);
}

async function checksum(path: string): Promise<string> {
  return createHash("sha256")
    .update(await readFile(path))
    .digest("hex");
}

async function files(directory: string): Promise<string[]> {
  const result: string[] = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) result.push(...(await files(path)).map((child) => `${entry.name}/${child}`));
    else if (entry.isFile()) result.push(entry.name);
    else throw new Error(`Unexpected non-file package entry: ${path}`);
  }
  return result.sort();
}

async function archive(directory: string, destination: string): Promise<void> {
  if (process.platform === "darwin") {
    await run(["ditto", "-c", "-k", "--keepParent", directory, destination]);
  } else if (process.platform === "win32") {
    // Pass paths as environment data: PowerShell syntax must not interpret an
    // output directory containing quotes, spaces, or interpolation characters.
    await run(
      [
        "powershell.exe",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$ErrorActionPreference = 'Stop'; Compress-Archive -LiteralPath $env:SOLID_GPUI_PACKAGE_SOURCE -DestinationPath $env:SOLID_GPUI_PACKAGE_ARCHIVE -CompressionLevel Optimal",
      ],
      root,
      { SOLID_GPUI_PACKAGE_SOURCE: directory, SOLID_GPUI_PACKAGE_ARCHIVE: destination },
    );
  } else {
    await run(["tar", "-czf", destination, "-C", dirname(directory), basename(directory)]);
  }
}

async function extract(archivePath: string, destination: string): Promise<void> {
  await mkdir(destination, { recursive: true });
  if (process.platform === "darwin") {
    await run(["ditto", "-x", "-k", archivePath, destination]);
  } else if (process.platform === "win32") {
    await run(
      [
        "powershell.exe",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$ErrorActionPreference = 'Stop'; Expand-Archive -LiteralPath $env:SOLID_GPUI_PACKAGE_ARCHIVE -DestinationPath $env:SOLID_GPUI_PACKAGE_EXTRACT",
      ],
      root,
      { SOLID_GPUI_PACKAGE_ARCHIVE: archivePath, SOLID_GPUI_PACKAGE_EXTRACT: destination },
    );
  } else {
    await run(["tar", "-xzf", archivePath, "-C", destination]);
  }
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    args: Bun.argv.slice(2),
    options: { out: { type: "string" }, help: { type: "boolean", short: "h" } },
    strict: true,
  });
  if (values.help) {
    console.log("Usage: bun scripts/gallery-package.ts [--out <directory>]");
    return;
  }
  if (!["darwin", "linux", "win32"].includes(process.platform)) {
    throw new Error(`Packaging is not implemented on ${process.platform}`);
  }
  const target = (await run(["rustc", "-vV"])).match(/^host: (.+)$/m)?.[1];
  if (!target) throw new Error("rustc did not report a host target");
  const metadata = JSON.parse(await run(["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"]));
  const version: string = metadata.packages.find((item: { name: string }) => item.name === "gallery-host").version;
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error("Gallery packages require a MAJOR.MINOR.PATCH version");

  const output = resolve(values.out ?? join(root, "dist/gallery"));
  await mkdir(output, { recursive: true });
  const temporary = await mkdtemp(join(output, ".package-"));
  let extracted: string | undefined;
  try {
    const bundle = join(temporary, "gallery.js");
    await buildApplication({
      runtime: "quickjs",
      entry: join(root, "examples/gallery/src/quickjs.tsx"),
      outfile: bundle,
      sourcemap: "none",
    });
    const environment: Record<string, string> = { SOLID_GPUI_GALLERY_BUNDLE: bundle };
    if (process.platform === "darwin") {
      // Record the selected target consistently in both Mach-O and Info.plist.
      environment.MACOSX_DEPLOYMENT_TARGET = process.env.MACOSX_DEPLOYMENT_TARGET ?? "13.0";
      if (!/^\d+\.\d+(?:\.\d+)?$/.test(environment.MACOSX_DEPLOYMENT_TARGET)) {
        throw new Error("MACOSX_DEPLOYMENT_TARGET must be a numeric macOS version");
      }
    }
    await run(
      [
        "cargo",
        "build",
        "--locked",
        "--release",
        "--target",
        target,
        "--package",
        "gallery-host",
        "--bin",
        executable,
        "--features",
        "distribution",
      ],
      root,
      environment,
    );

    const name = `${executable}-${version}-${target}`;
    const stage = join(temporary, name);
    const binaryName = executable + (process.platform === "win32" ? ".exe" : "");
    let binaryRelative: string;
    let resources: string;
    if (process.platform === "darwin") {
      const contents = `${appName}.app/Contents`;
      binaryRelative = `${contents}/MacOS/${binaryName}`;
      resources = `${contents}/Resources`;
      await write(
        join(stage, contents, "Info.plist"),
        `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>${executable}</string>
  <key>CFBundleIdentifier</key><string>${identifier}</string>
  <key>CFBundleName</key><string>${appName}</string>
  <key>CFBundleDisplayName</key><string>${appName}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleShortVersionString</key><string>${version}</string>
  <key>CFBundleVersion</key><string>${version}</string>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>LSMinimumSystemVersion</key><string>${environment.MACOSX_DEPLOYMENT_TARGET}</string>
  <key>NSHighResolutionCapable</key><true/>
</dict></plist>
`,
      );
    } else if (process.platform === "linux") {
      binaryRelative = `usr/bin/${binaryName}`;
      resources = `usr/share/doc/${executable}`;
      await write(
        join(stage, `usr/share/applications/${identifier}.desktop`),
        `[Desktop Entry]
Type=Application
Name=${appName}
Comment=Explore SolidJS components rendered by native GPUI
Exec=${executable}
Terminal=false
Categories=Development;
`,
      );
    } else {
      binaryRelative = binaryName;
      resources = ".";
    }
    const binary = join(stage, binaryRelative);
    await copy(join(metadata.target_directory, target, "release", binaryName), binary);
    if (process.platform !== "win32") await chmod(binary, 0o755);
    for (const file of ["LICENSE", "THIRD-PARTY-NOTICES.md"]) {
      await copy(join(root, file), join(stage, resources, file));
    }
    await copy(join(root, "docs/distribution.md"), join(stage, "README.md"));
    if (process.platform === "darwin") {
      const libraries = await run(["otool", "-L", binary]);
      const external = libraries
        .split("\n")
        .slice(1)
        .map((line) => line.trim())
        .filter(Boolean)
        .filter((line) => !line.startsWith("/System/Library/") && !line.startsWith("/usr/lib/"));
      if (external.length > 0)
        throw new Error(`Bundle contains unbundled non-system libraries:\n${external.join("\n")}`);
      await write(join(stage, "NATIVE-DEPENDENCIES.txt"), libraries.replace(binary, binaryRelative));
      await run(["plutil", "-lint", join(stage, `${appName}.app/Contents/Info.plist`)]);
      await run(["codesign", "--force", "--sign", "-", join(stage, `${appName}.app`)]);
      await run(["codesign", "--verify", "--strict", join(stage, `${appName}.app`)]);
    } else if (process.platform === "linux") {
      const libraries = await run(["ldd", binary]);
      if (libraries.includes("not found")) throw new Error(`Missing native dependencies:\n${libraries}`);
      await write(join(stage, "NATIVE-DEPENDENCIES.txt"), libraries);
    }
    const inventory = await files(stage);
    const sums = await Promise.all(inventory.map(async (file) => `${await checksum(join(stage, file))}  ${file}`));
    await write(join(stage, "SHA256SUMS"), sums.join("\n") + "\n");
    const extension = process.platform === "linux" ? ".tar.gz" : ".zip";
    const candidate = join(temporary, name + extension);
    await archive(stage, candidate);

    extracted = await mkdtemp(join(tmpdir(), "solid-gpui-package-check-"));
    await extract(candidate, extracted);
    const extractedStage = join(extracted, name);
    for (const [index, file] of inventory.entries()) {
      if (`${await checksum(join(extractedStage, file))}  ${file}` !== sums[index]) {
        throw new Error(`Archive changed package content: ${file}`);
      }
    }
    // Run outside the repository with no JS runtime on PATH. The extracted
    // executable must own its UI, rather than accidentally using build assets.
    const extractedBinary = join(extractedStage, binaryRelative);
    const isolated = { PATH: "" };
    const reportedVersion = await run([extractedBinary, "--version"], extracted, isolated);
    if (reportedVersion.trim() !== `${executable} ${version}`)
      throw new Error("Extracted application version mismatch");
    console.error((await run([extractedBinary, "--check-bundle"], extracted, isolated)).trim());
    const destination = join(output, name + extension);
    await rename(candidate, destination);
    await write(destination + ".sha256", `${await checksum(destination)}  ${basename(destination)}\n`);
    console.log(`Gallery package verified: ${destination}`);
  } finally {
    await rm(temporary, { recursive: true, force: true });
    if (extracted) await rm(extracted, { recursive: true, force: true });
  }
}

await main();
