import {
  COMMAND_INVOKE_NATIVE,
  COMMAND_CANCEL_NATIVE,
  MAX_NATIVE_CALL_BYTES,
  COMMAND_ACTIVATE_WINDOW,
  COMMAND_BLUR,
  COMMAND_FOCUS,
  COMMAND_FOCUS_NEXT,
  COMMAND_FOCUS_PREV,
  COMMAND_GET_FOCUS,
  COMMAND_GET_WINDOW_BOUNDS,
  COMMAND_GET_WINDOW_SIZE,
  COMMAND_GET_WINDOW_STATE,
  COMMAND_CLIPBOARD_READ,
  COMMAND_CLIPBOARD_READ_IMAGE,
  COMMAND_CLIPBOARD_WRITE,
  COMMAND_CLIPBOARD_WRITE_IMAGE,
  COMMAND_FILE_DIALOG_OPEN,
  COMMAND_FILE_DIALOG_SAVE,
  COMMAND_LOAD_FONT,
  COMMAND_MINIMIZE_WINDOW,
  COMMAND_OPEN_URL,
  COMMAND_OPEN_SURFACE,
  COMMAND_READ_TEXT_FILE,
  COMMAND_RESIZE_WINDOW,
  COMMAND_RESOLVE_CLOSE_REQUEST,
  COMMAND_SET_CLOSE_POLICY,
  COMMAND_SET_KEYBINDINGS,
  COMMAND_SET_MENUS,
  COMMAND_SET_SELECTION,
  COMMAND_SHOW_NOTIFICATION,
  COMMAND_SCROLL_TO_END,
  COMMAND_GET_SCROLL_OFFSET,
  COMMAND_SCROLL_TO_OFFSET,
  COMMAND_SCROLL_TO_INDEX,
  COMMAND_TOGGLE_FULLSCREEN,
  COMMAND_WRITE_TEXT_FILE,
  COMMAND_ZOOM_WINDOW,
  COMMAND_SET_TITLE,
  MAX_CLIPBOARD_IMAGE_BYTES,
  MAX_CLIPBOARD_TEXT_BYTES,
  MAX_FILE_WRITE_BYTES,
  encodeFrame,
  type ClipboardImage,
  type Command,
  type CommandKind,
  type CommandPayload,
  type CommandResult,
  type CommandValue,
  type Event,
  type WindowOpenOptions,
  utf8ByteLength,
} from "../protocol";
import { TransportTerminatedError, type TransportTerminationListener } from "../transport";
import type { Appearance } from "../hooks";
import { dispatchEvent, type DispatchContext } from "./dispatch";
import { CommandClient } from "./command-client";
import type { NativeCallOptions } from "../native-call";
import { HostTree } from "./host-tree";
import { afterRootCommit } from "./host-config";
import { assertU32Option, nextU32 } from "./props";
import type {
  HostNodeInternal,
  Keybinding,
  ListenerBinding,
  MenuDefinition,
  MenuItem,
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

function normalizeClipboardImage(image: ClipboardImage): ClipboardImage {
  if (image === null || typeof image !== "object") {
    throw new TypeError("clipboard image must be an object");
  }
  if (image.format !== "png" && image.format !== "jpeg" && image.format !== "gif" && image.format !== "svg") {
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
const TEXT_INPUT_COMMANDS: readonly CommandKind[] = [
  COMMAND_FOCUS,
  COMMAND_BLUR,
  COMMAND_SET_SELECTION,
  COMMAND_GET_FOCUS,
];
const VIEW_COMMANDS: readonly CommandKind[] = [COMMAND_FOCUS, COMMAND_BLUR, COMMAND_GET_FOCUS];
const VIRTUAL_LIST_COMMANDS: readonly CommandKind[] = [
  COMMAND_SCROLL_TO_INDEX,
  COMMAND_SCROLL_TO_END,
  COMMAND_GET_SCROLL_OFFSET,
  COMMAND_SCROLL_TO_OFFSET,
];
export type EventBatchListener = (events: readonly Event[]) => void;

export interface RootContainerOptions {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly scheduleDispatch: (dispatch: () => void) => void;
  readonly submitFrame: (frame: Uint8Array) => boolean;
  readonly onTransportTermination?: TransportTerminationListener;
  readonly onWindowResize?: WindowResizeHandler;
  readonly onWindowActivation?: WindowActivationHandler;
  readonly onSurfaceClosed?: () => void;
  readonly onCloseRequested?: (requestId: number) => void;
  readonly onAction?: (action: string) => void;
  readonly onAppearance?: (appearance: Appearance) => void;
  readonly onNotificationResponse?: (response: { readonly tag: string; readonly actionId: string | null }) => void;
}

export class RootContainer implements DispatchContext {
  readonly tree: HostTree;
  readonly surfaceId: number;
  readonly commandClient: CommandClient;
  readonly epoch: number;
  revision = 0;
  unmounted = false;
  unhandledError: Error | undefined;
  private lastEventSequence = 0;
  private hasEventSequence = false;
  private unsubscribe: (() => void) | undefined;
  private started = false;
  private readonly scheduleDispatch: (dispatch: () => void) => void;
  private readonly submitFrameImpl: (frame: Uint8Array) => boolean;
  private transportTerminated = false;
  private terminationError: TransportTerminatedError | undefined;
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
  constructor(options: RootContainerOptions) {
    this.surfaceId = assertU32Option("surfaceId", options.surfaceId);
    this.epoch = assertU32Option("epoch", options.epoch);
    this.scheduleDispatch = options.scheduleDispatch;
    this.submitFrameImpl = options.submitFrame;
    this.onTransportTermination = options.onTransportTermination;
    this.onWindowResize = options.onWindowResize;
    this.onWindowActivation = options.onWindowActivation;
    this.surfaceClosedHandler = options.onSurfaceClosed;
    this.onCloseRequested = options.onCloseRequested;
    this.onAction = options.onAction;
    this.onAppearance = options.onAppearance;
    this.onNotificationResponse = options.onNotificationResponse;
    this.commandClient = new CommandClient(this);
    this.tree = new HostTree({
      invokeNative: (moduleId, moduleDigest, functionId, args, options) =>
        this.invokeNative(moduleId, moduleDigest, functionId, args, options),
      surfaceId: this.surfaceId,
      epoch: this.epoch,
      getRevision: () => this.revision,
      submitCommit: (commit) => {
        if (!this.submitFrame(encodeFrame(commit))) return false;
        this.revision = commit.revision;
        return true;
      },
      onCommitError: (error) => this.recordUnhandledError(error),
      submitCommand: (node, kind, payload) => this.submitCommand(node, kind, payload),
      submitCommandValue: (node, kind, payload, options) => this.submitCommandValue(node, kind, payload, options),
    });
  }

  start(unsubscribeEvents: () => void): void {
    if (this.started || this.unmounted) {
      unsubscribeEvents();
      return;
    }
    this.started = true;
    this.unsubscribe = unsubscribeEvents;
  }
  terminate(error: TransportTerminatedError): void {
    if (this.unmounted || this.transportTerminated) return;
    this.transportTerminated = true;
    this.terminationError = error;
    this.tree.invalid = true;
    this.detachEventSubscription();
    this.commandClient.rejectAll(error);
    this.onTransportTermination?.(error);
  }

  private detachEventSubscription(): void {
    const unsubscribe = this.unsubscribe;
    this.unsubscribe = undefined;
    unsubscribe?.();
  }

  submitFrame(frame: Uint8Array): boolean {
    if (this.transportTerminated) return false;
    try {
      return this.submitFrameImpl(frame);
    } catch (error) {
      if (error instanceof TransportTerminatedError) {
        this.terminate(error);
        return false;
      }
      throw error;
    }
  }
  isTerminated(): boolean {
    return this.transportTerminated;
  }
  getTerminationError(): TransportTerminatedError | undefined {
    return this.terminationError;
  }

  submitCommand(node: HostNodeInternal, kind: CommandKind, payload: CommandPayload): Promise<void> {
    return this.submitCommandValue(node, kind, payload).then(() => undefined);
  }
  submitCommandValue(
    node: HostNodeInternal,
    kind: CommandKind,
    payload: CommandPayload,
    options?: NativeCallOptions,
  ): Promise<CommandValue | null> {
    if (this.transportTerminated) {
      return Promise.reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
    }
    if (this.unmounted) return Promise.reject(new SurfaceClosedError(this.surfaceId));
    if (!node.attached)
      return Promise.reject(
        new Error(`host node ${node.id} is unavailable; ensure the node is mounted before invoking commands`),
      );
    const isInput = node.kind === "TextInput";
    const isList = node.kind === "VirtualList";
    const isView = node.kind === "View";
    const isExtension = node.kind === "Extension";
    if (!isInput && !isList && !isView && !isExtension)
      return Promise.reject(new Error("host node does not support commands"));
    if (isExtension && (kind !== COMMAND_INVOKE_NATIVE || payload?.type !== "invoke-native"))
      return Promise.reject(new Error("unknown Extension command"));
    if (isView && !node.focusable)
      return Promise.reject(
        new Error(`View node ${node.id} is not focusable; set focusable={true} before calling focus`),
      );
    if (isInput && !TEXT_INPUT_COMMANDS.includes(kind)) return Promise.reject(new Error("unknown TextInput command"));
    if (isView && !VIEW_COMMANDS.includes(kind)) return Promise.reject(new Error("unknown View command"));
    if (isList && !VIRTUAL_LIST_COMMANDS.includes(kind))
      return Promise.reject(new Error("unknown VirtualList command"));
    if (
      kind === COMMAND_SCROLL_TO_OFFSET &&
      (payload === null || payload.type !== "number" || !Number.isFinite(payload.value) || payload.value < 0)
    )
      return Promise.reject(new TypeError("scroll offset must be finite and non-negative"));
    if (
      kind === COMMAND_SET_SELECTION &&
      (payload === null ||
        payload.type !== "selection" ||
        !Number.isInteger(payload.start) ||
        !Number.isInteger(payload.end) ||
        payload.start < 0 ||
        payload.end < payload.start ||
        payload.end > 0xffff_ffff)
    )
      return Promise.reject(
        new Error(`invalid UTF-16 selection for TextInput node ${node.id}; provide offsets within the text range`),
      );
    if (
      kind === COMMAND_SCROLL_TO_INDEX &&
      (payload === null ||
        payload.type !== "scroll-index" ||
        !Number.isInteger(payload.index) ||
        payload.index < 0 ||
        payload.index > 0xffff_ffff ||
        (node.hostProperties?.type === "virtual-list" && payload.index >= node.hostProperties.value.itemCount))
    )
      return Promise.reject(
        new Error(`VirtualList index is out of range for node ${node.id}; use an index below itemCount`),
      );
    let requestId: number;
    try {
      requestId = this.commandClient.allocateRequestId(nextU32);
    } catch (error) {
      return Promise.reject(error);
    }
    const command: Command = {
      type: "command",
      surfaceId: this.surfaceId,
      epoch: this.epoch,
      afterRevision: this.revision,
      requestId,
      nodeId: node.id,
      command: kind,
      payload,
    };
    return this.commandClient.submit(command, options);
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
      requestId = this.commandClient.allocateRequestId(nextU32);
    } catch (error) {
      return Promise.reject(error);
    }
    const command: Command = {
      type: "command",
      surfaceId: this.surfaceId,
      epoch: this.epoch,
      afterRevision: this.revision,
      requestId,
      nodeId: 1,
      command: COMMAND_SET_TITLE,
      payload: { type: "text", value: title },
    };
    return this.commandClient.submit(command).then(() => undefined);
  }
  private submitSurfaceCommand(kind: CommandKind, payload: CommandPayload): Promise<void> {
    return this.submitSurfaceCommandValue(kind, payload).then(() => undefined);
  }
  private submitSurfaceCommandValue(
    kind: CommandKind,
    payload: CommandPayload,
    options?: NativeCallOptions,
  ): Promise<CommandValue | null> {
    if (this.transportTerminated) {
      return Promise.reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
    }
    if (this.unmounted) return Promise.reject(new SurfaceClosedError(this.surfaceId));
    let requestId: number;
    try {
      requestId = this.commandClient.allocateRequestId(nextU32);
    } catch (error) {
      return Promise.reject(error);
    }
    const command: Command = {
      type: "command",
      surfaceId: this.surfaceId,
      epoch: this.epoch,
      afterRevision: this.revision,
      requestId,
      nodeId: 1,
      command: kind,
      payload,
    };
    return this.commandClient.submit(command, options);
  }

  cancelNative(requestId: number): void {
    if (this.transportTerminated || this.unmounted) return;
    // Cancellation is ordered with commits, but needs no result observer.
    try {
      if (
        !this.submitFrame(
          encodeFrame({
            type: "command",
            surfaceId: this.surfaceId,
            epoch: this.epoch,
            afterRevision: this.revision,
            requestId: this.commandClient.allocateRequestId(nextU32),
            nodeId: 1,
            command: COMMAND_CANCEL_NATIVE,
            payload: { type: "cancel-native", requestId },
          }),
        )
      )
        throw this.terminationError ?? new Error("transport rejected cancellation");
    } catch (error) {
      this.terminate(new TransportTerminatedError("Native cancellation could not be delivered", error));
    }
  }

  invokeNative(
    moduleId: Uint8Array,
    moduleDigest: Uint8Array,
    functionId: number,
    args: Uint8Array,
    options?: NativeCallOptions,
  ): Promise<Uint8Array> {
    if (!(moduleId instanceof Uint8Array) || moduleId.byteLength !== 16)
      return Promise.reject(new TypeError("native module id must contain exactly 16 bytes"));
    if (!(moduleDigest instanceof Uint8Array) || moduleDigest.byteLength !== 32)
      return Promise.reject(new TypeError("native module digest must contain exactly 32 bytes"));
    if (typeof functionId !== "number" || !Number.isInteger(functionId) || functionId <= 0 || functionId > 0xffff_ffff)
      return Promise.reject(new TypeError("native function id must be a positive u32"));
    if (!(args instanceof Uint8Array)) return Promise.reject(new TypeError("native arguments must be a Uint8Array"));
    if (args.byteLength > MAX_NATIVE_CALL_BYTES)
      return Promise.reject(new RangeError("native arguments exceed the supported size"));
    return afterRootCommit(this.tree, () =>
      this.submitSurfaceCommandValue(
        COMMAND_INVOKE_NATIVE,
        {
          type: "invoke-native",
          moduleId,
          moduleDigest,
          functionId,
          args,
        },
        options,
      ),
    ).then((value) => {
      if (
        value === null ||
        value.type !== "bytes" ||
        !(value.value instanceof Uint8Array) ||
        value.value.byteLength > MAX_NATIVE_CALL_BYTES
      )
        throw new Error("native invokeNative returned an invalid byte result");
      return value.value;
    });
  }

  recordUnhandledError(error: unknown): void {
    this.tree.invalid = true;
    this.unhandledError = error instanceof Error ? error : new Error(String(error));
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
    return this.submitSurfaceCommand(COMMAND_RESIZE_WINDOW, { type: "window-size", width, height });
  }
  setClosePolicy(policy: "allow" | "require-confirmation"): Promise<void> {
    if (policy !== "allow" && policy !== "require-confirmation") {
      return Promise.reject(new TypeError("close policy must be allow or require-confirmation"));
    }
    return this.submitSurfaceCommand(COMMAND_SET_CLOSE_POLICY, { type: "text", value: policy });
  }

  resolveCloseRequest(requestId: number, allow: boolean): Promise<void> {
    if (!Number.isInteger(requestId) || requestId < 0 || requestId > 0xffff_ffff) {
      return Promise.reject(new RangeError("close request id must be a u32"));
    }
    if (typeof allow !== "boolean") {
      return Promise.reject(new TypeError("close request allow must be a boolean"));
    }
    return this.submitSurfaceCommand(COMMAND_RESOLVE_CLOSE_REQUEST, {
      type: "close-resolution",
      requestId,
      allow,
    });
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
    return this.submitSurfaceCommand(COMMAND_OPEN_URL, { type: "text", value: url });
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
    const kindCode = options.kind === undefined ? null : ({ normal: 0, floating: 1, dialog: 2 } as const)[options.kind];
    const windowOptions: WindowOpenOptions | undefined = hasOptions
      ? {
          kind: kindCode,
          resizable: options.resizable ?? null,
          minWidth,
          minHeight,
        }
      : undefined;
    const payload: CommandPayload = {
      type: "open-surface",
      title,
      width,
      height,
      ...(windowOptions === undefined ? {} : { options: windowOptions }),
    };
    return this.submitSurfaceCommandValue(COMMAND_OPEN_SURFACE, payload).then((value) => {
      if (
        value === null ||
        value.type !== "number" ||
        !Number.isInteger(value.value) ||
        value.value < 1 ||
        value.value > 0xffff_ffff
      )
        throw new Error("native openSurface returned an invalid surface id");
      return value.value;
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
    const payload: CommandPayload = { type: "file-dialog-open", title, directories, multiple };
    return this.submitSurfaceCommandValue(COMMAND_FILE_DIALOG_OPEN, payload).then((value) => {
      if (value === null) return null;
      if (value.type !== "paths" || value.paths.length === 0 || !value.paths.every((path) => path.length > 0))
        throw new Error("native pickFiles returned an invalid value");
      return [...value.paths];
    });
  }

  pickSavePath(options: { readonly defaultName?: string } = {}): Promise<string | null> {
    const defaultName = options.defaultName ?? "";
    if (typeof defaultName !== "string" || [...defaultName].length > 256)
      return Promise.reject(new TypeError("save dialog defaultName must be at most 256 characters"));
    return this.submitSurfaceCommandValue(COMMAND_FILE_DIALOG_SAVE, {
      type: "text",
      value: defaultName,
    }).then((value) => {
      if (value === null || value.type !== "text" || value.value.length === 0)
        throw new Error("native pickSavePath returned an invalid value");
      return value.value;
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
    return this.submitSurfaceCommandValue(COMMAND_READ_TEXT_FILE, { type: "text", value: path }).then((value) => {
      if (value === null || value.type !== "file-text")
        throw new Error("native readTextFile returned an invalid value");
      return value.value;
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
    return this.submitSurfaceCommandValue(COMMAND_LOAD_FONT, { type: "text", value: path }).then((value) => {
      if (value === null || value.type !== "text" || value.value.length === 0)
        throw new Error("native loadFont returned an invalid family name");
      return value.value;
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
    return this.submitSurfaceCommandValue(COMMAND_WRITE_TEXT_FILE, { type: "file-write", path, content }).then(
      (value) => {
        if (
          value === null ||
          value.type !== "number" ||
          !Number.isInteger(value.value) ||
          value.value < 0 ||
          value.value > 0xffff_ffff
        )
          throw new Error("native writeTextFile returned an invalid byte count");
        return value.value;
      },
    );
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
    const payload: CommandPayload = {
      type: "notification",
      title,
      body,
      ...(actions === undefined ? {} : { actions }),
    };
    return this.submitSurfaceCommand(COMMAND_SHOW_NOTIFICATION, payload);
  }
  setMenus(menus: readonly MenuDefinition[]): Promise<void> {
    try {
      const encodeItem = (item: MenuItem): import("../protocol").MenuItemDefinition => {
        if (item.type === "separator") return { type: "separator" };
        if (item.type === "action") {
          if (typeof item.name !== "string" || item.name.length === 0 || [...item.name].length > 256)
            throw new TypeError("menu action name must be 1..256 Unicode scalar values");
          if (
            (item.disabled !== undefined && typeof item.disabled !== "boolean") ||
            (item.checked !== undefined && typeof item.checked !== "boolean")
          )
            throw new TypeError("menu action disabled/checked states must be boolean");
          return {
            type: "action",
            name: item.name,
            ...(item.disabled === undefined ? {} : { disabled: item.disabled }),
            ...(item.checked === undefined ? {} : { checked: item.checked }),
          };
        }
        if (
          typeof item.title !== "string" ||
          item.title.length === 0 ||
          [...item.title].length > 256 ||
          !Array.isArray(item.items)
        )
          throw new TypeError("submenu title must be 1..256 Unicode scalar values");
        return { type: "submenu", title: item.title, items: item.items.map(encodeItem) };
      };
      const normalizedMenus: import("../protocol").MenuDefinition[] = menus.map((menu) => {
        if (
          typeof menu.title !== "string" ||
          menu.title.length === 0 ||
          [...menu.title].length > 256 ||
          !Array.isArray(menu.items)
        )
          throw new TypeError("menu title must be 1..256 Unicode scalar values");
        return { title: menu.title, items: menu.items.map(encodeItem) };
      });
      return this.submitSurfaceCommand(COMMAND_SET_MENUS, { type: "menus", menus: normalizedMenus });
    } catch (error) {
      return Promise.reject(error);
    }
  }
  setKeybindings(bindings: readonly Keybinding[]): Promise<void> {
    try {
      if (!Array.isArray(bindings) || bindings.length > 64) {
        throw new RangeError("setKeybindings accepts at most 64 bindings");
      }
      const normalizedBindings: import("../protocol").KeybindingDefinition[] = bindings.map((binding, index) => {
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
        return { keystrokes: binding.keystrokes, actionName: binding.actionName };
      });
      return this.submitSurfaceCommand(COMMAND_SET_KEYBINDINGS, {
        type: "keybindings",
        bindings: normalizedBindings,
      });
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
      if (value === null || value.type !== "pair" || value.width < 0 || value.height < 0)
        throw new Error("native getWindowSize returned an invalid value");
      return [value.width, value.height];
    });
  }

  minimizeWindow(): Promise<void> {
    return this.submitSurfaceCommand(COMMAND_MINIMIZE_WINDOW, null);
  }

  getWindowBounds(): Promise<{ x: number; y: number; width: number; height: number }> {
    return this.submitSurfaceCommandValue(COMMAND_GET_WINDOW_BOUNDS, null).then((value) => {
      if (value === null || value.type !== "bounds" || value.width < 0 || value.height < 0)
        throw new Error("native getWindowBounds returned an invalid value");
      return { x: value.x, y: value.y, width: value.width, height: value.height };
    });
  }

  getWindowState(): Promise<{ fullscreen: boolean; maximized: boolean }> {
    return this.submitSurfaceCommandValue(COMMAND_GET_WINDOW_STATE, null).then((value) => {
      if (value === null || value.type !== "window-state")
        throw new Error("native getWindowState returned an invalid value");
      return { fullscreen: value.fullscreen, maximized: value.maximized };
    });
  }

  activateWindow(): Promise<void> {
    return this.submitSurfaceCommand(COMMAND_ACTIVATE_WINDOW, null);
  }

  setClipboardText(text: string): Promise<void> {
    if (typeof text !== "string") return Promise.reject(new TypeError("clipboard text must be a string"));
    if (utf8ByteLength(text) > MAX_CLIPBOARD_TEXT_BYTES)
      return Promise.reject(new RangeError("clipboard text exceeds the supported size"));
    return this.submitSurfaceCommand(COMMAND_CLIPBOARD_WRITE, { type: "text", value: text });
  }

  getClipboardText(): Promise<string> {
    return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_READ, null).then((value) => {
      if (value === null || value.type !== "text" || utf8ByteLength(value.value) > MAX_CLIPBOARD_TEXT_BYTES)
        throw new Error("native getClipboardText returned an invalid value");
      return value.value;
    });
  }

  setClipboardImage(image: ClipboardImage): Promise<void> {
    try {
      const normalized = normalizeClipboardImage(image);
      return this.submitSurfaceCommand(COMMAND_CLIPBOARD_WRITE_IMAGE, {
        type: "clipboard-image",
        image: normalized,
      });
    } catch (error) {
      return Promise.reject(error);
    }
  }

  getClipboardImage(): Promise<ClipboardImage | null> {
    return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_READ_IMAGE, null).then((value) => {
      if (value === null) return null;
      if (value.type !== "image") throw new Error("native getClipboardImage returned an invalid value");
      return value.image;
    });
  }

  throwIfUnhandledError(): void {
    if (this.unhandledError !== undefined) {
      const error = this.unhandledError;
      this.unhandledError = undefined;
      throw error;
    }
  }

  receiveEvents(events: readonly Event[]): void {
    if (this.transportTerminated || this.unmounted || events.length === 0) return;
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
    this.detachEventSubscription();
    this.commandClient.rejectAll(error);
    this.tree.dispose();
  }

  acceptEvent(event: Event): boolean {
    if (
      this.unmounted ||
      this.transportTerminated ||
      event.surfaceId !== this.surfaceId ||
      event.epoch !== this.epoch ||
      event.revision > this.revision ||
      (this.hasEventSequence && event.sequence <= this.lastEventSequence)
    )
      return false;
    this.lastEventSequence = event.sequence;
    this.hasEventSequence = true;
    return true;
  }

  findListener(listenerId: number, revision: number): ListenerBinding | undefined {
    return this.tree.findListener(listenerId, revision);
  }
  resolveCommandResult(result: CommandResult): void {
    this.commandClient.resolve(result);
  }
}
