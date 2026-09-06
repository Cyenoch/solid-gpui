import { createComponent as createSolidComponent } from "solid-js";
import type { SolidChild } from "./renderer/types";

export {
  batch,
  createEffect,
  createMemo,
  createRenderEffect,
  createRoot,
  createSignal,
  onCleanup,
  untrack,
} from "solid-js";

import { solidRenderer } from "./renderer";

export const render = solidRenderer.render;
export const effect = solidRenderer.effect;
export const memo = solidRenderer.memo;
// Solid's universal Renderer type narrows components to its DOM-shaped JSX
// element type, while the runtime accepts any universal host return value.
const createUniversalComponent = createSolidComponent as unknown as <Props extends object>(
  component: (props: Props) => SolidChild,
  props: Props,
) => SolidChild;

export function createComponent<Props extends object>(
  component: (props: Props) => SolidChild,
  props: Props,
): SolidChild {
  return createUniversalComponent(component, props);
}
export const createElement = solidRenderer.createElement;
export const createTextNode = solidRenderer.createTextNode;
export const insertNode = solidRenderer.insertNode;
export const insert = solidRenderer.insert;
export const spread = solidRenderer.spread;
export const setProp = solidRenderer.setProp;
export const mergeProps = solidRenderer.mergeProps;
export const use = solidRenderer.use;
