import type { WebHost } from "@solid-gpui/core/web";
export interface WasmHost extends WebHost {
  default(): Promise<unknown>;
  start(): Promise<void>;
}
export async function loadHost(): Promise<WasmHost> {
  const host: WasmHost = await import("./wasm/solid_gpui_web.js");
  await host.default();
  await host.start();
  return host;
}
