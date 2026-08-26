export type Position = "relative" | "absolute";
export type FlexDirection = "row" | "column" | "row-reverse" | "column-reverse";
export type CursorStyle =
  | "default"
  | "text"
  | "pointer"
  | "grab"
  | "grabbing"
  | "not-allowed"
  | "context-menu"
  | "crosshair"
  | "vertical-text"
  | "alias"
  | "copy"
  | "no-drop"
  | "move"
  | "ew-resize"
  | "ns-resize"
  | "nesw-resize"
  | "nwse-resize"
  | "col-resize"
  | "row-resize";
export type TextAlign = "left" | "center" | "right";
export type JustifyContent = "flex-start" | "center" | "flex-end" | "space-between" | "space-around" | "space-evenly";
export type AlignItems = "flex-start" | "center" | "flex-end" | "stretch" | "baseline";
export type FontWeight = "normal" | "medium" | "semibold" | "bold" | "heavy";
export type Overflow = "visible" | "hidden" | "scroll";
export type FontStyle = "normal" | "italic";
export interface BoxShadow {
  readonly offsetX: number;
  readonly offsetY: number;
  readonly blurRadius: number;
  readonly spreadRadius: number;
  readonly color: string;
  readonly inset?: boolean;
}

export type BoxShadowInput = BoxShadow | readonly [BoxShadow, BoxShadow];
export type TextDecoration = "none" | "underline" | "lineThrough";
export type AlignSelf = "start" | "end" | "flex-start" | "flex-end" | "center" | "baseline" | "stretch";
export type TransitionEasing = "linear" | "easeIn" | "easeOut" | "easeInOut";
export type TransitionProperty = "opacity" | "backgroundColor" | "width" | "height";

export interface Transition {
  readonly durationMs: number;
  readonly delayMs?: number;
  readonly easing?: TransitionEasing;
  readonly properties?: readonly TransitionProperty[];
  readonly onComplete?: (generation: number) => void;
}

export interface Style {
  readonly width?: number;
  readonly height?: number;
  readonly flexDirection?: FlexDirection;
  readonly flexGrow?: number;
  readonly padding?: number;
  readonly gap?: number;
  readonly justifyContent?: JustifyContent;
  readonly alignItems?: AlignItems;
  readonly borderRadius?: number;
  readonly borderWidth?: number;
  readonly borderColor?: string;
  readonly fontSize?: number;
  readonly fontWeight?: FontWeight;
  readonly overflow?: Overflow;
  readonly lineClamp?: number;
  readonly textOverflow?: "clip" | "ellipsis";
  readonly marginTop?: number;
  readonly marginRight?: number;
  readonly marginBottom?: number;
  readonly marginLeft?: number;
  readonly fontStyle?: FontStyle;
  readonly textDecoration?: TextDecoration;
  readonly lineHeight?: number;
  readonly minWidth?: number;
  readonly maxWidth?: number;
  readonly minHeight?: number;
  readonly maxHeight?: number;
  readonly flexShrink?: number;
  readonly alignSelf?: AlignSelf;
  readonly position?: Position;
  readonly left?: number;
  readonly top?: number;
  readonly right?: number;
  readonly bottom?: number;
  readonly cursor?: CursorStyle;
  readonly textAlign?: TextAlign;
  readonly backgroundColor?: string;
  readonly color?: string;
  readonly opacity?: number;
  readonly transition?: Transition;
  readonly boxShadow?: BoxShadowInput;
  readonly fontFamily?: string;
}

export type StyleProp = Style | null | undefined;

export type EncodedTransition = readonly [number, number, 0 | 1 | 2 | 3, number];
export type EncodedBoxShadowValue = readonly [number, number, number, number, number, 0 | 1];
export type EncodedBoxShadow =
  | readonly [1, EncodedBoxShadowValue]
  | readonly [2, readonly [EncodedBoxShadowValue, EncodedBoxShadowValue]];
export type EncodedStyle = readonly [
  number | null,
  number | null,
  0 | 1 | 2 | 3 | 4,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  EncodedTransition | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  0 | 1,
  number | null,
  number | null,
  number | null,
  number | null,
  0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18,
  0 | 1 | 2 | 3,
  EncodedBoxShadow | null,
  string | null,
];

