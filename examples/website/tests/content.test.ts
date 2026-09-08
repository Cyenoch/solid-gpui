import { expect, test } from "bun:test";
import { buildHighlights } from "../build-highlights";
import { componentCatalog } from "../component-catalog";
import { landingCode } from "../src/snippets";

test("build-time Shiki covers documentation, examples, excerpts and related types without changing code", async () => {
  const catalog = componentCatalog();
  const results = await buildHighlights(catalog);
  const types = new Map(results.filter((result) => result.language === "ts").map((result) => [result.code, result]));
  for (const page of catalog) {
    for (const member of page.members) {
      for (const field of [...member.properties, ...member.events, ...member.commands])
        expect(types.has(field.type)).toBe(true);
    }
  }
  const signature = catalog
    .find((page) => page.name === "Button")!
    .members[0].events.find((field) => field.name === "onHoverChange")!.type;
  expect(new Set(types.get(signature)!.runs.map((run) => run.color)).size).toBeGreaterThan(2);
  for (const result of results) expect(result.runs.map((run) => run.text).join("")).toBe(result.code);
  expect(results.some((result) => result.language === "sh")).toBe(true);
  expect(results.some((result) => result.language === "rust")).toBe(true);
  expect(
    new Set(results.find((result) => result.code === landingCode)!.runs.map((run) => run.color)).size,
  ).toBeGreaterThan(2);
}, 30_000);
