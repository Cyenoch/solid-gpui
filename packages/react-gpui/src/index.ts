import { createElement, forwardRef, useCallback, useRef, useState } from "react";
import type { ReactNode } from "react";
import type {
  HostNode,
  ImageProps,
  PressableProps,
  TextInputHandle,
  TextInputProps,
  TextProps,
  ViewHandle,
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
  type ImageObjectFit,
  type ImageProps,
  type KeyAction,
  type KeyEvent,
  type KeyHandler,
  type PointerAction,
  type PointerButton,
  type PointerEvent,
  type PointerHandler,
  type ScrollDeltaKind,
  type ScrollEvent,
  type ScrollHandler,
  type LayoutFrame,
  type LayoutHandler,
  type PressEventType,
  type PressHandler,
  type Root,
  type RootOptions,
  type TextInputHandle,
  type TextInputProps,
  type TextProps,
  type ViewHandle,
  type ViewProps,
  type PressableProps,
  type VirtualListHandle,
  type VirtualListProps,
  type WindowActivationHandler,
  type WindowResizeHandler,
} from "./renderer";
export type {
  MenuDefinition,
  MenuItem,
  NotificationOptions,
  PickFilesOptions,
  PickSavePathOptions,
  SurfaceOpenOptions,
} from "./renderer";
export { createSurfaceHost, type SurfaceHost, type SurfaceHostOptions } from "./surface-host";
export {
  DEFAULT_MAX_PENDING_BYTES,
  MemoryTransport,
  StdioTransport,
  TransportTerminatedError,
  createProcessTerminationHandler,
  type ByteInput,
  type ByteInputEventListener,
  type ByteInputListener,
  type ByteOutput,
  type ByteOutputEventListener,
  type DrainListener,
  type ExitFunction,
  type StdioTransportOptions,
  type Transport,
  type TransportChunk,
  type TransportListener,
  type TransportTerminationListener,
} from "./transport";
export {
  createStyleSheet,
  type AlignItems,
  type AlignSelf,
  type FlexDirection,
  type Position,
  type FontStyle,
  type FontWeight,
  type JustifyContent,
  type NamedStyles,
  type Overflow,
  type Style,
  type StyleProp,
  type TextDecoration,
  type Transition,
  type TransitionEasing,
  type TransitionProperty,
} from "./style";
export {
  createAppearanceStore,
  createWindowSizeStore,
  useAppearance,
  useWindowSize,
  type Appearance,
  type AppearanceStore,
  type WindowSize,
  type WindowSizeStore,
} from "./hooks";
export const View = forwardRef<ViewHandle, ViewProps>((props, ref) => createElement("View", { ...props, ref }));
export const Image = forwardRef<HostNode, ImageProps>((props, ref) => createElement("Image", { ...props, ref }));
export const TextInput = forwardRef<TextInputHandle, TextInputProps>((props, ref) =>
  createElement("TextInput", { ...props, ref }),
);
export const Text = forwardRef<HostNode, TextProps>((props, ref) => createElement("Text", { ...props, ref }));
export const Pressable = forwardRef<HostNode, PressableProps>((props, ref) =>
  createElement("Pressable", { ...props, ref }),
);

function VirtualListImpl<T>(props: VirtualListProps<T>, ref: React.ForwardedRef<VirtualListHandle>) {
  const defaultCount = Math.min(props.data.length, props.initialNumToRender ?? 10);
  const [range, setRange] = useState<readonly [number, number]>(() => [0, defaultCount]);
  const rangeStart = Math.min(range[0], props.data.length);
  const storedRangeEnd = range[1] === 0 && defaultCount > 0 ? defaultCount : range[1];
  const rangeEnd = Math.min(Math.max(rangeStart, storedRangeEnd), props.data.length);
  const endReached = useRef(false);
  const onVisibleRange = useCallback(
    (start: number, end: number) => {
      const nextStart = Math.max(0, Math.min(start, props.data.length));
      const nextEnd = Math.max(nextStart, Math.min(end, props.data.length));
      setRange((previous) => (previous[0] === nextStart && previous[1] === nextEnd ? previous : [nextStart, nextEnd]));
      if (nextEnd >= props.data.length) {
        if (!endReached.current) {
          endReached.current = true;
          props.onEndReached?.();
        }
      } else {
        endReached.current = false;
      }
    },
    [props.data.length, props.onEndReached],
  );
  if (props.data.length === 0) {
    endReached.current = false;
    return props.emptyState ?? null;
  }
  const children: ReactNode[] = [];
  const keys = new Set<string>();
  for (let index = rangeStart; index < rangeEnd; index += 1) {
    const item = props.data[index];
    if (item === undefined) continue;
    const key = props.itemKey(item, index);
    if (typeof key !== "string" && typeof key !== "number")
      throw new TypeError("VirtualList itemKey must return a string or number");
    const keyString = String(key);
    if (keys.has(keyString))
      throw new TypeError(`VirtualList itemKey must be unique in committed range (duplicate ${keyString})`);
    keys.add(keyString);
    children.push(createElement("View", { key: keyString }, props.renderItem(item, index)));
  }
  return createElement(
    "VirtualList",
    {
      ref,
      style: props.style,
      __itemCount: props.data.length,
      __rangeStart: rangeStart,
      __rangeEnd: rangeEnd,
      __estimatedItemSize: props.estimatedItemSize,
      __overscan: props.overscan ?? 2,
      __onVisibleRange: onVisibleRange,
    },
    children,
  );
}

export const VirtualList = forwardRef(VirtualListImpl) as <T>(
  props: VirtualListProps<T> & { ref?: React.Ref<VirtualListHandle> },
) => React.ReactElement | null;

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
      Image: ImageProps;
      VirtualList: VirtualListProps<unknown>;
    }
  }
}