const COLOR_PATTERN = /^#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?$/;
const STYLE_KEYS: Record<string, true> = {
  width: true,
  height: true,
  flexDirection: true,
  flexGrow: true,
  padding: true,
  gap: true,
  justifyContent: true,
  alignItems: true,
  borderRadius: true,
  borderWidth: true,
  borderColor: true,
  fontSize: true,
  fontWeight: true,
  overflow: true,
  lineClamp: true,
  textOverflow: true,
  marginTop: true,
  marginRight: true,
  marginBottom: true,
  marginLeft: true,
  fontStyle: true,
  textDecoration: true,
  lineHeight: true,
  minWidth: true,
  maxWidth: true,
  minHeight: true,
  maxHeight: true,
  flexShrink: true,
  alignSelf: true,
  position: true,
  boxShadow: true,
  fontFamily: true,
  left: true,
  top: true,
  textAlign: true,
  right: true,
  cursor: true,
  bottom: true,
  backgroundColor: true,
  color: true,
  opacity: true,
  transition: true,
};

function assertNumber(name: string, value: unknown, nonNegative: boolean): asserts value is number {
  if (typeof value !== "number" || !Number.isFinite(value) || (nonNegative && value < 0)) {
    throw new TypeError(`${name} must be a finite${nonNegative ? " non-negative" : ""} number`);
  }
}
const BOX_SHADOW_KEYS: Record<string, true> = {
  offsetX: true,
  offsetY: true,
  blurRadius: true,
  spreadRadius: true,
  color: true,
  inset: true,
};

function assertF32Number(name: string, value: unknown, nonNegative: boolean): asserts value is number {
  assertNumber(name, value, nonNegative);
  if (!Number.isFinite(Math.fround(value))) {
    throw new TypeError(`${name} must be representable as f32`);
  }
}

function validateBoxShadow(value: unknown, index: number): BoxShadow {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`boxShadow[${index}] must be an object`);
  }
  for (const key of Object.keys(value)) {
    if (!BOX_SHADOW_KEYS[key]) throw new TypeError(`Unsupported boxShadow field: ${key}`);
  }
  const shadow = value as BoxShadow;
  assertF32Number(`boxShadow[${index}].offsetX`, shadow.offsetX, false);
  assertF32Number(`boxShadow[${index}].offsetY`, shadow.offsetY, false);
  assertF32Number(`boxShadow[${index}].blurRadius`, shadow.blurRadius, true);
  assertF32Number(`boxShadow[${index}].spreadRadius`, shadow.spreadRadius, true);
  if (typeof shadow.color !== "string" || !COLOR_PATTERN.test(shadow.color)) {
    throw new TypeError(`boxShadow[${index}].color must be #RRGGBB or #RRGGBBAA`);
  }
  if (shadow.inset !== undefined && typeof shadow.inset !== "boolean") {
    throw new TypeError(`boxShadow[${index}].inset must be a boolean`);
  }
  return shadow;
}

function validateBoxShadowInput(value: unknown): BoxShadowInput {
  if (Array.isArray(value)) {
    if (value.length !== 2) throw new TypeError("boxShadow must contain one or two shadows");
    validateBoxShadow(value[0], 0);
    validateBoxShadow(value[1], 1);
    return value as unknown as readonly [BoxShadow, BoxShadow];
  }
  return validateBoxShadow(value, 0);
}

function validateFontFamily(value: unknown): asserts value is string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    [...value].length > 64 ||
    [...value].some((character) => /\p{Cc}/u.test(character))
  ) {
    throw new TypeError("fontFamily must be a non-empty string of at most 64 characters");
  }
}

