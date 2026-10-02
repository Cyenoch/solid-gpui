import type { Style as WireStyle } from "./generated/protocol";
import type { Style } from "../style";

const ENUMS = {
  flexDirection: { 1: "row", 2: "column", 3: "row-reverse", 4: "column-reverse" },
  justifyContent: {
    1: "flex-start",
    2: "center",
    3: "flex-end",
    4: "space-between",
    5: "space-around",
    6: "space-evenly",
  },
  alignItems: { 1: "flex-start", 2: "center", 3: "flex-end", 4: "stretch", 5: "baseline" },
  fontWeight: { 400: "normal", 500: "medium", 600: "semibold", 700: "bold", 900: "heavy" },
  overflow: { 1: "visible", 2: "hidden", 3: "scroll" },
  textOverflow: { 1: "clip", 2: "ellipsis" },
  fontStyle: { 0: "normal", 1: "italic" },
  textDecoration: { 0: "none", 1: "underline", 2: "lineThrough" },
  alignSelf: { 1: "start", 2: "end", 3: "flex-start", 4: "flex-end", 5: "center", 6: "baseline", 7: "stretch" },
  position: { 0: "relative", 1: "absolute", 2: "overlay" },
  cursor: Object.fromEntries(
    [
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
    ].map((name, index) => [index, name]),
  ),
  textAlign: { 1: "left", 2: "center", 3: "right" },
  flexWrap: { 0: "nowrap", 1: "wrap", 2: "wrap-reverse" },
} satisfies Partial<Record<keyof Style, Readonly<Record<number, string>>>>;
type EnumKey = keyof typeof ENUMS;
const enumTables: Readonly<Record<string, Readonly<Record<number, string>>>> = ENUMS;
const color = (value: number) => `#${value.toString(16).padStart(8, "0")}`;

const CODES = Object.fromEntries(
  Object.entries(ENUMS).map(([key, values]) => [
    key,
    Object.fromEntries(Object.entries(values).map(([code, name]) => [name, Number(code)])),
  ]),
);

export function styleEnum<K extends EnumKey>(key: K, value: Style[K]): number | undefined {
  return value === undefined ? undefined : CODES[key]![value];
}

/** Normalize committed wire values into the application's semantic style vocabulary. */
export function committedStyle(wire: WireStyle | undefined): Readonly<Style> | null {
  if (!wire) return null;
  const style: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(wire)) {
    if (key.endsWith("Unit")) continue;
    if (typeof value !== "number" && typeof value !== "string") continue;
    style[key] = enumTables[key] ? enumTables[key][Number(value)] : /color$/i.test(key) ? color(Number(value)) : value;
  }
  for (const key of ["width", "height", "minWidth", "maxWidth", "minHeight", "maxHeight"] as const) {
    if (wire[key] !== undefined) style[key] = committedLength(wire[key]!, wire[`${key}Unit`]!);
  }
  if (wire.flexBasis) style.flexBasis = committedLength(wire.flexBasis.value!, wire.flexBasis.unit!);
  for (const key of ["overflowX", "overflowY"] as const) {
    if (wire[key] !== undefined) style[key] = ENUMS.overflow[wire[key]! as 1 | 2 | 3];
  }
  for (const key of ["hover", "active", "focusVisible"] as const) {
    const value = wire[key];
    if (value)
      style[key] = Object.freeze(
        Object.fromEntries(
          Object.entries(value)
            .filter(([, v]) => v !== undefined)
            .map(([k, v]) => [k, k === "opacity" ? v : color(Number(v))]),
        ),
      );
  }
  if (wire.transition) {
    const value = wire.transition;
    style.transition = Object.freeze({
      durationMs: value.durationMs,
      delayMs: value.delayMs,
      easing: ["linear", "easeIn", "easeOut", "easeInOut"][value.easing!],
      properties: Object.freeze(
        ["opacity", "backgroundColor", "width", "height"].filter(
          (_, index) => (value.propertyMask! & (1 << index)) !== 0,
        ),
      ),
    });
  }
  if (wire.boxShadow) {
    const shadows = wire.boxShadow.values!.map((value) => Object.freeze({ ...value, color: color(value.color!) }));
    style.boxShadow = shadows.length === 1 ? shadows[0] : Object.freeze(shadows);
  }
  if (wire.linearGradient) {
    const value = wire.linearGradient;
    style.linearGradient = Object.freeze({
      angle: value.angle,
      stops: Object.freeze([
        Object.freeze({ color: color(value.startColor!), position: value.startPosition }),
        Object.freeze({ color: color(value.endColor!), position: value.endPosition }),
      ]),
    });
  }
  return Object.freeze(style) as Readonly<Style>;
}
function committedLength(value: number, unit: number): import("../style").Length {
  switch (unit) {
    case 0:
      return value;
    case 1:
      return Object.freeze({ unit: "rem", value });
    case 2:
      return Object.freeze({ unit: "percent", value });
    case 3:
      return "auto";
    default:
      throw new TypeError("Committed dimension requires a supported native unit");
  }
}
