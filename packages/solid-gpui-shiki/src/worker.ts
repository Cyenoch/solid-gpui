import { createHighlighter } from "shiki";
import type { HighlighterCore } from "shiki/core";
import type { WorkerRequest, WorkerResponse } from "./messages";
import { toHighlightResult } from "./tokens";

let highlighter: HighlighterCore;
const send = (message: WorkerResponse): void => postMessage(message);

globalThis.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const message = event.data;
  if (message.type === "init") {
    try {
      highlighter = await createHighlighter({
        langs: [...message.options.languages],
        themes: [...message.options.themes],
      });
      send({ type: "ready" });
    } catch (error) {
      send({ type: "fatal", error: String(error) });
    }
    return;
  }
  try {
    // The parent enforces a hard deadline by terminating this worker. Disabling
    // TextMate's soft deadline avoids publishing silently truncated line state.
    const tokens = highlighter.codeToTokens(message.request.code, {
      lang: message.request.language,
      theme: message.request.theme,
      tokenizeTimeLimit: 0,
    });
    send({ type: "result", id: message.id, result: toHighlightResult(message.request, tokens) });
  } catch (error) {
    send({ type: "error", id: message.id, error: String(error) });
  }
};
