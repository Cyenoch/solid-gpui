import {
  COMMAND_BLUR,
  COMMAND_FOCUS,
  COMMAND_FOCUS_NEXT,
  COMMAND_FOCUS_PREV,
  COMMAND_GET_FOCUS,
  COMMAND_GET_WINDOW_SIZE,
  COMMAND_RESIZE_WINDOW,
  COMMAND_CLIPBOARD_READ,
  COMMAND_CLIPBOARD_READ_IMAGE,
  COMMAND_CLIPBOARD_WRITE,
  COMMAND_CLIPBOARD_WRITE_IMAGE,
  COMMAND_FILE_DIALOG_OPEN,
  COMMAND_FILE_DIALOG_SAVE,
  COMMAND_KIND,
  COMMAND_OPEN_URL,
  COMMAND_OPEN_SURFACE,
  COMMAND_LOAD_FONT,
  COMMAND_READ_TEXT_FILE,
  COMMAND_SET_KEYBINDINGS,
  COMMAND_SET_MENUS,
  COMMAND_SET_CLOSE_POLICY,
  COMMAND_RESOLVE_CLOSE_REQUEST,
  COMMAND_SHOW_NOTIFICATION,
  COMMAND_SCROLL_TO_END,
  COMMAND_SCROLL_TO_INDEX,
  COMMAND_SET_SELECTION,
  COMMAND_TOGGLE_FULLSCREEN,
  COMMAND_WRITE_TEXT_FILE,
  COMMAND_ZOOM_WINDOW,
  CLIPBOARD_IMAGE_FORMAT_GIF,
  CLIPBOARD_IMAGE_FORMAT_JPEG,
  CLIPBOARD_IMAGE_FORMAT_PNG,
  CLIPBOARD_IMAGE_FORMAT_SVG,
  FrameDecoder,
  MAX_CLIPBOARD_IMAGE_BYTES,
  MAX_CLIPBOARD_TEXT_BYTES,
  MAX_FILE_WRITE_BYTES,
  MAX_FILE_READ_BYTES,
  PROTOCOL_VERSION,
  UPDATE_LISTENER,
  UPDATE_PROPERTIES,
  UPDATE_SELECTABLE,
  UPDATE_STYLE,
  UPDATE_TOOLTIP,
  UPDATE_ACCESSIBILITY,
  UPDATE_POINTER_MOVE,
  UPDATE_TEXT,
  decodeEvent,
  encodeFrame,
  type ClipboardImage,
  type ClipboardImageFormat,
  type ClipboardImageFormatCode,
  type ClipboardImagePayload,
  type Command,
  type KeybindingsPayload,
  type MenuItemPayload,
  type MenuPayload,
  type Patch,
  type PatchOperation,
  type PressEventFrame,
  type SnapshotNode,
  type WindowOpenOptionsPayload,
  utf8ByteLength,
} from "../protocol";
import { TransportTerminatedError, type Transport, type TransportTerminationListener } from "../transport";
import { accessibilityWire, assertU32Option, hostPropertiesWire, KIND_CODES, nextU32 } from "./props";
import type { Appearance } from "../hooks";
import { encodeStyle } from "../style";
import { dispatchEvent, type DispatchContext } from "./dispatch";
import { NodeGraph } from "./nodes";
import type {
  HostKind,
  HostNodeInternal,
  HostProps,
  Keybinding,
  MenuDefinition,
  MenuItem,
  PendingCommand,
  TextInputCallbacks,
  WindowActivationHandler,
  WindowResizeHandler,
} from "./types";
import type { SurfaceOpenOptions } from "../renderer";
export class SurfaceClosedError extends Error {
  readonly surfaceId: number;

  constructor(surfaceId: number) {
    super(`surface ${surfaceId} is closed`);
    this.name = "SurfaceClosedError";
    this.surfaceId = surfaceId;
  }
}

