import { createRoot, Pressable, Text, View, type Style } from "@solid-gpui/core";
import { createComponent, createSignal, ErrorBoundary, onCleanup } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { CodeBlock } from "@solid-gpui/shiki";
import { createBunHighlighter } from "@solid-gpui/shiki/bun";

const highlighter = await createBunHighlighter({
  languages: ["typescript", "rust"],
  themes: ["github-dark", "github-light"],
});
const root = createRoot(new StdioTransport());
const samples = {
  typescript:
    'import { createSignal } from "solid-js";\r\n\r\nconst 名称 = "😀e\u0301";\r\n/* Native text,\r\n   with shared selection. */\r\nconst [count, setCount] = createSignal(0);\r\n',
  rust: 'fn main() {\n    let message = "Hello, GPUI — 你好😀";\n    println!("{message}");\n}\n',
};

root.render(() => {
  onCleanup(() => highlighter.dispose());
  const [light, setLight] = createSignal(false);
  const [language, setLanguage] = createSignal<keyof typeof samples>("typescript");
  const button = (label: string, action: () => void) =>
    createComponent(Pressable, {
      accessibilityRole: "button",
      accessibilityLabel: label,
      onPress: action,
      style: { padding: 10, borderRadius: 6, backgroundColor: "#334155" },
      children: createComponent(Text, { style: { color: "#ffffff", fontSize: 13, lineHeight: 18 }, children: label }),
    });
  return createComponent(View, {
    get style(): Style {
      return {
        padding: 24,
        gap: 18,
        flexGrow: 1,
        minWidth: 0,
        overflow: "scroll",
        backgroundColor: light() ? "#f8fafc" : "#0f172a",
        color: light() ? "#111827" : "#e2e8f0",
      };
    },
    children: [
      createComponent(Text, { style: { fontSize: 22, fontWeight: "bold" }, children: "Shiki · Native code" }),
      createComponent(Text, {
        children: "Select and copy the code. Switch language or theme, then resize the window.",
      }),
      createComponent(View, {
        style: { flexDirection: "row", gap: 12 },
        children: [
          button("Switch theme", () => setLight((value) => !value)),
          button("Switch language", () => setLanguage((value) => (value === "typescript" ? "rust" : "typescript"))),
        ],
      }),
      createComponent(ErrorBoundary, {
        fallback: (error) => createComponent(Text, { children: `Highlighting failed: ${String(error)}` }),
        get children() {
          return createComponent(CodeBlock, {
            highlighter,
            get code() {
              return samples[language()];
            },
            get language() {
              return language();
            },
            get theme() {
              return light() ? "github-light" : "github-dark";
            },
            style: { borderRadius: 8, fontFamily: "Menlo", minWidth: 0 },
          });
        },
      }),
    ],
  });
});
void root.setTitle("Shiki · Native code");
