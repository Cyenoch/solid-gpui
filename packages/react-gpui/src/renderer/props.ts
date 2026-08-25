import { utf8ByteLength } from "../protocol";
import { validateStyle } from "../style";
import type { HostPropertiesWire } from "../protocol";

import type {
  AccessibilityProps,
  AccessibilityWire,
  HostKind,
  HostNodeInternal,
  HostProps,
  ImageObjectFit,
  ImageWire,
  TextInputProps,
  TextInputWire,
  VirtualListWire,
  DragWire,
} from "./types";

export const KIND_CODES: Record<HostKind, 1 | 2 | 3 | 4 | 5 | 6 | 7> = {
  View: 1,
  Text: 2,
  Pressable: 3,
  TextInput: 5,
  RawText: 4,
  VirtualList: 6,
  Image: 7,
};
export const VALID_HOST_TYPES: Record<string, true> = {
  View: true,
  Text: true,
  Pressable: true,
  TextInput: true,
  VirtualList: true,
  Image: true,
};
const ACCESSIBILITY_PROPS: Record<string, true> = {
  accessibilityRole: true,
  accessibilityLabel: true,
  accessibilityDescription: true,
  accessibilityDisabled: true,
  accessibilityChecked: true,
  accessibilitySelected: true,
  accessibilityValue: true,
};
const ALLOWED_PROPS: Record<HostKind, Record<string, true>> = {
  View: {
    style: true,
    onLayout: true,
    draggable: true,
    onDragOver: true,
    onDrop: true,
    onExternalFileDrop: true,
    focusable: true,
    onKeyDown: true,
    onPointerDown: true,
    onPointerUp: true,
    onHoverChange: true,
    onScroll: true,
    children: true,
    ref: true,
    ...ACCESSIBILITY_PROPS,
  },
  Text: { style: true, onLayout: true, children: true, ref: true, ...ACCESSIBILITY_PROPS },
  Pressable: {
    style: true,
    onLayout: true,
    onPress: true,
    disabled: true,
    draggable: true,
    onDragOver: true,
    onDrop: true,
    onExternalFileDrop: true,
    focusable: true,
    onKeyDown: true,
    onPointerDown: true,
    onPointerUp: true,
    onHoverChange: true,
    children: true,
    ref: true,
    ...ACCESSIBILITY_PROPS,
  },
  TextInput: {
    style: true,
    children: true,
    ref: true,
    value: true,
    defaultValue: true,
    placeholder: true,
    onChangeText: true,
    onSelectionChange: true,
    onFocus: true,
    onSubmitEditing: true,
    onKeyDown: true,
    multiline: true,
    disabled: true,
    maxLength: true,
    ...ACCESSIBILITY_PROPS,
  },
  VirtualList: {
    style: true,
    children: true,
    ref: true,
    __itemCount: true,
    __rangeStart: true,
    __rangeEnd: true,
    __estimatedItemSize: true,
    __overscan: true,
    __onVisibleRange: true,
    __onAnimationComplete: true,
    ...ACCESSIBILITY_PROPS,
  },
  Image: { style: true, source: true, objectFit: true, onLayout: true, ref: true, ...ACCESSIBILITY_PROPS },
  RawText: { children: true, ref: true },
};
const ROLE_CODES: Record<NonNullable<AccessibilityProps["accessibilityRole"]>, number> = {
  generic: 1,
  button: 2,
  text: 3,
  textbox: 4,
  checkbox: 5,
  heading: 6,
};

