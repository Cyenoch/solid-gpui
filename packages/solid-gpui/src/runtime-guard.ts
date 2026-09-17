import { createComputed, createRenderEffect, createSignal } from "solid-js";

/**
 * Importing the SDK under a non-client resolution used to leave signals inert:
 * Solid's server build never schedules the renderer, so a mounted application
 * submits a snapshot and then nothing at all. A duplicated Solid link fails the
 * same silent way, with signals on one side invisible to the other.
 *
 * Both are decided structurally, not by observing effects: Solid's server build
 * aliases `createRenderEffect` to `createComputed`, and a second Solid copy
 * brings its own functions. Neither check schedules work, so they hold whatever
 * batching context a root is created in, and tools that import the SDK for its
 * utilities or types are unaffected because the checks run only when a renderer
 * root is created.
 */
const SERVER_SOLID =
  "solid-gpui: solid-js resolved to its server build, so signals never notify this renderer. Resolve Solid's client build (Bun: --conditions=browser, Vite: resolve.conditions) or run tests through @solid-gpui/vite/test, which does it for you.";
const DUPLICATE_SOLID =
  'solid-gpui: two copies of solid-js are loaded, so signals created by one copy never notify the other. Deduplicate solid-js (Vite: resolve.dedupe: ["solid-js"]) so the application and the SDK share one reactive graph.';

const REGISTRY = Symbol.for("@solid-gpui/core/solid");

interface SolidInstance {
  readonly createSignal: typeof createSignal;
  readonly createRenderEffect: typeof createRenderEffect;
  readonly createComputed: typeof createComputed;
}

const solid: SolidInstance = { createSignal, createRenderEffect, createComputed };

// Registration happens on import; only a renderer turns it into a diagnosis.
const registry = globalThis as { [key: symbol]: SolidInstance | undefined };
const previous = registry[REGISTRY];
if (!previous) registry[REGISTRY] = solid;

let verified = false;

/** Refuse to start a renderer that cannot observe the application's signals. */
export function assertSolidRuntime(): void {
  if (verified) return;
  if (createRenderEffect === createComputed) throw new Error(SERVER_SOLID);
  if (previous && previous.createSignal !== createSignal) throw new Error(DUPLICATE_SOLID);
  verified = true;
}
