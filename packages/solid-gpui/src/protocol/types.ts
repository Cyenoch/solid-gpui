import type { StyleProp } from "../style";
import {
  COMMAND_KINDS,
  MAX_EXTENSION_BYTES,
  MAX_EXTENSION_EVENTS,
  MAX_EXTENSION_FIELDS,
  MAX_EXTENSION_TEXT_BYTES,
  type ClipboardImageFormat,
} from "./constants";

export type { ClipboardImageFormat } from "./constants";

export type CommandKind = (typeof COMMAND_KINDS)[number];

export const ICON_NAMES = [
  "lucide:arrow-left",
  "lucide:bell",
  "lucide:book-open",
  "lucide:box",
  "lucide:brush",
  "lucide:check",
  "lucide:check-square",
  "lucide:chevron-down",
  "lucide:chevron-right",
  "lucide:clipboard-list",
  "lucide:clock",
  "lucide:copy",
  "lucide:cpu",
  "lucide:download",
  "lucide:ellipsis",
  "lucide:external-link",
  "lucide:file",
  "lucide:folder",
  "lucide:folder-open",
  "lucide:folder-plus",
  "lucide:gauge",
  "lucide:globe",
  "lucide:hard-drive",
  "lucide:heart",
  "lucide:home",
  "lucide:house",
  "lucide:image",
  "lucide:info",
  "lucide:layers",
  "lucide:layout",
  "lucide:list",
  "lucide:minus",
  "lucide:moon",
  "lucide:more-horizontal",
  "lucide:monitor",
  "lucide:mouse-pointer-click",
  "lucide:move",
  "lucide:package",
  "lucide:palette",
  "lucide:pause",
  "lucide:play",
  "lucide:plus",
  "lucide:power",
  "lucide:puzzle",
  "lucide:refresh-cw",
  "lucide:rocket",
  "lucide:search",
  "lucide:settings",
  "lucide:sliders-horizontal",
  "lucide:square",
  "lucide:star",
  "lucide:sun",
  "lucide:text-cursor-input",
  "lucide:trash-2",
  "lucide:triangle-alert",
  "lucide:type",
  "lucide:upload",
  "lucide:user",
  "lucide:users",
  "lucide:x",
  "lucide:zap",
] as const;
declare const applicationIcon: unique symbol;
export type ApplicationIconName = string & { readonly [applicationIcon]: true };
export type IconName = (typeof ICON_NAMES)[number] | ApplicationIconName;
const applicationIcons = new Set<string>();

/** Register names from the application's embedded Rust catalog before rendering. */
export function registerIconNames<const T extends readonly string[]>(
  names: T,
): { readonly [K in keyof T]: T[K] & ApplicationIconName } {
  const candidate = new Set(applicationIcons);
  for (const name of names) {
    if (
      typeof name !== "string" ||
      name.length > 128 ||
      !/^[a-z0-9-]+:[a-z0-9-]+$/.test(name) ||
      (ICON_NAMES as readonly string[]).includes(name)
    )
      throw new TypeError(`Invalid or reserved application icon: ${name}`);
    candidate.add(name);
  }
  if (candidate.size > 256) throw new RangeError("Application icon catalog exceeds 256 names");
  for (const name of candidate) applicationIcons.add(name);
  return Object.freeze([...names]) as unknown as { readonly [K in keyof T]: T[K] & ApplicationIconName };
}
export function isIconName(name: unknown): name is IconName {
  return typeof name === "string" && ((ICON_NAMES as readonly string[]).includes(name) || applicationIcons.has(name));
}
export type NodeKind =
  | "View"
  | "Text"
  | "Pressable"
  | "RawText"
  | "TextInput"
  | "VirtualList"
  | "Image"
  | "Extension"
  | "Icon";

export type ExtensionValue =
  | { readonly type: "bool"; readonly value: boolean }
  | { readonly type: "i32"; readonly value: number }
  | { readonly type: "u32"; readonly value: number }
  | { readonly type: "f32"; readonly value: number }
  | { readonly type: "text"; readonly value: string }
  | { readonly type: "bytes"; readonly value: Uint8Array };

export interface ExtensionField {
  readonly id: number;
  readonly value: ExtensionValue;
}

export interface ExtensionProperties {
  readonly providerId: Uint8Array;
  readonly catalogDigest: Uint8Array;
  readonly entryId: number;
  readonly entryVersion: number;
  readonly fields: readonly ExtensionField[];
  readonly eventIds: readonly number[];
}

