export type FlexDirection = "row" | "column";
export type TransitionEasing = "linear" | "easeIn" | "easeOut" | "easeInOut";
export type TransitionProperty = "opacity" | "backgroundColor";

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
  readonly backgroundColor?: string;
  readonly color?: string;
  readonly opacity?: number;
  readonly transition?: Transition;
}

export type StyleProp = Style | null | undefined;

export type EncodedTransition = readonly [number, number, 0 | 1 | 2 | 3, number];
export type EncodedStyle = readonly [
  number | null,
  number | null,
  0 | 1 | 2,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  number | null,
  EncodedTransition | null,
];

const COLOR_PATTERN = /^#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?$/;
const STYLE_KEYS: Record<string, true> = {
  width: true,
  height: true,
  flexDirection: true,
  flexGrow: true,
  padding: true,
  gap: true,
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
  if (style.flexGrow !== undefined) assertNumber("flexGrow", style.flexGrow, true);
  if (style.padding !== undefined) assertNumber("padding", style.padding, true);
  if (style.gap !== undefined) assertNumber("gap", style.gap, true);
  if (style.opacity !== undefined) {
    assertNumber("opacity", style.opacity, true);
    if (style.opacity > 1) throw new TypeError("opacity must be between 0 and 1");
  }
  if (style.flexDirection !== undefined && style.flexDirection !== "row" && style.flexDirection !== "column") {
    throw new TypeError("flexDirection must be row or column");
  }
  for (const key of ["backgroundColor", "color"] as const) {
    const color = style[key];
    if (color !== undefined && (typeof color !== "string" || !COLOR_PATTERN.test(color))) {
      throw new TypeError(`${key} must be #RRGGBB or #RRGGBBAA`);
    }
  }
  if (style.transition !== undefined) {
    const transition = style.transition;
    if (transition === null || typeof transition !== "object" || Array.isArray(transition)) throw new TypeError("transition must be an object");
    assertNumber("transition.durationMs", transition.durationMs, true);
    if (!Number.isInteger(transition.durationMs) || transition.durationMs > 0xffff_ffff) throw new TypeError("transition.durationMs must be a u32");
    if (transition.delayMs !== undefined) {
      assertNumber("transition.delayMs", transition.delayMs, true);
      if (!Number.isInteger(transition.delayMs) || transition.delayMs > 0xffff_ffff) throw new TypeError("transition.delayMs must be a u32");
    }
    if (transition.easing !== undefined && transition.easing !== "linear" && transition.easing !== "easeIn" && transition.easing !== "easeOut" && transition.easing !== "easeInOut") {
      throw new TypeError("transition.easing is invalid");
    }
    if (transition.onComplete !== undefined && typeof transition.onComplete !== "function") throw new TypeError("transition.onComplete must be a function");
    if (transition.properties !== undefined) {
      if (!Array.isArray(transition.properties) || transition.properties.length === 0) throw new TypeError("transition.properties must not be empty");
      const seen = new Set<string>();
      for (const property of transition.properties) {
        if (property !== "opacity" && property !== "backgroundColor") throw new TypeError("transition.properties contains an unsupported property");
        if (seen.has(property)) throw new TypeError("transition.properties contains duplicates");
        seen.add(property);
      }
    }
  }
  return style;
}

export function encodeColor(color: string): number {
  if (!COLOR_PATTERN.test(color)) throw new TypeError("color must be #RRGGBB or #RRGGBBAA");
  const hex = color.slice(1);
  const alpha = hex.length === 6 ? "ff" : hex.slice(6);
  return Number.parseInt(`${hex.slice(0, 6)}${alpha}`, 16) >>> 0;
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
  const easing = transition?.easing === "linear" ? 0 : transition?.easing === "easeIn" ? 1 : transition?.easing === "easeOut" ? 2 : 3;
  const encoded = Object.freeze([
    style.width ?? null,
    style.height ?? null,
    style.flexDirection === undefined ? 0 : style.flexDirection === "row" ? 1 : 2,
    style.flexGrow ?? null,
    style.padding ?? null,
    style.gap ?? null,
    style.backgroundColor === undefined ? null : encodeColor(style.backgroundColor),
    style.color === undefined ? null : encodeColor(style.color),
    style.opacity ?? null,
    transition === undefined ? null : [transition.durationMs, transition.delayMs ?? 0, easing, propertyMask],
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