export function validateStyle(value: StyleProp): Style | null | undefined {
  if (value == null) return value;
  if (typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError("style must be an object, null, or undefined");
  }
  for (const key of Object.keys(value)) {
    if (!STYLE_KEYS[key]) throw new TypeError(`Unsupported style field: ${key}`);
  }
  const style = value as Style;
  if (style.width !== undefined) assertNumber("width", style.width, true);
  if (style.height !== undefined) assertNumber("height", style.height, true);
  if (style.position !== undefined && style.position !== "relative" && style.position !== "absolute")
    throw new TypeError("position must be relative or absolute");
  for (const [name, offset] of [
    ["left", style.left],
    ["top", style.top],
    ["right", style.right],
    ["bottom", style.bottom],
  ] as const) {
    if (offset !== undefined) assertNumber(name, offset, false);
  }
  if (style.flexGrow !== undefined) assertNumber("flexGrow", style.flexGrow, true);
  if (style.padding !== undefined) assertNumber("padding", style.padding, true);
  if (style.gap !== undefined) assertNumber("gap", style.gap, true);
  for (const [name, amount] of [
    ["marginTop", style.marginTop],
    ["marginRight", style.marginRight],
    ["marginBottom", style.marginBottom],
    ["marginLeft", style.marginLeft],
    ["lineHeight", style.lineHeight],
    ["minWidth", style.minWidth],
    ["maxWidth", style.maxWidth],
    ["minHeight", style.minHeight],
    ["maxHeight", style.maxHeight],
    ["flexShrink", style.flexShrink],
  ] as const) {
    if (amount !== undefined) assertNumber(name, amount, true);
  }
  if (
    style.justifyContent !== undefined &&
    !["flex-start", "center", "flex-end", "space-between", "space-around", "space-evenly"].includes(
      style.justifyContent,
    )
  ) {
    throw new TypeError("justifyContent is invalid");
  }
  if (
    style.alignItems !== undefined &&
    !["flex-start", "center", "flex-end", "stretch", "baseline"].includes(style.alignItems)
  ) {
    throw new TypeError("alignItems is invalid");
  }
  if (style.borderRadius !== undefined) assertNumber("borderRadius", style.borderRadius, true);
  if (style.borderWidth !== undefined) assertNumber("borderWidth", style.borderWidth, true);
  if (style.fontSize !== undefined) {
    assertNumber("fontSize", style.fontSize, false);
    if (style.fontSize <= 0) throw new TypeError("fontSize must be positive");
  }
  if (style.fontWeight !== undefined && !["normal", "medium", "semibold", "bold", "heavy"].includes(style.fontWeight)) {
    throw new TypeError("fontWeight is invalid");
  }
  if (style.overflow !== undefined && !["visible", "hidden", "scroll"].includes(style.overflow)) {
    throw new TypeError("overflow is invalid");
  }
  if (style.lineClamp !== undefined) {
    assertNumber("lineClamp", style.lineClamp, true);
    if (!Number.isInteger(style.lineClamp) || style.lineClamp < 1 || style.lineClamp > 100)
      throw new TypeError("lineClamp must be an integer between 1 and 100");
  }
  if (style.textOverflow !== undefined && style.textOverflow !== "clip" && style.textOverflow !== "ellipsis") {
    throw new TypeError("textOverflow is invalid");
  }
  if (style.fontStyle !== undefined && style.fontStyle !== "normal" && style.fontStyle !== "italic") {
    throw new TypeError("fontStyle is invalid");
  }
  if (
    style.textDecoration !== undefined &&
    style.textDecoration !== "none" &&
    style.textDecoration !== "underline" &&
    style.textDecoration !== "lineThrough"
  ) {
    throw new TypeError("textDecoration is invalid");
  }
  if (
    style.alignSelf !== undefined &&
    !["start", "end", "flex-start", "flex-end", "center", "baseline", "stretch"].includes(style.alignSelf)
  ) {
    throw new TypeError("alignSelf is invalid");
  }
  if (
    style.cursor !== undefined &&
    ![
      "default",
      "text",
      "pointer",
      "grab",
      "grabbing",
      "not-allowed",
      "context-menu",
      "crosshair",
      "vertical-text",
      "alias",
      "copy",
      "no-drop",
      "move",
      "ew-resize",
      "ns-resize",
      "nesw-resize",
      "nwse-resize",
      "col-resize",
      "row-resize",
    ].includes(style.cursor)
  ) {
    throw new TypeError("cursor is invalid");
  }
  if (style.opacity !== undefined) {
    assertNumber("opacity", style.opacity, true);
    if (style.opacity > 1) throw new TypeError("opacity must be between 0 and 1");
  }
  if (
    style.flexDirection !== undefined &&
    !["row", "column", "row-reverse", "column-reverse"].includes(style.flexDirection)
  ) {
    throw new TypeError("flexDirection is invalid");
  }
  if (style.textAlign !== undefined && !["left", "center", "right"].includes(style.textAlign)) {
    throw new TypeError("textAlign is invalid");
  }
  for (const key of ["borderColor", "backgroundColor", "color"] as const) {
    const color = style[key];
    if (color !== undefined && (typeof color !== "string" || !COLOR_PATTERN.test(color))) {
      throw new TypeError(`${key} must be #RRGGBB or #RRGGBBAA`);
    }
  }
  if (style.transition !== undefined) {
    const transition = style.transition;
    if (transition === null || typeof transition !== "object" || Array.isArray(transition))
      throw new TypeError("transition must be an object");
    assertNumber("transition.durationMs", transition.durationMs, true);
    if (!Number.isInteger(transition.durationMs) || transition.durationMs > 0xffff_ffff)
      throw new TypeError("transition.durationMs must be a u32");
    if (transition.delayMs !== undefined) {
      assertNumber("transition.delayMs", transition.delayMs, true);
      if (!Number.isInteger(transition.delayMs) || transition.delayMs > 0xffff_ffff)
        throw new TypeError("transition.delayMs must be a u32");
    }
    if (
      transition.easing !== undefined &&
      transition.easing !== "linear" &&
      transition.easing !== "easeIn" &&
      transition.easing !== "easeOut" &&
      transition.easing !== "easeInOut"
    ) {
      throw new TypeError("transition.easing is invalid");
    }
    if (transition.onComplete !== undefined && typeof transition.onComplete !== "function")
      throw new TypeError("transition.onComplete must be a function");
    if (transition.properties !== undefined) {
      if (!Array.isArray(transition.properties) || transition.properties.length === 0)
        throw new TypeError("transition.properties must not be empty");
      const seen = new Set<string>();
      for (const property of transition.properties) {
        if (property !== "opacity" && property !== "backgroundColor" && property !== "width" && property !== "height")
          throw new TypeError("transition.properties contains an unsupported property");
        if (seen.has(property)) throw new TypeError("transition.properties contains duplicates");
        seen.add(property);
      }
    }
  }
  if (style.boxShadow !== undefined) validateBoxShadowInput(style.boxShadow);
  if (style.fontFamily !== undefined) validateFontFamily(style.fontFamily);
  return style;
}

