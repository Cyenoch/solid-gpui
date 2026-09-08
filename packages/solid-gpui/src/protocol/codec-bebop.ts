import { EVENT_APPLICATION_ACTIVATION, COMMAND_CONFIGURE_APPLICATION } from "./constants";
import { BebopView } from "bebop";
import { boundedBebopDecode } from "./bebop-guard";
import {
  AccessibilityProperties as WireAccessibilityProperties,
  Body,
  Body as WireBody,
  BoxShadowSet as WireBoxShadowSet,
  BoxShadowValue as WireBoxShadowValue,
  ClearStyle,
  ClipboardImageCommand as WireClipboardImageCommand,
  Command as WireCommand,
  CommandPayload as WireCommandPayload,
  CommandResult as WireCommandResult,
  CommandValue as WireCommandValue,
  Envelope,
  Event as WireEvent,
  EventPayload as WireEventPayload,
  ExtensionValue as WireExtensionValue,
  HostProperties as WireHostProperties,
  KeybindingDefinition as WireKeybindingDefinition,
  MenuDefinition as WireMenuDefinition,
  MenuItem as WireMenuItem,
  MenuItemValue as WireMenuItemValue,
  Node as WireNode,
  NotificationActionDefinition as WireNotificationActionDefinition,
  Patch as WirePatch,
  PatchOperation as WirePatchOperation,
  PatchOperationValue as WirePatchOperationValue,
  Snapshot as WireSnapshot,
  Style as WireStyle,
  Transition as WireTransition,
  WindowOpenOptions as WireWindowOpenOptions,
  WindowAppearance,
  NodeKind,
  type CommandPayload,
  type CommandValue,
  type ExtensionField as WireExtensionField,
  type EventPayload,
  type HostProperties,
  type TextInputEventData as WireTextInputEventData,
} from "./generated/protocol";
import {
  COMMAND_ACTIVATE_WINDOW,
  COMMAND_BLUR,
  COMMAND_CLIPBOARD_READ,
  COMMAND_CLIPBOARD_READ_IMAGE,
  COMMAND_CLIPBOARD_WRITE,
  COMMAND_CLIPBOARD_WRITE_IMAGE,
  COMMAND_FILE_DIALOG_OPEN,
  COMMAND_FILE_DIALOG_SAVE,
  COMMAND_INVOKE_NATIVE,
  COMMAND_CANCEL_NATIVE,
  MAX_NATIVE_CALL_BYTES,
  COMMAND_FOCUS,
  COMMAND_FOCUS_NEXT,
  COMMAND_FOCUS_PREV,
  COMMAND_GET_FOCUS,
  COMMAND_GET_SCROLL_OFFSET,
  COMMAND_GET_WINDOW_BOUNDS,
  COMMAND_GET_WINDOW_SIZE,
  COMMAND_GET_WINDOW_STATE,
  COMMAND_LOAD_FONT,
  COMMAND_MINIMIZE_WINDOW,
  COMMAND_OPEN_SURFACE,
  COMMAND_OPEN_POPUP,
  COMMAND_CLOSE_POPUP,
  COMMAND_OPEN_URL,
  COMMAND_READ_TEXT_FILE,
  COMMAND_RESIZE_WINDOW,
  COMMAND_RESOLVE_CLOSE_REQUEST,
  COMMAND_SCROLL_TO_END,
  COMMAND_SCROLL_TO_INDEX,
  COMMAND_SCROLL_TO_OFFSET,
  COMMAND_SET_CLOSE_POLICY,
  COMMAND_SET_KEYBINDINGS,
  COMMAND_SET_MENUS,
  COMMAND_SET_SELECTION,
  COMMAND_SET_TITLE,
  COMMAND_SHOW_NOTIFICATION,
  COMMAND_TOGGLE_FULLSCREEN,
  COMMAND_WRITE_TEXT_FILE,
  COMMAND_ZOOM_WINDOW,
  CLIPBOARD_IMAGE_FORMAT_GIF,
  CLIPBOARD_IMAGE_FORMAT_JPEG,
  CLIPBOARD_IMAGE_FORMAT_PNG,
  CLIPBOARD_IMAGE_FORMAT_SVG,
  COMMAND_KINDS,
  DRAG_DROP,
  DRAG_EXTERNAL_FILE_DROP,
  DRAG_OVER,
  EVENT_ACTION,
  EVENT_ANIMATION_COMPLETE,
  EVENT_BLUR,
  EVENT_CHANGE,
  EVENT_CLOSE_REQUESTED,
  EVENT_COMMAND_RESULT,
  EVENT_DRAG,
  EVENT_FOCUS,
  EVENT_HOVER,
  EVENT_KEY,
  EVENT_LAYOUT,
  EVENT_NOTIFICATION_RESPONSE,
  EVENT_POINTER,
  EVENT_POINTER_DOWN_OUTSIDE,
  EVENT_PRESS,
  EVENT_SCROLL,
  EVENT_SELECTION,
  EVENT_SUBMIT,
  EVENT_SURFACE_CLOSED,
  EVENT_VISIBLE_RANGE,
  EVENT_WINDOW_ACTIVATION,
  EVENT_WINDOW_APPEARANCE,
  EVENT_WINDOW_RESIZE,
  EVENT_KEY_DOWN,
  EVENT_KEY_REPEAT,
  EVENT_KEY_UP,
  EVENT_POINTER_DOWN,
  EVENT_POINTER_UP,
  KEY_MODIFIER_NAMES,
  MAX_CLIPBOARD_IMAGE_BYTES,
  MAX_CLIPBOARD_TEXT_BYTES,
  MAX_FILE_READ_BYTES,
  MAX_FILE_WRITE_BYTES,
  MAX_FRAME_SIZE,
  MAX_EXTENSION_FIELDS,
  MAX_EXTENSION_TEXT_BYTES,
  PROTOCOL_VERSION,
  SCROLL_DELTA_LINES,
  SCROLL_DELTA_PIXELS,
  UPDATE_ACCESSIBILITY,
  UPDATE_FOCUSABLE,
  UPDATE_LISTENER,
  UPDATE_POINTER_MOVE,
  UPDATE_PROPERTIES,
  UPDATE_SELECTABLE,
  UPDATE_STYLE,
  UPDATE_TEXT,
  UPDATE_TOOLTIP,
  EVENT_EXTENSION,
  EVENT_KIND,
  PATCH_KIND,
  SNAPSHOT_KIND,
} from "./constants";
import type {
  ClipboardImage,
  Event as SemanticEvent,
  EventPayload as SemanticEventPayload,
  ExtensionField,
  ExtensionProperties,
  ExtensionValue,
  IconName,
  OutboundMessage,
  Patch as SemanticPatch,
  PatchOperation as SemanticPatchOperation,
  Snapshot as SemanticSnapshot,
  SnapshotNode as SemanticNode,
  Command as SemanticCommand,
  CommandPayload as SemanticCommandPayload,
  CommandResult as SemanticCommandResult,
  CommandValue as SemanticCommandValue,
  AccessibilityProperties as SemanticAccessibilityProperties,
  HostProperties as SemanticHostProperties,
  IconProperties as SemanticIconProperties,
  MenuDefinition as SemanticMenuDefinition,
  MenuItemDefinition as SemanticMenuItemDefinition,
  NotificationActionDefinition as SemanticNotificationActionDefinition,
  KeybindingDefinition as SemanticKeybindingDefinition,
  WindowOpenOptions as SemanticWindowOpenOptions,
  TextInputEventData as SemanticTextInputEventData,
} from "./types";
import { validateExtensionFields, validateExtensionProperties, validateExtensionValue } from "./types";
import { bytesFrom, FrameDecoder, framePayload, type FrameChunk } from "./frame";
import { encodeColor, validateStyle, type BoxShadowInput, type StyleProp } from "../style";
import { ProtocolVersionMismatchError, validateEvent } from "./validate";

const NODE_KIND_CODES: Record<SemanticNode["kind"], NodeKind> = {
  View: NodeKind.View,
  Text: NodeKind.Text,
  Pressable: NodeKind.Pressable,
  RawText: NodeKind.RawText,
  TextInput: NodeKind.TextInput,
  VirtualList: NodeKind.VirtualList,
  Image: NodeKind.Image,
  Extension: NodeKind.Extension,
  Icon: NodeKind.Icon,
};
const FORMAT_CODES = {
  png: CLIPBOARD_IMAGE_FORMAT_PNG,
  jpeg: CLIPBOARD_IMAGE_FORMAT_JPEG,
  gif: CLIPBOARD_IMAGE_FORMAT_GIF,
  svg: CLIPBOARD_IMAGE_FORMAT_SVG,
} as const;
const FORMAT_NAMES = ["png", "jpeg", "gif", "svg"] as const;
const UTF8_ENCODER = new TextEncoder();
const MAX_REPEATED_ITEMS = 1 << 20;

