import { isIconName, MAX_CLIPBOARD_TEXT_BYTES, utf8ByteLength } from "../protocol";
import { encodeColor, validateStyle } from "../style";
import type { AccessibilityProperties, HostProperties, IconProperties } from "../protocol";
import type { HostKind, HostNodeInternal, HostProps, TextInputProps } from "./types";

import {
  ROLE_CODES,
  defaultAccessibilityRole,
  hasAllowedProp,
  isAccessibilityProp,
  validateHostKindValues,
} from "./facts";

const OBJECT_FIT_CODES = {
  fill: 1,
  contain: 2,
  cover: 3,
  scaleDown: 4,
  none: 5,
} as const;

function assertAccessibilityText(name: string, value: unknown, maxBytes: number): asserts value is string | undefined {
  if (
    value !== undefined &&
    (typeof value !== "string" || utf8ByteLength(value) > maxBytes || /[\u0000-\u001f\u007f]/.test(value))
  ) {
    throw new TypeError(`${name} must be a safe string of at most ${maxBytes} UTF-8 bytes`);
  }
}

export function accessibilityFor(kind: HostKind, props: HostProps): AccessibilityProperties | null {
  const roleValue = Object.hasOwn(props, "accessibilityRole") ? props.accessibilityRole : undefined;
  if (roleValue !== undefined && !Object.hasOwn(ROLE_CODES, roleValue)) {
    throw new TypeError("accessibilityRole is invalid");
  }
  const role = roleValue ?? defaultAccessibilityRole(kind);
  const label = Object.hasOwn(props, "accessibilityLabel") ? props.accessibilityLabel : undefined;
  const description = Object.hasOwn(props, "accessibilityDescription") ? props.accessibilityDescription : undefined;
  const disabled = Object.hasOwn(props, "accessibilityDisabled") ? props.accessibilityDisabled : undefined;
  const checked = Object.hasOwn(props, "accessibilityChecked") ? props.accessibilityChecked : undefined;
  const selected = Object.hasOwn(props, "accessibilitySelected") ? props.accessibilitySelected : undefined;
  const value = Object.hasOwn(props, "accessibilityValue") ? props.accessibilityValue : undefined;
  const expanded = Object.hasOwn(props, "accessibilityExpanded") ? props.accessibilityExpanded : undefined;
  const level = Object.hasOwn(props, "accessibilityLevel") ? props.accessibilityLevel : undefined;
  const inputDisabled =
    kind === "TextInput" && Object.hasOwn(props, "disabled") ? (props as TextInputProps).disabled : undefined;
  assertAccessibilityText("accessibilityLabel", label, 1024);
  assertAccessibilityText("accessibilityDescription", description, MAX_CLIPBOARD_TEXT_BYTES);
  assertAccessibilityText("accessibilityValue", value, MAX_CLIPBOARD_TEXT_BYTES);
  if (disabled !== undefined && typeof disabled !== "boolean")
    throw new TypeError("accessibilityDisabled must be a boolean");
  if (checked !== undefined && typeof checked !== "boolean")
    throw new TypeError("accessibilityChecked must be a boolean");
  if (selected !== undefined && typeof selected !== "boolean")
    throw new TypeError("accessibilitySelected must be a boolean");
  if (expanded !== undefined && typeof expanded !== "boolean")
    throw new TypeError("accessibilityExpanded must be a boolean");
  if (level !== undefined && (!Number.isInteger(level) || level < 1 || level > 0xffff_ffff))
    throw new TypeError("accessibilityLevel must be a positive u32");
  if (checked !== undefined && role !== "checkbox")
    throw new TypeError("accessibilityChecked requires accessibilityRole=checkbox");
  if (level !== undefined && role !== "heading")
    throw new TypeError("accessibilityLevel requires accessibilityRole=heading");
  if (role === undefined && !Object.keys(props).some(isAccessibilityProp)) return null;
  return {
    role: role === undefined ? 0 : ROLE_CODES[role],
    label: label ?? null,
    description: description ?? null,
    disabled: disabled ?? inputDisabled ?? false,
    checked: checked ?? null,
    selected: selected ?? null,
    value: value ?? null,
    expanded: expanded ?? null,
    level: level ?? null,
  };
}
export function inputFor(node: HostNodeInternal, props: HostProps): HostProperties | null {
  if (node.kind !== "TextInput") return null;
  const input = props as TextInputProps;
  const previous = node.hostProperties;
  const previousInput = previous?.type === "text-input" ? previous.value : null;
  const maxLength = input.maxLength ?? null;
  const value = truncateUtf16(
    input.value ?? node.latestNativeText ?? previousInput?.value ?? input.defaultValue ?? "",
    maxLength,
  );
  return {
    type: "text-input",
    value: {
      value,
      placeholder: input.placeholder ?? null,
      multiline: input.multiline ?? false,
      disabled: input.disabled ?? false,
      controlled: input.value !== undefined,
      ackEditSeq: node.latestNativeEditSeq || previousInput?.ackEditSeq || 0,
      selectionStart: node.latestNativeSelection?.start ?? previousInput?.selectionStart ?? 0,
      selectionEnd: node.latestNativeSelection?.end ?? previousInput?.selectionEnd ?? 0,
      markedStart: node.latestNativeSelection?.markedStart ?? previousInput?.markedStart ?? null,
      markedEnd: node.latestNativeSelection?.markedEnd ?? previousInput?.markedEnd ?? null,
      selectionReversed: node.latestNativeSelection?.reversed ?? previousInput?.selectionReversed ?? false,
      maxLength,
    },
  };
}
function assertImageSource(name: string, value: unknown): asserts value is string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    utf8ByteLength(value) > 1024 ||
    /[\u0000-\u001f\u007f]/.test(value)
  ) {
    throw new TypeError(`Image ${name} must be a non-empty path of at most 1024 UTF-8 bytes`);
  }
}