export type PointerAction = 1 | 2;
export type KeyActionCode = 1 | 2 | 3;
export type ScrollDeltaCode = 1 | 2;

export interface ClipboardImage {
  readonly format: ClipboardImageFormat;
  readonly bytes: Uint8Array;
}

export interface TextInputProperties {
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

export interface VirtualListProperties {
  readonly itemCount: number;
  readonly rangeStart: number;
  readonly rangeEnd: number;
  readonly estimatedItemSize: number;
  readonly overscan: number;
}

export interface ImageProperties {
  readonly source: string;
  readonly objectFit: 1 | 2 | 3 | 4 | 5;
  readonly fallbackSource: string | null;
}

export interface IconProperties {
  readonly name: IconName;
  readonly size: number;
  readonly color: number | null;
}

export interface DragProperties {
  readonly dragType: string | null;
  readonly exportFiles: readonly string[] | null;
  readonly acceptsDragOver: boolean;
  readonly acceptsDrop: boolean;
}

export type HostProperties =
  | { readonly type: "text-input"; readonly value: TextInputProperties }
  | { readonly type: "virtual-list"; readonly value: VirtualListProperties }
  | { readonly type: "image"; readonly value: ImageProperties }
  | { readonly type: "drag"; readonly value: DragProperties }
  | { readonly type: "extension"; readonly value: ExtensionProperties }
  | { readonly type: "icon"; readonly value: IconProperties };
export interface AccessibilityProperties {
  readonly role: number;
  readonly label: string | null;
  readonly description: string | null;
  readonly disabled: boolean;
  readonly checked: boolean | null;
  readonly selected: boolean | null;
  readonly value: string | null;
  readonly expanded: boolean | null;
  readonly level: number | null;
}

export interface SnapshotNode {
  readonly id: number;
  readonly parentId: number;
  readonly index: number;
  readonly kind: NodeKind;
  readonly style: StyleProp;
  readonly text: string | null;
  readonly listenerId: number;
  readonly hostProperties: HostProperties | null;
  readonly accessibility: AccessibilityProperties | null;
  readonly focusable: boolean;
  readonly selectable: boolean;
  readonly tooltip: string | null;
  readonly acceptsPointerMove: boolean;
}

export interface Snapshot {
  readonly type: "snapshot";
  readonly surfaceId: number;
  readonly epoch: number;
  readonly baseRevision: number;
  readonly revision: number;
  readonly nodes: readonly SnapshotNode[];
}

export interface PatchCreate {
  readonly type: "create";
  readonly node: SnapshotNode;
}

export interface PatchUpdate {
  readonly type: "update";
  readonly id: number;
  readonly mask: number;
  readonly style: StyleProp;
  readonly text: string | null;
  readonly listenerId: number;
  readonly hostProperties: HostProperties | null;
  readonly accessibility: AccessibilityProperties | null;
  readonly focusable: boolean;
  readonly selectable: boolean;
  readonly tooltip: string | null;
  readonly acceptsPointerMove: boolean;
}

export interface PatchMove {
  readonly type: "move";
  readonly id: number;
  readonly parentId: number;
  readonly index: number;
}

export interface PatchDelete {
  readonly type: "delete";
  readonly id: number;
}

export type PatchOperation = PatchCreate | PatchUpdate | PatchMove | PatchDelete;

export interface Patch {
  readonly type: "patch";
  readonly surfaceId: number;
  readonly epoch: number;
  readonly baseRevision: number;
  readonly revision: number;
  readonly operations: readonly PatchOperation[];
}

export type MenuItemDefinition =
  | { readonly type: "separator" }
  | {
      readonly type: "action";
      readonly name: string;
      readonly disabled?: boolean;
      readonly checked?: boolean;
    }
  | {
      readonly type: "submenu";
      readonly title: string;
      readonly items: readonly MenuItemDefinition[];
    };

export interface MenuDefinition {
  readonly title: string;
  readonly items: readonly MenuItemDefinition[];
}

export interface KeybindingDefinition {
  readonly keystrokes: string;
  readonly actionName: string;
}

export interface NotificationActionDefinition {
  readonly id: string;
  readonly label: string;
}

export interface WindowOpenOptions {
  readonly kind: 0 | 1 | 2 | null;
  readonly resizable: boolean | null;
  readonly minWidth: number | null;
  readonly minHeight: number | null;
}

export type CommandPayload =
  | null
  | {
      readonly type: "invoke-native";
      readonly moduleId: Uint8Array;
      readonly moduleDigest: Uint8Array;
      readonly functionId: number;
      readonly args: Uint8Array;
    }
  | { readonly type: "selection"; readonly start: number; readonly end: number }
  | { readonly type: "scroll-index"; readonly index: number; readonly alignment: number }
  | { readonly type: "window-size"; readonly width: number; readonly height: number }
  | {
      readonly type: "open-surface";
      readonly title: string;
      readonly width: number;
      readonly height: number;
      readonly options?: WindowOpenOptions;
    }
  | {
      readonly type: "file-dialog-open";
      readonly title: string;
      readonly directories: boolean;
      readonly multiple: boolean;
    }
  | {
      readonly type: "notification";
      readonly title: string;
      readonly body: string;
      readonly actions?: readonly NotificationActionDefinition[];
    }
  | { readonly type: "menus"; readonly menus: readonly MenuDefinition[] }
  | { readonly type: "keybindings"; readonly bindings: readonly KeybindingDefinition[] }
  | { readonly type: "clipboard-image"; readonly image: ClipboardImage }
  | { readonly type: "file-write"; readonly path: string; readonly content: string }
  | { readonly type: "close-resolution"; readonly requestId: number; readonly allow: boolean }
  | { readonly type: "text"; readonly value: string }
  | { readonly type: "number"; readonly value: number };

export interface Command {
  readonly type: "command";
  readonly surfaceId: number;
  readonly epoch: number;
  readonly afterRevision: number;
  readonly requestId: number;
  readonly nodeId: number;
  readonly command: CommandKind;
  readonly payload: CommandPayload;
}

export type CommandValue =
  | { readonly type: "bytes"; readonly value: Uint8Array }
  | { readonly type: "number"; readonly value: number }
  | { readonly type: "pair"; readonly width: number; readonly height: number }
  | { readonly type: "boolean"; readonly value: boolean }
  | { readonly type: "text"; readonly value: string }
  | { readonly type: "paths"; readonly paths: readonly string[] }
  | { readonly type: "file-text"; readonly value: string }
  | { readonly type: "image"; readonly image: ClipboardImage }
  | { readonly type: "bounds"; readonly x: number; readonly y: number; readonly width: number; readonly height: number }
  | { readonly type: "window-state"; readonly fullscreen: boolean; readonly maximized: boolean }
  | { readonly type: "scroll-offset"; readonly value: number };

export interface CommandResult {
  readonly requestId: number;
  readonly command: number;
  readonly nodeId: number;
  readonly success: boolean;
  readonly error: string | null;
  readonly value: CommandValue | null;
}

export interface TextInputEventData {
  readonly text: string;
  readonly selectionStart: number;
  readonly selectionEnd: number;
  readonly markedStart: number | null;
  readonly markedEnd: number | null;
  readonly editSeq: number;
  readonly reversed: boolean;
}

export type EventPayload =
  | { readonly type: "press" }
  | { readonly type: "change"; readonly data: TextInputEventData }
  | { readonly type: "selection"; readonly data: TextInputEventData }
  | { readonly type: "focus"; readonly data?: TextInputEventData }
  | { readonly type: "blur"; readonly data?: TextInputEventData }
  | { readonly type: "command-result"; readonly result: CommandResult }
  | { readonly type: "visible-range"; readonly start: number; readonly end: number }
  | { readonly type: "animation-complete"; readonly generation: number }
  | {
      readonly type: "key";
      readonly key: string;
      readonly modifiers: readonly string[];
      readonly action: KeyActionCode;
    }
  | {
      readonly type: "pointer";
      readonly button: 1 | 2 | 3 | 4 | 5;
      readonly modifiers: readonly string[];
      readonly action: PointerAction;
      readonly clickCount: number;
      readonly x: number;
      readonly y: number;
    }
  | { readonly type: "pointer-move"; readonly x: number; readonly y: number; readonly modifiers: readonly string[] }
  | { readonly type: "hover" }
  | {
      readonly type: "scroll";
      readonly deltaKind: ScrollDeltaCode;
      readonly dx: number;
      readonly dy: number;
      readonly x: number;
      readonly y: number;
      readonly modifiers: readonly string[];
    }
  | { readonly type: "submit"; readonly text: string }
  | { readonly type: "window-resize"; readonly width: number; readonly height: number; readonly scaleFactor: number }
  | { readonly type: "window-activation"; readonly active: boolean }
  | { readonly type: "surface-closed" }
  | { readonly type: "action"; readonly action: string }
  | { readonly type: "window-appearance"; readonly appearance: "light" | "dark" }
  | { readonly type: "layout"; readonly x: number; readonly y: number; readonly width: number; readonly height: number }
  | { readonly type: "drag-over"; readonly dragType: string }
  | { readonly type: "drag-drop"; readonly dragType: string }
  | { readonly type: "external-file-drop"; readonly paths: readonly string[] }
  | { readonly type: "notification-response"; readonly tag: string; readonly actionId: string | null }
  | { readonly type: "pointer-down-outside"; readonly x: number; readonly y: number }
  | { readonly type: "close-requested"; readonly requestId: number }
  | { readonly type: "extension"; readonly eventId: number; readonly fields: readonly ExtensionField[] };

export interface Event {
  readonly type: "event";
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
  readonly sequence: number;
  readonly nodeId: number;
  readonly listenerId: number;
  readonly payload: EventPayload;
}

export type ProtocolMessage = Snapshot | Patch | Command | Event;
export type OutboundMessage = Snapshot | Patch | Command;

const EXTENSION_ENCODER = new TextEncoder();

function isU32(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff;
}

/** Validate one provider-neutral extension value without throwing on malformed input. */
export function validateExtensionValue(value: unknown): value is ExtensionValue {
  try {
    if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
    const candidate = value as { readonly type?: unknown; readonly value?: unknown };
    switch (candidate.type) {
      case "bool":
        return typeof candidate.value === "boolean";
      case "i32":
        return (
          typeof candidate.value === "number" &&
          Number.isInteger(candidate.value) &&
          candidate.value >= -0x8000_0000 &&
          candidate.value <= 0x7fff_ffff
        );
      case "u32":
        return isU32(candidate.value);
      case "f32":
        return (
          typeof candidate.value === "number" &&
          Number.isFinite(candidate.value) &&
          Number.isFinite(Math.fround(candidate.value))
        );
      case "text":
        return (
          typeof candidate.value === "string" &&
          EXTENSION_ENCODER.encode(candidate.value).byteLength <= MAX_EXTENSION_TEXT_BYTES
        );
      case "bytes":
        return candidate.value instanceof Uint8Array && candidate.value.byteLength <= MAX_EXTENSION_BYTES;
      default:
        return false;
    }
  } catch {
    return false;
  }
}

/** Validate sorted extension fields and their independent aggregate payload caps. */
export function validateExtensionFields(fields: unknown): fields is readonly ExtensionField[] {
  try {
    if (!Array.isArray(fields) || fields.length > MAX_EXTENSION_FIELDS) return false;
    let previous = 0;
    let textBytes = 0;
    let bytes = 0;
    for (const field of fields) {
      if (field === null || typeof field !== "object" || Array.isArray(field)) return false;
      const candidate = field as { readonly id?: unknown; readonly value?: unknown };
      if (!isU32(candidate.id) || candidate.id <= previous || !validateExtensionValue(candidate.value)) return false;
      previous = candidate.id;
      const value = candidate.value;
      if (value.type === "text") textBytes += EXTENSION_ENCODER.encode(value.value).byteLength;
      if (value.type === "bytes") bytes += value.value.byteLength;
      if (textBytes > MAX_EXTENSION_TEXT_BYTES || bytes > MAX_EXTENSION_BYTES) return false;
    }
    return true;
  } catch {
    return false;
  }
}

/** Validate complete host extension properties without throwing on malformed input. */
export function validateExtensionProperties(value: unknown): value is ExtensionProperties {
  try {
    if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
    const candidate = value as {
      readonly providerId?: unknown;
      readonly catalogDigest?: unknown;
      readonly entryId?: unknown;
      readonly entryVersion?: unknown;
      readonly fields?: unknown;
      readonly eventIds?: unknown;
    };
    if (!(candidate.providerId instanceof Uint8Array) || candidate.providerId.byteLength !== 16) return false;
    if (!(candidate.catalogDigest instanceof Uint8Array) || candidate.catalogDigest.byteLength !== 32) return false;
    if (
      candidate.entryId === 0 ||
      !isU32(candidate.entryId) ||
      !isU32(candidate.entryVersion) ||
      candidate.entryVersion === 0
    )
      return false;
    if (!validateExtensionFields(candidate.fields)) return false;
    if (!Array.isArray(candidate.eventIds) || candidate.eventIds.length > MAX_EXTENSION_EVENTS) return false;
    let previous = 0;
    for (const id of candidate.eventIds) {
      if (!isU32(id) || id <= previous) return false;
      previous = id;
    }
    return true;
  } catch {
    return false;
  }
}