function finite(value: number, name: string): number {
  if (!Number.isFinite(value)) throw new TypeError(`${name} must be finite`);
  return value;
}
function f32(value: number, name: string): number {
  if (!Number.isFinite(value)) throw new TypeError(`${name} must be finite`);
  const rounded = Math.fround(value);
  if (!Number.isFinite(rounded)) throw new RangeError(`${name} must be representable as float32`);
  return rounded;
}
function boundedString(value: string, maxBytes: number, name: string): string {
  if (typeof value !== "string" || UTF8_ENCODER.encode(value).byteLength > maxBytes)
    throw new RangeError(`${name} exceeds its byte limit`);
  return value;
}
function boundedArray<T>(value: readonly T[] | undefined, name: string): T[] | undefined {
  if (value === undefined) return undefined;
  if (value.length > MAX_REPEATED_ITEMS) throw new RangeError(`${name} exceeds the repeated-field limit`);
  return [...value];
}

function withPath<T>(path: string, operation: () => T): T {
  try {
    return operation();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new TypeError(`${path}: ${message}`, { cause: error });
  }
}
function wireExtensionValue(value: ExtensionValue): WireExtensionValue {
  if (!validateExtensionValue(value)) throw new TypeError("extension value is invalid");
  switch (value.type) {
    case "bool":
      return WireExtensionValue.fromExtensionBoolValue({ value: value.value });
    case "i32":
      return WireExtensionValue.fromExtensionInt32Value({ value: value.value });
    case "u32":
      return WireExtensionValue.fromExtensionU32Value({ value: value.value });
    case "f32":
      return WireExtensionValue.fromExtensionF32Value({ value: f32(value.value, "extension f32") });
    case "text":
      return WireExtensionValue.fromExtensionTextValue({
        value: boundedString(value.value, MAX_EXTENSION_TEXT_BYTES, "extension text"),
      });
    case "bytes":
      return WireExtensionValue.fromExtensionBytesValue({ value: value.value });
  }
}
function wireExtensionField(value: ExtensionField): WireExtensionField {
  if (!validateExtensionValue(value.value) || !Number.isInteger(value.id) || value.id < 0 || value.id > 0xffff_ffff)
    throw new TypeError("extension field is invalid");
  return { id: value.id, value: wireExtensionValue(value.value) };
}