export function imageFor(node: HostNodeInternal, props: HostProps): HostProperties | null {
  if (node.kind !== "Image") return null;
  const source = props.source;
  assertImageSource("source", source);
  const fallbackSource = props.fallbackSource;
  if (fallbackSource !== undefined) assertImageSource("fallbackSource", fallbackSource);
  const objectFit = props.objectFit ?? "contain";
  if (!["fill", "contain", "cover", "scaleDown", "none"].includes(objectFit))
    throw new TypeError("Image objectFit is invalid");
  const objectFitCode = OBJECT_FIT_CODES[objectFit];
  return { type: "image", value: { source, objectFit: objectFitCode, fallbackSource: fallbackSource ?? null } };
}
export function iconFor(node: HostNodeInternal, props: HostProps): HostProperties | null {
  if (node.kind !== "Icon") return null;
  const name = props.name;
  if (!isIconName(name)) {
    throw new TypeError("Icon name must be a built-in or registered application icon");
  }
  const size = props.size ?? 16;
  if (typeof size !== "number" || !Number.isFinite(size) || size <= 0 || !Number.isFinite(Math.fround(size))) {
    throw new RangeError("Icon size must be a positive float32");
  }
  const color = props.color;
  if (color !== undefined && typeof color !== "string") throw new TypeError("Icon color must be a CSS color");
  return {
    type: "icon",
    value: {
      name,
      size: Math.fround(size),
      color: color === undefined ? null : encodeColor(color),
    } satisfies IconProperties,
  };
}

export function virtualListFor(node: HostNodeInternal, props: HostProps): HostProperties | null {
  if (node.kind !== "VirtualList") return null;
  const itemCount = props.__itemCount;
  const rangeStart = props.__rangeStart;
  const rangeEnd = props.__rangeEnd;
  const estimatedItemSize = props.__estimatedItemSize;
  const overscan = props.__overscan ?? 0;
  if (
    itemCount === undefined ||
    rangeStart === undefined ||
    rangeEnd === undefined ||
    estimatedItemSize === undefined
  ) {
    throw new TypeError("VirtualList host properties are incomplete");
  }
  if (
    ![itemCount, rangeStart, rangeEnd, overscan].every(Number.isInteger) ||
    itemCount < 0 ||
    rangeStart < 0 ||
    rangeEnd < rangeStart ||
    rangeEnd > itemCount ||
    overscan < 0 ||
    estimatedItemSize <= 0 ||
    !Number.isFinite(estimatedItemSize) ||
    !Number.isFinite(Math.fround(estimatedItemSize))
  ) {
    throw new RangeError("VirtualList host properties are invalid");
  }
  assertU32Option("VirtualList itemCount", itemCount);
  assertU32Option("VirtualList rangeStart", rangeStart);
  assertU32Option("VirtualList rangeEnd", rangeEnd);
  assertU32Option("VirtualList overscan", overscan);
  return { type: "virtual-list", value: { itemCount, rangeStart, rangeEnd, estimatedItemSize, overscan } };
}
function exportFilesFor(value: unknown): readonly string[] | null {
  if (value === undefined) return null;
  if (!Array.isArray(value) || value.length === 0 || value.length > 8) {
    throw new RangeError("draggable.exportFiles must contain 1..8 paths");
  }
  for (const [index, path] of value.entries()) {
    if (
      typeof path !== "string" ||
      path.length === 0 ||
      utf8ByteLength(path) > 1024 ||
      /[\u0000-\u001f\u007f]/.test(path)
    ) {
      throw new TypeError(`draggable.exportFiles[${index}] must be a non-empty path of at most 1024 UTF-8 bytes`);
    }
  }
  return value;
}

