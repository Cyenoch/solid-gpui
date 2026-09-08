import { expect, test } from "bun:test";
import { createRoot, MemoryTransport, Text } from "@solid-gpui/core";
import { createComponent, createSignal, ErrorBoundary } from "@solid-gpui/core/runtime";
import { Envelope } from "../../solid-gpui/src/protocol/generated/protocol";
import { CodeBlock, type CodeHighlighter, type HighlightRequest, type HighlightResult } from "../src";

test("code/theme changes and unmount ignore stale completions and keep one selectable paragraph", async () => {
  const requests: { request: HighlightRequest; signal?: AbortSignal; resolve: (result: HighlightResult) => void }[] =
    [];
  const highlighter: CodeHighlighter = {
    highlight(request, options) {
      return new Promise((resolve) => requests.push({ request, signal: options?.signal, resolve }));
    },
    dispose() {
      throw new Error("A component does not own its shared highlighter");
    },
  };
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  const [code, setCode] = createSignal("old");
  const [theme, setTheme] = createSignal("github-dark");
  const [fontSize, setFontSize] = createSignal(13);
  const complete = (index: number): void => {
    const { request, resolve } = requests[index];
    resolve({
      ...request,
      foreground: "#ffffff",
      background: "#000000",
      runs: [{ text: request.code, color: "#ff0000", fontStyle: 3 }],
    });
  };
  const flush = async (): Promise<void> => {
    await Promise.resolve();
    await Promise.resolve();
  };
  try {
    root.render(() =>
      createComponent(CodeBlock, {
        highlighter,
        get code() {
          return code();
        },
        language: "typescript",
        get theme() {
          return theme();
        },
        get style() {
          return { fontSize: fontSize() };
        },
      }),
    );
    const snapshot = Envelope.decode(transport.submitted[0].subarray(4)).body;
    if (snapshot?.tag !== 1) throw new Error("Expected a Snapshot");
    expect(snapshot.value.nodes?.filter((node) => node.selectable)).toHaveLength(1);
    setCode("new 😀");
    await flush();
    expect(requests[0].signal?.aborted).toBe(true);
    complete(1);
    await flush();
    const patch = Envelope.decode(transport.submitted.at(-1)!.subarray(4)).body;
    expect(JSON.stringify(patch)).toContain("new 😀");
    if (patch?.tag !== 3) throw new Error("Expected a Patch");
    const styledRun = patch.value.operations
      ?.map((item) => item.operation)
      .find((operation) => operation?.tag === 1 && operation.value.node?.style?.color === 0xff0000ff);
    expect(styledRun?.tag).toBe(1);
    if (styledRun?.tag === 1) {
      expect(styledRun.value.node?.style?.fontWeight).toBe(700);
      expect(styledRun.value.node?.style?.fontStyle).toBe(1);
    }
    const count = transport.submitted.length;
    complete(0);
    await flush();
    expect(transport.submitted.length).toBe(count);
    setFontSize(18);
    await flush();
    expect(requests).toHaveLength(2);
    setTheme("github-light");
    await flush();
    expect(requests[2].request.theme).toBe("github-light");
    root.unmount();
    expect(requests[2].signal?.aborted).toBe(true);
    const closedCount = transport.submitted.length;
    complete(2);
    await flush();
    expect(transport.submitted.length).toBe(closedCount);
  } finally {
    root.unmount();
  }
});

test("highlighting errors reach the native Solid error boundary", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  const highlighter: CodeHighlighter = {
    highlight: () => Promise.reject(new Error("Unsupported grammar")),
    dispose() {},
  };
  let failure: unknown;
  try {
    root.render(() =>
      createComponent(ErrorBoundary, {
        fallback(error) {
          failure = error;
          return createComponent(Text, { children: "Highlighting failed" });
        },
        get children() {
          return createComponent(CodeBlock, { highlighter, code: "code", language: "unknown", theme: "github-dark" });
        },
      }),
    );
    await Promise.resolve();
    await Promise.resolve();
    expect(failure).toBeInstanceOf(Error);
    expect(String(failure)).toContain("Unsupported grammar");
    expect(JSON.stringify(Envelope.decode(transport.submitted.at(-1)!.subarray(4)).body)).toContain(
      "Highlighting failed",
    );
  } finally {
    root.unmount();
  }
});