function wireTransition(value: NonNullable<NonNullable<StyleProp>["transition"]>): WireTransition {
  const properties = value.properties;
  let propertyMask = 0;
  if (properties === undefined || properties.includes("opacity")) propertyMask |= 1;
  if (properties === undefined || properties.includes("backgroundColor")) propertyMask |= 2;
  if (properties === undefined || properties.includes("width")) propertyMask |= 4;
  if (properties === undefined || properties.includes("height")) propertyMask |= 8;
  const easing = value.easing === "linear" ? 0 : value.easing === "easeIn" ? 1 : value.easing === "easeOut" ? 2 : 3;
  return { durationMs: value.durationMs, delayMs: value.delayMs ?? 0, easing, propertyMask };
}
function wireShadow(value: BoxShadowInput): WireBoxShadowSet {
  const values = (Array.isArray(value) ? value : [value]).map(
    (shadow) =>
      ({
        offsetX: Math.fround(shadow.offsetX),
        offsetY: Math.fround(shadow.offsetY),
        blurRadius: Math.fround(shadow.blurRadius),
        spreadRadius: Math.fround(shadow.spreadRadius),
        color: encodeColor(shadow.color),
        inset: shadow.inset ?? false,
      }) satisfies WireBoxShadowValue,
  );
  return { values };
}
function wireStyle(value: StyleProp): WireStyle | undefined {
  if (value == null) return undefined;
  validateStyle(value);
  return {
    width: value.width === undefined ? undefined : Math.fround(value.width),
    height: value.height === undefined ? undefined : Math.fround(value.height),
    gridColumns: value.gridColumns,
    gridRows: value.gridRows,
    gridColumnSpan: value.gridColumnSpan,
    gridRowSpan: value.gridRowSpan,
    flexDirection:
      value.flexDirection === undefined
        ? undefined
        : { row: 1, column: 2, "row-reverse": 3, "column-reverse": 4 }[value.flexDirection],
    flexGrow: value.flexGrow === undefined ? undefined : Math.fround(value.flexGrow),
    padding: value.padding === undefined ? undefined : Math.fround(value.padding),
    gap: value.gap === undefined ? undefined : Math.fround(value.gap),
    backgroundColor: value.backgroundColor === undefined ? undefined : encodeColor(value.backgroundColor),
    color: value.color === undefined ? undefined : encodeColor(value.color),
    opacity: value.opacity === undefined ? undefined : Math.fround(value.opacity),
    transition: value.transition === undefined ? undefined : wireTransition(value.transition),
    justifyContent:
      value.justifyContent === undefined
        ? undefined
        : { "flex-start": 1, center: 2, "flex-end": 3, "space-between": 4, "space-around": 5, "space-evenly": 6 }[
            value.justifyContent
          ],
    alignItems:
      value.alignItems === undefined
        ? undefined
        : { "flex-start": 1, center: 2, "flex-end": 3, stretch: 4, baseline: 5 }[value.alignItems],
    borderRadius: value.borderRadius === undefined ? undefined : Math.fround(value.borderRadius),
    borderWidth: value.borderWidth === undefined ? undefined : Math.fround(value.borderWidth),
    borderColor: value.borderColor === undefined ? undefined : encodeColor(value.borderColor),
    fontSize: value.fontSize === undefined ? undefined : Math.fround(value.fontSize),
    fontWeight:
      value.fontWeight === undefined
        ? undefined
        : { normal: 400, medium: 500, semibold: 600, bold: 700, heavy: 900 }[value.fontWeight],
    overflow: value.overflow === undefined ? undefined : { visible: 1, hidden: 2, scroll: 3 }[value.overflow],
    lineClamp: value.lineClamp,
    textOverflow: value.textOverflow === undefined ? undefined : { clip: 1, ellipsis: 2 }[value.textOverflow],
    marginTop: value.marginTop === undefined ? undefined : Math.fround(value.marginTop),
    marginRight: value.marginRight === undefined ? undefined : Math.fround(value.marginRight),
    marginBottom: value.marginBottom === undefined ? undefined : Math.fround(value.marginBottom),
    marginLeft: value.marginLeft === undefined ? undefined : Math.fround(value.marginLeft),
    fontStyle: value.fontStyle === undefined ? undefined : value.fontStyle === "italic" ? 1 : 0,
    textDecoration:
      value.textDecoration === undefined
        ? undefined
        : value.textDecoration === "none"
          ? 0
          : value.textDecoration === "underline"
            ? 1
            : 2,
    lineHeight: value.lineHeight === undefined ? undefined : Math.fround(value.lineHeight),
    minWidth: value.minWidth === undefined ? undefined : Math.fround(value.minWidth),
    maxWidth: value.maxWidth === undefined ? undefined : Math.fround(value.maxWidth),
    minHeight: value.minHeight === undefined ? undefined : Math.fround(value.minHeight),
    maxHeight: value.maxHeight === undefined ? undefined : Math.fround(value.maxHeight),
    flexShrink: value.flexShrink === undefined ? undefined : Math.fround(value.flexShrink),
    alignSelf:
      value.alignSelf === undefined
        ? undefined
        : { start: 1, end: 2, "flex-start": 3, "flex-end": 4, center: 5, baseline: 6, stretch: 7 }[value.alignSelf],
    position:
      value.position === undefined
        ? undefined
        : value.position === "relative"
          ? 0
          : value.position === "absolute"
            ? 1
            : 2,
    left: value.left === undefined ? undefined : Math.fround(value.left),
    top: value.top === undefined ? undefined : Math.fround(value.top),
    right: value.right === undefined ? undefined : Math.fround(value.right),
    bottom: value.bottom === undefined ? undefined : Math.fround(value.bottom),
    cursor:
      value.cursor === undefined
        ? undefined
        : {
            default: 0,
            text: 1,
            pointer: 2,
            grab: 3,
            grabbing: 4,
            "not-allowed": 5,
            "context-menu": 6,
            crosshair: 7,
            "vertical-text": 8,
            alias: 9,
            copy: 10,
            "no-drop": 11,
            move: 12,
            "ew-resize": 13,
            "ns-resize": 14,
            "nesw-resize": 15,
            "nwse-resize": 16,
            "col-resize": 17,
            "row-resize": 18,
          }[value.cursor],
    textAlign: value.textAlign === undefined ? undefined : { left: 1, center: 2, right: 3 }[value.textAlign],
    boxShadow: value.boxShadow === undefined ? undefined : wireShadow(value.boxShadow),
    linearGradient:
      value.linearGradient === undefined
        ? undefined
        : {
            angle: Math.fround(value.linearGradient.angle),
            startColor: encodeColor(value.linearGradient.stops[0].color),
            startPosition: Math.fround(value.linearGradient.stops[0].position),
            endColor: encodeColor(value.linearGradient.stops[1].color),
            endPosition: Math.fround(value.linearGradient.stops[1].position),
          },
    fontFamily: value.fontFamily,
    borderTopColor: value.borderTopColor === undefined ? undefined : encodeColor(value.borderTopColor),
    borderRightColor: value.borderRightColor === undefined ? undefined : encodeColor(value.borderRightColor),
    borderBottomColor: value.borderBottomColor === undefined ? undefined : encodeColor(value.borderBottomColor),
    borderLeftColor: value.borderLeftColor === undefined ? undefined : encodeColor(value.borderLeftColor),

    paddingTop: value.paddingTop === undefined ? undefined : Math.fround(value.paddingTop),
    paddingRight: value.paddingRight === undefined ? undefined : Math.fround(value.paddingRight),
    paddingBottom: value.paddingBottom === undefined ? undefined : Math.fround(value.paddingBottom),
    paddingLeft: value.paddingLeft === undefined ? undefined : Math.fround(value.paddingLeft),
    borderTopWidth: value.borderTopWidth === undefined ? undefined : Math.fround(value.borderTopWidth),
    borderRightWidth: value.borderRightWidth === undefined ? undefined : Math.fround(value.borderRightWidth),
    borderBottomWidth: value.borderBottomWidth === undefined ? undefined : Math.fround(value.borderBottomWidth),
    borderLeftWidth: value.borderLeftWidth === undefined ? undefined : Math.fround(value.borderLeftWidth),
    borderTopLeftRadius: value.borderTopLeftRadius === undefined ? undefined : Math.fround(value.borderTopLeftRadius),
    borderTopRightRadius:
      value.borderTopRightRadius === undefined ? undefined : Math.fround(value.borderTopRightRadius),
    borderBottomRightRadius:
      value.borderBottomRightRadius === undefined ? undefined : Math.fround(value.borderBottomRightRadius),
    borderBottomLeftRadius:
      value.borderBottomLeftRadius === undefined ? undefined : Math.fround(value.borderBottomLeftRadius),
    widthPercent: value.widthPercent === undefined ? undefined : Math.fround(value.widthPercent),
    heightPercent: value.heightPercent === undefined ? undefined : Math.fround(value.heightPercent),
    flexWrap: value.flexWrap === undefined ? undefined : { nowrap: 0, wrap: 1, "wrap-reverse": 2 }[value.flexWrap],
  };
}
function wireAccessibility(value: SemanticAccessibilityProperties | null): WireAccessibilityProperties | undefined {
  if (value === null) return undefined;
  return {
    role: value.role,
    label: value.label ?? undefined,
    description: value.description ?? undefined,
    disabled: value.disabled,
    checked: value.checked ?? undefined,
    selected: value.selected ?? undefined,
    value: value.value ?? undefined,
    expanded: value.expanded ?? undefined,
    level: value.level ?? undefined,
    live: value.live ?? undefined,
  };
}
function wireHost(value: SemanticHostProperties | null): HostProperties | undefined {
  if (value === null) return undefined;
  if (value.type === "text-input")
    return WireHostProperties.fromTextInputProperties({
      value: value.value.value,
      placeholder: value.value.placeholder ?? undefined,
      multiline: value.value.multiline,
      disabled: value.value.disabled,
      controlled: value.value.controlled,
      ackEditSeq: value.value.ackEditSeq,
      selectionStart: value.value.selectionStart,
      selectionEnd: value.value.selectionEnd,
      markedStart: value.value.markedStart ?? undefined,
      markedEnd: value.value.markedEnd ?? undefined,
      maxLength: value.value.maxLength ?? undefined,
      selectionReversed: value.value.selectionReversed,
    });
  if (value.type === "virtual-list")
    return WireHostProperties.fromVirtualListProperties({
      itemCount: value.value.itemCount,
      rangeStart: value.value.rangeStart,
      rangeEnd: value.value.rangeEnd,
      estimatedItemSize: f32(value.value.estimatedItemSize, "estimated item size"),
      overscan: value.value.overscan,
    });
  if (value.type === "image")
    return WireHostProperties.fromImageProperties({
      source: value.value.source,
      objectFit: value.value.objectFit,
      fallbackSource: value.value.fallbackSource ?? undefined,
    });
  if (value.type === "icon")
    return WireHostProperties.fromIconProperties({
      name: value.value.name,
      size: f32(value.value.size, "icon size"),
      color: value.value.color ?? undefined,
    });
  if (value.type === "drag")
    return WireHostProperties.fromDragProperties({
      dragType: value.value.dragType ?? undefined,
      exportFiles: boundedArray(value.value.exportFiles ?? undefined, "exportFiles"),
      acceptsDragOver: value.value.acceptsDragOver,
      acceptsDrop: value.value.acceptsDrop,
    });
  if (!validateExtensionProperties(value.value)) throw new TypeError("extension properties are invalid");
  return WireHostProperties.fromExtensionProperties({
    providerId: value.value.providerId,
    catalogDigest: value.value.catalogDigest,
    entryId: value.value.entryId,
    entryVersion: value.value.entryVersion,
    fields: value.value.fields.map((field) => wireExtensionField(field)),
    eventIds: [...value.value.eventIds],
  });
}
function wireNode(value: SemanticNode): WireNode {
  return {
    id: value.id,
    parentId: value.parentId,
    index: value.index,
    kind: NODE_KIND_CODES[value.kind],
    style: withPath(`node ${value.id}.style`, () => wireStyle(value.style)),
    text: value.text ?? undefined,
    listenerId: value.listenerId,
    hostProperties: withPath(`node ${value.id}.hostProperties`, () => wireHost(value.hostProperties)),
    accessibility: withPath(`node ${value.id}.accessibility`, () => wireAccessibility(value.accessibility)),
    focusable: value.focusable,
    selectable: value.selectable,
    tooltip: value.tooltip ?? undefined,
    acceptsPointerMove: value.acceptsPointerMove,
  };
}
function wireSnapshot(value: SemanticSnapshot): WireSnapshot {
  return {
    surfaceId: value.surfaceId,
    epoch: value.epoch,
    baseRevision: value.baseRevision,
    revision: value.revision,
    nodes: value.nodes.map((node, index) => withPath(`body.snapshot.nodes[${index}]`, () => wireNode(node))),
  };
}
function wirePatchOperation(value: SemanticPatchOperation): WirePatchOperation {
  if (value.type === "create")
    return {
      operation: WirePatchOperationValue.fromPatchCreate({
        node: withPath(`patch node ${value.node.id}`, () => wireNode(value.node)),
      }),
    };
  if (value.type === "move")
    return {
      operation: WirePatchOperationValue.fromPatchMove({ id: value.id, parentId: value.parentId, index: value.index }),
    };
  if (value.type === "delete") return { operation: WirePatchOperationValue.fromPatchDelete({ id: value.id }) };
  const clearsStyle = value.style == null;
  const style =
    value.mask & UPDATE_STYLE && !clearsStyle
      ? withPath(`patch node ${value.id}.style`, () => wireStyle(value.style))
      : undefined;
  const clearStyle = value.mask & UPDATE_STYLE && clearsStyle ? {} : undefined;
  return {
    operation: WirePatchOperationValue.fromPatchUpdate({
      id: value.id,
      mask: value.mask,
      style,
      clearStyle,
      text: value.mask & UPDATE_TEXT && value.text !== null ? value.text : undefined,
      listenerId: value.mask & UPDATE_LISTENER ? value.listenerId : undefined,
      hostProperties:
        value.mask & UPDATE_PROPERTIES
          ? withPath(`patch node ${value.id}.hostProperties`, () => wireHost(value.hostProperties))
          : undefined,
      accessibility:
        value.mask & UPDATE_ACCESSIBILITY
          ? withPath(`patch node ${value.id}.accessibility`, () => wireAccessibility(value.accessibility))
          : undefined,
      focusable: value.mask & UPDATE_FOCUSABLE ? value.focusable : undefined,
      selectable: value.mask & UPDATE_SELECTABLE ? value.selectable : undefined,
      tooltip: value.mask & UPDATE_TOOLTIP ? (value.tooltip ?? undefined) : undefined,
      acceptsPointerMove: value.mask & UPDATE_POINTER_MOVE ? value.acceptsPointerMove : undefined,
    }),
  };
}
function wirePatch(value: SemanticPatch): WirePatch {
  return {
    surfaceId: value.surfaceId,
    epoch: value.epoch,
    baseRevision: value.baseRevision,
    revision: value.revision,
    operations: value.operations.map((operation, index) =>
      withPath(`body.patch.operations[${index}]`, () => wirePatchOperation(operation)),
    ),
  };
}
function wireMenuItem(value: SemanticMenuItemDefinition): WireMenuItem {
  const item: WireMenuItemValue =
    value.type === "separator"
      ? WireMenuItemValue.fromMenuSeparator({})
      : value.type === "action"
        ? WireMenuItemValue.fromMenuAction({
            name: value.name,
            disabled: value.disabled ?? false,
            checked: value.checked ?? false,
          })
        : WireMenuItemValue.fromMenuSubmenu({ menu: wireMenu({ title: value.title, items: value.items }) });
  return { value: item };
}
function wireMenu(value: SemanticMenuDefinition): WireMenuDefinition {
  return { title: value.title, items: value.items.map(wireMenuItem) };
}
function wireNotificationActions(
  value: readonly SemanticNotificationActionDefinition[] | undefined,
): WireNotificationActionDefinition[] | undefined {
  return boundedArray(value, "notification actions")?.map((action) => ({ id: action.id, label: action.label }));
}
function wireKeybindings(value: readonly SemanticKeybindingDefinition[]): WireKeybindingDefinition[] {
  return value.map((binding) => ({ keystrokes: binding.keystrokes, actionName: binding.actionName }));
}
function wireWindowOptions(value: SemanticWindowOpenOptions | undefined): WireWindowOpenOptions | undefined {
  if (value === undefined) return undefined;
  return {
    kind: value.kind ?? undefined,
    resizable: value.resizable ?? undefined,
    minWidth: value.minWidth ?? undefined,
    minHeight: value.minHeight ?? undefined,
  };
}
function wireImage(value: ClipboardImage): WireClipboardImageCommand {
  return { format: FORMAT_CODES[value.format], bytes: value.bytes };
}
function wireCommandPayload(value: SemanticCommandPayload): CommandPayload | undefined {
  if (value === null) return undefined;
  if (value.type === "open-popup")
    return WireCommandPayload.fromOpenPopupCommand({
      anchorNodeId: value.anchorNodeId,
      width: value.width,
      height: value.height,
      placement: value.placement,
      gap: value.gap,
    });
  if (value.type === "close-popup") return WireCommandPayload.fromClosePopupCommand({ requestId: value.requestId });
  if (value.type === "configure-application")
    return WireCommandPayload.fromConfigureApplicationCommand({
      keepAlive: value.keepAlive,
      quit: value.quit,
      acknowledgedSequence: value.acknowledgedSequence,
    });
  if (value.type === "cancel-native") return WireCommandPayload.fromCancelNativeCommand({ requestId: value.requestId });
  if (value.type === "invoke-native")
    return WireCommandPayload.fromInvokeNativeCommand({
      moduleId: value.moduleId,
      moduleDigest: value.moduleDigest,
      functionId: value.functionId,
      args: value.args,
    });
  if (value.type === "selection")
    return WireCommandPayload.fromU32PairCommand({ first: value.start, second: value.end });
  if (value.type === "scroll-index")
    return WireCommandPayload.fromU32PairCommand({ first: value.index, second: value.alignment });
  if (value.type === "window-size")
    return WireCommandPayload.fromU32PairCommand({ first: value.width, second: value.height });
  if (value.type === "open-surface")
    return WireCommandPayload.fromOpenSurfaceCommand({
      title: value.title,
      width: value.width,
      height: value.height,
      options: wireWindowOptions(value.options),
    });
  if (value.type === "file-dialog-open")
    return WireCommandPayload.fromFileDialogOpenCommand({
      title: value.title,
      directories: value.directories,
      multiple: value.multiple,
    });
  if (value.type === "notification")
    return WireCommandPayload.fromNotificationCommand({
      title: value.title,
      body: value.body,
      actions: wireNotificationActions(value.actions),
    });
  if (value.type === "menus") return WireCommandPayload.fromMenusCommand({ menus: value.menus.map(wireMenu) });
  if (value.type === "keybindings")
    return WireCommandPayload.fromKeybindingsCommand({ bindings: wireKeybindings(value.bindings) });
  if (value.type === "clipboard-image") return WireCommandPayload.fromClipboardImageCommand(wireImage(value.image));
  if (value.type === "file-write")
    return WireCommandPayload.fromStringPairCommand({ path: value.path, content: value.content });
  if (value.type === "close-resolution")
    return WireCommandPayload.fromCloseResolutionCommand({ requestId: value.requestId, allow: value.allow });
  if (value.type === "text") return WireCommandPayload.fromTextCommand({ value: value.value });
  return WireCommandPayload.fromFloatCommand({ value: value.value });
}
function isU32(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff;
}
function validWindowSize(width: unknown, height: unknown): boolean {
  return (
    isU32(width) &&
    isU32(height) &&
    ((width === 0 && height === 0) || (width > 0 && width <= 16_384 && height > 0 && height <= 16_384))
  );
}
type NonNullCommandPayload = Exclude<SemanticCommandPayload, null>;
function requirePayloadType<T extends NonNullCommandPayload["type"]>(
  payload: SemanticCommandPayload,
  type: T,
): Extract<NonNullCommandPayload, { readonly type: T }> {
  if (payload === null || payload.type !== type) throw new TypeError(`command payload must be ${type}`);
  return payload as Extract<NonNullCommandPayload, { readonly type: T }>;
}
function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
function validProtocolText(
  value: unknown,
  maxBytes: number,
  nonEmpty = false,
  maxCodePoints?: number,
): value is string {
  return (
    typeof value === "string" &&
    (!nonEmpty || value.length > 0) &&
    (maxCodePoints === undefined || [...value].length <= maxCodePoints) &&
    UTF8_ENCODER.encode(value).byteLength <= maxBytes &&
    !/[\u0000-\u001f\u007f]/.test(value)
  );
}
function validMenuItems(items: unknown[], depth: number): boolean {
  if (depth > 16 || items.length > 1024) return false;
  for (const item of items) {
    if (!isRecord(item)) return false;
    if (item.type === "separator") continue;
    if (
      item.type === "action" &&
      validProtocolText(item.name, 1024, true, 256) &&
      (item.disabled === undefined || typeof item.disabled === "boolean") &&
      (item.checked === undefined || typeof item.checked === "boolean")
    )
      continue;
    if (
      item.type === "submenu" &&
      validProtocolText(item.title, 1024, true, 256) &&
      Array.isArray(item.items) &&
      validMenuItems(item.items, depth + 1)
    )
      continue;
    return false;
  }
  return true;
}
function validMenu(value: unknown, depth = 0): value is SemanticMenuDefinition {
  return (
    isRecord(value) &&
    depth <= 16 &&
    validProtocolText(value.title, 1024, true, 256) &&
    Array.isArray(value.items) &&
    validMenuItems(value.items, depth)
  );
}
function validKeybinding(value: unknown): value is SemanticKeybindingDefinition {
  if (!isRecord(value)) return false;
  return validProtocolText(value.keystrokes, 64, true) && validProtocolText(value.actionName, 64, true, 64);
}
function validateCommand(value: SemanticCommand): void {
  if (
    !isU32(value.surfaceId) ||
    !isU32(value.epoch) ||
    !isU32(value.afterRevision) ||
    !isU32(value.requestId) ||
    !isU32(value.nodeId) ||
    !COMMAND_KINDS.includes(value.command as (typeof COMMAND_KINDS)[number])
  )
    throw new TypeError("command header is invalid");
  if (value.command === COMMAND_INVOKE_NATIVE && value.nodeId === 0)
    throw new TypeError("native invocation requires a live root or component node");
  const payload = value.payload;
  switch (value.command) {
    case COMMAND_CONFIGURE_APPLICATION: {
      const control = requirePayloadType(payload, "configure-application");
      if (
        !isU32(control.acknowledgedSequence) ||
        value.surfaceId !== 0 ||
        value.nodeId !== 0 ||
        value.afterRevision !== 0 ||
        value.epoch === 0 ||
        value.requestId === 0 ||
        typeof control.keepAlive !== "boolean" ||
        typeof control.quit !== "boolean" ||
        (control.quit && control.keepAlive)
      )
        throw new TypeError("application control header or policy is invalid");
      return;
    }
    case COMMAND_CANCEL_NATIVE: {
      const target = requirePayloadType(payload, "cancel-native");
      if (value.nodeId !== 1 || !isU32(target.requestId) || target.requestId === 0)
        throw new TypeError("native cancellation requires a root and positive request ID");
      return;
    }
    case COMMAND_INVOKE_NATIVE: {
      const value = requirePayloadType(payload, "invoke-native");
      if (
        !(value.moduleId instanceof Uint8Array) ||
        value.moduleId.byteLength !== 16 ||
        !(value.moduleDigest instanceof Uint8Array) ||
        value.moduleDigest.byteLength !== 32 ||
        !isU32(value.functionId) ||
        value.functionId === 0 ||
        !(value.args instanceof Uint8Array) ||
        value.args.byteLength > MAX_NATIVE_CALL_BYTES
      )
        throw new TypeError("native invocation payload is invalid");
      return;
    }
    case COMMAND_FOCUS:
    case COMMAND_BLUR:
    case COMMAND_SCROLL_TO_END:
    case COMMAND_ZOOM_WINDOW:
    case COMMAND_TOGGLE_FULLSCREEN:
    case COMMAND_FOCUS_NEXT:
    case COMMAND_FOCUS_PREV:
    case COMMAND_GET_WINDOW_SIZE:
    case COMMAND_GET_FOCUS:
    case COMMAND_CLIPBOARD_READ:
    case COMMAND_CLIPBOARD_READ_IMAGE:
    case COMMAND_MINIMIZE_WINDOW:
    case COMMAND_GET_WINDOW_BOUNDS:
    case COMMAND_GET_WINDOW_STATE:
    case COMMAND_ACTIVATE_WINDOW:
    case COMMAND_GET_SCROLL_OFFSET:
      if (payload !== null) throw new TypeError("command does not accept a payload");
      return;
    case COMMAND_SET_SELECTION: {
      const value = requirePayloadType(payload, "selection");
      if (!isU32(value.start) || !isU32(value.end) || value.start > value.end)
        throw new TypeError("selection payload is invalid");
      return;
    }
    case COMMAND_SCROLL_TO_INDEX: {
      const value = requirePayloadType(payload, "scroll-index");
      if (!isU32(value.index) || !isU32(value.alignment)) throw new TypeError("scroll index payload is invalid");
      return;
    }
    case COMMAND_RESIZE_WINDOW: {
      const value = requirePayloadType(payload, "window-size");
      if (!validWindowSize(value.width, value.height)) throw new TypeError("window size payload is invalid");
      return;
    }
    case COMMAND_SCROLL_TO_OFFSET: {
      const value = requirePayloadType(payload, "number");
      if (!Number.isFinite(value.value) || value.value < 0 || !Number.isFinite(Math.fround(value.value)))
        throw new TypeError("scroll offset payload is invalid");
      return;
    }
    case COMMAND_OPEN_POPUP: {
      if (value.nodeId !== 1) throw new TypeError("popup creation requires the Surface root");
      const options = requirePayloadType(payload, "open-popup");
      if (
        !isU32(options.anchorNodeId) ||
        options.anchorNodeId < 2 ||
        !validWindowSize(options.width, options.height) ||
        options.width === 0 ||
        options.height === 0 ||
        !isU32(options.placement) ||
        options.placement > 11 ||
        !Number.isFinite(options.gap) ||
        options.gap < 0 ||
        options.gap > 1024
      )
        throw new TypeError("popup options are invalid");
      return;
    }
    case COMMAND_CLOSE_POPUP: {
      if (value.nodeId !== 1) throw new TypeError("popup cancellation requires the Surface root");
      const options = requirePayloadType(payload, "close-popup");
      if (!isU32(options.requestId) || options.requestId === 0) throw new TypeError("popup request is invalid");
      return;
    }
    case COMMAND_OPEN_SURFACE: {
      const value = requirePayloadType(payload, "open-surface");
      if (
        typeof value.title !== "string" ||
        [...value.title].length > 256 ||
        !validWindowSize(value.width, value.height) ||
        (value.options !== undefined &&
          ((value.options.kind !== null && ![0, 1, 2].includes(value.options.kind)) ||
            (value.options.resizable !== null && typeof value.options.resizable !== "boolean") ||
            (value.options.minWidth === null) !== (value.options.minHeight === null) ||
            (value.options.minWidth !== null &&
              (!isU32(value.options.minWidth) ||
                !isU32(value.options.minHeight) ||
                value.options.minWidth === 0 ||
                value.options.minHeight === 0 ||
                value.options.minWidth > 16_384 ||
                value.options.minHeight > 16_384))))
      )
        throw new TypeError("open surface payload is invalid");
      return;
    }
    case COMMAND_FILE_DIALOG_OPEN: {
      const value = requirePayloadType(payload, "file-dialog-open");
      if (
        typeof value.title !== "string" ||
        [...value.title].length > 256 ||
        typeof value.directories !== "boolean" ||
        typeof value.multiple !== "boolean"
      )
        throw new TypeError("file dialog payload is invalid");
      return;
    }
    case COMMAND_SHOW_NOTIFICATION: {
      const value = requirePayloadType(payload, "notification");
      if (
        typeof value.title !== "string" ||
        UTF8_ENCODER.encode(value.title).byteLength > 256 ||
        typeof value.body !== "string" ||
        UTF8_ENCODER.encode(value.body).byteLength > 1024 ||
        (value.actions !== undefined &&
          (!Array.isArray(value.actions) ||
            value.actions.length > 3 ||
            value.actions.some(
              (action) =>
                !isRecord(action) ||
                !validProtocolText(action.id, 64, true) ||
                !validProtocolText(action.label, 256, true, 256),
            )))
      )
        throw new TypeError("notification payload is invalid");
      return;
    }
    case COMMAND_SET_MENUS: {
      const value = requirePayloadType(payload, "menus");
      if (!Array.isArray(value.menus) || value.menus.length > 64 || value.menus.some((menu) => !validMenu(menu)))
        throw new TypeError("menus payload is invalid");
      return;
    }
    case COMMAND_SET_KEYBINDINGS: {
      const value = requirePayloadType(payload, "keybindings");
      if (
        !Array.isArray(value.bindings) ||
        value.bindings.length > 64 ||
        value.bindings.some((binding) => !validKeybinding(binding))
      )
        throw new TypeError("keybindings payload is invalid");
      return;
    }
    case COMMAND_CLIPBOARD_WRITE_IMAGE: {
      const value = requirePayloadType(payload, "clipboard-image");
      if (
        !isRecord(value.image) ||
        !["png", "jpeg", "gif", "svg"].includes(value.image.format) ||
        !(value.image.bytes instanceof Uint8Array) ||
        value.image.bytes.byteLength === 0 ||
        value.image.bytes.byteLength > MAX_CLIPBOARD_IMAGE_BYTES
      )
        throw new TypeError("clipboard image payload is invalid");
      return;
    }
    case COMMAND_WRITE_TEXT_FILE: {
      const value = requirePayloadType(payload, "file-write");
      if (
        typeof value.path !== "string" ||
        value.path.length === 0 ||
        UTF8_ENCODER.encode(value.path).byteLength > 1024 ||
        /[\u0000-\u001f\u007f]/.test(value.path) ||
        !value.path.startsWith("/") ||
        typeof value.content !== "string" ||
        UTF8_ENCODER.encode(value.content).byteLength > MAX_FILE_WRITE_BYTES
      )
        throw new TypeError("file write payload is invalid");
      return;
    }
    case COMMAND_RESOLVE_CLOSE_REQUEST: {
      const value = requirePayloadType(payload, "close-resolution");
      if (!isU32(value.requestId) || typeof value.allow !== "boolean")
        throw new TypeError("close resolution payload is invalid");
      return;
    }
    case COMMAND_SET_TITLE: {
      const value = requirePayloadType(payload, "text");
      if (typeof value.value !== "string" || [...value.value].length > 256 || value.value.length === 0)
        throw new TypeError("title command payload is invalid");
      return;
    }
    case COMMAND_FILE_DIALOG_SAVE: {
      const value = requirePayloadType(payload, "text");
      if (typeof value.value !== "string" || [...value.value].length > 256)
        throw new TypeError("file dialog save payload is invalid");
      return;
    }
    case COMMAND_OPEN_URL: {
      const value = requirePayloadType(payload, "text");
      if (
        typeof value.value !== "string" ||
        value.value.length === 0 ||
        UTF8_ENCODER.encode(value.value).byteLength > 2048 ||
        /\s/.test(value.value) ||
        (!value.value.startsWith("http://") && !value.value.startsWith("https://")) ||
        value.value.slice(value.value.indexOf("://") + 3).length === 0
      )
        throw new TypeError("URL command payload is invalid");
      return;
    }
    case COMMAND_CLIPBOARD_WRITE: {
      const value = requirePayloadType(payload, "text");
      if (typeof value.value !== "string" || UTF8_ENCODER.encode(value.value).byteLength > MAX_CLIPBOARD_TEXT_BYTES)
        throw new TypeError("clipboard text payload is invalid");
      return;
    }
    case COMMAND_SET_CLOSE_POLICY: {
      const value = requirePayloadType(payload, "text");
      if (value.value !== "allow" && value.value !== "require-confirmation")
        throw new TypeError("close policy payload is invalid");
      return;
    }
    case COMMAND_READ_TEXT_FILE:
    case COMMAND_LOAD_FONT: {
      const value = requirePayloadType(payload, "text");
      if (
        typeof value.value !== "string" ||
        value.value.length === 0 ||
        UTF8_ENCODER.encode(value.value).byteLength > 1024 ||
        /[\u0000-\u001f\u007f]/.test(value.value) ||
        !value.value.startsWith("/")
      )
        throw new TypeError("file path command payload is invalid");
      return;
    }
    default:
      throw new TypeError("unknown command");
  }
}
function wireCommand(value: SemanticCommand): WireCommand {
  return withPath(`body.command(kind=${value.command})`, () => {
    validateCommand(value);
    return {
      surfaceId: value.surfaceId,
      epoch: value.epoch,
      afterRevision: value.afterRevision,
      requestId: value.requestId,
      nodeId: value.nodeId,
      kind: value.command,
      payload: wireCommandPayload(value.payload),
    };
  });
}
function wireCommandValue(value: SemanticCommandValue): CommandValue {
  if (value.type === "bytes") {
    if (!(value.value instanceof Uint8Array) || value.value.byteLength > MAX_NATIVE_CALL_BYTES)
      throw new TypeError("native byte result is invalid");
    return WireCommandValue.fromBytesValue({ value: value.value });
  }
  if (value.type === "number") {
    if (!isU32(value.value)) throw new TypeError("command number must be a u32");
    return WireCommandValue.fromNumberValue({ value: value.value });
  }
  if (value.type === "pair")
    return WireCommandValue.fromPairValue({
      width: f32(value.width, "command pair width"),
      height: f32(value.height, "command pair height"),
    });
  if (value.type === "boolean") return WireCommandValue.fromBoolValue({ value: value.value });
  if (value.type === "text") return WireCommandValue.fromTextValue({ value: value.value });
  if (value.type === "paths") return WireCommandValue.fromPathsValue({ paths: [...value.paths] });
  if (value.type === "file-text") return WireCommandValue.fromFileTextValue({ value: value.value });
  if (value.type === "image") return WireCommandValue.fromImageValue(wireImage(value.image));
  if (value.type === "bounds")
    return WireCommandValue.fromBoundsValue({
      x: f32(value.x, "bounds x"),
      y: f32(value.y, "bounds y"),
      width: f32(value.width, "bounds width"),
      height: f32(value.height, "bounds height"),
    });
  if (value.type === "window-state")
    return WireCommandValue.fromWindowStateValue({ fullscreen: value.fullscreen, maximized: value.maximized });
  if (value.type === "scroll-offset")
    return WireCommandValue.fromScrollOffsetValue({ value: f32(value.value, "scroll offset") });
  throw new TypeError("unknown command value");
}
function wireTextInput(value: SemanticTextInputEventData): WireTextInputEventData {
  return {
    text: value.text,
    selectionStart: value.selectionStart,
    selectionEnd: value.selectionEnd,
    markedStart: value.markedStart ?? undefined,
    markedEnd: value.markedEnd ?? undefined,
    editSeq: value.editSeq,
    reversed: value.reversed,
  };
}
function eventType(value: SemanticEventPayload): number {
  switch (value.type) {
    case "application-activation":
      return EVENT_APPLICATION_ACTIVATION;
    case "press":
      return EVENT_PRESS;
    case "change":
      return EVENT_CHANGE;
    case "selection":
      return EVENT_SELECTION;
    case "focus":
      return EVENT_FOCUS;
    case "blur":
      return EVENT_BLUR;
    case "command-result":
      return EVENT_COMMAND_RESULT;
    case "visible-range":
      return EVENT_VISIBLE_RANGE;
    case "animation-complete":
      return EVENT_ANIMATION_COMPLETE;
    case "key":
      return EVENT_KEY;
    case "pointer":
    case "pointer-move":
      return EVENT_POINTER;
    case "hover":
      return EVENT_HOVER;
    case "scroll":
      return EVENT_SCROLL;
    case "submit":
      return EVENT_SUBMIT;
    case "window-resize":
      return EVENT_WINDOW_RESIZE;
    case "window-activation":
      return EVENT_WINDOW_ACTIVATION;
    case "surface-closed":
      return EVENT_SURFACE_CLOSED;
    case "action":
      return EVENT_ACTION;
    case "window-appearance":
      return EVENT_WINDOW_APPEARANCE;
    case "layout":
      return EVENT_LAYOUT;
    case "drag-over":
    case "drag-drop":
    case "external-file-drop":
      return EVENT_DRAG;
    case "notification-response":
      return EVENT_NOTIFICATION_RESPONSE;
    case "pointer-down-outside":
      return EVENT_POINTER_DOWN_OUTSIDE;
    case "close-requested":
      return EVENT_CLOSE_REQUESTED;
  }
  return EVENT_EXTENSION;
}
function wireEventPayload(value: SemanticEventPayload): EventPayload | undefined {
  switch (value.type) {
    case "application-activation":
      return WireEventPayload.fromApplicationActivationEvent({
        targetSurfaceId: value.targetSurfaceId,
        reason: value.reason,
        urls: [...value.urls],
      });
    case "press":
    case "hover":
    case "surface-closed":
      return undefined;
    case "change":
    case "selection":
    case "focus":
    case "blur":
      return value.data === undefined ? undefined : WireEventPayload.fromTextInputEventData(wireTextInput(value.data));
    case "command-result": {
      const result = value.result;
      return WireEventPayload.fromCommandResult({
        requestId: result.requestId,
        command: result.command,
        nodeId: result.nodeId,
        success: result.success,
        error: result.error ?? undefined,
        value: result.value === null ? undefined : wireCommandValue(result.value),
      });
    }
    case "visible-range":
      return WireEventPayload.fromVisibleRangeEvent({ start: value.start, end: value.end });
    case "animation-complete":
      return WireEventPayload.fromAnimationCompleteEvent({ generation: value.generation });
    case "key":
      return WireEventPayload.fromKeyEvent({ key: value.key, modifiers: [...value.modifiers], action: value.action });
    case "pointer":
      return WireEventPayload.fromPointerEvent({
        button: value.button,
        modifiers: [...value.modifiers],
        action: value.action,
        clickCount: value.clickCount,
        x: f32(value.x, "pointer x"),
        y: f32(value.y, "pointer y"),
      });
    case "pointer-move":
      return WireEventPayload.fromPointerMoveEvent({
        modifiers: [...value.modifiers],
        x: f32(value.x, "pointer move x"),
        y: f32(value.y, "pointer move y"),
      });
    case "scroll":
      return WireEventPayload.fromScrollEvent({
        deltaKind: value.deltaKind,
        dx: f32(value.dx, "scroll dx"),
        dy: f32(value.dy, "scroll dy"),
        x: f32(value.x, "scroll x"),
        y: f32(value.y, "scroll y"),
        modifiers: [...value.modifiers],
      });
    case "submit":
      return WireEventPayload.fromSubmitEvent({ text: value.text });
    case "window-resize":
      return WireEventPayload.fromWindowResizeEvent({
        width: f32(value.width, "window width"),
        height: f32(value.height, "window height"),
        scaleFactor: f32(value.scaleFactor, "window scale factor"),
      });
    case "window-activation":
      return WireEventPayload.fromWindowActivationEvent({ active: value.active });
    case "action":
      return WireEventPayload.fromActionEvent({ action: value.action });
    case "window-appearance":
      return WireEventPayload.fromWindowAppearanceEvent({
        appearance: value.appearance === "light" ? WindowAppearance.Light : WindowAppearance.Dark,
      });
    case "layout":
      return WireEventPayload.fromLayoutEvent({
        x: f32(value.x, "layout x"),
        y: f32(value.y, "layout y"),
        width: f32(value.width, "layout width"),
        height: f32(value.height, "layout height"),
      });
    case "drag-over":
      return WireEventPayload.fromDragOverEvent({ dragType: value.dragType });
    case "drag-drop":
      return WireEventPayload.fromDragDropEvent({ dragType: value.dragType });
    case "external-file-drop":
      return WireEventPayload.fromExternalFileDropEvent({ paths: [...value.paths] });
    case "notification-response":
      return WireEventPayload.fromNotificationResponseEvent({ tag: value.tag, actionId: value.actionId ?? undefined });
    case "pointer-down-outside":
      return WireEventPayload.fromPointerDownOutsideEvent({
        x: f32(value.x, "pointer-down-outside x"),
        y: f32(value.y, "pointer-down-outside y"),
      });
    case "close-requested":
      return WireEventPayload.fromCloseRequestedEvent({ requestId: value.requestId });
    case "extension":
      return WireEventPayload.fromExtensionEvent({
        eventId: value.eventId,
        fields: value.fields.map((field) => wireExtensionField(field)),
      });
  }
}
function wireEvent(value: SemanticEvent): WireEvent {
  return withPath(`body.event(sequence=${value.sequence})`, () => {
    if (validateEvent(value) === null) throw new TypeError("event is invalid");
    return {
      surfaceId: value.surfaceId,
      epoch: value.epoch,
      revision: value.revision,
      sequence: value.sequence,
      nodeId: value.nodeId,
      listenerId: value.listenerId,
      eventType: eventType(value.payload),
      payload: wireEventPayload(value.payload),
    };
  });
}
function wireEnvelope(value: OutboundMessage | SemanticEvent): Envelope {
  const body: WireBody =
    value.type === "snapshot"
      ? Body.fromSnapshot(wireSnapshot(value))
      : value.type === "patch"
        ? Body.fromPatch(wirePatch(value))
        : value.type === "command"
          ? Body.fromCommand(wireCommand(value))
          : Body.fromEvent(wireEvent(value));
  return { protocolVersion: PROTOCOL_VERSION, body };
}

