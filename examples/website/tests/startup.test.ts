import { expect, test } from "bun:test";
import { isUnsupportedBrowser } from "../src/startup";

test("only exhausted graphics backends are reported as unsupported", () => {
  expect(
    isUnsupportedBrowser(new Error("No browser graphics backend could be initialized. Tried WebGPU, then WebGL2.")),
  ).toBe(true);
  expect(isUnsupportedBrowser("No browser graphics backend could be initialized.")).toBe(true);
  expect(isUnsupportedBrowser(new TypeError("Failed to fetch dynamically imported module"))).toBe(false);
  expect(isUnsupportedBrowser(new Error("Application render failed"))).toBe(false);
});
