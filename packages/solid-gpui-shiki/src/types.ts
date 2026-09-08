export interface HighlightRequest {
  readonly code: string;
  readonly language: string;
  readonly theme: string;
}

export interface HighlightRun {
  readonly text: string;
  readonly color: string;
  /** Shiki's resolved font flags: italic 1, bold 2, underline 4, strikethrough 8. */
  readonly fontStyle: number;
}

export interface HighlightResult extends HighlightRequest {
  readonly foreground: string;
  readonly background: string;
  /** Includes original line separators; concatenation exactly reproduces code. */
  readonly runs: readonly HighlightRun[];
}

/** A runtime-independent service. The caller owns and disposes its backend. */
export interface CodeHighlighter {
  highlight(request: HighlightRequest, options?: { readonly signal?: AbortSignal }): Promise<HighlightResult>;
  dispose(): void;
}

export const MAX_CODE_BYTES = 64 * 1024;
export const MAX_CODE_RUNS = 4096;

export function validateRequest(request: HighlightRequest): void {
  if (typeof request.code !== "string") throw new Error("Code must be a string");
  if (request.code.length > MAX_CODE_BYTES) throw new Error(`Code exceeds the ${MAX_CODE_BYTES}-byte code block limit`);
  if (!request.code.isWellFormed()) {
    throw new Error("Code must be a well-formed Unicode string");
  }
  if (new TextEncoder().encode(request.code).byteLength > MAX_CODE_BYTES) {
    throw new Error(`Code exceeds the ${MAX_CODE_BYTES}-byte code block limit`);
  }
  if (!request.language || !request.theme) throw new Error("Language and theme are required");
}
