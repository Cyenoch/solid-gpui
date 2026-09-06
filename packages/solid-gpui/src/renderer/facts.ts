import type { AccessibilityProps, HostKind, HostProps, TextInputProps } from "./types";

export const VALID_HOST_TYPES: Record<string, true> = {
  View: true,
  Text: true,
  Pressable: true,
  TextInput: true,
  VirtualList: true,
  Image: true,
  Extension: true,
  Icon: true,
};
export const ROLE_CODES: Record<NonNullable<AccessibilityProps["accessibilityRole"]>, number> = {
  generic: 1,
  button: 2,
  text: 3,
  textbox: 4,
  checkbox: 5,
  heading: 6,
  link: 7,
};

const PROP_GROUP_STYLE = 1;
const PROP_GROUP_ACCESSIBILITY = 2;
const PROP_GROUP_INTERACTION = 4;
const PROP_GROUP_HOST_PROPERTIES = 8;
const PROP_GROUP_TOOLTIP = 16;

const ACCESSIBILITY_PROPS: Record<string, true> = {
  accessibilityRole: true,
  accessibilityLabel: true,
  accessibilityDescription: true,
  accessibilityDisabled: true,
  accessibilityChecked: true,
  accessibilitySelected: true,
  accessibilityValue: true,
  accessibilityExpanded: true,
  accessibilityLevel: true,
};

const BASE_PROPS = {
  style: true,
  children: true,
  ref: true,
  ...ACCESSIBILITY_PROPS,
} as const;

export interface HostKindSpec {
  readonly allowedProps: Readonly<Record<string, true>>;
  readonly defaultAccessibilityRole?: NonNullable<AccessibilityProps["accessibilityRole"]>;
  readonly projector: "none" | "input" | "virtualList" | "image" | "drag" | "icon";
  readonly allowsLayout: boolean;
}

export const HOST_KIND_SPECS: Record<HostKind, HostKindSpec> = {
  View: {
    allowedProps: {
      ...BASE_PROPS,
      tooltip: true,
      onLayout: true,
      draggable: true,
      onDragOver: true,
      onDrop: true,
      onExternalFileDrop: true,
      focusable: true,
      onKeyDown: true,
      onPointerDown: true,
      onPointerUp: true,
      onPointerMove: true,
      onHoverChange: true,
      onScroll: true,
      onFocus: true,
      onBlur: true,
      onPointerDownOutside: true,
    },
    projector: "drag",
    allowsLayout: true,
  },
  Text: {
    allowedProps: {
      ...BASE_PROPS,
      selectable: true,
      onPress: true,
      onFocus: true,
      onBlur: true,
      onLayout: true,
    },
    projector: "none",
    allowsLayout: true,
  },
  Pressable: {
    allowedProps: {
      ...BASE_PROPS,
      tooltip: true,
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
      onPointerMove: true,
      onHoverChange: true,
      onFocus: true,
      onBlur: true,
    },
    defaultAccessibilityRole: "button",
    projector: "drag",
    allowsLayout: true,
  },
  TextInput: {
    allowedProps: {
      ...BASE_PROPS,
      value: true,
      defaultValue: true,
      placeholder: true,
      onChangeText: true,
      onSelectionChange: true,
      onFocus: true,
      onBlur: true,
      onSubmitEditing: true,
      onKeyDown: true,
      multiline: true,
      disabled: true,
      maxLength: true,
    },
    defaultAccessibilityRole: "textbox",
    projector: "input",
    allowsLayout: false,
  },
  VirtualList: {
    allowedProps: {
      ...BASE_PROPS,
      __itemCount: true,
      __rangeStart: true,
      __rangeEnd: true,
      __estimatedItemSize: true,
      __overscan: true,
      __onVisibleRange: true,
      __onAnimationComplete: true,
    },
    projector: "virtualList",
    allowsLayout: false,
  },
  Image: {
    allowedProps: {
      ...BASE_PROPS,
      source: true,
      fallbackSource: true,
      objectFit: true,
      onLayout: true,
    },
    projector: "image",
    allowsLayout: true,
  },
  Icon: {
    allowedProps: {
      ...BASE_PROPS,
      name: true,
      size: true,
      color: true,
      onLayout: true,
    },
    projector: "icon",
    allowsLayout: true,
  },
  RawText: {
    allowedProps: {
      children: true,
      ref: true,
    },
    projector: "none",
    allowsLayout: false,
  },
  Extension: {
    allowedProps: {
      ...BASE_PROPS,
      onExtensionEvent: true,
      onLayout: true,
      __extensionDescriptor: true,
    },
    projector: "none",
    allowsLayout: true,
  },
};

