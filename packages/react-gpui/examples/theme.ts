import type { Appearance } from "../src/index";

/**
 * Shared semantic colors for the examples. Palette policy stays in the
 * application: each root selects a theme from its appearance store.
 */
export const lightTheme = {
  canvas: "#f7f8fa",
  surface: "#ffffff",
  surfaceRaised: "#fbfcfe",
  surfaceSubtle: "#f1f3f6",
  surfaceMuted: "#e7ebf0",
  border: "#d8e0ea",
  borderSubtle: "#e2e7ee",
  borderInput: "#b9c6d6",
  text: "#172033",
  textMuted: "#5b6b7f",
  textSubtle: "#64748b",
  accent: "#2d6cdf",
  accentText: "#2458b8",
  accentHover: "#2458b8",
  accentPressed: "#1e4a9b",
  accentSoft: "#eaf1ff",
  accentSoftHover: "#f4f7fb",
  accentSoftPressed: "#dbe8ff",
  focusRing: "#2d6cdf",
  success: "#0f8a5f",
  successSoft: "#eaf6f1",
  danger: "#dc2626",
  dangerSoft: "#fee2e2",
  disabled: "#e7ebf0",
  disabledText: "#64748b",
  input: "#ffffff",
  inputFocus: "#eaf1ff",
  onAccent: "#ffffff",
  shadow: "#17203318",
  shadowStrong: "#17203324",
  shadowSoft: "#17203310",
} as const;

export const darkTheme = {
  canvas: "#0f172a",
  surface: "#1e293b",
  surfaceRaised: "#243447",
  surfaceSubtle: "#273449",
  surfaceMuted: "#334155",
  border: "#475569",
  borderSubtle: "#3b4a60",
  borderInput: "#64748b",
  text: "#f8fafc",
  textMuted: "#cbd5e1",
  textSubtle: "#94a3b8",
  accent: "#2563eb",
  accentText: "#93c5fd",
  accentHover: "#3b82f6",
  accentPressed: "#1d4ed8",
  accentSoft: "#1e3a5f",
  accentSoftHover: "#254a73",
  accentSoftPressed: "#2d5f90",
  focusRing: "#93c5fd",
  success: "#4ade80",
  successSoft: "#14532d",
  danger: "#f87171",
  dangerSoft: "#4c1d1d",
  disabled: "#334155",
  disabledText: "#94a3b8",
  input: "#0f172a",
  inputFocus: "#172554",
  onAccent: "#ffffff",
  shadow: "#00000040",
  shadowStrong: "#00000066",
  shadowSoft: "#00000026",
} as const;

export type Theme = typeof lightTheme | typeof darkTheme;

/** Pick the example palette for the current root appearance. */
export function useTheme(appearance: Appearance): Theme {
  return appearance === "dark" ? darkTheme : lightTheme;
}
