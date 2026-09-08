import { Text, type SolidElement, type Style, type TextProps } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import { validateRequest, type CodeHighlighter, type HighlightResult, type HighlightRun } from "./types";

export type { CodeHighlighter, HighlightRequest, HighlightResult, HighlightRun } from "./types";
export { MAX_CODE_BYTES, MAX_CODE_RUNS } from "./types";

interface CodeAppearanceProps {
  readonly style?: Style;
  readonly selectable?: boolean;
  readonly accessibilityLabel?: string;
  readonly onLayout?: TextProps["onLayout"];
}

export interface CodeBlockProps extends CodeAppearanceProps {
  readonly highlighter: CodeHighlighter;
  readonly code: string;
  readonly language: string;
  readonly theme: string;
}

export interface HighlightedCodeProps extends CodeAppearanceProps {
  readonly highlighted: HighlightResult;
}

/** Render a previously computed result without loading a highlighting runtime. */
export function HighlightedCode(props: HighlightedCodeProps): SolidElement {
  const children = createMemo(() =>
    props.highlighted.runs.map((run) =>
      createComponent(Text, {
        style: runStyle(run),
        children: run.text,
      }),
    ),
  );
  return renderParagraph(
    props,
    () => props.highlighted,
    () => props.highlighted.language,
    children,
  );
}

function runStyle(run: HighlightRun): Style {
  return {
    color: run.color,
    fontWeight: run.fontStyle & 2 ? "bold" : "normal",
    fontStyle: run.fontStyle & 1 ? "italic" : "normal",
    textDecoration: run.fontStyle & 4 ? "underline" : run.fontStyle & 8 ? "lineThrough" : "none",
  };
}

/** A bounded, selectable native paragraph. Errors propagate to Solid's ErrorBoundary. */
export function CodeBlock(props: CodeBlockProps): SolidElement {
  const [result, setResult] = createSignal<HighlightResult>();
  const [error, setError] = createSignal<unknown>();
  createEffect(() => {
    const request = { code: props.code, language: props.language, theme: props.theme };
    const highlighter = props.highlighter;
    validateRequest(request);
    const controller = new AbortController();
    setResult(undefined);
    setError(undefined);
    highlighter.highlight(request, { signal: controller.signal }).then(
      (value) => {
        if (!controller.signal.aborted) setResult(value);
      },
      (reason) => {
        if (!controller.signal.aborted) setError(() => reason);
      },
    );
    onCleanup(() => controller.abort());
  });
  const children = createMemo(() => {
    if (error()) throw error();
    const highlighted = result();
    if (!highlighted) return props.code;
    return highlighted.runs.map((run) => createComponent(Text, { style: runStyle(run), children: run.text }));
  });
  return renderParagraph(props, result, () => props.language, children);
}

function renderParagraph(
  props: CodeAppearanceProps,
  result: () => HighlightResult | undefined,
  language: () => string,
  children: () => SolidElement,
): SolidElement {
  return createComponent(Text, {
    get style() {
      return {
        fontFamily: "monospace",
        fontSize: 13,
        lineHeight: 20,
        padding: 16,
        ...props.style,
        ...(result() ? { color: result()!.foreground, backgroundColor: result()!.background } : {}),
      };
    },
    get selectable() {
      return props.selectable ?? true;
    },
    get accessibilityLabel() {
      return props.accessibilityLabel ?? `${language()} code`;
    },
    get onLayout() {
      return props.onLayout;
    },
    get children() {
      return children();
    },
  });
}