export function propGroup(kind: HostKind, name: string): number {
  if (name === "style") return PROP_GROUP_STYLE;
  if (name.startsWith("accessibility")) return PROP_GROUP_ACCESSIBILITY;
  if (name === "onExtensionEvent") return PROP_GROUP_INTERACTION;
  if (kind === "Extension") {
    if (name === "children" || name === "ref" || name === "onLayout") return PROP_GROUP_INTERACTION;
    return PROP_GROUP_HOST_PROPERTIES;
  }
  if (name === "tooltip") return PROP_GROUP_TOOLTIP;
  if (
    name === "onPress" ||
    name === "focusable" ||
    name === "selectable" ||
    name === "disabled" ||
    name === "onKeyDown" ||
    name === "onPointerDown" ||
    name === "onPointerUp" ||
    name === "onPointerMove" ||
    name === "onHoverChange" ||
    name === "onFocus" ||
    name === "onBlur" ||
    name === "onPointerDownOutside" ||
    name === "onScroll" ||
    name === "onLayout" ||
    name === "draggable" ||
    name === "onDragOver" ||
    name === "onDrop" ||
    name === "onExternalFileDrop"
  )
    return PROP_GROUP_INTERACTION;
  if (kind === "TextInput" || kind === "VirtualList" || kind === "Image" || kind === "Icon")
    return PROP_GROUP_HOST_PROPERTIES;
  return PROP_GROUP_INTERACTION;
}

export function isAccessibilityProp(name: string): boolean {
  return Object.hasOwn(ACCESSIBILITY_PROPS, name);
}

export function hostKindSpec(kind: HostKind): HostKindSpec {
  return HOST_KIND_SPECS[kind];
}

export function hasAllowedProp(kind: HostKind, name: string): boolean {
  return Object.hasOwn(HOST_KIND_SPECS[kind].allowedProps, name);
}

export function defaultAccessibilityRole(
  kind: HostKind,
): NonNullable<AccessibilityProps["accessibilityRole"]> | undefined {
  return HOST_KIND_SPECS[kind].defaultAccessibilityRole;
}

export function assertChildKind(parentKind: HostKind | null, childKind: HostKind, textDepth = 0): void {
  if (parentKind === null) return;
  if (parentKind === "Text" && childKind !== "RawText" && childKind !== "Text") {
    throw new TypeError("Text children must be raw text or one-level nested Text runs");
  }
  if (parentKind === "Text" && childKind === "Text" && textDepth > 1) {
    throw new TypeError("Nested Text may contain only raw text; deeper Text nesting is unsupported");
  }
  if (parentKind === "Image" || parentKind === "Icon")
    throw new TypeError(`${parentKind} nodes cannot contain children`);
  if (childKind === "RawText" && parentKind !== "Text") {
    throw new TypeError("Raw text is only valid directly under Text");
  }
}

export function validateHostKindValues(kind: HostKind, props: HostProps): void {
  if (kind === "Pressable" && props.onPress !== undefined && typeof props.onPress !== "function")
    throw new TypeError("Pressable onPress must be a function");
  if (kind !== "TextInput") return;
  const input = props as TextInputProps;
  for (const [name, callback] of [
    ["onChangeText", input.onChangeText],
    ["onSelectionChange", input.onSelectionChange],
    ["onFocus", input.onFocus],
    ["onBlur", input.onBlur],
    ["onSubmitEditing", input.onSubmitEditing],
    ["onKeyDown", input.onKeyDown],
  ] as const) {
    if (callback !== undefined && typeof callback !== "function")
      throw new TypeError(`TextInput ${name} must be a function`);
  }
  for (const [name, value] of [
    ["value", input.value],
    ["defaultValue", input.defaultValue],
    ["placeholder", input.placeholder],
  ] as const) {
    if (value !== undefined && typeof value !== "string") throw new TypeError(`TextInput ${name} must be a string`);
  }
  if (input.multiline !== undefined && typeof input.multiline !== "boolean")
    throw new TypeError("TextInput multiline must be a boolean");
  if (input.disabled !== undefined && typeof input.disabled !== "boolean")
    throw new TypeError("TextInput disabled must be a boolean");
}
