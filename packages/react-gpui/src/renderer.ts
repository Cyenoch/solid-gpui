import type { ReactNode } from "react";
import Reconciler from "react-reconciler";
import { LegacyRoot } from "react-reconciler/constants";

import { hostConfig } from "./renderer/host-config";
import { RootContainer } from "./renderer/root-container";
import type {
  AccessibilityProps,
  AnimationCompleteEvent,
  HostKind,
  HostNode,
  Draggable,
  DragDropHandler,
  DragOverHandler,
  ExternalFileDropHandler,
  KeyAction,
  KeyEvent,
  KeyHandler,
  PointerAction,
  PointerButton,
  PointerEvent,
  PointerHandler,
  PressEventType,
  PressHandler,
  ScrollDeltaKind,
  ScrollEvent,
  ScrollHandler,
  LayoutFrame,
  LayoutHandler,
  TextInputHandle,
  TextInputProps,
  TextProps,
  ViewHandle,
  ViewProps,
  PressableProps,
  VirtualListHandle,
  VirtualListProps,
  WindowActivationHandler,
  WindowResizeHandler,
  Keybinding,
  NotificationResponseHandler,
  MenuDefinition,
} from "./renderer/types";
import type { Transport, TransportTerminationListener } from "./transport";
import type { Appearance } from "./hooks";
export type {
  AccessibilityProps,
  AnimationCompleteEvent,
  HostKind,
  HostNode,
  Draggable,
  DragDropHandler,
  DragOverHandler,
  ExternalFileDropHandler,
  ImageObjectFit,
  ImageProps,
  HoverHandler,
  KeyAction,
  KeyEvent,
  KeyHandler,
  PointerAction,
  PointerButton,
  PointerEvent,
  PointerHandler,
  PressEventType,
  PressHandler,
  ScrollDeltaKind,
  ScrollEvent,
  ScrollHandler,
  LayoutFrame,
  LayoutHandler,
  TextInputHandle,
  TextInputProps,
  TextProps,
  ViewHandle,
  ViewProps,
  PressableProps,
  VirtualListHandle,
  VirtualListProps,
  WindowActivationHandler,
  WindowResizeHandler,
  Keybinding,
  NotificationResponse,
  NotificationResponseHandler,
  MenuDefinition,
  MenuItem,
} from "./renderer/types";
const renderer = Reconciler(hostConfig);
let nextSurfaceId = 1;

export interface RootOptions {
  readonly surfaceId?: number;
  readonly epoch?: number;
  readonly maxFrameSize?: number;
  readonly onTransportTermination?: TransportTerminationListener;
  readonly onClose?: () => void;
  readonly onWindowResize?: WindowResizeHandler;
  readonly onWindowActivation?: WindowActivationHandler;
  readonly onNotificationResponse?: NotificationResponseHandler;
  readonly onAction?: (action: string) => void;
  readonly onAppearance?: (appearance: Appearance) => void;
}
export type SurfaceKind = "normal" | "floating" | "dialog";
export interface SurfaceOptions {
  readonly kind?: SurfaceKind;
  readonly resizable?: boolean;
  readonly minSize?: readonly [number, number];
}
export interface SurfaceOpenOptions extends SurfaceOptions {
  readonly title?: string;
  readonly width?: number;
  readonly height?: number;
}
export interface PickFilesOptions {
  readonly title?: string;
  readonly directories?: boolean;
  readonly multiple?: boolean;
}

export interface PickSavePathOptions {
  readonly defaultName?: string;
}
export interface NotificationAction {
  readonly id: string;
  readonly label: string;
}
export interface NotificationOptions {
  readonly title: string;
  readonly body: string;
  readonly actions?: readonly NotificationAction[];
}
export interface Root {
  render(element: ReactNode): void;
  setTitle(title: string): Promise<void>;
  resize(width: number, height: number): Promise<void>;
  getWindowSize(): Promise<[number, number]>;
  setClipboardText(text: string): Promise<void>;
  getClipboardText(): Promise<string>;
  zoom(): Promise<void>;
  toggleFullscreen(): Promise<void>;
  openSurface(options?: SurfaceOpenOptions): Promise<number>;
  pickFiles(options?: PickFilesOptions): Promise<string[] | null>;
  pickSavePath(options?: PickSavePathOptions): Promise<string | null>;
  showNotification(options: NotificationOptions): Promise<void>;
  setMenus(menus: readonly MenuDefinition[]): Promise<void>;
  setKeybindings(bindings: readonly Keybinding[]): Promise<void>;
  openUrl(url: string): Promise<void>;
  focusNext(): Promise<void>;
  focusPrev(): Promise<void>;
  unmount(): void;
}

