import { createMemo } from "solid-js";
import { createHostElement } from "./renderer";
import type { HostNodeInternal, HostProps, ViewProps } from "./renderer/types";
import type { AlignItems, FlexDirection, JustifyContent, StyleProp } from "./style";

/** The flex-container shorthands `Row` and `Column` accept beside every `View` prop. */
export interface FlexContainerProps extends ViewProps {
  /** Space between children, in logical pixels. */
  readonly gap?: number;
  /** Cross-axis alignment of children (`alignItems`). */
  readonly align?: AlignItems;
  /** Main-axis distribution of children (`justifyContent`). */
  readonly justify?: JustifyContent;
  /** Space inside every edge of the container, in logical pixels. */
  readonly padding?: number;
}
export type RowProps = FlexContainerProps;
export type ColumnProps = FlexContainerProps;

/** Shorthands are consumed by the helper, so they never reach the host element. */
const SHORTHANDS: Record<string, true> = { gap: true, align: true, justify: true, padding: true };

/**
 * The container's style: the caller's style plus the explicit axis and shorthands.
 * A `style.flexDirection` already states the axis, so it wins over the helper's
 * default; each shorthand is more specific than the style field it feeds.
 */
function axisStyle(axis: FlexDirection, props: FlexContainerProps): StyleProp {
  const style = props.style;
  const shorthand =
    props.gap !== undefined || props.align !== undefined || props.justify !== undefined || props.padding !== undefined;
  // Nothing to add, and the caller already stated an axis: pass the caller's own
  // style object through, so an unchanged layout keeps its identity.
  if (!shorthand && style?.flexDirection !== undefined) return style;
  return {
    ...style,
    flexDirection: style?.flexDirection ?? axis,
    ...(props.gap === undefined ? undefined : { gap: props.gap }),
    ...(props.align === undefined ? undefined : { alignItems: props.align }),
    ...(props.justify === undefined ? undefined : { justifyContent: props.justify }),
    ...(props.padding === undefined ? undefined : { padding: props.padding }),
  };
}

/**
 * The props the host element sees: the caller's props, with the axis and
 * shorthands folded into `style` and the shorthands themselves withheld — a
 * `View` rejects unknown props by name.
 *
 * The renderer publishes props by enumerating own keys, so `style` has to be
 * reported as one even when the caller never passed it.
 */
function hostProps(props: FlexContainerProps, style: () => StyleProp): HostProps {
  return new Proxy(props as unknown as HostProps, {
    get(target, property, receiver) {
      if (property === "style") return style();
      if (typeof property === "string" && Object.hasOwn(SHORTHANDS, property)) return undefined;
      return Reflect.get(target, property, receiver);
    },
    has(target, property) {
      return property === "style" || Reflect.has(target, property);
    },
    ownKeys(target) {
      const keys = Reflect.ownKeys(target);
      return keys.includes("style") ? keys : [...keys, "style"];
    },
    getOwnPropertyDescriptor(target, property) {
      if (property === "style" && !Object.hasOwn(target, property))
        return { configurable: true, enumerable: true, get: () => style() };
      return Reflect.getOwnPropertyDescriptor(target, property);
    },
  });
}

/**
 * Explicit flex container. A raw `View` becomes a flex container from any flex
 * style and falls back to a column when no direction is set; `Row`/`Column`
 * state the axis, so `gap` cannot pick one by accident.
 */
function flexContainer(axis: FlexDirection, props: FlexContainerProps): HostNodeInternal {
  const style = createMemo(() => axisStyle(axis, props));
  return createHostElement("View", hostProps(props, style));
}

/** Children laid out left to right: a native `View` with `flexDirection: "row"`. */
export const Row = (props: RowProps): HostNodeInternal => flexContainer("row", props);
/** Children laid out top to bottom: a native `View` with `flexDirection: "column"`. */
export const Column = (props: ColumnProps): HostNodeInternal => flexContainer("column", props);