function encodeCursor(cursor: CursorStyle | undefined): number | null {
  switch (cursor) {
    case undefined:
    case "default":
      return null;
    case "text":
      return 1;
    case "pointer":
      return 2;
    case "grab":
      return 3;
    case "grabbing":
      return 4;
    case "not-allowed":
      return 5;
    case "context-menu":
      return 6;
    case "crosshair":
      return 7;
    case "vertical-text":
      return 8;
    case "alias":
      return 9;
    case "copy":
      return 10;
    case "no-drop":
      return 11;
    case "move":
      return 12;
    case "ew-resize":
      return 13;
    case "ns-resize":
      return 14;
    case "nesw-resize":
      return 15;
    case "nwse-resize":
      return 16;
    case "col-resize":
      return 17;
    case "row-resize":
      return 18;
  }
}

export function encodeColor(color: string): number {
  if (!COLOR_PATTERN.test(color)) throw new TypeError("color must be #RRGGBB or #RRGGBBAA");
  const hex = color.slice(1);
  const alpha = hex.length === 6 ? "ff" : hex.slice(6);
  return Number.parseInt(`${hex.slice(0, 6)}${alpha}`, 16) >>> 0;
}
function encodeBoxShadow(input: BoxShadowInput): EncodedBoxShadow {
  const encode = (shadow: BoxShadow): EncodedBoxShadowValue => [
    Math.fround(shadow.offsetX),
    Math.fround(shadow.offsetY),
    Math.fround(shadow.blurRadius),
    Math.fround(shadow.spreadRadius),
    encodeColor(shadow.color),
    shadow.inset === true ? 1 : 0,
  ];
  if (Array.isArray(input)) {
    const shadows = input as readonly [BoxShadow, BoxShadow];
    return [2, [encode(shadows[0]), encode(shadows[1])]];
  }
  return [1, encode(input as BoxShadow)];
}

const ENCODED_STYLE_CACHE = new WeakMap<object, EncodedStyle>();

