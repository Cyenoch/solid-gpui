import {
  ErrorBoundary as solidErrorBoundary,
  For as solidFor,
  Index as solidIndex,
  Match as solidMatch,
  Show as solidShow,
  Suspense as solidSuspense,
  Switch as solidSwitch,
  lazy as solidLazy,
  createComponent as createSolidComponent,
} from "solid-js";
import type { Accessor } from "solid-js";
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

// Solid declares its control flow against its own DOM-shaped JSX namespace:
// children are constrained to and returned as `solid-js`'s element type, which
// native JSX neither accepts nor produces. The runtime objects below are Solid's
// own renderer-agnostic implementations; only their public types are restated
// against `SolidChild`, so authored JSX gets real inference instead of casts.
export const For = solidFor as unknown as <T>(props: {
  each: readonly T[] | undefined | null | false;
  fallback?: SolidChild;
  children: (item: T, index: Accessor<number>) => SolidChild;
}) => SolidChild;

export const Index = solidIndex as unknown as <T>(props: {
  each: readonly T[] | undefined | null | false;
  fallback?: SolidChild;
  children: (item: Accessor<T>, index: number) => SolidChild;
}) => SolidChild;

export const Show = solidShow as unknown as {
  <T>(props: {
    when: T | undefined | null | false;
    keyed?: false;
    fallback?: SolidChild;
    children: SolidChild | ((item: Accessor<NonNullable<T>>) => SolidChild);
  }): SolidChild;
  <T>(props: {
    when: T | undefined | null | false;
    keyed: true;
    fallback?: SolidChild;
    children: SolidChild | ((item: NonNullable<T>) => SolidChild);
  }): SolidChild;
};

export const Switch = solidSwitch as unknown as (props: { fallback?: SolidChild; children: SolidChild }) => SolidChild;

export const Match = solidMatch as unknown as {
  <T>(props: {
    when: T | undefined | null | false;
    keyed?: false;
    children: SolidChild | ((item: Accessor<NonNullable<T>>) => SolidChild);
  }): SolidChild;
  <T>(props: {
    when: T | undefined | null | false;
    keyed: true;
    children: SolidChild | ((item: NonNullable<T>) => SolidChild);
  }): SolidChild;
};

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
