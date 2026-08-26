import type { ReactNode } from "react";

import type { StyleProp } from "../style";

export interface RootOwner {
  invalid: boolean;
  validationError: Error | undefined;
  submitCommand(node: HostNodeInternal, kind: number, payload: readonly [number, number] | null): Promise<void>;
  submitCommandValue(node: HostNodeInternal, kind: number, payload: readonly [number, number] | null): Promise<unknown>;
  setNodeProps(node: HostNodeInternal, props: HostProps): void;
  updateNodeProps(node: HostNodeInternal, props: HostProps): number;
  detachFromParent(node: HostNodeInternal): void;
  refreshChildIndexes(parent: HostNodeInternal): void;
  detachSubtree(node: HostNodeInternal): void;
  markMoved(node: HostNodeInternal): void;
  markUpdated(node: HostNodeInternal, mask: number): void;
  markDeleted(node: HostNodeInternal): void;
}

export type HostKind = "View" | "Text" | "Pressable" | "TextInput" | "RawText" | "VirtualList" | "Image";
export type ImageObjectFit = "fill" | "contain" | "cover" | "scaleDown" | "none";
export type PressEventType = "press";

export interface HostNode {
  readonly id: number;
  readonly kind: HostKind;
}
export interface AccessibilityProps {
  readonly accessibilityRole?: "button" | "text" | "textbox" | "checkbox" | "heading" | "generic";
  readonly accessibilityLabel?: string;
  readonly accessibilityDescription?: string;
  readonly accessibilityDisabled?: boolean;
  readonly accessibilityChecked?: boolean;
  readonly accessibilitySelected?: boolean;
  readonly accessibilityValue?: string;
}

export interface TextInputProps extends AccessibilityProps {
  readonly value?: string;
  readonly defaultValue?: string;
  readonly placeholder?: string;
  readonly onChangeText?: (value: string) => void;
  readonly onSelectionChange?: (selection: {
    start: number;
    end: number;
    reversed: boolean;
    composing?: { start: number; end: number } | null;
  }) => void;
  readonly onFocus?: () => void;
  readonly onBlur?: () => void;
  readonly onSubmitEditing?: (value: string) => void;
  readonly onKeyDown?: KeyHandler;
  readonly multiline?: boolean;
  readonly disabled?: boolean;
  readonly maxLength?: number;
  readonly style?: StyleProp;
  readonly children?: ReactNode;
}
export interface TextInputEvent {
  readonly type: "change" | "selection" | "focus" | "blur";
  readonly text: string;
  readonly selection: {
    start: number;
    end: number;
    reversed: boolean;
    composing: { start: number; end: number } | null;
  };
  readonly editSeq: number;
  readonly target: HostNode;
}
export interface TextInputHandle extends HostNode {
  focus(): Promise<void>;
  blur(): Promise<void>;
  setSelection(start: number, end: number): Promise<void>;
}

export interface VirtualListProps<T> extends AccessibilityProps {
  readonly data: readonly T[];
  readonly itemKey: (item: T, index: number) => string | number;
  readonly renderItem: (item: T, index: number) => ReactNode;
  readonly estimatedItemSize: number;
  readonly overscan?: number;
  readonly initialNumToRender?: number;
  readonly onEndReached?: () => void;
  /** Rendered instead of a native VirtualList when `data` is empty. */
  readonly emptyState?: ReactNode;
  readonly style?: StyleProp;
}
export interface VirtualListHandle extends HostNode {
  scrollToIndex(index: number): Promise<void>;
  scrollToEnd(): Promise<void>;
}
export interface AnimationCompleteEvent {
  readonly generation: number;
  readonly target: HostNode;
}

/** A press is a semantic notification; native cancellation is intentionally unavailable. */
export interface PressEvent {
  readonly type: PressEventType;
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
  readonly sequence: number;
  readonly target: HostNode;
}