export function encodeStyle(style: StyleProp): EncodedStyle | null {
  if (style == null) return null;
  const cached = ENCODED_STYLE_CACHE.get(style);
  if (cached !== undefined) return cached;
  validateStyle(style);
  const transition = style.transition;
  const properties = transition?.properties;
  let propertyMask = 0;
  if (properties === undefined || properties.includes("opacity")) propertyMask |= 1;
  if (properties === undefined || properties.includes("backgroundColor")) propertyMask |= 2;
  if (properties === undefined || properties.includes("width")) propertyMask |= 4;
  if (properties === undefined || properties.includes("height")) propertyMask |= 8;
  const easing =
    transition?.easing === "linear"
      ? 0
      : transition?.easing === "easeIn"
        ? 1
        : transition?.easing === "easeOut"
          ? 2
          : 3;
  const justifyContent =
    style.justifyContent === undefined
      ? null
      : style.justifyContent === "flex-start"
        ? 1
        : style.justifyContent === "center"
          ? 2
          : style.justifyContent === "flex-end"
            ? 3
            : style.justifyContent === "space-between"
              ? 4
              : style.justifyContent === "space-around"
                ? 5
                : 6;
  const alignItems =
    style.alignItems === undefined
      ? null
      : style.alignItems === "flex-start"
        ? 1
        : style.alignItems === "center"
          ? 2
          : style.alignItems === "flex-end"
            ? 3
            : style.alignItems === "stretch"
              ? 4
              : 5;
  const fontWeight =
    style.fontWeight === undefined
      ? null
      : style.fontWeight === "normal"
        ? 400
        : style.fontWeight === "medium"
          ? 500
          : style.fontWeight === "semibold"
            ? 600
            : style.fontWeight === "bold"
              ? 700
              : 900;
  const fontStyle = style.fontStyle === undefined ? null : style.fontStyle === "italic" ? 1 : 0;
  const textDecoration =
    style.textDecoration === undefined
      ? null
      : style.textDecoration === "none"
        ? 0
        : style.textDecoration === "underline"
          ? 1
          : 2;
  const alignSelf =
    style.alignSelf === undefined
      ? null
      : style.alignSelf === "start"
        ? 1
        : style.alignSelf === "end"
          ? 2
          : style.alignSelf === "flex-start"
            ? 3
            : style.alignSelf === "flex-end"
              ? 4
              : style.alignSelf === "center"
                ? 5
                : style.alignSelf === "baseline"
                  ? 6
                  : 7;
  const encoded = Object.freeze([
    style.width ?? null,
    style.height ?? null,
    style.flexDirection === undefined
      ? 0
      : style.flexDirection === "row"
        ? 1
        : style.flexDirection === "column"
          ? 2
          : style.flexDirection === "row-reverse"
            ? 3
            : 4,
    style.flexGrow ?? null,
    style.padding ?? null,
    style.gap ?? null,
    style.backgroundColor === undefined ? null : encodeColor(style.backgroundColor),
    style.color === undefined ? null : encodeColor(style.color),
    style.opacity ?? null,
    transition === undefined ? null : [transition.durationMs, transition.delayMs ?? 0, easing, propertyMask],
    justifyContent,
    alignItems,
    style.borderRadius ?? null,
    style.borderWidth ?? null,
    style.borderColor === undefined ? null : encodeColor(style.borderColor),
    style.fontSize ?? null,
    fontWeight,
    style.overflow === undefined ? null : style.overflow === "visible" ? 1 : style.overflow === "hidden" ? 2 : 3,
    style.lineClamp ?? null,
    style.textOverflow === undefined ? null : style.textOverflow === "clip" ? 1 : 2,
    style.marginTop ?? null,
    style.marginRight ?? null,
    style.marginBottom ?? null,
    style.marginLeft ?? null,
    fontStyle,
    textDecoration,
    style.lineHeight ?? null,
    style.minWidth ?? null,
    style.maxWidth ?? null,
    style.minHeight ?? null,
    style.maxHeight ?? null,
    style.flexShrink ?? null,
    alignSelf,
    style.position === undefined ? 0 : style.position === "relative" ? 0 : 1,
    style.left ?? null,
    style.top ?? null,
    style.right ?? null,
    style.bottom ?? null,
    encodeCursor(style.cursor),
    style.textAlign === undefined ? 0 : style.textAlign === "left" ? 1 : style.textAlign === "center" ? 2 : 3,
    style.boxShadow === undefined ? null : encodeBoxShadow(style.boxShadow),
    style.fontFamily ?? null,
  ]) as EncodedStyle;
  ENCODED_STYLE_CACHE.set(style, encoded);
  return encoded;
}

export type NamedStyles<T extends Record<string, Style>> = {
  readonly [K in keyof T]: T[K];
};

export function createStyleSheet<T extends Record<string, Style>>(recipes: T): NamedStyles<T> {
  if (recipes == null || typeof recipes !== "object" || Array.isArray(recipes)) {
    throw new TypeError("StyleSheet.create expects an object of style recipes");
  }
  const result: Record<string, Style> = {};
  for (const [name, recipe] of Object.entries(recipes)) {
    validateStyle(recipe);
    result[name] = Object.freeze({ ...recipe });
  }
  return Object.freeze(result) as NamedStyles<T>;
}