function encodeInto(value: OutboundMessage | SemanticEvent, view: BebopView): Uint8Array {
  view.startWriting();
  Envelope.encodeInto(wireEnvelope(value), view);
  return view.toArray();
}
export function encodePayload(value: OutboundMessage | SemanticEvent): Uint8Array {
  return encodeInto(value, BebopView.getInstance()).slice();
}
export function encodeFrame(value: OutboundMessage | SemanticEvent): Uint8Array {
  const payload = encodeInto(value, BebopView.getInstance());
  if (payload.byteLength > MAX_FRAME_SIZE) throw new RangeError("Bebop frame exceeds maximum size");
  const frame = new Uint8Array(payload.byteLength + 4);
  new DataView(frame.buffer).setUint32(0, payload.byteLength, true);
  frame.set(payload, 4);
  return frame;
}
function decodeEnvelope(payload: Uint8Array): Envelope {
  if (payload.byteLength > MAX_FRAME_SIZE) throw new RangeError("Bebop payload exceeds maximum size");
  boundedBebopDecode(payload);
  const value = Envelope.decode(payload);
  if (value.protocolVersion !== PROTOCOL_VERSION) throw new ProtocolVersionMismatchError(value.protocolVersion ?? 0);
  if (value.body === undefined) throw new TypeError("Bebop envelope body is missing");
  return value;
}
function decodeImage(value: { format?: number; bytes?: Uint8Array }): ClipboardImage | null {
  if (
    value.format === undefined ||
    value.bytes === undefined ||
    !Number.isInteger(value.format) ||
    value.format < 1 ||
    value.format > 4 ||
    value.bytes.byteLength === 0 ||
    value.bytes.byteLength > MAX_CLIPBOARD_IMAGE_BYTES
  )
    return null;
  return { format: FORMAT_NAMES[value.format - 1], bytes: value.bytes.slice() };
}
function semanticCommandValue(value: CommandValue): SemanticCommandValue | null {
  switch (value.tag) {
    case 1:
      return value.value.value === undefined || !isU32(value.value.value)
        ? null
        : { type: "number", value: value.value.value };
    case 2:
      return value.value.width === undefined || value.value.height === undefined
        ? null
        : {
            type: "pair",
            width: finite(value.value.width, "command pair width"),
            height: finite(value.value.height, "command pair height"),
          };
    case 3:
      return value.value.value === undefined ? null : { type: "boolean", value: value.value.value };
    case 4:
      return value.value.value === undefined
        ? null
        : { type: "text", value: boundedString(value.value.value, MAX_CLIPBOARD_TEXT_BYTES, "command text") };
    case 5:
      return value.value.paths === undefined || value.value.paths.length === 0
        ? null
        : {
            type: "paths",
            paths: value.value.paths.map((path) => boundedString(path, MAX_FILE_READ_BYTES, "command path")),
          };
    case 6:
      return value.value.value === undefined
        ? null
        : { type: "file-text", value: boundedString(value.value.value, MAX_FILE_READ_BYTES, "file text") };
    case 7: {
      const image = decodeImage(value.value);
      return image === null ? null : { type: "image", image };
    }
    case 8:
      return value.value.x === undefined ||
        value.value.y === undefined ||
        value.value.width === undefined ||
        value.value.height === undefined
        ? null
        : {
            type: "bounds",
            x: finite(value.value.x, "bounds x"),
            y: finite(value.value.y, "bounds y"),
            width: finite(value.value.width, "bounds width"),
            height: finite(value.value.height, "bounds height"),
          };
    case 9:
      return value.value.fullscreen === undefined || value.value.maximized === undefined
        ? null
        : { type: "window-state", fullscreen: value.value.fullscreen, maximized: value.value.maximized };
    case 10:
      return value.value.value === undefined
        ? null
        : { type: "scroll-offset", value: finite(value.value.value, "scroll offset") };
    case 11:
      return value.value.value instanceof Uint8Array && value.value.value.byteLength <= MAX_NATIVE_CALL_BYTES
        ? { type: "bytes", value: value.value.value }
        : null;
    default:
      return null;
  }
}
function semanticCommandResult(value: WireCommandResult): SemanticCommandResult | null {
  if (
    value.requestId === undefined ||
    value.command === undefined ||
    value.nodeId === undefined ||
    value.success === undefined ||
    !COMMAND_KINDS.includes(value.command as never)
  )
    return null;
  const commandValue = value.value === undefined ? null : semanticCommandValue(value.value);
  if (value.value !== undefined && commandValue === null) return null;
  return {
    requestId: value.requestId,
    command: value.command,
    nodeId: value.nodeId,
    success: value.success,
    error: value.error ?? null,
    value: commandValue,
  };
}