const CLIPBOARD_IMAGE_FORMAT_CODES: Record<ClipboardImageFormat, ClipboardImageFormatCode> = {
  png: CLIPBOARD_IMAGE_FORMAT_PNG,
  jpeg: CLIPBOARD_IMAGE_FORMAT_JPEG,
  gif: CLIPBOARD_IMAGE_FORMAT_GIF,
  svg: CLIPBOARD_IMAGE_FORMAT_SVG,
};
const CLIPBOARD_IMAGE_FORMAT_NAMES: Record<ClipboardImageFormatCode, ClipboardImageFormat> = {
  [CLIPBOARD_IMAGE_FORMAT_PNG]: "png",
  [CLIPBOARD_IMAGE_FORMAT_JPEG]: "jpeg",
  [CLIPBOARD_IMAGE_FORMAT_GIF]: "gif",
  [CLIPBOARD_IMAGE_FORMAT_SVG]: "svg",
};
function normalizeClipboardImage(image: ClipboardImage): ClipboardImage {
  if (image === null || typeof image !== "object") {
    throw new TypeError("clipboard image must be an object");
  }
  if (!Object.hasOwn(CLIPBOARD_IMAGE_FORMAT_CODES, image.format)) {
    throw new TypeError("clipboard image format must be png, jpeg, gif, or svg");
  }
  const normalized = image.bytes;
  if (!(normalized instanceof Uint8Array)) {
    throw new TypeError("clipboard image bytes must be a Uint8Array");
  }
  if (normalized.byteLength === 0 || normalized.byteLength > MAX_CLIPBOARD_IMAGE_BYTES) {
    throw new RangeError("clipboard image bytes must be non-empty and within the supported size");
  }
  return { format: image.format, bytes: normalized };
}
export class RootContainer implements DispatchContext {
  readonly nodes: NodeGraph;
  readonly children: HostNodeInternal[];
  readonly syntheticRoot: HostNodeInternal;
  readonly listeners: Map<number, HostNodeInternal>;
  readonly nodesById: Map<number, HostNodeInternal>;
  readonly surfaceId: number;
  private readonly pendingCommands = new Map<number, PendingCommand>();
  private nextRequestId = 1;
  readonly epoch: number;
  revision = 0;
  bootstrapped = false;
  readonly inputListeners: Map<number, TextInputCallbacks>;
  private readonly createdIds: Set<number>;
  private readonly deletedRoots: Set<number>;
  private readonly movedIds: Set<number>;
  private readonly updatedMasks: Map<number, number>;
  private readonly scheduleDispatch: (dispatch: () => void) => void;
  private transportTerminated = false;
  private terminationError: TransportTerminatedError | undefined;
  invalid = false;
  unmounted = false;
  validationError: Error | undefined;
  unhandledError: Error | undefined;
  private lastEventSequence = 0;
  private hasEventSequence = false;
  private readonly decoder: FrameDecoder;
  private unsubscribe: (() => void) | undefined;
  private unsubscribeTermination: (() => void) | undefined;
  private readonly onTransportTermination: TransportTerminationListener | undefined;
  private readonly surfaceClosedHandler: (() => void) | undefined;
  readonly onCloseRequested: ((requestId: number) => void) | undefined;
  readonly onWindowResize: WindowResizeHandler | undefined;
  readonly onWindowActivation: WindowActivationHandler | undefined;
  readonly onAction: ((action: string) => void) | undefined;
  readonly onAppearance: ((appearance: Appearance) => void) | undefined;
  readonly onNotificationResponse:
    | ((response: { readonly tag: string; readonly actionId: string | null }) => void)
    | undefined;
  constructor(
    readonly transport: Transport,
    surfaceId: number,
    epoch: number,
    maxFrameSize: number,
    scheduleDispatch: (dispatch: () => void) => void,
    onTransportTermination?: TransportTerminationListener,
    onWindowResize?: WindowResizeHandler,
    onWindowActivation?: WindowActivationHandler,
    onSurfaceClosed?: () => void,
    onCloseRequested?: (requestId: number) => void,
    onAction?: (action: string) => void,
    onAppearance?: (appearance: Appearance) => void,
    onNotificationResponse?: (response: { readonly tag: string; readonly actionId: string | null }) => void,
  ) {
    this.surfaceId = assertU32Option("surfaceId", surfaceId);
    this.epoch = assertU32Option("epoch", epoch);
    this.nodes = new NodeGraph(this);
    this.children = this.nodes.children;
    this.syntheticRoot = this.nodes.syntheticRoot;
    this.listeners = this.nodes.listeners;
    this.nodesById = this.nodes.nodesById;
    this.inputListeners = this.nodes.inputListeners;
    this.createdIds = this.nodes.createdIds;
    this.deletedRoots = this.nodes.deletedRoots;
    this.movedIds = this.nodes.movedIds;
    this.updatedMasks = this.nodes.updatedMasks;
    this.scheduleDispatch = scheduleDispatch;
    this.onTransportTermination = onTransportTermination;
    this.onWindowResize = onWindowResize;
    this.onWindowActivation = onWindowActivation;
    this.surfaceClosedHandler = onSurfaceClosed;
    this.onCloseRequested = onCloseRequested;
    this.onAction = onAction;
    this.onAppearance = onAppearance;
    this.onNotificationResponse = onNotificationResponse;
    this.decoder = new FrameDecoder(maxFrameSize);
    this.unsubscribe = transport.onData((chunk) => this.receive(chunk));
    this.unsubscribeTermination = transport.onTermination((error) => this.handleTransportTermination(error));
  }
  private handleTransportTermination(error: TransportTerminatedError): void {
    if (this.unmounted || this.transportTerminated) return;
    this.transportTerminated = true;
    this.terminationError = error;
    this.invalid = true;
    this.detachTransportListeners();
    for (const pending of this.pendingCommands.values()) pending.reject(error);
    this.pendingCommands.clear();
    this.onTransportTermination?.(error);
  }
  private detachTransportListeners(): void {
    const unsubscribe = this.unsubscribe;
    this.unsubscribe = undefined;
    unsubscribe?.();
    const unsubscribeTermination = this.unsubscribeTermination;
    this.unsubscribeTermination = undefined;
    unsubscribeTermination?.();
  }
  private failProtocol(detail: unknown): void {
    const message = detail instanceof Error ? detail.message : String(detail);
    this.handleTransportTermination(
      new TransportTerminatedError(`RootContainer protocol failure: ${message}`, {
        kind: "protocol",
        detail: message,
      }),
    );
  }

