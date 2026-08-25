import { createElement, forwardRef, useCallback, useRef, useState } from "react";
import type { ReactNode } from "react";
import type {
  HostNode,
  PressableProps,
  TextInputHandle,
  TextInputProps,
  TextProps,
  ViewProps,
  VirtualListHandle,
  VirtualListProps,
} from "./renderer";
import { createStyleSheet } from "./style";

export {
  createRoot,
  type AccessibilityProps,
  type AnimationCompleteEvent,
  type HostKind,
  type HostNode,
  type PressEvent,
  type PressEventType,
  type PressHandler,
  type Root,
  type RootOptions,
  type TextInputHandle,
  type TextInputProps,
  type TextProps,
  type ViewProps,
  type PressableProps,
  type VirtualListHandle,
  type VirtualListProps,
} from "./renderer";
export {
  DEFAULT_MAX_PENDING_BYTES,
  MemoryTransport,
  StdioTransport,
  type ByteInput,
  type ByteInputListener,
  type ByteOutput,
  type DrainListener,
  type StdioTransportOptions,
  type Transport,
  type TransportChunk,
  type TransportListener,
} from "./transport";
export { createStyleSheet, type FlexDirection, type NamedStyles, type Style, type StyleProp, type Transition, type TransitionEasing, type TransitionProperty } from "./style";
export const View = forwardRef<HostNode, ViewProps>((props, ref) => createElement("View", { ...props, ref }));
export const TextInput = forwardRef<TextInputHandle, TextInputProps>((props, ref) => createElement("TextInput", { ...props, ref }));
export const Text = forwardRef<HostNode, TextProps>((props, ref) => createElement("Text", { ...props, ref }));
export const Pressable = forwardRef<HostNode, PressableProps>((props, ref) => createElement("Pressable", { ...props, ref }));

function VirtualListImpl<T>(props: VirtualListProps<T>, ref: React.ForwardedRef<VirtualListHandle>) {
  const defaultCount = Math.min(props.data.length, props.initialNumToRender ?? 10);
  const [range, setRange] = useState<readonly [number, number]>(() => [0, defaultCount]);
  const rangeStart = Math.min(range[0], props.data.length);
  const rangeEnd = Math.min(Math.max(rangeStart, range[1]), props.data.length);
  const endReached = useRef(false);
  const onVisibleRange = useCallback((start: number, end: number) => {
    const nextStart = Math.max(0, Math.min(start, props.data.length));
    const nextEnd = Math.max(nextStart, Math.min(end, props.data.length));
    setRange((previous) => previous[0] === nextStart && previous[1] === nextEnd ? previous : [nextStart, nextEnd]);
    if (nextEnd >= props.data.length) {
      if (!endReached.current) {
        endReached.current = true;
        props.onEndReached?.();
      }
    } else {
      endReached.current = false;
    }
  }, [props.data.length, props.onEndReached]);
  const children: ReactNode[] = [];
  const keys = new Set<string>();
  for (let index = rangeStart; index < rangeEnd; index += 1) {
    const item = props.data[index];
    if (item === undefined) continue;
    const key = props.itemKey(item, index);
    if (typeof key !== "string" && typeof key !== "number") throw new TypeError("VirtualList itemKey must return a string or number");
    const keyString = String(key);
    if (keys.has(keyString)) throw new TypeError(`VirtualList itemKey must be unique in committed range (duplicate ${keyString})`);
    keys.add(keyString);
    children.push(createElement("View", { key: keyString }, props.renderItem(item, index)));
  }
  return createElement("VirtualList", {
    ref,
    style: props.style,
    __itemCount: props.data.length,
    __rangeStart: rangeStart,
    __rangeEnd: rangeEnd,
    __estimatedItemSize: props.estimatedItemSize,
    __overscan: props.overscan ?? 2,
    __onVisibleRange: onVisibleRange,
  }, children);
}

export const VirtualList = forwardRef(VirtualListImpl) as <T>(props: VirtualListProps<T> & { ref?: React.Ref<VirtualListHandle> }) => React.ReactElement | null;

export const StyleSheet = {
  create: createStyleSheet,
} as const;

declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      View: ViewProps;
      TextInput: TextInputProps;
      Text: TextProps;
      Pressable: PressableProps;
      VirtualList: VirtualListProps<unknown>;
    }
  }
}
