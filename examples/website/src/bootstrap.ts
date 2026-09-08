import { showStartupError } from "./startup";

if (typeof WebAssembly === "undefined" || (!("gpu" in navigator) && typeof WebGL2RenderingContext === "undefined")) {
  showStartupError("Required browser APIs are unavailable: WebAssembly and WebGPU or WebGL2.", true);
} else {
  void import("./main").catch((error: unknown) => showStartupError(error));
}