  private submitFrame(frame: Uint8Array): boolean {
    if (this.transportTerminated) return false;
    try {
      this.transport.submit(frame);
      return true;
    } catch (error) {
      if (error instanceof TransportTerminatedError) {
        this.handleTransportTermination(error);
        return false;
      }
      throw error;
    }
  }
  allocateNode(kind: HostKind): HostNodeInternal {
    const node = this.nodes.allocateNode(kind);
    if (this.bootstrapped) this.createdIds.add(node.id);
    return node;
  }

  allocateListener(node: HostNodeInternal): number {
    return this.nodes.allocateListener(node);
  }
  setNodeProps(node: HostNodeInternal, props: HostProps): void {
    this.nodes.setNodeProps(node, props);
  }
  updateNodeProps(node: HostNodeInternal, props: HostProps): number {
    return this.nodes.updateNodeProps(node, props);
  }
  detachFromParent(node: HostNodeInternal): void {
    this.nodes.detachFromParent(node);
  }
  refreshChildIndexes(parent: HostNodeInternal): void {
    this.nodes.refreshChildIndexes(parent);
  }
  detachSubtree(node: HostNodeInternal): void {
    this.nodes.detachSubtree(node);
  }
  releaseDetachedFocus(node: HostNodeInternal): void {
    this.nodes.releaseDetachedFocus(node);
  }

