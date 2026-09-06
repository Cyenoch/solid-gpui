import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const generatedPath = resolve("crates/solid-gpui/src/protocol/generated/protocol.rs");
const source = await readFile(generatedPath, "utf8");
const normalized = source
  .replace(/\bref\s+/g, "")
  .replace(/match zelf \{/g, "match &zelf {")
  .replace(/if let Some\(v\) = ([^&\n{]+)\{/g, "if let Some(v) = &$1{");
const withClippyAllowance = normalized.startsWith("#![allow(clippy::never_loop)]")
  ? normalized
  : `#![allow(clippy::never_loop)]\n${normalized}`;
if (withClippyAllowance === source) {
  console.log("Bebop Rust bindings already normalized");
} else {
  await writeFile(generatedPath, withClippyAllowance);
  console.log(`normalized ${generatedPath}`);
}