export function dragFor(node: HostNodeInternal, props: HostProps): HostProperties | null {
  if (node.kind === "Pressable" && props.disabled === true) return null;
  if (
    props.draggable === undefined &&
    props.onDragOver === undefined &&
    props.onDrop === undefined &&
    props.onExternalFileDrop === undefined
  )
    return null;
  return {
    type: "drag",
    value: {
      dragType: props.draggable?.type ?? null,
      exportFiles: exportFilesFor(props.draggable?.exportFiles),
      acceptsDragOver: props.onDragOver !== undefined,
      acceptsDrop: props.onDrop !== undefined,
    },
  };
}

export function nextU32(value: number, name: string): number {
  if (value >= 0xffff_ffff) throw new RangeError(`${name} exhausted u32 range`);
  return value + 1;
}

export function assertU32Option(name: string, value: number): number {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) throw new RangeError(`${name} must be a u32`);
  return value;
}
export function truncateUtf16(value: string, maxLength: number | null): string {
  if (maxLength === null) return value;
  let units = 0;
  let end = 0;
  for (const character of value) {
    const nextUnits = units + character.length;
    if (nextUnits > maxLength) break;
    units = nextUnits;
    end += character.length;
  }
  return end === value.length ? value : value.slice(0, end);
}

export function validateProps(kind: HostKind, props: HostProps): void {
  validateHostKindValues(kind, props);
  if (kind === "Extension") {
    if (props.__extensionDescriptor === undefined) throw new TypeError("Extension descriptor is required");
    if (typeof props.__extensionDescriptor !== "object") throw new TypeError("Extension descriptor is invalid");
    if (props.onExtensionEvent !== undefined && typeof props.onExtensionEvent !== "function")
      throw new TypeError("Extension onExtensionEvent must be a function");
    if (props.onLayout !== undefined && typeof props.onLayout !== "function")
      throw new TypeError("Extension onLayout must be a function");
    if (props.style !== undefined) validateStyle(props.style);
    accessibilityFor(kind, props);
    return;
  }
  for (const key of Object.keys(props)) {
    if (!hasAllowedProp(kind, key)) throw new TypeError(`Unsupported ${kind} prop: ${key}`);
  }
  if (
    (kind === "View" || kind === "Text" || kind === "Pressable" || kind === "Image") &&
    props.onLayout !== undefined &&
    typeof props.onLayout !== "function"
  )
    throw new TypeError(`${kind} onLayout must be a function`);
  if (kind !== "RawText") validateStyle(props.style);
  if (kind === "Text" && props.onPress !== undefined && typeof props.onPress !== "function")
    throw new TypeError("Text onPress must be a function");
  if (kind === "Text" && props.selectable !== undefined && typeof props.selectable !== "boolean")
    throw new TypeError("Text selectable must be a boolean");
  if (kind === "Text") {
    if (props.onFocus !== undefined && typeof props.onFocus !== "function")
      throw new TypeError("Text onFocus must be a function");
    if (props.onBlur !== undefined && typeof props.onBlur !== "function")
      throw new TypeError("Text onBlur must be a function");
    if (props.onFocus !== undefined || props.onBlur !== undefined) {
      if (props.onPress === undefined) throw new TypeError("Text onFocus/onBlur requires onPress");
    }
  }
  if ((kind === "View" || kind === "Pressable") && props.tooltip !== undefined) {
    if (
      typeof props.tooltip !== "string" ||
      props.tooltip.length === 0 ||
      utf8ByteLength(props.tooltip) > 256 ||
      /[\u0000-\u001f\u007f-\u009f]/.test(props.tooltip)
    )
      throw new TypeError(`${kind} tooltip must be a non-empty safe string of at most 256 UTF-8 bytes`);
  }
  if (kind === "View" || kind === "Pressable") {
    if (props.onPointerDown !== undefined && typeof props.onPointerDown !== "function")
      throw new TypeError(`${kind} onPointerDown must be a function`);
    if (props.onPointerMove !== undefined && typeof props.onPointerMove !== "function")
      throw new TypeError(`${kind} onPointerMove must be a function`);
    if (props.onPointerUp !== undefined && typeof props.onPointerUp !== "function")
      throw new TypeError(`${kind} onPointerUp must be a function`);
    if (props.onHoverChange !== undefined && typeof props.onHoverChange !== "function")
      throw new TypeError(`${kind} onHoverChange must be a function`);
    if (props.onFocus !== undefined && typeof props.onFocus !== "function")
      throw new TypeError(`${kind} onFocus must be a function`);
    if (props.onBlur !== undefined && typeof props.onBlur !== "function")
      throw new TypeError(`${kind} onBlur must be a function`);
    if ((props.onFocus !== undefined || props.onBlur !== undefined) && props.focusable !== true)
      throw new TypeError(`${kind} onFocus/onBlur requires focusable=true`);
    if (kind === "View" && props.onPointerDownOutside !== undefined && typeof props.onPointerDownOutside !== "function")
      throw new TypeError(`${kind} onPointerDownOutside must be a function`);
    if (props.draggable !== undefined) {
      const draggable = props.draggable;
      if (
        draggable === null ||
        typeof draggable !== "object" ||
        typeof draggable.type !== "string" ||
        draggable.type.length === 0 ||
        [...draggable.type].length > 128 ||
        /[\u0000-\u001f\u007f]/.test(draggable.type)
      )
        throw new TypeError(`${kind} draggable.type must be a non-empty safe string`);
    }
    if (props.onDragOver !== undefined && typeof props.onDragOver !== "function")
      throw new TypeError(`${kind} onDragOver must be a function`);
    if (props.onDrop !== undefined && typeof props.onDrop !== "function")
      throw new TypeError(`${kind} onDrop must be a function`);
    if (props.onExternalFileDrop !== undefined && typeof props.onExternalFileDrop !== "function")
      throw new TypeError(`${kind} onExternalFileDrop must be a function`);
    if (props.focusable !== undefined && typeof props.focusable !== "boolean")
      throw new TypeError("Pressable focusable must be a boolean");
    if (props.onKeyDown !== undefined && typeof props.onKeyDown !== "function")
      throw new TypeError("Pressable onKeyDown must be a function");
    if (props.onKeyDown !== undefined && props.focusable !== true)
      throw new TypeError("Pressable onKeyDown requires focusable=true");
    if (props.disabled !== undefined && typeof props.disabled !== "boolean")
      throw new TypeError("Pressable disabled must be a boolean");
  }
  if (kind === "Image") {
    imageFor({ kind, hostProperties: null } as HostNodeInternal, props);
  }
  if (kind === "Icon") iconFor({ kind, hostProperties: null } as HostNodeInternal, props);
  if (kind === "TextInput") {
    const input = props as TextInputProps;
    for (const [name, value] of [
      ["value", input.value],
      ["defaultValue", input.defaultValue],
      ["placeholder", input.placeholder],
    ] as const) {
      if (value !== undefined && (typeof value !== "string" || utf8ByteLength(value) > MAX_CLIPBOARD_TEXT_BYTES))
        throw new TypeError(`TextInput ${name} must be a string of at most ${MAX_CLIPBOARD_TEXT_BYTES} UTF-8 bytes`);
    }
    if (
      input.maxLength !== undefined &&
      (!Number.isInteger(input.maxLength) || input.maxLength < 0 || input.maxLength > 0xffff_ffff)
    )
      throw new TypeError("TextInput maxLength must be a non-negative u32");
  }
  if (kind === "View") {
    if (props.focusable !== undefined && typeof props.focusable !== "boolean")
      throw new TypeError("View focusable must be a boolean");
    if (props.onKeyDown !== undefined && typeof props.onKeyDown !== "function")
      throw new TypeError("View onKeyDown must be a function");
    if (props.onKeyDown !== undefined && props.focusable !== true)
      throw new TypeError("View onKeyDown requires focusable=true");
    if (props.onScroll !== undefined && typeof props.onScroll !== "function")
      throw new TypeError("View onScroll must be a function");
  }
  if (kind === "VirtualList") {
    virtualListFor({ kind, hostProperties: null } as HostNodeInternal, props);
    if (props.__onVisibleRange !== undefined && typeof props.__onVisibleRange !== "function")
      throw new TypeError("VirtualList range callback must be a function");
  }
  accessibilityFor(kind, props);
}
