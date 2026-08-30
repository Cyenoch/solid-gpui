import type { ReactNode } from "react";
import Reconciler from "react-reconciler";
import { LegacyRoot } from "react-reconciler/constants";

import { hostConfig } from "./renderer/host-config";
import { SurfaceClosedError, RootContainer } from "./renderer/root-container";
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
  FocusEvent,
  FocusHandler,
  PointerDownOutsideEvent,
  PointerDownOutsideHandler,
  PointerButton,
  PointerEvent,
  PointerMoveEvent,
  PointerHandler,
  PointerMoveHandler,
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
import type { ClipboardImage } from "./protocol";
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
  FocusEvent,
  FocusHandler,
  PointerDownOutsideEvent,
  PointerDownOutsideHandler,
  PointerButton,
  PointerEvent,
  PointerMoveEvent,
  PointerHandler,
  PointerMoveHandler,
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
export { SurfaceClosedError } from "./renderer/root-container";
const renderer = Reconciler(hostConfig);
let nextSurfaceId = 1;

export interface RootOptions {
  readonly surfaceId?: number;
  readonly epoch?: number;
  readonly maxFrameSize?: number;
  readonly onTransportTermination?: TransportTerminationListener;
  readonly onClose?: () => void;
  readonly onCloseRequested?: (requestId: number) => void;
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
  minimizeWindow(): Promise<void>;
  getWindowBounds(): Promise<{ x: number; y: number; width: number; height: number }>;
  getWindowState(): Promise<{ fullscreen: boolean; maximized: boolean }>;
  activateWindow(): Promise<void>;
  setClipboardText(text: string): Promise<void>;
  getClipboardText(): Promise<string>;
  setClipboardImage(image: ClipboardImage): Promise<void>;
  getClipboardImage(): Promise<ClipboardImage | null>;
  zoom(): Promise<void>;
  toggleFullscreen(): Promise<void>;
  setClosePolicy(policy: "allow" | "require-confirmation"): Promise<void>;
  resolveCloseRequest(requestId: number, allow: boolean): Promise<void>;
  openSurface(options?: SurfaceOpenOptions): Promise<number>;
  pickFiles(options?: PickFilesOptions): Promise<string[] | null>;
  pickSavePath(options?: PickSavePathOptions): Promise<string | null>;
  readTextFile(path: string): Promise<string>;
  loadFont(path: string): Promise<string>;
  writeTextFile(path: string, content: string): Promise<number>;
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
    options.onCloseRequested,
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
      if (closed) throw new SurfaceClosedError(surfaceId);
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
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.setTitle(title);
    },
    resize(width: number, height: number): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.resize(width, height);
    },
    getWindowSize(): Promise<[number, number]> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.getWindowSize();
    },
    minimizeWindow(): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.minimizeWindow();
    },
    getWindowBounds(): Promise<{ x: number; y: number; width: number; height: number }> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.getWindowBounds();
    },
    getWindowState(): Promise<{ fullscreen: boolean; maximized: boolean }> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.getWindowState();
    },
    activateWindow(): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.activateWindow();
    },
    setClipboardText(text: string): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.setClipboardText(text);
    },
    getClipboardText(): Promise<string> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.getClipboardText();
    },
    setClipboardImage(image: ClipboardImage): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.setClipboardImage(image);
    },
    getClipboardImage(): Promise<ClipboardImage | null> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.getClipboardImage();
    },
    zoom(): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.zoom();
    },
    setClosePolicy(policy: "allow" | "require-confirmation"): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.setClosePolicy(policy);
    },
    resolveCloseRequest(requestId: number, allow: boolean): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.resolveCloseRequest(requestId, allow);
    },
    openSurface(options: SurfaceOpenOptions = {}): Promise<number> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.openSurface(options);
    },
    pickFiles(options: PickFilesOptions = {}): Promise<string[] | null> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.pickFiles(options);
    },
    pickSavePath(options: PickSavePathOptions = {}): Promise<string | null> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.pickSavePath(options);
    },
    readTextFile(path: string): Promise<string> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.readTextFile(path);
    },
    loadFont(path: string): Promise<string> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.loadFont(path);
    },
    writeTextFile(path: string, content: string): Promise<number> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.writeTextFile(path, content);
    },
    showNotification(options: NotificationOptions): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.showNotification(options);
    },
    setMenus(menus: readonly MenuDefinition[]): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.setMenus(menus);
    },
    setKeybindings(bindings: readonly Keybinding[]): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.setKeybindings(bindings);
    },
    toggleFullscreen(): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.toggleFullscreen();
    },
    openUrl(url: string): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.openUrl(url);
    },
    focusNext(): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.focusNext();
    },
    focusPrev(): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
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