export type PressHandler = (event: PressEvent) => void;
export type KeyAction = "down" | "repeat" | "up";
export interface KeyEvent {
  readonly key: string;
  readonly modifiers: string[];
  readonly action: KeyAction;
}
export type KeyHandler = (event: KeyEvent) => void;
export type PointerButton = "left" | "right" | "middle" | "back" | "forward";
export type PointerAction = "down" | "up";
export interface PointerEvent {
  readonly type: "pointerdown" | "pointerup";
  readonly button: PointerButton;
  readonly modifiers: string[];
  readonly clickCount: number;
  readonly target: HostNode;
}
export type PointerHandler = (event: PointerEvent) => void;
export type HoverHandler = (hovered: boolean) => void;
export interface Draggable {
  readonly type: string;
  readonly data?: unknown;
  readonly exportFiles?: readonly string[];
}
export type DragOverHandler = (dragType: string) => void;
export type DragDropHandler = (dragType: string) => void;
export type ExternalFileDropHandler = (paths: string[]) => void;
export interface ViewHandle extends HostNode {
  focus(): Promise<void>;
  blur(): Promise<void>;
  isFocused(): Promise<boolean>;
}
export type ScrollDeltaKind = "pixels" | "lines";
export interface ScrollEvent {
  readonly deltaKind: ScrollDeltaKind;
  readonly dx: number;
  readonly dy: number;
  readonly x: number;
  readonly y: number;
  readonly modifiers: string[];
}
export type WindowResizeHandler = (width: number, height: number, scaleFactor?: number) => void;
export type WindowActivationHandler = (active: boolean) => void;
export interface NotificationResponse {
  readonly tag: string;
  readonly actionId: string | null;
}
export type NotificationResponseHandler = (response: NotificationResponse) => void;
export type ScrollHandler = (event: ScrollEvent) => void;
export interface LayoutFrame {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}
export type LayoutHandler = (frame: LayoutFrame) => void;
export type MenuItem =
  | { readonly type: "separator" }
  | { readonly type: "action"; readonly name: string; readonly disabled?: boolean; readonly checked?: boolean }
  | { readonly type: "submenu"; readonly title: string; readonly items: readonly MenuItem[] };
export interface MenuDefinition {
  readonly title: string;
  readonly items: readonly MenuItem[];
}
export interface Keybinding {
  readonly keystrokes: string;
  readonly actionName: string;
}

