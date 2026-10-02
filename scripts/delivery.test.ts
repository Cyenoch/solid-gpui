import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import schema from "../packages/solid-gpui/src/protocol/schema-lock.json";
import {
  deliveryHash,
  deliveryVersion,
  installStockHost,
  scaffoldApplication,
  stockTarget,
} from "../packages/solid-gpui-vite/src/delivery.ts";

test("an unpublished scaffold refuses to install old public packages under the same version", async () => {
  const root = await mkdtemp(join(process.env.TMPDIR!, "delivery-scaffold-"));
  try {
    await expect(scaffoldApplication({ directory: join(root, "counter"), runtime: "quickjs" })).rejects.toThrow(
      "unpublished scaffold requires --manifest",
    );
    await expect(installStockHost({ root })).rejects.toThrow("unpublished 0.5.2 candidate");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("stock acquisition rejects a corrupt host before attempting to execute or select it", async () => {
  const root = await mkdtemp(join(process.env.TMPDIR!, "delivery-corrupt-"));
  try {
    const host = join(root, "host.bin");
    await writeFile(host, "untrusted bytes");
    const bindings = join(root, "native.ts");
    await writeFile(bindings, "export {};\n");
    const manifest = join(root, "candidate.json");
    await writeFile(
      manifest,
      JSON.stringify({
        format: 1,
        version: deliveryVersion(),
        revision: "a".repeat(40),
        target: stockTarget(),
        protocolVersion: schema.protocolVersion,
        schemaDigest: schema.sha256,
        runtimes: ["bun", "quickjs"],
        host: { file: "host.bin", bytes: 15, sha256: "c".repeat(64) },
        bindings: { file: "native.ts", bytes: 11, sha256: await deliveryHash(bindings) },
        source: { file: "source.tar.gz", bytes: 1, sha256: "d".repeat(64) },
        packages: {
          core: { file: "core.tgz", bytes: 1, sha256: "e".repeat(64) },
          vite: { file: "vite.tgz", bytes: 1, sha256: "f".repeat(64) },
        },
        javascript: { core: "a".repeat(64), vite: "b".repeat(64) },
      }),
    );
    await expect(installStockHost({ root, manifest })).rejects.toThrow("checksum or size mismatch");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
