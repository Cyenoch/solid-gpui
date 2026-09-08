globalThis.onmessage = (event: MessageEvent<{ type: string }>) => {
  if (event.data.type === "init") postMessage({ type: "ready" });
  // A stalled backend must not keep its callers or the Bun process alive.
};