  private submitCommandFrame(command: Command): Promise<unknown> {
    const requestId = command[5];
    return new Promise<unknown>((resolve, reject) => {
      this.pendingCommands.set(requestId, { resolve, reject });
      try {
        if (!this.submitFrame(encodeFrame(command))) {
          this.pendingCommands.delete(requestId);
          reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
        }
      } catch (error) {
        this.pendingCommands.delete(requestId);
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  submitCommand(node: HostNodeInternal, kind: number, payload: readonly [number, number] | null): Promise<void> {
    return this.submitCommandValue(node, kind, payload).then(() => undefined);
  }
  submitCommandValue(
    node: HostNodeInternal,
    kind: number,
    payload: readonly [number, number] | null,
  ): Promise<unknown> {
    if (this.transportTerminated) {
      return Promise.reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
    }
    if (this.unmounted) return Promise.reject(new SurfaceClosedError(this.surfaceId));
    if (!node.attached) return Promise.reject(new Error("host node is unavailable"));
    const isInput = node.kind === "TextInput";
    const isList = node.kind === "VirtualList";
    const isView = node.kind === "View";
    if (!isInput && !isList && !isView) return Promise.reject(new Error("host node does not support commands"));
    if (isView && !node.focusable) return Promise.reject(new Error("View is not focusable"));
    if (
      isInput &&
      !([COMMAND_FOCUS, COMMAND_BLUR, COMMAND_SET_SELECTION, COMMAND_GET_FOCUS] as number[]).includes(kind)
    )
      return Promise.reject(new Error("unknown TextInput command"));
    if (isView && !([COMMAND_FOCUS, COMMAND_BLUR, COMMAND_GET_FOCUS] as number[]).includes(kind))
      return Promise.reject(new Error("unknown View command"));
    if (isList && !([COMMAND_SCROLL_TO_INDEX, COMMAND_SCROLL_TO_END] as number[]).includes(kind))
      return Promise.reject(new Error("unknown VirtualList command"));
    if (
      kind === COMMAND_SET_SELECTION &&
      (payload === null ||
        !payload.every((value) => Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff) ||
        payload[0] > payload[1])
    )
      return Promise.reject(new Error("invalid UTF-16 selection"));
    if (
      kind === COMMAND_SCROLL_TO_INDEX &&
      (payload === null ||
        !Number.isInteger(payload[0]) ||
        payload[0] < 0 ||
        payload[0] > 0xffff_ffff ||
        (node.hostProperties && "itemCount" in node.hostProperties && payload[0] >= node.hostProperties.itemCount))
    )
      return Promise.reject(new Error("VirtualList index is out of range"));
    let requestId: number;
    try {
      requestId = this.nextRequestId;
      this.nextRequestId = nextU32(this.nextRequestId, "command request id");
    } catch (error) {
      return Promise.reject(error);
    }
    const command: Command = [
      PROTOCOL_VERSION,
      COMMAND_KIND,
      this.surfaceId,
      this.epoch,
      this.revision,
      requestId,
      node.id,
      kind as Command[7],
      payload,
    ];
    return this.submitCommandFrame(command);
  }
  setTitle(title: string): Promise<void> {
    if (this.transportTerminated) {
      return Promise.reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
    }
    if (this.unmounted) return Promise.reject(new SurfaceClosedError(this.surfaceId));
    if (typeof title !== "string" || title.length === 0 || [...title].length > 256) {
      return Promise.reject(new TypeError("title must be a non-empty string of at most 256 characters"));
    }
    let requestId: number;
    try {
      requestId = this.nextRequestId;
      this.nextRequestId = nextU32(this.nextRequestId, "command request id");
    } catch (error) {
      return Promise.reject(error);
    }
    const command: Command = [
      PROTOCOL_VERSION,
      COMMAND_KIND,
      this.surfaceId,
      this.epoch,
      this.revision,
      requestId,
      1,
      6,
      title,
    ];
    return this.submitCommandFrame(command).then(() => undefined);
  }

  beginRender(): void {
    this.invalid = false;
    this.validationError = undefined;
    this.unhandledError = undefined;
    this.clearMutations();
  }

  recordUnhandledError(error: unknown): void {
    this.unhandledError = error instanceof Error ? error : new Error(String(error));
  }
  private submitSurfaceCommandValue(
    kind:
      | typeof COMMAND_RESIZE_WINDOW
      | typeof COMMAND_ZOOM_WINDOW
      | typeof COMMAND_TOGGLE_FULLSCREEN
      | typeof COMMAND_OPEN_URL
      | typeof COMMAND_FOCUS_NEXT
      | typeof COMMAND_FOCUS_PREV
      | typeof COMMAND_GET_WINDOW_SIZE
      | typeof COMMAND_CLIPBOARD_WRITE
      | typeof COMMAND_CLIPBOARD_READ
      | typeof COMMAND_CLIPBOARD_WRITE_IMAGE
      | typeof COMMAND_CLIPBOARD_READ_IMAGE
      | typeof COMMAND_OPEN_SURFACE
      | typeof COMMAND_FILE_DIALOG_OPEN
      | typeof COMMAND_FILE_DIALOG_SAVE
      | typeof COMMAND_SHOW_NOTIFICATION
      | typeof COMMAND_SET_MENUS
      | typeof COMMAND_SET_KEYBINDINGS
      | typeof COMMAND_SET_CLOSE_POLICY
      | typeof COMMAND_RESOLVE_CLOSE_REQUEST
      | typeof COMMAND_READ_TEXT_FILE
      | typeof COMMAND_WRITE_TEXT_FILE
      | typeof COMMAND_LOAD_FONT,
    payload:
      | readonly [number, number]
      | readonly [string, readonly [number, number]]
      | readonly [string, readonly [number, number], WindowOpenOptionsPayload]
      | readonly [string, string]
      | readonly [string, string, readonly (readonly [string, string])[]]
      | ClipboardImagePayload
      | KeybindingsPayload
      | string
      | MenuPayload
      | null,
  ): Promise<unknown> {
    if (this.transportTerminated) {
      return Promise.reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
    }
    if (this.unmounted) return Promise.reject(new SurfaceClosedError(this.surfaceId));
    let requestId: number;
    try {
      requestId = this.nextRequestId;
      this.nextRequestId = nextU32(this.nextRequestId, "command request id");
    } catch (error) {
      return Promise.reject(error);
    }
    const command: Command = [
      PROTOCOL_VERSION,
      COMMAND_KIND,
      this.surfaceId,
      this.epoch,
      this.revision,
      requestId,
      1,
      kind as Command[7],
      payload,
    ];
    return this.submitCommandFrame(command);
  }
  private submitSurfaceCommand(
    kind:
      | typeof COMMAND_RESIZE_WINDOW
      | typeof COMMAND_ZOOM_WINDOW
      | typeof COMMAND_TOGGLE_FULLSCREEN
      | typeof COMMAND_OPEN_URL
      | typeof COMMAND_FOCUS_NEXT
      | typeof COMMAND_FOCUS_PREV,
    payload: readonly [number, number] | string | null,
  ): Promise<void> {
    return this.submitSurfaceCommandValue(kind, payload).then(() => undefined);
  }

  resize(width: number, height: number): Promise<void> {
    if (
      !Number.isInteger(width) ||
      !Number.isInteger(height) ||
      width < 1 ||
      width > 16_384 ||
      height < 1 ||
      height > 16_384
    )
      return Promise.reject(new RangeError("window size must be integer pixels in the range 1..16384"));
    return this.submitSurfaceCommand(COMMAND_RESIZE_WINDOW, [width, height]);
  }
  setClosePolicy(policy: "allow" | "require-confirmation"): Promise<void> {
    if (policy !== "allow" && policy !== "require-confirmation") {
      return Promise.reject(new TypeError("close policy must be allow or require-confirmation"));
    }
    return this.submitSurfaceCommandValue(COMMAND_SET_CLOSE_POLICY, policy).then(() => undefined);
  }

  resolveCloseRequest(requestId: number, allow: boolean): Promise<void> {
    if (!Number.isInteger(requestId) || requestId < 0 || requestId > 0xffff_ffff) {
      return Promise.reject(new RangeError("close request id must be a u32"));
    }
    if (typeof allow !== "boolean") {
      return Promise.reject(new TypeError("close request allow must be a boolean"));
    }
    return this.submitSurfaceCommandValue(COMMAND_RESOLVE_CLOSE_REQUEST, [requestId, allow ? 1 : 0]).then(
      () => undefined,
    );
  }

  zoom(): Promise<void> {
    return this.submitSurfaceCommand(COMMAND_ZOOM_WINDOW, null);
  }

  toggleFullscreen(): Promise<void> {
    return this.submitSurfaceCommand(COMMAND_TOGGLE_FULLSCREEN, null);
  }

  openUrl(url: string): Promise<void> {
    if (
      typeof url !== "string" ||
      utf8ByteLength(url) > 2048 ||
      /\s/.test(url) ||
      (!url.startsWith("http://") && !url.startsWith("https://")) ||
      url.slice(url.indexOf("://") + 3).length === 0
    )
      return Promise.reject(new TypeError("url must be a non-empty http or https URL of at most 2048 UTF-8 bytes"));
    return this.submitSurfaceCommand(COMMAND_OPEN_URL, url);
  }
  openSurface(options: SurfaceOpenOptions = {}): Promise<number> {
    const title = options.title ?? "";
    const width = options.width ?? 0;
    const height = options.height ?? 0;
    if (typeof title !== "string" || [...title].length > 256)
      return Promise.reject(new TypeError("surface title must be at most 256 characters"));
    if (
      !Number.isInteger(width) ||
      !Number.isInteger(height) ||
      width < 0 ||
      width > 16_384 ||
      height < 0 ||
      height > 16_384 ||
      (width === 0) !== (height === 0)
    )
      return Promise.reject(new RangeError("surface size must be 0x0 or integer pixels in the range 1..16384"));
    if (options.kind !== undefined && !["normal", "floating", "dialog"].includes(options.kind))
      return Promise.reject(new TypeError("surface kind must be normal, floating, or dialog"));
    if (options.resizable !== undefined && typeof options.resizable !== "boolean")
      return Promise.reject(new TypeError("surface resizable must be a boolean"));
    let minWidth: number | null = null;
    let minHeight: number | null = null;
    if (options.minSize !== undefined) {
      if (
        !Array.isArray(options.minSize) ||
        options.minSize.length !== 2 ||
        !Number.isInteger(options.minSize[0]) ||
        !Number.isInteger(options.minSize[1]) ||
        options.minSize[0] <= 0 ||
        options.minSize[1] <= 0 ||
        options.minSize[0] > 16_384 ||
        options.minSize[1] > 16_384
      )
        return Promise.reject(new RangeError("surface minSize must be positive integer pixels in the range 1..16384"));
      [minWidth, minHeight] = options.minSize;
    }
    const hasOptions = options.kind !== undefined || options.resizable !== undefined || options.minSize !== undefined;
    const kind = options.kind === undefined ? null : ({ normal: 0, floating: 1, dialog: 2 } as const)[options.kind];
    const windowOptions: WindowOpenOptionsPayload | undefined = hasOptions
      ? [kind, options.resizable ?? null, minWidth, minHeight]
      : undefined;
    const payload = windowOptions
      ? ([title, [width, height], windowOptions] as const)
      : ([title, [width, height]] as const);
    return this.submitSurfaceCommandValue(COMMAND_OPEN_SURFACE, payload).then((value) => {
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 1 ||
        typeof value[1] !== "number" ||
        !Number.isInteger(value[1]) ||
        value[1] < 1 ||
        value[1] > 0xffff_ffff
      )
        throw new Error("native openSurface returned an invalid surface id");
      return value[1];
    });
  }
  pickFiles(
    options: { readonly title?: string; readonly directories?: boolean; readonly multiple?: boolean } = {},
  ): Promise<string[] | null> {
    const title = options.title ?? "";
    const directories = options.directories ?? false;
    const multiple = options.multiple ?? false;
    if (typeof title !== "string" || [...title].length > 256)
      return Promise.reject(new TypeError("file dialog title must be at most 256 characters"));
    if (typeof directories !== "boolean" || typeof multiple !== "boolean")
      return Promise.reject(new TypeError("file dialog options must be boolean"));
    const payload: readonly [string, readonly [number, number]] = [title, [directories ? 1 : 0, multiple ? 1 : 0]];
    return this.submitSurfaceCommandValue(COMMAND_FILE_DIALOG_OPEN, payload).then((value) => {
      if (value === undefined || value === null) return null;
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 5 ||
        !Array.isArray(value[1]) ||
        value[1].length === 0 ||
        !value[1].every((path): path is string => typeof path === "string" && path.length > 0)
      )
        throw new Error("native pickFiles returned an invalid value");
      return [...value[1]];
    });
  }

  pickSavePath(options: { readonly defaultName?: string } = {}): Promise<string | null> {
    const defaultName = options.defaultName ?? "";
    if (typeof defaultName !== "string" || [...defaultName].length > 256)
      return Promise.reject(new TypeError("save dialog defaultName must be at most 256 characters"));
    return this.submitSurfaceCommandValue(COMMAND_FILE_DIALOG_SAVE, defaultName).then((value) => {
      if (value === undefined || value === null) return null;
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 4 ||
        typeof value[1] !== "string" ||
        value[1].length === 0
      )
        throw new Error("native pickSavePath returned an invalid value");
      return value[1];
    });
  }

  readTextFile(path: string): Promise<string> {
    if (
      typeof path !== "string" ||
      path.length === 0 ||
      utf8ByteLength(path) > 1024 ||
      /[\u0000-\u001f\u007f]/.test(path) ||
      !path.startsWith("/")
    )
      return Promise.reject(new TypeError("file path must be a non-empty absolute path of at most 1024 UTF-8 bytes"));
    return this.submitSurfaceCommandValue(COMMAND_READ_TEXT_FILE, path).then((value) => {
      if (!Array.isArray(value) || value.length !== 2 || value[0] !== 6 || typeof value[1] !== "string")
        throw new Error("native readTextFile returned an invalid value");
      return value[1];
    });
  }
  loadFont(path: string): Promise<string> {
    if (
      typeof path !== "string" ||
      path.length === 0 ||
      utf8ByteLength(path) > 1024 ||
      /[\u0000-\u001f\u007f]/.test(path) ||
      !path.startsWith("/")
    )
      return Promise.reject(new TypeError("font path must be a non-empty absolute path of at most 1024 UTF-8 bytes"));
    return this.submitSurfaceCommandValue(COMMAND_LOAD_FONT, path).then((value) => {
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 4 ||
        typeof value[1] !== "string" ||
        value[1].length === 0
      )
        throw new Error("native loadFont returned an invalid family name");
      return value[1];
    });
  }
  writeTextFile(path: string, content: string): Promise<number> {
    if (
      typeof path !== "string" ||
      path.length === 0 ||
      utf8ByteLength(path) > 1024 ||
      /[\u0000-\u001f\u007f]/.test(path) ||
      !path.startsWith("/")
    )
      return Promise.reject(new TypeError("file path must be a non-empty absolute path of at most 1024 UTF-8 bytes"));
    if (typeof content !== "string" || utf8ByteLength(content) > MAX_FILE_WRITE_BYTES)
      return Promise.reject(new RangeError("file content exceeds the supported size"));
    return this.submitSurfaceCommandValue(COMMAND_WRITE_TEXT_FILE, [path, content]).then((value) => {
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 1 ||
        typeof value[1] !== "number" ||
        !Number.isFinite(value[1])
      )
        throw new Error("native writeTextFile returned an invalid byte count");
      return value[1];
    });
  }

  showNotification(options: {
    readonly title: string;
    readonly body: string;
    readonly actions?: readonly { readonly id: string; readonly label: string }[];
  }): Promise<void> {
    const { title, body, actions } = options;
    if (typeof title !== "string" || utf8ByteLength(title) > 256)
      return Promise.reject(new TypeError("notification title must be at most 256 UTF-8 bytes"));
    if (typeof body !== "string" || utf8ByteLength(body) > 1024)
      return Promise.reject(new TypeError("notification body must be at most 1024 UTF-8 bytes"));
    if (
      actions !== undefined &&
      (!Array.isArray(actions) ||
        actions.length > 3 ||
        actions.some(
          (action) =>
            typeof action.id !== "string" ||
            action.id.length === 0 ||
            utf8ByteLength(action.id) > 64 ||
            typeof action.label !== "string" ||
            action.label.length === 0 ||
            utf8ByteLength(action.label) > 256,
        ))
    )
      return Promise.reject(new TypeError("notification actions must contain at most three bounded id/label pairs"));
    const payload: readonly [string, string] | readonly [string, string, readonly (readonly [string, string])[]] =
      actions === undefined
        ? [title, body]
        : [title, body, actions.map((action) => [action.id, action.label] as const)];
    return this.submitSurfaceCommandValue(COMMAND_SHOW_NOTIFICATION, payload).then(() => undefined);
  }
  setMenus(menus: readonly MenuDefinition[]): Promise<void> {
    try {
      const encodeItem = (item: MenuItem): MenuItemPayload => {
        if (item.type === "separator") return [0];
        if (item.type === "action") {
          if (typeof item.name !== "string" || item.name.length === 0 || [...item.name].length > 256)
            throw new TypeError("menu action name must be 1..256 Unicode scalar values");
          if (
            (item.disabled !== undefined && typeof item.disabled !== "boolean") ||
            (item.checked !== undefined && typeof item.checked !== "boolean")
          )
            throw new TypeError("menu action disabled/checked states must be boolean");
          const disabled = item.disabled ?? false;
          const checked = item.checked ?? false;
          return disabled || checked ? [1, item.name, [disabled, checked]] : [1, item.name];
        }
        if (
          typeof item.title !== "string" ||
          item.title.length === 0 ||
          [...item.title].length > 256 ||
          !Array.isArray(item.items)
        )
          throw new TypeError("submenu title must be 1..256 Unicode scalar values");
        return [2, [item.title, item.items.map(encodeItem)]];
      };
      const payload: MenuPayload = menus.map((menu) => {
        if (
          typeof menu.title !== "string" ||
          menu.title.length === 0 ||
          [...menu.title].length > 256 ||
          !Array.isArray(menu.items)
        )
          throw new TypeError("menu title must be 1..256 Unicode scalar values");
        return [menu.title, menu.items.map(encodeItem)];
      });
      return this.submitSurfaceCommandValue(COMMAND_SET_MENUS, payload).then(() => undefined);
    } catch (error) {
      return Promise.reject(error);
    }
  }
  setKeybindings(bindings: readonly Keybinding[]): Promise<void> {
    try {
      if (!Array.isArray(bindings) || bindings.length > 64) {
        throw new RangeError("setKeybindings accepts at most 64 bindings");
      }
      const payload: KeybindingsPayload = bindings.map((binding, index) => {
        if (
          binding === null ||
          typeof binding !== "object" ||
          typeof binding.keystrokes !== "string" ||
          binding.keystrokes.trim().length === 0 ||
          utf8ByteLength(binding.keystrokes) > 64 ||
          /[\u0000-\u001f\u007f]/.test(binding.keystrokes)
        ) {
          throw new TypeError(`keybinding[${index}].keystrokes must be a non-empty string of at most 64 UTF-8 bytes`);
        }
        if (
          typeof binding.actionName !== "string" ||
          binding.actionName.length === 0 ||
          [...binding.actionName].length > 64 ||
          /[\u0000-\u001f\u007f]/.test(binding.actionName)
        ) {
          throw new TypeError(`keybinding[${index}].actionName must be 1..64 Unicode characters`);
        }
        return [binding.keystrokes, binding.actionName];
      });
      return this.submitSurfaceCommandValue(COMMAND_SET_KEYBINDINGS, payload).then(() => undefined);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  focusNext(): Promise<void> {
    return this.submitSurfaceCommand(COMMAND_FOCUS_NEXT, null);
  }

  focusPrev(): Promise<void> {
    return this.submitSurfaceCommand(COMMAND_FOCUS_PREV, null);
  }
  getWindowSize(): Promise<[number, number]> {
    return this.submitSurfaceCommandValue(COMMAND_GET_WINDOW_SIZE, null).then((value) => {
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 2 ||
        !Array.isArray(value[1]) ||
        value[1].length !== 2 ||
        typeof value[1][0] !== "number" ||
        !Number.isFinite(value[1][0]) ||
        value[1][0] < 0 ||
        typeof value[1][1] !== "number" ||
        !Number.isFinite(value[1][1]) ||
        value[1][1] < 0
      )
        throw new Error("native getWindowSize returned an invalid value");
      return [value[1][0], value[1][1]];
    });
  }

  setClipboardText(text: string): Promise<void> {
    if (typeof text !== "string") return Promise.reject(new TypeError("clipboard text must be a string"));
    if (utf8ByteLength(text) > MAX_CLIPBOARD_TEXT_BYTES)
      return Promise.reject(new RangeError("clipboard text exceeds the supported size"));
    return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_WRITE, text).then(() => undefined);
  }

  getClipboardText(): Promise<string> {
    return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_READ, null).then((value) => {
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 4 ||
        typeof value[1] !== "string" ||
        utf8ByteLength(value[1]) > MAX_CLIPBOARD_TEXT_BYTES
      )
        throw new Error("native getClipboardText returned an invalid value");
      return value[1];
    });
  }

  setClipboardImage(image: ClipboardImage): Promise<void> {
    try {
      const normalized = normalizeClipboardImage(image);
      const payload: ClipboardImagePayload = [CLIPBOARD_IMAGE_FORMAT_CODES[normalized.format], normalized.bytes];
      return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_WRITE_IMAGE, payload).then(() => undefined);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  getClipboardImage(): Promise<ClipboardImage | null> {
    return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_READ_IMAGE, null).then((value) => {
      if (value === undefined || value === null) return null;
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        value[0] !== 7 ||
        !Array.isArray(value[1]) ||
        value[1].length !== 2 ||
        typeof value[1][0] !== "number" ||
        !Number.isInteger(value[1][0]) ||
        value[1][0] < CLIPBOARD_IMAGE_FORMAT_PNG ||
        value[1][0] > CLIPBOARD_IMAGE_FORMAT_SVG ||
        !(value[1][1] instanceof Uint8Array) ||
        value[1][1].byteLength === 0 ||
        value[1][1].byteLength > MAX_CLIPBOARD_IMAGE_BYTES
      ) {
        throw new Error("native getClipboardImage returned an invalid value");
      }
      const formatCode = value[1][0] as ClipboardImageFormatCode;
      return { format: CLIPBOARD_IMAGE_FORMAT_NAMES[formatCode], bytes: value[1][1] };
    });
  }

  throwIfUnhandledError(): void {
    if (this.unhandledError !== undefined) {
      const error = this.unhandledError;
      this.unhandledError = undefined;
      throw error;
    }
  }

  markMoved(node: HostNodeInternal): void {
    this.nodes.markMoved(node, this.bootstrapped);
  }

  markUpdated(node: HostNodeInternal, mask: number): void {
    this.nodes.markUpdated(node, mask, this.bootstrapped);
  }

  markDeleted(node: HostNodeInternal): void {
    this.nodes.markDeleted(node, this.bootstrapped);
  }

  private clearMutations(): void {
    this.nodes.clearMutations();
  }
  private snapshotNodes(): SnapshotNode[] {
    return this.nodes.snapshotNodes();
  }

  private nodeDepth(node: HostNodeInternal): number {
    return this.nodes.nodeDepth(node);
  }

  private nativeParentId(node: HostNodeInternal): number {
    return this.nodes.nativeParentId(node);
  }

  commit(): void {
    if (this.invalid) {
      this.invalid = false;
      this.clearMutations();
      return;
    }
    const baseRevision = this.revision;
    const revision = nextU32(baseRevision, "revision");
    if (!this.bootstrapped) {
      if (
        !this.submitFrame(
          encodeFrame([PROTOCOL_VERSION, 1, this.surfaceId, this.epoch, baseRevision, revision, this.snapshotNodes()]),
        )
      ) {
        this.clearMutations();
        return;
      }
    } else {
      const operations: PatchOperation[] = [];
      const created = [...this.createdIds]
        .map((id) => this.nodesById.get(id))
        .filter((node): node is HostNodeInternal => node !== undefined)
        .sort((a, b) => this.nodeDepth(a) - this.nodeDepth(b) || a.id - b.id);
      for (const node of created) {
        const base = [
          1,
          node.id,
          this.nativeParentId(node),
          node.index,
          KIND_CODES[node.kind],
          encodeStyle(node.style),
          node.kind === "RawText" ? node.text : null,
          node.listenerId,
          hostPropertiesWire(node.hostProperties),
          accessibilityWire(node.accessibility),
          node.focusable,
        ] as const;
        if (node.tooltip !== null) {
          operations.push(
            node.acceptsPointerMove
              ? [...base, node.selectable, node.tooltip, true]
              : [...base, node.selectable, node.tooltip],
          );
        } else if (node.selectable) {
          operations.push(node.acceptsPointerMove ? [...base, true, null, true] : [...base, true]);
        } else if (node.acceptsPointerMove) {
          operations.push([...base, false, null, true]);
        } else {
          operations.push(base);
        }
      }
      const moved = [...this.movedIds]
        .map((id) => this.nodesById.get(id))
        .filter((node): node is HostNodeInternal => node !== undefined && !this.createdIds.has(node.id))
        .sort((a, b) => this.nodeDepth(a) - this.nodeDepth(b) || a.id - b.id);
      for (const node of moved) {
        operations.push([3, node.id, this.nativeParentId(node), node.index]);
      }
      for (const [id, mask] of [...this.updatedMasks.entries()].sort(([a], [b]) => a - b)) {
        const node = this.nodesById.get(id);
        if (node === undefined) continue;
        const base = [
          2,
          id,
          mask,
          mask & UPDATE_STYLE ? encodeStyle(node.style) : null,
          mask & UPDATE_TEXT && node.kind === "RawText" ? node.text : null,
          mask & UPDATE_LISTENER ? node.listenerId : 0,
          mask & UPDATE_PROPERTIES ? hostPropertiesWire(node.hostProperties) : null,
          mask & UPDATE_ACCESSIBILITY ? accessibilityWire(node.accessibility) : null,
          node.focusable,
        ] as const;
        if (mask & UPDATE_TOOLTIP || mask & UPDATE_POINTER_MOVE) {
          if (node.tooltip !== null) {
            operations.push(
              node.acceptsPointerMove
                ? [...base, node.selectable, node.tooltip, true]
                : [...base, node.selectable, node.tooltip],
            );
          } else if (mask & UPDATE_TOOLTIP || node.selectable) {
            operations.push(
              node.acceptsPointerMove ? [...base, node.selectable, null, true] : [...base, node.selectable, null],
            );
          } else if (node.acceptsPointerMove) {
            operations.push([...base, false, null, true]);
          } else {
            operations.push(base);
          }
        } else if (mask & UPDATE_SELECTABLE && node.selectable) operations.push([...base, true]);
        else operations.push(base);
      }
      for (const id of [...this.deletedRoots].sort((a, b) => a - b)) {
        if (!this.createdIds.has(id) && !this.nodesById.has(id)) operations.push([4, id]);
      }
      const patch: Patch = [PROTOCOL_VERSION, 3, this.surfaceId, this.epoch, baseRevision, revision, operations];
      if (!this.submitFrame(encodeFrame(patch))) {
        this.clearMutations();
        return;
      }
    }
    this.revision = revision;
    this.bootstrapped = true;
    this.clearMutations();
  }

  receive(chunk: Uint8Array | ArrayBuffer): void {
    if (this.transportTerminated || this.unmounted) return;
    let payloads: Uint8Array[];
    try {
      payloads = this.decoder.push(chunk);
    } catch (error) {
      this.failProtocol(error);
      return;
    }
    if (payloads.length === 0) return;
    const events: PressEventFrame[] = [];
    for (const payload of payloads) {
      let event: PressEventFrame | null;
      try {
        event = decodeEvent(payload);
      } catch (error) {
        this.failProtocol(error);
        return;
      }
      if (event === null) {
        this.failProtocol("received malformed event frame");
        return;
      }
      events.push(event);
    }
    this.scheduleDispatch(() => {
      for (const event of events) dispatchEvent(this, event);
    });
  }
  onSurfaceClosed(): void {
    if (this.unmounted) return;
    this.dispose(new SurfaceClosedError(this.surfaceId));
    this.surfaceClosedHandler?.();
  }

  dispose(error: Error = new SurfaceClosedError(this.surfaceId)): void {
    if (this.unmounted) return;
    this.unmounted = true;
    this.detachTransportListeners();
    for (const pending of this.pendingCommands.values()) pending.reject(error);
    this.pendingCommands.clear();
    for (const node of this.children) this.detachSubtree(node);
    this.children.length = 0;
    this.listeners.clear();
  }
  acceptEvent(event: PressEventFrame): boolean {
    if (
      this.unmounted ||
      this.transportTerminated ||
      event[2] !== this.surfaceId ||
      event[3] !== this.epoch ||
      event[4] > this.revision ||
      (this.hasEventSequence && event[5] <= this.lastEventSequence)
    )
      return false;
    this.lastEventSequence = event[5];
    this.hasEventSequence = true;
    return true;
  }
  findListener(listenerId: number): HostNodeInternal | undefined {
    return this.listeners.get(listenerId);
  }
  findNode(nodeId: number): HostNodeInternal | undefined {
    return this.nodesById.get(nodeId);
  }
  findInputCallbacks(listenerId: number): TextInputCallbacks | undefined {
    return this.inputListeners.get(listenerId);
  }
  resolveCommandResult(requestId: number, success: boolean, errorPayload: unknown, value?: unknown): void {
    const pending = this.pendingCommands.get(requestId);
    if (pending === undefined) return;
    this.pendingCommands.delete(requestId);
    if (success) pending.resolve(value);
    else pending.reject(new Error(String(errorPayload ?? "native command failed")));
  }
}