export function accessibilityFor(kind: HostKind, props: HostProps): AccessibilityWire | null {
  const role =
    props.accessibilityRole ?? (kind === "TextInput" ? "textbox" : kind === "Pressable" ? "button" : undefined);
  const checked = props.accessibilityChecked;
  const input = props as TextInputProps;
  if (checked !== undefined && role !== "checkbox")
    throw new TypeError("accessibilityChecked requires accessibilityRole=checkbox");
  if (role === undefined && Object.keys(props).every((key) => !ACCESSIBILITY_PROPS[key])) return null;
  return {
    role: role === undefined ? 0 : ROLE_CODES[role],
    label: props.accessibilityLabel ?? null,
    description: props.accessibilityDescription ?? null,
    disabled: props.accessibilityDisabled ?? input.disabled ?? false,
    checked: checked ?? null,
    selected: props.accessibilitySelected ?? null,
    value: props.accessibilityValue ?? null,
  };
}
export function inputFor(node: HostNodeInternal, props: HostProps): TextInputWire | null {
  if (node.kind !== "TextInput") return null;
  const input = props as TextInputProps;
  const previous = node.hostProperties;
  const previousInput = previous && "value" in previous ? previous : null;
  const maxLength = input.maxLength ?? null;
  const value = truncateUtf16(
    input.value ?? node.latestNativeText ?? previousInput?.value ?? input.defaultValue ?? "",
    maxLength,
  );
  return {
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
    maxLength,
  };
}
export function imageFor(node: HostNodeInternal, props: HostProps): ImageWire | null {
  if (node.kind !== "Image") return null;
  const source = props.source;
  if (
    typeof source !== "string" ||
    source.length === 0 ||
    utf8ByteLength(source) > 1024 ||
    /[\u0000-\u001f\u007f]/.test(source)
  )
    throw new TypeError("Image source must be a non-empty path of at most 1024 UTF-8 bytes");
  const objectFit = props.objectFit ?? "contain";
  if (!["fill", "contain", "cover", "scaleDown", "none"].includes(objectFit))
    throw new TypeError("Image objectFit is invalid");
  const objectFitCode =
    objectFit === "fill"
      ? 1
      : objectFit === "contain"
        ? 2
        : objectFit === "cover"
          ? 3
          : objectFit === "scaleDown"
            ? 4
            : 5;
  return { source, objectFit: objectFitCode };
}
export function virtualListFor(node: HostNodeInternal, props: HostProps): VirtualListWire | null {
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
    !Number.isFinite(estimatedItemSize)
  ) {
    throw new RangeError("VirtualList host properties are invalid");
  }
  assertU32Option("VirtualList itemCount", itemCount);
  assertU32Option("VirtualList rangeStart", rangeStart);
  assertU32Option("VirtualList rangeEnd", rangeEnd);
  assertU32Option("VirtualList overscan", overscan);
  return { itemCount, rangeStart, rangeEnd, estimatedItemSize, overscan };
}
export function dragFor(node: HostNodeInternal, props: HostProps): DragWire | null {
  if (node.kind === "Pressable" && props.disabled === true) return null;
  if (
    props.draggable === undefined &&
    props.onDragOver === undefined &&
    props.onDrop === undefined &&
    props.onExternalFileDrop === undefined
  )
    return null;
  return { dragType: props.draggable?.type ?? null };
}

export function hostPropertiesWire(
  value: TextInputWire | VirtualListWire | ImageWire | DragWire | null,
): HostPropertiesWire | null {
  if (value === null) return null;
  if ("value" in value)
    return [
      1,
      value.value,
      value.placeholder,
      value.multiline,
      value.disabled,
      value.controlled,
      value.ackEditSeq,
      value.selectionStart,
      value.selectionEnd,
      value.markedStart,
      value.markedEnd,
      value.maxLength,
    ];
  if ("source" in value) return [3, value.source, value.objectFit];
  if ("dragType" in value) return [4, value.dragType];
  return [2, value.itemCount, value.rangeStart, value.rangeEnd, value.estimatedItemSize, value.overscan];
}
export function accessibilityWire(value: AccessibilityWire | null): readonly unknown[] | null {
  return value === null
    ? null
    : [value.role, value.label, value.description, value.disabled, value.checked, value.selected, value.value];
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
  for (const key of Object.keys(props)) {
    if (!ALLOWED_PROPS[kind][key]) throw new TypeError(`Unsupported ${kind} prop: ${key}`);
  }
  if (
    (kind === "View" || kind === "Text" || kind === "Pressable" || kind === "Image") &&
    props.onLayout !== undefined &&
    typeof props.onLayout !== "function"
  )
    throw new TypeError(`${kind} onLayout must be a function`);
  if (kind !== "RawText") validateStyle(props.style);
  if (kind === "Pressable" && props.onPress !== undefined && typeof props.onPress !== "function") {
    throw new TypeError("Pressable onPress must be a function");
  }
  if (kind === "View" || kind === "Pressable") {
    if (props.onPointerDown !== undefined && typeof props.onPointerDown !== "function")
      throw new TypeError(`${kind} onPointerDown must be a function`);
    if (props.onPointerUp !== undefined && typeof props.onPointerUp !== "function")
      throw new TypeError(`${kind} onPointerUp must be a function`);
    if (props.onHoverChange !== undefined && typeof props.onHoverChange !== "function")
      throw new TypeError(`${kind} onHoverChange must be a function`);
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
  }
  if (kind === "Pressable") {
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
  if (kind === "TextInput") {
    if (props.onSubmitEditing !== undefined && typeof props.onSubmitEditing !== "function")
      throw new TypeError("TextInput onSubmitEditing must be a function");
    if (props.onKeyDown !== undefined && typeof props.onKeyDown !== "function")
      throw new TypeError("TextInput onKeyDown must be a function");
    if (
      props.maxLength !== undefined &&
      (!Number.isInteger(props.maxLength) || props.maxLength < 0 || props.maxLength > 0xffff_ffff)
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
}
