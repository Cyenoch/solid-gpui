import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { createBunHighlighter } from "../../packages/solid-gpui-shiki/src/bun";
import type { HighlightResult } from "../../packages/solid-gpui-shiki/src/types";
import type { componentCatalog } from "./component-catalog";
import { examples } from "./src/examples";
import { markdownBlocks } from "./src/markdown";
import { excerptCode, importCode, installCode, landingCode, snippetKey } from "./src/snippets";

/** One build-time service supplies every rendered code block, including excerpts. */
export async function buildHighlights(
  catalog: ReturnType<typeof componentCatalog>,
  watch: (path: string) => void = () => {},
): Promise<HighlightResult[]> {
  const inputs = new Map<string, { code: string; language: string; theme: string }>();
  const add = (code: string, language = "tsx") =>
    inputs.set(snippetKey(code, language), { code, language, theme: "github-dark" });
  for (const name of ["Workspace", "Account", "Collections"]) {
    const path = resolve(import.meta.dirname, `src/showcase/${name}.tsx`);
    watch(path);
    const source = await readFile(path, "utf8");
    add(source);
    add(excerptCode(source));
  }
  add(landingCode);
  add(installCode, "sh");
  for (const code of Object.values(examples)) add(code);
  for (const entry of catalog) {
    for (const member of entry.members) {
      for (const field of [...member.properties, ...member.events, ...member.commands]) add(field.type, "ts");
    }
    for (const example of entry.examples) {
      add(example.source);
      add(excerptCode(example.source));
    }
    add(entry.source);
    add(excerptCode(entry.source));
    add(importCode(entry.name));
    if (entry.definitions.length) add(entry.definitions.join("\n\n"), "ts");
  }
  const docs = resolve(import.meta.dirname, "../../docs");
  for (const name of await readdir(docs)) {
    if (!name.endsWith(".md") || name === "README.md") continue;
    const path = resolve(docs, name);
    watch(path);
    for (const block of markdownBlocks(await readFile(path, "utf8"))) {
      if (block.kind === "code") add(block.text, block.language);
    }
  }
  const highlighter = await createBunHighlighter({
    languages: ["javascript", "tsx", "typescript", "rust", "bash", "toml", "json"],
    themes: ["github-dark"],
    workerURL: new URL("../../packages/solid-gpui-shiki/src/worker.ts", import.meta.url),
  });
  try {
    const results: HighlightResult[] = [];
    for (const request of inputs.values()) results.push(await highlighter.highlight(request));
    return results;
  } finally {
    highlighter.dispose();
  }
}
