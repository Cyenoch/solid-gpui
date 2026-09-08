import {
  ErrorBoundary as solidErrorBoundary,
  Suspense as solidSuspense,
  lazy as solidLazy,
  createComponent as createSolidComponent,
} from "solid-js";
import type { SolidChild } from "./renderer/types";

export {
  batch,
  createEffect,
  createMemo,
  createRenderEffect,
  createRoot,
  createSignal,
  createResource,
  useTransition,
  startTransition,
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

// Solid control flow is host-independent; these types describe native children.
export type Component<Props = {}> = (props: Props) => SolidChild;
export const Suspense = solidSuspense as unknown as Component<{
  children: SolidChild;
  fallback?: SolidChild;
}>;
export const ErrorBoundary = solidErrorBoundary as unknown as Component<{
  children: SolidChild;
  fallback: SolidChild | ((error: unknown, reset: () => void) => SolidChild);
}>;
export const lazy = solidLazy as unknown as <Props>(
  load: () => Promise<{ default: Component<Props> }>,
) => Component<Props> & { preload(): Promise<{ default: Component<Props> }> };
