export type ThemeMode = "dark" | "light";

export interface ThemeColors {
  readonly bgApp: string;
  readonly bgSidebar: string;
  readonly bgHeader: string;
  readonly bgCard: string;
  readonly bgCardHover: string;
  readonly bgHover: string;
  readonly bgMuted: string;
  readonly bgActive: string;
  readonly bgOverlay: string;
  readonly bgInput: string;
  readonly bgCode: string;
  readonly codeBg: string;
  readonly codeText: string;

  readonly border: string;
  readonly borderMuted: string;
  readonly borderFocus: string;

  readonly textPrimary: string;
  readonly textSecondary: string;
  readonly textMuted: string;
  readonly textInverse: string;

  readonly accent: string;
  readonly accentHover: string;
  readonly accentMuted: string;
  readonly accentText: string;

  readonly primary: string;
  readonly primaryHover: string;
  readonly primaryText: string;

  readonly success: string;
  readonly successMuted: string;
  readonly successText: string;

  readonly warning: string;
  readonly warningMuted: string;
  readonly warningText: string;

  readonly danger: string;
  readonly dangerMuted: string;
  readonly dangerText: string;

  readonly info: string;
  readonly infoMuted: string;
  readonly infoText: string;
}

export const DARK_THEME: ThemeColors = {
  bgApp: "#191A1C",
  bgSidebar: "#1D1E20",
  bgHeader: "#202123",
  bgCard: "#232527",
  bgCardHover: "#2C2E31",
  bgHover: "#2C2E31",
  bgMuted: "#252729",
  bgActive: "#34373A",
  bgOverlay: "#191A1CDD",
  bgInput: "#1A1C1E",
  bgCode: "#161719",
  codeBg: "#161719",
  codeText: "#D5DBD8",

  border: "#393C3F",
  borderMuted: "#2C2E31",
  borderFocus: "#8DCAB1",

  textPrimary: "#EDEDEB",
  textSecondary: "#B1B4B6",
  textMuted: "#858B8E",
  textInverse: "#191A1C",

  accent: "#8DCAB1",
  accentHover: "#A8D9C4",
  accentMuted: "#293E36",
  accentText: "#16251E",

  primary: "#8DCAB1",
  primaryHover: "#A8D9C4",
  primaryText: "#16251E",

  success: "#10B981",
  successMuted: "#064E3B44",
  successText: "#34D399",

  warning: "#F59E0B",
  warningMuted: "#78350F44",
  warningText: "#FBBF24",

  danger: "#EF4444",
  dangerMuted: "#7F1D1D44",
  dangerText: "#F87171",

  info: "#06B6D4",
  infoMuted: "#164E6344",
  infoText: "#22D3EE",
};

export const LIGHT_THEME: ThemeColors = {
  bgApp: "#F4F4F1",
  bgSidebar: "#EBECE8",
  bgHeader: "#FAFAF8",
  bgCard: "#FFFFFF",
  bgCardHover: "#F3F4F0",
  bgHover: "#E7E9E3",
  bgMuted: "#F0F1EC",
  bgActive: "#DFE4DB",
  bgOverlay: "#F4F4F1DD",
  bgInput: "#FFFFFF",
  bgCode: "#202523",
  codeBg: "#202523",
  codeText: "#E2EAE5",

  border: "#D5D9D0",
  borderMuted: "#E1E4DC",
  borderFocus: "#326F56",

  textPrimary: "#252A26",
  textSecondary: "#5D655F",
  textMuted: "#737C74",
  textInverse: "#FFFFFF",

  accent: "#326F56",
  accentHover: "#245640",
  accentMuted: "#DFEEE5",
  accentText: "#FFFFFF",

  primary: "#326F56",
  primaryHover: "#245640",
  primaryText: "#FFFFFF",

  success: "#059669",
  successMuted: "#D1FAE5",
  successText: "#065F46",

  warning: "#D97706",
  warningMuted: "#FEF3C7",
  warningText: "#92400E",

  danger: "#DC2626",
  dangerMuted: "#FEE2E2",
  dangerText: "#991B1B",

  info: "#0891B2",
  infoMuted: "#CFFAFE",
  infoText: "#155E75",
};

export function getTheme(mode: ThemeMode): ThemeColors {
  return mode === "dark" ? DARK_THEME : LIGHT_THEME;
}

// Fixed swatches need a foreground derived from their own background, not app mode.
export function contrastingText(hex: string): "#000000" | "#FFFFFF" {
  const channels = [1, 3, 5].map((start) => {
    const value = Number.parseInt(hex.slice(start, start + 2), 16) / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  const luminance = channels[0]! * 0.2126 + channels[1]! * 0.7152 + channels[2]! * 0.0722;
  return (luminance + 0.05) / 0.05 >= 1.05 / (luminance + 0.05) ? "#000000" : "#FFFFFF";
}
