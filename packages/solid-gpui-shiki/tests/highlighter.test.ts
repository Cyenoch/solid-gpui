import { afterAll, beforeAll, expect, test } from "bun:test";
import { createBunHighlighter } from "../src/bun";
import { MAX_CODE_BYTES, type CodeHighlighter } from "../src/types";

let highlighter: CodeHighlighter;
beforeAll(async () => {
  highlighter = await createBunHighlighter({
    languages: ["typescript", "tsx", "rust", "json", "markdown"],
    themes: ["github-dark", "github-light"],
  });
});
afterAll(() => highlighter.dispose());

test("Oniguruma preserves Unicode and CRLF and closes strings before multiline comments", async () => {
  const code = 'const 名称 = "😀e\u0301";\r\n/* open\r\nclose */ const n = 42;\r\n';
  const dark = await highlighter.highlight({ code, language: "typescript", theme: "github-dark" });
  expect(dark.runs.map((run) => run.text).join("")).toBe(code);
  expect(dark.runs.find((run) => run.text.includes('"😀'))?.text).toBe('"😀e\u0301"');
  expect(dark.runs.find((run) => run.text.includes("/* open"))?.color).toBe("#6A737D");
  expect(dark.runs.find((run) => run.text.includes("close */"))?.color).toBe("#6A737D");
  const light = await highlighter.highlight({ code, language: "typescript", theme: "github-light" });
  expect(light.background).not.toBe(dark.background);
  expect(light.runs.map((run) => run.text).join("")).toBe(code);
  expect(light.runs[0].color).not.toBe(dark.runs[0].color);
});

test("loaded language dependencies, empty lines and lone CR retain the exact source", async () => {
  for (const [language, code] of [
    ["tsx", 'const App = () => <div title="你好">{42}</div>;'],
    ["rust", 'fn main() { let text = r#"你好😀"#; }'],
    ["json", '{"emoji":"😀", "enabled":true}'],
    ["markdown", "# Title\n\n```typescript\nconst n = 1;\n```\n"],
    ["typescript", "\n\r\n\r"],
    ["typescript", ""],
  ]) {
    const result = await highlighter.highlight({ code, language, theme: "github-dark" });
    expect(result.runs.map((run) => run.text).join("")).toBe(code);
  }
});

test("invalid requests reject explicitly and leave the worker usable", async () => {
  const request = { code: "const n = 1;", language: "typescript", theme: "github-dark" };
  await expect(highlighter.highlight({ ...request, language: "not-a-language" })).rejects.toThrow();
  await expect(highlighter.highlight({ ...request, theme: "not-a-theme" })).rejects.toThrow();
  await expect(highlighter.highlight({ ...request, code: "😀".repeat(MAX_CODE_BYTES / 4 + 1) })).rejects.toThrow(
    "limit",
  );
  await expect(highlighter.highlight({ ...request, code: "\ud800" })).rejects.toThrow("Unicode");
  expect((await highlighter.highlight(request)).runs.map((run) => run.text).join("")).toBe(request.code);
});

test("active and queued cancellation settle promptly without cancelling another consumer", async () => {
  const active = new AbortController();
  const queued = new AbortController();
  const request = { code: "const n = 1;\n".repeat(100), language: "typescript", theme: "github-dark" };
  const first = highlighter.highlight(request, { signal: active.signal });
  const second = highlighter.highlight(request, { signal: queued.signal });
  const last = highlighter.highlight({ ...request, code: "const final = true;" });
  active.abort();
  queued.abort();
  await expect(first).rejects.toHaveProperty("name", "AbortError");
  await expect(second).rejects.toHaveProperty("name", "AbortError");
  expect((await last).code).toBe("const final = true;");
});

test("disposal rejects outstanding work and future requests", async () => {
  const service = await createBunHighlighter({ languages: ["json"], themes: ["github-dark"] });
  const request = { code: "{}", language: "json", theme: "github-dark" };
  const pending = [service.highlight(request), service.highlight(request)];
  service.dispose();
  service.dispose();
  expect((await Promise.allSettled(pending)).every((result) => result.status === "rejected")).toBe(true);
  await expect(service.highlight(request)).rejects.toThrow("disposed");
});

test("a stalled worker expires the service and rejects queued work", async () => {
  const service = await createBunHighlighter({
    languages: ["json"],
    themes: ["github-dark"],
    timeoutMs: 1000,
    workerURL: new URL("./fixtures/stalled-worker.ts", import.meta.url),
  });
  try {
    const request = { code: "{}", language: "json", theme: "github-dark" };
    const results = await Promise.allSettled([service.highlight(request), service.highlight(request)]);
    expect(results.every((result) => result.status === "rejected" && String(result.reason).includes("deadline"))).toBe(
      true,
    );
    await expect(service.highlight(request)).rejects.toThrow("deadline");
  } finally {
    service.dispose();
  }
});
