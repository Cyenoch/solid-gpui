import { generationHost, afterGenerationActivation } from "./generation";
import type { NativeCallOptions } from "./native-call";
import { createRoot as createSolidRoot, createSignal, runWithOwner, type Owner } from "solid-js";
import { COMMAND_OPEN_POPUP, COMMAND_CLOSE_POPUP, type CommandPayload } from "./protocol";
import type { HostTree } from "./renderer/host-tree";
import { createRenderer, type Renderer } from "solid-js/universal";

import {
  hostConfig,
  bindRootOwner,
  afterRootCommit,
  cancelScheduledCommit,
  withRoot,
  withRootTransaction,
} from "./renderer/host-config";
import { SurfaceClosedError, RootContainer } from "./renderer/root-container";
import { SurfaceRouter } from "./renderer/surface-router";

import type {
  HostKind,
  WindowActivationHandler,
  WindowResizeHandler,
  Keybinding,
  NotificationResponseHandler,
  MenuDefinition,
  HostNodeInternal,
  HostProps,
  SolidChild,
} from "./renderer/types";
import type { ExtensionDescriptor, ExtensionProps } from "./renderer/extension";
import type { Transport, TransportTerminationListener } from "./transport";
import type { Appearance } from "./hooks";
import type { ClipboardImage } from "./protocol";
export type {
  AccessibilityProps,
  AnimationCompleteEvent,
  HostKind,
  HostNode,
  SolidChild,
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
export type { IconProps } from "./renderer/types";
export type { IconName } from "./protocol";
export type SolidElement = SolidChild;
export type {
  ExtensionDescriptor,
  ExtensionEvent,
  ExtensionEventHandler,
  ExtensionProps,
  ExtensionComponentProps,
} from "./renderer/extension";
export { SurfaceClosedError } from "./renderer/root-container";
export const solidRenderer: Renderer<HostNodeInternal> = createRenderer(hostConfig);

const surfaceContexts = new WeakMap<
  HostTree,
  {
    router: SurfaceRouter;
    container: RootContainer;
    children: Set<() => void>;
  }
>();

/** Internal presentation seam: content is constructed in its destination HostTree. */
export function mountPopupSurface(
  tree: HostTree,
  options: Extract<CommandPayload, { type: "open-popup" }>,
  owner: Owner,
  content: () => SolidElement,
  closed: () => void,
  failed: (error: unknown) => void,
): () => void {
  const context = surfaceContexts.get(tree);
  if (!context || tree.isDisposed()) throw new Error("SystemPopover requires a mounted Surface");
  let disposed = false;
  let requestId: number | undefined;
  let root: Root | undefined;
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    stopWaiting();
    context.children.delete(dispose);
    if (requestId !== undefined && !context.container.unmounted) {
      void context.container
        .submitSurfaceCommand(COMMAND_CLOSE_POPUP, { type: "close-popup", requestId })
        .catch((error) => {
          if (!(error instanceof SurfaceClosedError) && !tree.isDisposed()) failed(error);
        });
    }
    root?.unmount();
  };
  const start = () => {
    void afterRootCommit(tree, () => {
      if (disposed || tree.isDisposed()) return Promise.resolve(null);
      const pending = context.container.beginSurfaceCommand(COMMAND_OPEN_POPUP, options);
      requestId = pending.requestId;
      return pending.result.catch((error) => {
        requestId = undefined;
        throw error;
      });
    })
      .then((value) => {
        if (disposed || tree.isDisposed()) return;
        if (!value || value.type !== "number" || !Number.isInteger(value.value) || value.value <= 0)
          throw new Error("native popup creation returned an invalid Surface ID");
        runWithOwner(owner, () => {
          root = createRootWithRouter(context.router, {
            surfaceId: value.value,
            epoch: context.container.epoch,
            onClose: () => {
              if (disposed) return;
              disposed = true;
              context.children.delete(dispose);
              closed();
            },
          });
          root.render(content);
        });
      })
      .catch((error) => {
        const report = !disposed && !tree.isDisposed();
        dispose();
        if (report) failed(error);
      });
  };
  context.children.add(dispose);
  const stopWaiting = afterGenerationActivation(start);
  return dispose;
}

function normalizeRefProps(props: HostProps): HostProps {
  const ref = props.ref;
  if (ref === undefined || typeof ref === "function") return props;
  if (typeof ref === "object" && ref !== null && "current" in ref) {
    const callback = (node: HostNodeInternal): void => {
      ref.current = node;
    };
    return new Proxy(props, {
      get(target, property, receiver) {
        return property === "ref" ? callback : Reflect.get(target, property, receiver);
      },
    });
  }
  throw new TypeError("Solid GPUI ref must be a callback or mutable ref object");
}

export function createHostElement(type: HostKind, props: HostProps = {}): HostNodeInternal {
  const node = solidRenderer.createElement(type);
  solidRenderer.spread(node, normalizeRefProps(props));
  return node;
}

export function createExtensionElement<Props extends object>(
  descriptor: ExtensionDescriptor<Props>,
  props: ExtensionProps<Props> = {} as ExtensionProps<Props>,
): HostNodeInternal {
  // Keep the caller's object intact: Solid getter descriptors must remain live so
  // the descriptor can observe signal-backed props during dirty finalization.
  const node = createHostElement("Extension", props as HostProps);
  solidRenderer.setProp(node, "__extensionDescriptor", descriptor);
  return node;
}

