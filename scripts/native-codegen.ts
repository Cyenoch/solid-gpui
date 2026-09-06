import { Command } from "commander";
import { fileURLToPath } from "node:url";
import { exportNativeBindings } from "../packages/solid-gpui/src/vite/native-export";

const root = fileURLToPath(new URL("../", import.meta.url));
const program = new Command()
  .description("Generate TypeScript from an actual Rust host; no arguments generate SDK and Gallery bindings")
  .option("--check", "verify outputs without writing")
  .option("--manifest <path>", "Cargo manifest for a custom host")
  .option("--package <name>", "Cargo package for a custom host")
  .option("--bin <name>", "Cargo binary for a custom host")
  .option("--features <features>", "comma-separated Cargo features")
  .option("--out <path>", "custom TypeScript output path");
program.parse();
const options = program.opts<{
  check?: boolean;
  manifest?: string;
  package?: string;
  bin?: string;
  features?: string;
  out?: string;
}>();
if (options.manifest || options.package || options.bin || options.features || options.out) {
  if (!options.out) program.error("Custom host generation requires --out");
  await exportNativeBindings(
    {
      manifestPath: options.manifest ?? "Cargo.toml",
      package: options.package,
      bin: options.bin,
      features: options.features?.split(","),
      output: options.out!,
      check: options.check,
    },
    root,
  );
} else {
  for (const target of [
    {
      package: "solid-gpui",
      bin: "solid-gpui-host",
      features: ["gpui-component"],
      output: "packages/solid-gpui/src/components.ts",
    },
    { package: "gallery-host", output: "examples/gallery/src/gallery/generated/native.ts" },
  ]) {
    await exportNativeBindings({ manifestPath: "Cargo.toml", ...target, check: options.check }, root);
  }
}
console.error(options.check ? "Native bindings match their hosts" : "Generated native bindings from their hosts");