function semanticExtensionValue(value: WireExtensionValue): ExtensionValue | null {
  let candidate: ExtensionValue | null;
  switch (value.tag) {
    case 1:
      candidate = value.value.value === undefined ? null : { type: "bool", value: value.value.value };
      break;
    case 2:
      candidate = value.value.value === undefined ? null : { type: "i32", value: value.value.value };
      break;
    case 3:
      candidate = value.value.value === undefined ? null : { type: "u32", value: value.value.value };
      break;
    case 4:
      candidate = value.value.value === undefined ? null : { type: "f32", value: value.value.value };
      break;
    case 5:
      candidate = value.value.value === undefined ? null : { type: "text", value: value.value.value };
      break;
    case 6:
      candidate = value.value.value === undefined ? null : { type: "bytes", value: value.value.value };
      break;
    default:
      return null;
  }
  return candidate !== null && validateExtensionValue(candidate) ? candidate : null;
}

function semanticExtensionFields(fields: readonly WireExtensionField[] | undefined): ExtensionField[] | null {
  if (fields === undefined || fields.length > MAX_EXTENSION_FIELDS) return null;
  const result: ExtensionField[] = [];
  for (const field of fields) {
    if (field.id === undefined || field.value === undefined) return null;
    const value = semanticExtensionValue(field.value);
    if (value === null) return null;
    result.push({ id: field.id, value });
  }
  return validateExtensionFields(result) ? result : null;
}