export function defineExtensionComponent<Props extends object>(
  descriptor: ExtensionDescriptor<Props>,
): (props: ExtensionProps<Props>) => HostNodeInternal {
  if (arguments.length !== 1) throw new TypeError("defineExtensionComponent accepts only a descriptor");
  return (props: ExtensionProps<Props>): HostNodeInternal => createExtensionElement(descriptor, props);
}
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
  invokeNative(
    moduleId: Uint8Array,
    moduleDigest: Uint8Array,
    functionId: number,
    args: Uint8Array,
    options?: NativeCallOptions,
  ): Promise<Uint8Array>;
  render(element: SolidElement | null): void;
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

function unwrapElement(value: unknown): unknown {
  return typeof value === "function" ? (value as () => unknown)() : value;
}

export function createRoot(transport: Transport, options: RootOptions = {}): Root {
  const router = new SurfaceRouter(transport, { maxFrameSize: options.maxFrameSize });
  return createRootWithRouter(router, options, true);
}

export function createRootWithRouter(router: SurfaceRouter, options: RootOptions = {}, startRouter = false): Root {
  const surfaceId = options.surfaceId ?? nextSurfaceId++;
  const epoch = options.epoch ?? generationHost()?.epoch ?? 1;
  let closed = false;
  let setElement: (element: unknown) => unknown;
  let disposeRender: (() => void) | undefined;
  let solidDisposed = false;
  let container: RootContainer;
  const disposeSolid = (): void => {
    if (solidDisposed) return;
    solidDisposed = true;
    try {
      disposeRender?.();
    } finally {
      disposeRender = undefined;
      if (startRouter) router.dispose();
    }
  };
  const scheduleDispatch = (dispatch: () => void): void => {
    if (closed) return;
    const hadPendingMutations = cancelScheduledCommit(container.tree);
    if (!hadPendingMutations) container.tree.beginRender();
    try {
      withRootTransaction(container.tree, dispatch);
    } catch (error) {
      container.recordUnhandledError(error);
    } finally {
      if (!container.unmounted) container.tree.commit();
    }
    container.throwIfUnhandledError();
  };
  container = new RootContainer({
    surfaceId,
    epoch,
    scheduleDispatch,
    submitFrame: (frame) => {
      router.submit(frame);
      return true;
    },
    onTransportTermination: (error) => {
      if (!closed) {
        closed = true;
        cancelScheduledCommit(container.tree);
        container.dispose(error);
        disposeSolid();
      }
      options.onTransportTermination?.(error);
    },
    onWindowResize: options.onWindowResize,
    onWindowActivation: options.onWindowActivation,
    onSurfaceClosed: () => {
      if (closed) return;
      closed = true;
      disposeSolid();
      options.onClose?.();
    },
    onCloseRequested: options.onCloseRequested,
    onAction: options.onAction,
    onAppearance: options.onAppearance,
    onNotificationResponse: options.onNotificationResponse,
  });
  const unregister = router.register(surfaceId, {
    deliver(events) {
      container.receiveEvents(events);
    },
    terminate(error) {
      container.terminate(error);
    },
  });
  container.start(unregister);
  const children = new Set<() => void>();
  surfaceContexts.set(container.tree, { router, container, children });
  if (startRouter) router.start();
  if (!closed) {
    createSolidRoot((dispose) => {
      disposeRender = dispose;
      // Bind the new Solid root once. Rendering it must not rebind the caller's owner.
      bindRootOwner(container.tree);
      withRoot(container.tree, () => {
        const [element, set] = createSignal<unknown>(null, { equals: false });
        setElement = set;
        solidRenderer.insert(container.tree.syntheticRoot, () =>
          withRoot(container.tree, () => unwrapElement(element()) as HostNodeInternal | null),
        );
      });
    });
  }
  return {
    invokeNative(
      moduleId: Uint8Array,
      moduleDigest: Uint8Array,
      functionId: number,
      args: Uint8Array,
      options?: NativeCallOptions,
    ): Promise<Uint8Array> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.invokeNative(moduleId, moduleDigest, functionId, args, options);
    },
    render(element: SolidElement | null): void {
      if (closed) throw new SurfaceClosedError(surfaceId);
      const hadPendingMutations = cancelScheduledCommit(container.tree);
      if (!hadPendingMutations) container.tree.beginRender();
      try {
        withRootTransaction(container.tree, () => setElement(() => element));
      } catch (error) {
        container.recordUnhandledError(error);
      } finally {
        container.tree.commit();
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
    toggleFullscreen(): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.toggleFullscreen();
    },
    setClosePolicy(policy: "allow" | "require-confirmation"): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.setClosePolicy(policy);
    },
    resolveCloseRequest(requestId: number, allow: boolean): Promise<void> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.resolveCloseRequest(requestId, allow);
    },
    openSurface(options?: SurfaceOpenOptions): Promise<number> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.openSurface(options);
    },
    pickFiles(options?: PickFilesOptions): Promise<string[] | null> {
      if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
      return container.pickFiles(options);
    },
    pickSavePath(options?: PickSavePathOptions): Promise<string | null> {
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
      for (const dispose of children) dispose();
      children.clear();
      closed = true;
      cancelScheduledCommit(container.tree);
      container.dispose();
      disposeSolid();
    },
  };
}