export function createRoot(transport: Transport, options: RootOptions = {}): Root {
  const surfaceId = options.surfaceId ?? nextSurfaceId++;
  const epoch = options.epoch ?? 1;
  let closed = false;
  const scheduleDispatch = (dispatch: () => void): void => {
    const batch = renderer.batchedUpdates;
    const dispatchBatch = (): void => {
      if (typeof batch === "function") batch(dispatch);
      else dispatch();
    };
    const flush = renderer.flushSyncFromReconciler;
    if (typeof flush === "function") flush(dispatchBatch);
    else dispatchBatch();
  };
  const container = new RootContainer(
    transport,
    surfaceId,
    epoch,
    options.maxFrameSize ?? 16 * 1024 * 1024,
    scheduleDispatch,
    options.onTransportTermination,
    options.onWindowResize,
    options.onWindowActivation,
    () => {
      if (closed) return;
      closed = true;
      options.onClose?.();
    },
    options.onAction,
    options.onAppearance,
    options.onNotificationResponse,
  );
  const reconcilerRoot = renderer.createContainer(
    container,
    LegacyRoot,
    null,
    false,
    null,
    "",
    (error: unknown) => {
      container.recordUnhandledError(error);
    },
    console.error,
    console.error,
    null,
  );
  return {
    render(element: ReactNode): void {
      if (closed) throw new Error("Cannot render into an unmounted root");
      container.beginRender();
      const flush = renderer.flushSyncFromReconciler;
      if (typeof flush === "function") {
        flush(() => renderer.updateContainerSync(element, reconcilerRoot, null, null));
      } else {
        renderer.updateContainerSync(element, reconcilerRoot, null, null);
      }
      container.throwIfUnhandledError();
    },
    setTitle(title: string): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot set title on an unmounted root"));
      return container.setTitle(title);
    },
    resize(width: number, height: number): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot resize an unmounted root"));
      return container.resize(width, height);
    },
    getWindowSize(): Promise<[number, number]> {
      if (closed) return Promise.reject(new Error("Cannot get window size from an unmounted root"));
      return container.getWindowSize();
    },
    setClipboardText(text: string): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot set clipboard on an unmounted root"));
      return container.setClipboardText(text);
    },
    getClipboardText(): Promise<string> {
      if (closed) return Promise.reject(new Error("Cannot get clipboard from an unmounted root"));
      return container.getClipboardText();
    },
    zoom(): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot zoom an unmounted root"));
      return container.zoom();
    },
    openSurface(options: SurfaceOpenOptions = {}): Promise<number> {
      if (closed) return Promise.reject(new Error("Cannot open a surface from an unmounted root"));
      return container.openSurface(options);
    },
    pickFiles(options: PickFilesOptions = {}): Promise<string[] | null> {
      if (closed) return Promise.reject(new Error("Cannot pick files from an unmounted root"));
      return container.pickFiles(options);
    },
    pickSavePath(options: PickSavePathOptions = {}): Promise<string | null> {
      if (closed) return Promise.reject(new Error("Cannot pick a save path from an unmounted root"));
      return container.pickSavePath(options);
    },
    showNotification(options: NotificationOptions): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot show a notification from an unmounted root"));
      return container.showNotification(options);
    },
    setMenus(menus: readonly MenuDefinition[]): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot set menus on an unmounted root"));
      return container.setMenus(menus);
    },
    setKeybindings(bindings: readonly Keybinding[]): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot set keybindings on an unmounted root"));
      return container.setKeybindings(bindings);
    },
    toggleFullscreen(): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot toggle fullscreen on an unmounted root"));
      return container.toggleFullscreen();
    },
    openUrl(url: string): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot open a URL on an unmounted root"));
      return container.openUrl(url);
    },
    focusNext(): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot focus next on an unmounted root"));
      return container.focusNext();
    },
    focusPrev(): Promise<void> {
      if (closed) return Promise.reject(new Error("Cannot focus previous on an unmounted root"));
      return container.focusPrev();
    },
    unmount(): void {
      if (closed) return;
      container.beginRender();
      const flush = renderer.flushSyncFromReconciler;
      if (typeof flush === "function") {
        flush(() => renderer.updateContainerSync(null, reconcilerRoot, null, null));
      } else {
        renderer.updateContainerSync(null, reconcilerRoot, null, null);
      }
      closed = true;
      container.dispose();
    },
  };
}
export { createSurfaceHost, type SurfaceHost, type SurfaceHostOptions } from "./surface-host";