function semanticExtensionProperties(value: Extract<HostProperties, { tag: 5 }>["value"]): ExtensionProperties | null {
  if (
    value.providerId === undefined ||
    value.catalogDigest === undefined ||
    value.entryId === undefined ||
    value.entryVersion === undefined ||
    value.fields === undefined ||
    value.eventIds === undefined
  )
    return null;
  const fields = semanticExtensionFields(value.fields);
  const properties: ExtensionProperties = {
    providerId: value.providerId.slice(),
    catalogDigest: value.catalogDigest.slice(),
    entryId: value.entryId,
    entryVersion: value.entryVersion,
    fields: fields ?? [],
    eventIds: [...value.eventIds],
  };
  return fields !== null && validateExtensionProperties(properties) ? properties : null;
}
function semanticTextInput(value: WireTextInputEventData): SemanticTextInputEventData | null {
  if (
    value.text === undefined ||
    value.selectionStart === undefined ||
    value.selectionEnd === undefined ||
    value.editSeq === undefined ||
    value.reversed === undefined ||
    value.selectionStart > value.selectionEnd ||
    (value.markedStart === undefined) !== (value.markedEnd === undefined) ||
    (value.markedStart !== undefined && value.markedEnd !== undefined && value.markedStart > value.markedEnd)
  )
    return null;
  return {
    text: value.text,
    selectionStart: value.selectionStart,
    selectionEnd: value.selectionEnd,
    markedStart: value.markedStart ?? null,
    markedEnd: value.markedEnd ?? null,
    editSeq: value.editSeq,
    reversed: value.reversed,
  };
}
function semanticEventPayload(eventType: number, value: WireEventPayload | undefined): SemanticEventPayload | null {
  if (eventType === EVENT_PRESS) return value === undefined ? { type: "press" } : null;
  if (eventType === EVENT_HOVER) return value === undefined ? { type: "hover" } : null;
  if (eventType === EVENT_SURFACE_CLOSED) return value === undefined ? { type: "surface-closed" } : null;
  if (eventType === EVENT_FOCUS || eventType === EVENT_BLUR) {
    if (value === undefined) return eventType === EVENT_FOCUS ? { type: "focus" } : { type: "blur" };
    if (value.tag !== 1) return null;
    const data = semanticTextInput(value.value);
    return data === null ? null : eventType === EVENT_FOCUS ? { type: "focus", data } : { type: "blur", data };
  }
  if (value === undefined) return null;
  switch (eventType) {
    case EVENT_APPLICATION_ACTIVATION:
      if (
        value.tag !== 22 ||
        value.value.targetSurfaceId === undefined ||
        (value.value.reason !== "launch" && value.value.reason !== "reopen" && value.value.reason !== "open-urls") ||
        value.value.urls === undefined
      )
        return null;
      return {
        type: "application-activation",
        targetSurfaceId: value.value.targetSurfaceId,
        reason: value.value.reason,
        urls: value.value.urls,
      };
    case EVENT_CHANGE:
    case EVENT_SELECTION: {
      if (value.tag !== 1) return null;
      const data = semanticTextInput(value.value);
      return data === null ? null : eventType === EVENT_CHANGE ? { type: "change", data } : { type: "selection", data };
    }
    case EVENT_COMMAND_RESULT: {
      if (value.tag !== 2) return null;
      const result = semanticCommandResult(value.value);
      return result === null ? null : { type: "command-result", result };
    }
    case EVENT_VISIBLE_RANGE:
      return value.tag === 3 &&
        value.value.start !== undefined &&
        value.value.end !== undefined &&
        value.value.start <= value.value.end
        ? { type: "visible-range", start: value.value.start, end: value.value.end }
        : null;
    case EVENT_ANIMATION_COMPLETE:
      return value.tag === 4 && value.value.generation !== undefined
        ? { type: "animation-complete", generation: value.value.generation }
        : null;
    case EVENT_KEY:
      return value.tag === 5 &&
        value.value.key !== undefined &&
        value.value.key.length > 0 &&
        value.value.modifiers !== undefined &&
        value.value.action !== undefined &&
        ([EVENT_KEY_DOWN, EVENT_KEY_REPEAT, EVENT_KEY_UP] as readonly number[]).includes(value.value.action)
        ? {
            type: "key",
            key: value.value.key,
            modifiers: value.value.modifiers,
            action: value.value.action as 1 | 2 | 3,
          }
        : null;
    case EVENT_POINTER: {
      if (value.tag === 6)
        return value.value.button !== undefined &&
          value.value.modifiers !== undefined &&
          value.value.action !== undefined &&
          value.value.clickCount !== undefined &&
          value.value.clickCount > 0 &&
          value.value.x !== undefined &&
          value.value.y !== undefined
          ? {
              type: "pointer",
              button: value.value.button as 1 | 2 | 3 | 4 | 5,
              modifiers: value.value.modifiers,
              action: value.value.action as 1 | 2,
              clickCount: value.value.clickCount,
              x: value.value.x,
              y: value.value.y,
            }
          : null;
      return value.tag === 7 &&
        value.value.modifiers !== undefined &&
        value.value.x !== undefined &&
        value.value.y !== undefined
        ? { type: "pointer-move", x: value.value.x, y: value.value.y, modifiers: value.value.modifiers }
        : null;
    }
    case EVENT_SCROLL:
      return value.tag === 8 &&
        value.value.deltaKind !== undefined &&
        ([SCROLL_DELTA_PIXELS, SCROLL_DELTA_LINES] as readonly number[]).includes(value.value.deltaKind) &&
        value.value.dx !== undefined &&
        value.value.dy !== undefined &&
        value.value.x !== undefined &&
        value.value.y !== undefined &&
        value.value.modifiers !== undefined
        ? {
            type: "scroll",
            deltaKind: value.value.deltaKind as 1 | 2,
            dx: value.value.dx,
            dy: value.value.dy,
            x: value.value.x,
            y: value.value.y,
            modifiers: value.value.modifiers,
          }
        : null;
    case EVENT_SUBMIT:
      return value.tag === 9 && value.value.text !== undefined ? { type: "submit", text: value.value.text } : null;
    case EVENT_WINDOW_RESIZE:
      return value.tag === 10 &&
        value.value.width !== undefined &&
        value.value.height !== undefined &&
        value.value.scaleFactor !== undefined &&
        value.value.width >= 0 &&
        value.value.height >= 0 &&
        value.value.scaleFactor > 0
        ? {
            type: "window-resize",
            width: value.value.width,
            height: value.value.height,
            scaleFactor: value.value.scaleFactor,
          }
        : null;
    case EVENT_WINDOW_ACTIVATION:
      return value.tag === 11 && value.value.active !== undefined
        ? { type: "window-activation", active: value.value.active }
        : null;
    case EVENT_ACTION:
      return value.tag === 12 && value.value.action !== undefined && value.value.action.length > 0
        ? { type: "action", action: value.value.action }
        : null;
    case EVENT_WINDOW_APPEARANCE:
      return value.tag === 13 &&
        value.value.appearance !== undefined &&
        (value.value.appearance === WindowAppearance.Light || value.value.appearance === WindowAppearance.Dark)
        ? {
            type: "window-appearance",
            appearance: value.value.appearance === WindowAppearance.Light ? "light" : "dark",
          }
        : null;
    case EVENT_LAYOUT:
      return value.tag === 14 &&
        value.value.x !== undefined &&
        value.value.y !== undefined &&
        value.value.width !== undefined &&
        value.value.height !== undefined
        ? { type: "layout", x: value.value.x, y: value.value.y, width: value.value.width, height: value.value.height }
        : null;
    case EVENT_DRAG:
      if (value.tag === 15 && value.value.dragType !== undefined)
        return { type: "drag-over", dragType: value.value.dragType };
      if (value.tag === 16 && value.value.dragType !== undefined)
        return { type: "drag-drop", dragType: value.value.dragType };
      return value.tag === 17 && value.value.paths !== undefined && value.value.paths.length > 0
        ? { type: "external-file-drop", paths: value.value.paths }
        : null;
    case EVENT_EXTENSION:
      if (value.tag !== 21 || value.value.eventId === undefined) return null;
      {
        const fields = semanticExtensionFields(value.value.fields);
        return fields === null ? null : { type: "extension", eventId: value.value.eventId, fields };
      }
    case EVENT_NOTIFICATION_RESPONSE:
      return value.tag === 18 && value.value.tag !== undefined
        ? { type: "notification-response", tag: value.value.tag, actionId: value.value.actionId ?? null }
        : null;
    case EVENT_POINTER_DOWN_OUTSIDE:
      return value.tag === 19 && value.value.x !== undefined && value.value.y !== undefined
        ? { type: "pointer-down-outside", x: value.value.x, y: value.value.y }
        : null;
    case EVENT_CLOSE_REQUESTED:
      return value.tag === 20 && value.value.requestId !== undefined
        ? { type: "close-requested", requestId: value.value.requestId }
        : null;
    default:
      return null;
  }
}
function semanticEvent(value: WireEvent): SemanticEvent | null {
  if (
    value.surfaceId === undefined ||
    value.epoch === undefined ||
    value.revision === undefined ||
    value.sequence === undefined ||
    value.nodeId === undefined ||
    value.listenerId === undefined ||
    value.eventType === undefined
  )
    return null;
  const payload = semanticEventPayload(value.eventType, value.payload);
  if (payload === null) return null;
  return validateEvent({
    type: "event",
    surfaceId: value.surfaceId,
    epoch: value.epoch,
    revision: value.revision,
    sequence: value.sequence,
    nodeId: value.nodeId,
    listenerId: value.listenerId,
    payload,
  });
}
export function decodeEvent(payload: Uint8Array): SemanticEvent | null {
  const envelope = decodeEnvelope(payload);
  if (envelope.body?.tag !== 2) return null;
  return semanticEvent(envelope.body.value);
}

