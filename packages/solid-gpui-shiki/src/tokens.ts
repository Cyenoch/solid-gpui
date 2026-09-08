import type { TokensResult } from "shiki/core";
import { MAX_CODE_RUNS, type HighlightRequest, type HighlightResult, type HighlightRun } from "./types";

function color(value: string | undefined): string {
  if (!value || !/^#(?:[0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(value)) {
    throw new Error("Code blocks require concrete hexadecimal theme colors");
  }
  return value.length <= 5 ? `#${[...value.slice(1)].map((digit) => digit + digit).join("")}` : value;
}

/** Convert UTF-16 token positions into lossless text runs; Rust owns UTF-8 run lengths. */
export function toHighlightResult(request: HighlightRequest, result: TokensResult): HighlightResult {
  const foreground = color(result.fg);
  const background = color(result.bg);
  const runs: HighlightRun[] = [];
  const append = (text: string, color: string, fontStyle: number): void => {
    if (!text) return;
    if (!text.isWellFormed()) throw new Error("Shiki token boundary splits a Unicode character");
    if (!Number.isInteger(fontStyle) || fontStyle < 0 || fontStyle > 15) {
      throw new Error("Unsupported Shiki token style");
    }
    if ((fontStyle & 12) === 12) {
      throw new Error("Solid GPUI text runs cannot combine underline and strikethrough");
    }
    const previous = runs.at(-1);
    if (previous?.color === color && previous.fontStyle === fontStyle) {
      runs[runs.length - 1] = { ...previous, text: previous.text + text };
    } else {
      if (runs.length >= MAX_CODE_RUNS) throw new Error(`Code exceeds the ${MAX_CODE_RUNS}-run code block limit`);
      runs.push({ text, color, fontStyle });
    }
  };
  let offset = 0;
  for (const line of result.tokens) {
    for (const token of line) {
      const end = token.offset + token.content.length;
      if (
        token.offset < offset ||
        end > request.code.length ||
        request.code.slice(token.offset, end) !== token.content
      ) {
        throw new Error("Shiki token ranges do not match the source");
      }
      if (token.bgColor) throw new Error("Solid GPUI inline text does not support token backgrounds");
      append(request.code.slice(offset, token.offset), foreground, 0);
      append(token.content, color(token.color ?? result.fg), token.fontStyle ?? 0);
      offset = end;
    }
  }
  append(request.code.slice(offset), foreground, 0);
  return { ...request, foreground, background, runs };
}
