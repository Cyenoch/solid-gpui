import { readFile, writeFile } from "node:fs/promises";
import { generateDeliveryRelease } from "./delivery-release.ts";

const messages = (await readFile("delivery-cargo.jsonl", "utf8"))
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line));
const hosts = messages.filter(
  (message) =>
    message.reason === "compiler-artifact" && message.target.name === "solid-gpui-host" && message.executable,
);
if (hosts.length !== 1) throw new Error("Cargo must report exactly one stock host executable");
const manifest = await generateDeliveryRelease({ output: "dist/delivery", executable: hosts[0].executable });
await writeFile("dist/delivery/manifest-path.txt", manifest);