export interface ViewProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly focusable?: boolean;
  readonly onKeyDown?: KeyHandler;
  readonly onPointerDown?: PointerHandler;
  readonly onPointerUp?: PointerHandler;
  readonly onHoverChange?: HoverHandler;
  readonly onScroll?: ScrollHandler;
  readonly onLayout?: LayoutHandler;
  readonly draggable?: Draggable;
  readonly onDragOver?: DragOverHandler;
  readonly onDrop?: DragDropHandler;
  readonly onExternalFileDrop?: ExternalFileDropHandler;
  readonly children?: ReactNode;
}
export interface ImageProps extends AccessibilityProps {
  readonly source: string;
  readonly fallbackSource?: string;
  readonly objectFit?: ImageObjectFit;
  readonly style?: StyleProp;
  readonly onLayout?: LayoutHandler;
  readonly children?: never;
}
export interface TextProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly selectable?: boolean;
  readonly onLayout?: LayoutHandler;
  readonly children?: ReactNode;
}
export interface PressableProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly onPress?: PressHandler;
  readonly focusable?: boolean;
  readonly onKeyDown?: KeyHandler;
  readonly disabled?: boolean;
  readonly onLayout?: LayoutHandler;
  readonly draggable?: Draggable;
  readonly onDragOver?: DragOverHandler;
  readonly onDrop?: DragDropHandler;
  readonly onExternalFileDrop?: ExternalFileDropHandler;
  readonly onPointerDown?: PointerHandler;
  readonly onPointerUp?: PointerHandler;
  readonly onHoverChange?: HoverHandler;
  readonly children?: ReactNode;
}
export interface HostProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly onPress?: PressHandler;
  readonly focusable?: boolean;
  readonly selectable?: boolean;
  readonly onKeyDown?: KeyHandler;
  readonly disabled?: boolean;
  readonly onPointerDown?: PointerHandler;
  readonly onPointerUp?: PointerHandler;
  readonly onHoverChange?: HoverHandler;
  readonly onScroll?: ScrollHandler;
  readonly onSubmitEditing?: (value: string) => void;
  readonly maxLength?: number;
  readonly draggable?: Draggable;
  readonly onDragOver?: DragOverHandler;
  readonly onDrop?: DragDropHandler;
  readonly onExternalFileDrop?: ExternalFileDropHandler;
  readonly source?: string;
  readonly fallbackSource?: string;
  readonly objectFit?: ImageObjectFit;
  readonly onLayout?: LayoutHandler;
  readonly children?: ReactNode;
  readonly __itemCount?: number;
  readonly __rangeStart?: number;
  readonly __rangeEnd?: number;
  readonly __estimatedItemSize?: number;
  readonly __overscan?: number;
  readonly __onVisibleRange?: (start: number, end: number) => void;
  readonly __onAnimationComplete?: (generation: number) => void;
  readonly [key: string]: unknown;
}
export interface TextInputWire {
  readonly value: string;
  readonly placeholder: string | null;
  readonly multiline: boolean;
  readonly disabled: boolean;
  readonly controlled: boolean;
  readonly ackEditSeq: number;
  readonly selectionStart: number;
  readonly selectionEnd: number;
  readonly markedStart: number | null;
  readonly markedEnd: number | null;
  readonly maxLength: number | null;
  readonly selectionReversed: boolean;
}
export interface ImageWire {
  readonly source: string;
  readonly objectFit: 1 | 2 | 3 | 4 | 5;
  readonly fallbackSource: string | null;
}
export interface VirtualListWire {
  readonly itemCount: number;
  readonly rangeStart: number;
  readonly rangeEnd: number;
  readonly estimatedItemSize: number;
  readonly overscan: number;
}
export interface DragWire {
  readonly dragType: string | null;
  readonly exportFiles: readonly string[] | null;
  readonly acceptsDragOver: boolean;
  readonly acceptsDrop: boolean;
}
export interface AccessibilityWire {
  readonly role: number;
  readonly label: string | null;
  readonly description: string | null;
  readonly disabled: boolean;
  readonly checked: boolean | null;
  readonly selected: boolean | null;
  readonly value: string | null;
}
export interface PendingCommand {
  readonly resolve: (value: unknown) => void;
  readonly reject: (error: Error) => void;
}
export interface TextInputCallbacks {
  readonly change?: (event: TextInputEvent) => void;
  readonly selection?: (event: TextInputEvent) => void;
  readonly focus?: (event: TextInputEvent) => void;
  readonly blur?: (event: TextInputEvent) => void;
  readonly submit?: (value: string) => void;
}
export interface PointerCallbacks {
  readonly down?: (event: PointerEvent) => void;
  readonly up?: (event: PointerEvent) => void;
}
export interface DragCallbacks {
  readonly over?: DragOverHandler;
  readonly drop?: DragDropHandler;
  readonly externalFileDrop?: ExternalFileDropHandler;
}
export interface HostNodeInternal extends HostNode {
  readonly kind: HostKind;
  readonly root: RootOwner;
  parent: HostNodeInternal | null;
  children: HostNodeInternal[];
  index: number;
  style: StyleProp;
  text: string | null;
  listenerId: number;
  keyListener: KeyHandler | undefined;
  pointerCallbacks: PointerCallbacks | null;
  hoverCallback: HoverHandler | undefined;
  hovered: boolean;
  scrollCallback: ScrollHandler | undefined;
  listener: PressHandler | undefined;
  focusable: boolean;
  selectable: boolean;
  disabled: boolean;
  focus?: () => Promise<void>;
  dragCallbacks: DragCallbacks;
  hostProperties: TextInputWire | VirtualListWire | ImageWire | DragWire | null;
  latestNativeText: string | null;
  latestNativeEditSeq: number;
  layoutCallback?: LayoutHandler;
  latestNativeSelection: {
    start: number;
    end: number;
    reversed: boolean;
    markedStart: number | null;
    markedEnd: number | null;
  } | null;
  blur?: () => Promise<void>;
  isFocused?: () => Promise<boolean>;
  setSelection?: (start: number, end: number) => Promise<void>;
  scrollToIndex?: (index: number) => Promise<void>;
  scrollToEnd?: () => Promise<void>;
  inputCallbacks: TextInputCallbacks | null;
  visibleRangeCallback?: (start: number, end: number) => void;
  animationCompleteCallback?: (generation: number) => void;
  lastAnimationGeneration: number | null;
  accessibility: AccessibilityWire | null;
  attached: boolean;
}
export interface HostContext {
  readonly root: RootOwner;
  readonly parentKind: HostKind | null;
}
