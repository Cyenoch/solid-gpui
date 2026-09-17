import { expect, test } from "bun:test";
import { MODE_LABEL } from "#mode";

/** Replaced by the fixture config's `define`, through the application's own transform. */
declare const __SOLID_GPUI_FIXTURE_MODE__: string;

// The runner states the mode it asked for. The config's alias, its `define` and
// Vite's own mode all have to agree with that request - not merely with each other,
// which a mode that never left the runner would also satisfy.
const expected = process.env.SOLID_GPUI_FIXTURE_MODE;
if (expected === undefined) throw new Error("the runner must set SOLID_GPUI_FIXTURE_MODE");

test("the requested mode reaches the config, the alias and the transform", () => {
  expect(MODE_LABEL).toBe(expected);
  expect(__SOLID_GPUI_FIXTURE_MODE__).toBe(expected);
  expect(import.meta.env.MODE).toBe(expected);
});
