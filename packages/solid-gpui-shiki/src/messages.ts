import type { BundledLanguage, BundledTheme } from "shiki";
import type { HighlightRequest, HighlightResult } from "./types";

export interface BunHighlighterOptions {
  readonly languages: readonly BundledLanguage[];
  readonly themes: readonly BundledTheme[];
  /** Deadline for initialization and each active request. Expiry terminates the service. */
  readonly timeoutMs?: number;
  /** Override the installed worker asset when a build tool supplies its own entry URL. */
  readonly workerURL?: string | URL;
}

export type WorkerRequest =
  | { readonly type: "init"; readonly options: Pick<BunHighlighterOptions, "languages" | "themes"> }
  | { readonly type: "highlight"; readonly id: number; readonly request: HighlightRequest };

export type WorkerResponse =
  | { readonly type: "ready" }
  | { readonly type: "fatal"; readonly error: string }
  | { readonly type: "result"; readonly id: number; readonly result: HighlightResult }
  | { readonly type: "error"; readonly id: number; readonly error: string };
