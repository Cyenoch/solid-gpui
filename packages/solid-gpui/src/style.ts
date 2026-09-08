export type Position = "relative" | "absolute" | "overlay";
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

export interface LinearGradient {
  /** Degrees clockwise from up: 180 paints from top to bottom. */
  readonly angle: number;
  readonly stops: readonly [
    { readonly color: string; readonly position: number },
    { readonly color: string; readonly position: number },
  ];
}

export interface Style {
  readonly borderTopColor?: string;
  readonly borderRightColor?: string;
  readonly borderBottomColor?: string;
  readonly borderLeftColor?: string;

  readonly linearGradient?: LinearGradient;
  readonly paddingTop?: number;
  readonly paddingRight?: number;
  readonly paddingBottom?: number;
  readonly paddingLeft?: number;
  readonly borderTopWidth?: number;
  readonly borderRightWidth?: number;
  readonly borderBottomWidth?: number;
  readonly borderLeftWidth?: number;
  readonly borderTopLeftRadius?: number;
  readonly borderTopRightRadius?: number;
  readonly borderBottomRightRadius?: number;
  readonly borderBottomLeftRadius?: number;
  readonly widthPercent?: number;
  readonly heightPercent?: number;
  readonly flexWrap?: "nowrap" | "wrap" | "wrap-reverse";
  /** Equal native grid tracks and item spans, each from 1 through 64. */
  readonly gridColumns?: number;
  readonly gridRows?: number;
  readonly gridColumnSpan?: number;
  readonly gridRowSpan?: number;

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

const COLOR_PATTERN = /^#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?$/;
const STYLE_KEYS: Record<string, true> = {
  borderTopColor: true,
  borderRightColor: true,
  borderBottomColor: true,
  borderLeftColor: true,

  linearGradient: true,
  paddingTop: true,
  paddingRight: true,
  paddingBottom: true,
  paddingLeft: true,
  borderTopWidth: true,
  borderRightWidth: true,
  borderBottomWidth: true,
  borderLeftWidth: true,
  borderTopLeftRadius: true,
  borderTopRightRadius: true,
  borderBottomRightRadius: true,
  borderBottomLeftRadius: true,
  widthPercent: true,
  heightPercent: true,
  flexWrap: true,
  gridColumns: true,
  gridRows: true,
  gridColumnSpan: true,
  gridRowSpan: true,

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
  if (!Number.isFinite(Math.fround(value))) throw new TypeError(`${name} must be representable as float32`);
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
  if (style.linearGradient !== undefined) {
    const gradient = style.linearGradient;
    if (
      !gradient ||
      typeof gradient !== "object" ||
      Object.keys(gradient).some((key) => key !== "angle" && key !== "stops")
    )
      throw new TypeError("linearGradient is invalid");
    assertNumber("linearGradient.angle", gradient.angle, true);
    if (gradient.angle > 360 || !Array.isArray(gradient.stops) || gradient.stops.length !== 2)
      throw new TypeError("linearGradient requires an angle in [0, 360] and exactly two stops");
    for (const stop of gradient.stops) {
      if (
        !stop ||
        typeof stop !== "object" ||
        Object.keys(stop).some((key) => key !== "color" && key !== "position") ||
        typeof stop.color !== "string" ||
        !COLOR_PATTERN.test(stop.color)
      )
        throw new TypeError("linearGradient stop is invalid");
      assertNumber("linearGradient stop position", stop.position, true);
      if (stop.position > 1) throw new TypeError("linearGradient stop position must be in [0, 1]");
    }
    if (gradient.stops[0].position >= gradient.stops[1].position)
      throw new TypeError("linearGradient stops must be strictly increasing");
  }
  if (style.paddingTop !== undefined) assertNumber("paddingTop", style.paddingTop, true);
  if (style.paddingRight !== undefined) assertNumber("paddingRight", style.paddingRight, true);
  if (style.paddingBottom !== undefined) assertNumber("paddingBottom", style.paddingBottom, true);
  if (style.paddingLeft !== undefined) assertNumber("paddingLeft", style.paddingLeft, true);
  if (style.borderTopWidth !== undefined) assertNumber("borderTopWidth", style.borderTopWidth, true);
  if (style.borderRightWidth !== undefined) assertNumber("borderRightWidth", style.borderRightWidth, true);
  if (style.borderBottomWidth !== undefined) assertNumber("borderBottomWidth", style.borderBottomWidth, true);
  if (style.borderLeftWidth !== undefined) assertNumber("borderLeftWidth", style.borderLeftWidth, true);
  if (style.borderTopLeftRadius !== undefined) assertNumber("borderTopLeftRadius", style.borderTopLeftRadius, true);
  if (style.borderTopRightRadius !== undefined) assertNumber("borderTopRightRadius", style.borderTopRightRadius, true);
  if (style.borderBottomRightRadius !== undefined)
    assertNumber("borderBottomRightRadius", style.borderBottomRightRadius, true);
  if (style.borderBottomLeftRadius !== undefined)
    assertNumber("borderBottomLeftRadius", style.borderBottomLeftRadius, true);
  if (style.widthPercent !== undefined) assertNumber("widthPercent", style.widthPercent, true);
  if (style.heightPercent !== undefined) assertNumber("heightPercent", style.heightPercent, true);
  if (style.width !== undefined && style.widthPercent !== undefined)
    throw new TypeError("width and widthPercent are mutually exclusive");
  if (style.height !== undefined && style.heightPercent !== undefined)
    throw new TypeError("height and heightPercent are mutually exclusive");
  if (style.flexWrap !== undefined && !["nowrap", "wrap", "wrap-reverse"].includes(style.flexWrap))
    throw new TypeError("flexWrap is invalid");

  if (style.width !== undefined) assertNumber("width", style.width, true);
  if (style.height !== undefined) assertNumber("height", style.height, true);
  if (
    style.position !== undefined &&
    style.position !== "relative" &&
    style.position !== "absolute" &&
    style.position !== "overlay"
  )
    throw new TypeError("position must be relative, absolute, or overlay");
  if (style.position === "overlay" && (style.right !== undefined || style.bottom !== undefined)) {
    throw new TypeError("overlay position supports left and top offsets only");
  }
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
  for (const key of ["gridColumns", "gridRows", "gridColumnSpan", "gridRowSpan"] as const) {
    const value = style[key];
    if (value !== undefined && (!Number.isInteger(value) || value < 1 || value > 64)) {
      throw new TypeError(`${key} must be an integer from 1 through 64`);
    }
  }
  if ((style.gridColumns !== undefined || style.gridRows !== undefined) && style.flexDirection !== undefined) {
    throw new TypeError("grid tracks and flexDirection cannot be combined");
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
  for (const key of [
    "borderColor",
    "backgroundColor",
    "color",
    "borderTopColor",
    "borderRightColor",
    "borderBottomColor",
    "borderLeftColor",
  ] as const) {
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
