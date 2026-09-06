export {
  mountApplication,
  type ApplicationDefinition,
  type ApplicationOptions,
  type MountedApplication,
} from "./application";
export type {
  ClipboardImage,
  ClipboardImageFormat,
  ExtensionField,
  ExtensionProperties,
  ExtensionValue,
} from "./protocol";
export { ICON_NAMES } from "./protocol";
import { createMemo, createSignal, mapArray } from "solid-js";
import { createHostElement } from "./renderer";
import type {
  HostNode,
  HostKind,
  HostNodeInternal,
  HostProps,
  ImageProps,
  IconProps,
  PressableProps,
  TextInputHandle,
  TextInputProps,
  TextProps,
  ViewHandle,
  ViewProps,
  VirtualListHandle,
  SolidChild,
  VirtualListProps,
} from "./renderer/types";
import { createStyleSheet } from "./style";

export {
  createRoot,
  createHostElement,
  createExtensionElement,
  defineExtensionComponent,
  solidRenderer,
  SurfaceClosedError,
  type AccessibilityProps,
  type AnimationCompleteEvent,
  type HostKind,
  type HostNode,
  type SolidChild,
  type SolidElement,
  type Draggable,
  type DragDropHandler,
  type DragOverHandler,
  type ExternalFileDropHandler,
  type ImageObjectFit,
  type ImageProps,
  type IconProps,
  type IconName,
  type KeyAction,
  type KeyEvent,
  type FocusEvent,
  type FocusHandler,
  type KeyHandler,
  type PointerButton,
  type PointerEvent,
  type PointerMoveEvent,
  type PointerDownOutsideEvent,
  type PointerDownOutsideHandler,
  type PointerHandler,
  type PointerMoveHandler,
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
  type Keybinding,
  type NotificationResponse,
  type NotificationResponseHandler,
  type NotificationAction,
  type MenuDefinition,
  type MenuItem,
  type NotificationOptions,
  type PickFilesOptions,
  type PickSavePathOptions,
  type SurfaceKind,
  type SurfaceOpenOptions,
  type SurfaceOptions,
  type ExtensionDescriptor,
  type ExtensionEvent,
  type ExtensionEventHandler,
  type ExtensionProps,
  type ExtensionComponentProps,
} from "./renderer";
export { createSurfaceHost, SurfaceIdReusedError, type SurfaceHost, type SurfaceHostOptions } from "./surface-host";
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
  type TransportTerminationCause,
  type TransportTerminationDetails,
  type TransportTerminationListener,
} from "./transport";
export {
  type BoxShadow,
  type BoxShadowInput,
  createStyleSheet,
  type AlignItems,
  type AlignSelf,
  type CursorStyle,
  type FlexDirection,
  type TextAlign,
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

function hostComponent<Props extends object>(kind: HostKind): (props: Props) => HostNodeInternal {
  return (props: Props): HostNodeInternal => createHostElement(kind, props as HostProps);
}

export const View = hostComponent<ViewProps>("View");
export const Image = hostComponent<ImageProps>("Image");
export const TextInput = hostComponent<TextInputProps>("TextInput");
export const Icon = hostComponent<IconProps>("Icon");
export const Text = hostComponent<TextProps>("Text");
export const Pressable = hostComponent<PressableProps>("Pressable");

export function VirtualList<T>(props: VirtualListProps<T>): SolidChild {
  const initialCount = Math.min(props.data.length, props.initialNumToRender ?? 10);
  const [range, setRange] = createSignal<readonly [number, number]>([0, initialCount]);
  let endReached = false;
  const onVisibleRange = (start: number, end: number): void => {
    const nextStart = Math.max(0, Math.min(start, props.data.length));
    const nextEnd = Math.max(nextStart, Math.min(end, props.data.length));
    setRange([nextStart, nextEnd]);
    if (nextEnd >= props.data.length) {
      if (!endReached) {
        endReached = true;
        props.onEndReached?.();
      }
    } else {
      endReached = false;
    }
  };
  const committedRange = createMemo<readonly [number, number]>(() => {
    const count = props.data.length;
    const [start, end] = range();
    // A previously empty list or a filter shorter than the old viewport must
    // publish real initial rows, never an inverted/out-of-bounds wire range.
    if (start >= count || end <= start) return [0, Math.min(count, props.initialNumToRender ?? 10)];
    return [start, Math.min(end, count)];
  });
  // Retain only the current window. mapArray gives each row its own Solid owner
  // and disposes it when it leaves; overlapping rows retain their host identity.
  type Row = { key: string; item: T; index: number };
  let previousRows = new Map<string, Row>();
  const visibleRows = createMemo(() => {
    const [start, end] = committedRange();
    const rows: Row[] = [];
    const nextRows = new Map<string, Row>();
    const keys = new Set<string>();
    for (let index = start; index < Math.min(end, props.data.length); index += 1) {
      const item = props.data[index];
      if (item === undefined) continue;
      const key = props.itemKey(item, index);
      if (typeof key !== "string" && typeof key !== "number") {
        throw new TypeError("VirtualList itemKey must return a string or number");
      }
      const normalized = String(key);
      if (keys.has(normalized)) {
        throw new TypeError(`VirtualList itemKey must be unique in committed range (duplicate ${normalized})`);
      }
      keys.add(normalized);
      const previous = previousRows.get(normalized);
      const row =
        previous && Object.is(previous.item, item) && previous.index === index
          ? previous
          : { key: normalized, item, index };
      rows.push(row);
      nextRows.set(normalized, row);
    }
    previousRows = nextRows;
    return rows;
  });
  const visibleChildren = mapArray(visibleRows, (row) =>
    createHostElement("View", { key: row.key, children: props.renderItem(row.item, row.index) }),
  );
  const node = createHostElement("VirtualList", {
    get style() {
      return props.style;
    },
    get ref() {
      return props.ref;
    },
    get __itemCount() {
      return props.data.length;
    },
    get __rangeStart() {
      return committedRange()[0];
    },
    get __rangeEnd() {
      return committedRange()[1];
    },
    get __estimatedItemSize() {
      return props.estimatedItemSize;
    },
    get __overscan() {
      return props.overscan ?? 2;
    },
    __onVisibleRange: onVisibleRange,
    get children() {
      return visibleChildren();
    },
  });
  return () => {
    if (props.data.length === 0) {
      endReached = false;
      return props.emptyState ?? null;
    }
    return node;
  };
}

export const StyleSheet = { create: createStyleSheet } as const;