export interface MessageClassification {
  readonly kind: "snapshot" | "patch" | "event" | "command" | "unknown";
  readonly event_type?: number;
  readonly command_kind?: number;
  readonly request_id?: number;
  readonly success?: boolean;
}
export function classifyPayload(payload: Uint8Array): MessageClassification {
  try {
    const body = decodeEnvelope(payload).body;
    if (body === undefined) return { kind: "unknown" };
    if (body.tag === 1) return { kind: "snapshot" };
    if (body.tag === 3) return { kind: "patch" };
    if (body.tag === 2) {
      const event = body.value;
      if (event.eventType === undefined) return { kind: "unknown" };
      const result = event.payload?.tag === 2 ? event.payload.value : undefined;
      return {
        kind: "event",
        event_type: event.eventType,
        ...(result?.requestId === undefined ? {} : { request_id: result.requestId }),
        ...(result?.success === undefined ? {} : { success: result.success }),
      };
    }
    if (body.tag === 4) {
      const result: MessageClassification = {
        kind: "command",
        command_kind: body.value.kind,
        request_id: body.value.requestId,
      };
      return result;
    }
    return { kind: "unknown" };
  } catch {
    return { kind: "unknown" };
  }
}
export interface ProtocolCodec {
  encodeFrame(message: OutboundMessage): Uint8Array;
  push(chunk: FrameChunk): SemanticEvent[];
}
export class BebopProtocolCodec implements ProtocolCodec {
  private readonly frames: FrameDecoder;
  constructor(maxFrameSize?: number) {
    this.frames = new FrameDecoder(maxFrameSize);
  }
  encodeFrame(message: OutboundMessage): Uint8Array {
    return encodeFrame(message);
  }
  push(chunk: FrameChunk): SemanticEvent[] {
    const events: SemanticEvent[] = [];
    for (const payload of this.frames.push(bytesFrom(chunk))) {
      const event = decodeEvent(payload);
      if (event === null) throw new Error("received malformed event frame");
      events.push(event);
    }
    return events;
  }
}

export { framePayload, MAX_FRAME_SIZE, PROTOCOL_VERSION, EVENT_KIND, PATCH_KIND, SNAPSHOT_KIND };
