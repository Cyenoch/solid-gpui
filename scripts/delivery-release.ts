#!/usr/bin/env bun
import { chmod, copyFile, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import { parseArgs } from "node:util";
import {
  deliveryHash,
  deliveryRun,
  deliverySourceDigest,
  deliveryVersion,
  stockTarget,
  type DeliveryFile,
  type DeliveryManifest,
} from "../packages/solid-gpui-vite/src/delivery.ts";

const root = resolve(import.meta.dirname, "..");

/** Generate consuming-root requirements from the source manifest instead of duplicating SDK knowledge. */
function cargoRequirements(source: string): string {
  const sections = source.split(/(?=^\[)/m).filter((section) => /^\[(?:profile\.|patch\.crates-io)/.test(section));
  return sections.join("").replace(/path = "(vendor\/[^"]+)"/g, 'path = ".solid-gpui/sdk/$1"');
}

export async function generateDeliveryRelease(options: {
  output: string;
  executable: string;
  revision?: string;
}): Promise<string> {
  const output = resolve(options.output);
  await mkdir(output, { recursive: true });
  const temporary = await mkdtemp(join(output, ".delivery-"));
  try {
    const revision = options.revision ?? (await deliveryRun(["git", "rev-parse", "HEAD"], root)).trim();
    if (!/^[a-f0-9]{40}$/.test(revision)) throw new Error("Delivery requires an immutable source commit");
    if (
      (
        await deliveryRun(
          [
            "git",
            "diff",
            "--name-only",
            revision,
            "--",
            "Cargo.toml",
            "Cargo.lock",
            "crates",
            "vendor",
            "packages/solid-gpui",
            "packages/solid-gpui-vite",
          ],
          root,
        )
      ).trim()
    )
      throw new Error("Commit native inputs before generating paired release artifacts");
    const version = deliveryVersion();
    const target = stockTarget();
    const sourceManifest = await deliveryRun(["git", "show", `${revision}:Cargo.toml`], root);
    if (Bun.TOML.parse(sourceManifest).workspace && !sourceManifest.includes(`version = "${version}"`))
      throw new Error("Source and npm delivery versions differ");
    const lock = JSON.parse(
      await deliveryRun(["git", "show", `${revision}:packages/solid-gpui/src/protocol/schema-lock.json`], root),
    );
    const executableName = `solid-gpui-host-${version}-${target}${process.platform === "win32" ? ".exe" : ""}`;
    const binary = join(output, executableName);
    await copyFile(options.executable, binary);
    if (process.platform !== "win32") await chmod(binary, 0o755);
    if (
      (await deliveryRun([binary, "--version"], temporary)).trim() !==
      `solid-gpui-host ${version} protocol=v${lock.protocolVersion}`
    )
      throw new Error("Release host does not match source version/protocol");
    const bindings = join(output, `solid-gpui-native-${version}-${target}.ts`);
    await writeFile(bindings, await deliveryRun([binary, "--export-native"], temporary));
    const tar = join(temporary, "source.tar");
    await deliveryRun(
      [
        "git",
        "archive",
        "--format=tar",
        "--prefix=sdk/",
        `--output=${tar}`,
        revision,
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "LICENSE",
        "THIRD-PARTY-NOTICES.md",
        "crates",
        "vendor",
      ],
      root,
    );
    await deliveryRun(["tar", "-xf", tar, "-C", temporary], root);
    const sdk = join(temporary, "sdk");
    const sourceCargo = sourceManifest.replace(
      /members = \[[\s\S]*?\]/,
      'members = ["crates/solid-gpui", "crates/solid-gpui-macros", "crates/solid-gpui-bun-sys", "crates/gpui-iconify"]',
    );
    await writeFile(join(sdk, "Cargo.toml"), sourceCargo);
    await writeFile(join(sdk, "consumer-cargo.toml"), cargoRequirements(sourceManifest));
    await writeFile(
      join(sdk, "delivery-source.json"),
      JSON.stringify({ version, revision, protocolVersion: lock.protocolVersion, schemaDigest: lock.sha256 }, null, 2) +
        "\n",
    );
    const source = join(output, `solid-gpui-sdk-${version}-${target}.tar.gz`);
    await deliveryRun(["tar", "-czf", source, "-C", temporary, "sdk"], root);
    const record = async (path: string): Promise<DeliveryFile> => ({
      file: basename(path),
      sha256: await deliveryHash(path),
      bytes: (await readFile(path)).length,
    });
    const packages: { core?: DeliveryFile; vite?: DeliveryFile } = {};
    for (const [name, directory] of [
      ["core", "solid-gpui"],
      ["vite", "solid-gpui-vite"],
    ] as const) {
      const stage = join(temporary, name);
      await mkdir(stage);
      await deliveryRun(
        [process.execPath, "pm", "pack", "--destination", stage, "--quiet"],
        join(root, "packages", directory),
      );
      const entries = await readdir(stage);
      if (entries.length !== 1) throw new Error("Candidate packing must emit exactly one tarball");
      const destination = join(output, entries[0]!);
      await copyFile(join(stage, entries[0]!), destination);
      packages[name] = await record(destination);
    }
    const manifest: DeliveryManifest = {
      format: 1,
      version,
      revision,
      target,
      protocolVersion: lock.protocolVersion,
      schemaDigest: lock.sha256,
      runtimes: ["bun", "quickjs"],
      host: await record(binary),
      bindings: await record(bindings),
      source: await record(source),
      packages: { core: packages.core!, vite: packages.vite! },
      javascript: {
        core: await deliverySourceDigest(join(root, "packages/solid-gpui")),
        vite: await deliverySourceDigest(join(root, "packages/solid-gpui-vite")),
      },
    };
    const path = join(output, `solid-gpui-delivery-${version}-${target}.json`);
    await writeFile(path, JSON.stringify(manifest, null, 2) + "\n");
    for (const artifact of [binary, bindings, source, path])
      await writeFile(artifact + ".sha256", `${await deliveryHash(artifact)}  ${basename(artifact)}\n`);
    return path;
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  const { values } = parseArgs({ options: { out: { type: "string" }, executable: { type: "string" } }, strict: true });
  if (!values.executable)
    throw new Error("Build solid-gpui-host with gpui-component,quickjs, then pass --executable <Cargo-reported-path>");
  console.log(
    await generateDeliveryRelease({
      output: values.out ?? join(root, "dist/delivery"),
      executable: resolve(values.executable),
    }),
  );
}
