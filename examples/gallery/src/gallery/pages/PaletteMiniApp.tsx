import { Pressable, Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import type { SolidChild } from "../types";
import { contrastingText } from "../theme";
import { useGallery } from "../context";
import { Button, Card, DenseRow, Divider, ResponsiveRow, SectionHeader } from "../components/ui";

interface ColorSwatchItem {
  readonly name: string;
  readonly hex: string;
}

export function PaletteMiniApp(): SolidChild {
  const { theme, showStatus } = useGallery();
  const [selectedHex, setSelectedHex] = createSignal("#3B82F6");

  const palettes: { readonly name: string; readonly colors: readonly ColorSwatchItem[] }[] = [
    {
      name: "Blue (Primary)",
      colors: [
        { name: "50", hex: "#EFF6FF" },
        { name: "100", hex: "#DBEAFE" },
        { name: "300", hex: "#93C5FD" },
        { name: "500", hex: "#3B82F6" },
        { name: "600", hex: "#2563EB" },
        { name: "800", hex: "#1E40AF" },
        { name: "900", hex: "#1E3A8A" },
      ],
    },
    {
      name: "Emerald (Success)",
      colors: [
        { name: "50", hex: "#ECFDF5" },
        { name: "100", hex: "#D1FAE5" },
        { name: "300", hex: "#6EE7B7" },
        { name: "500", hex: "#10B981" },
        { name: "600", hex: "#059669" },
        { name: "800", hex: "#065F46" },
        { name: "900", hex: "#064E3B" },
      ],
    },
    {
      name: "Violet (Accent)",
      colors: [
        { name: "50", hex: "#F5F3FF" },
        { name: "100", hex: "#EDE9FE" },
        { name: "300", hex: "#C4B5FD" },
        { name: "500", hex: "#8B5CF6" },
        { name: "600", hex: "#7C3AED" },
        { name: "800", hex: "#5B21B6" },
        { name: "900", hex: "#4C1D95" },
      ],
    },
    {
      name: "Amber (Warning)",
      colors: [
        { name: "50", hex: "#FFFBEB" },
        { name: "100", hex: "#FEF3C7" },
        { name: "300", hex: "#FCD34D" },
        { name: "500", hex: "#F59E0B" },
        { name: "600", hex: "#D97706" },
        { name: "800", hex: "#92400E" },
        { name: "900", hex: "#78350F" },
      ],
    },
    {
      name: "Rose (Danger)",
      colors: [
        { name: "50", hex: "#FFF1F2" },
        { name: "100", hex: "#FFE4E6" },
        { name: "300", hex: "#FDA4AF" },
        { name: "500", hex: "#F43F5E" },
        { name: "600", hex: "#E11D48" },
        { name: "800", hex: "#9F1239" },
        { name: "900", hex: "#881337" },
      ],
    },
    {
      name: "Slate (Neutral)",
      colors: [
        { name: "50", hex: "#F8FAFC" },
        { name: "100", hex: "#F1F5F9" },
        { name: "300", hex: "#CBD5E1" },
        { name: "500", hex: "#64748B" },
        { name: "700", hex: "#334155" },
        { name: "800", hex: "#1E293B" },
        { name: "900", hex: "#0F172A" },
      ],
    },
  ];

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Color Palette Studio"
        description="Design-system color tokens and palette preview. Click a color chip to copy its hexadecimal value."
        tag="Design System"
      />

      {/* Selected Color Showcase */}
      <ResponsiveRow
        gap={16}
        grow={[1, 0]}
        style={{
          backgroundColor: selectedHex(),
          borderRadius: 10,
          padding: 24,
          flexDirection: "row",
          justifyContent: "space-between",
          alignItems: "center",
          minWidth: 0,
          flexShrink: 0,
          gap: 16,
        }}
      >
        <View style={{ gap: 6, minWidth: 0, flexShrink: 1 }}>
          <Text style={{ color: contrastingText(selectedHex()), fontSize: 12, fontWeight: "bold", opacity: 0.8 }}>
            SELECTED COLOR
          </Text>
          <Text
            style={{
              color: contrastingText(selectedHex()),
              fontSize: 30,
              fontWeight: "bold",
              minWidth: 0,
              flexShrink: 1,
            }}
          >
            {selectedHex()}
          </Text>
          <Text style={{ color: contrastingText(selectedHex()), fontSize: 14, fontFamily: "monospace" }}>
            {selectedHex()}
          </Text>
        </View>
        <Button variant="secondary" onPress={() => showStatus(`Copied ${selectedHex()} to clipboard`, "success")}>
          Copy Hex
        </Button>
      </ResponsiveRow>

      {/* Palettes Grid */}
      <Card title="Design Tokens & Scales" description="Core design-system color scales.">
        <View style={{ gap: 16 }}>
          {palettes.map((pal) => (
            <View style={{ gap: 8 }}>
              <Text style={{ color: theme().textPrimary, fontSize: 13, fontWeight: "semibold" }}>{pal.name}</Text>
              <DenseRow gap={8}>
                {pal.colors.map((color) => (
                  <Pressable
                    onPress={() => {
                      setSelectedHex(color.hex);
                      showStatus(`Selected ${color.hex}`, "info");
                    }}
                    style={{
                      minWidth: 80,
                      flexGrow: 1,
                      flexShrink: 0,
                      height: 64,
                      backgroundColor: color.hex,
                      borderRadius: 6,
                      padding: 10,
                      justifyContent: "flex-end",
                      cursor: "pointer",
                      borderWidth: selectedHex() === color.hex ? 3 : 0,
                      borderColor: theme().accent,
                    }}
                  >
                    <Text
                      style={{
                        color: contrastingText(color.hex),
                        fontSize: 11,
                        fontWeight: "bold",
                      }}
                    >
                      {color.name}
                    </Text>
                    <Text
                      style={{
                        color: contrastingText(color.hex),
                        fontSize: 10,
                        fontFamily: "monospace",
                        opacity: 0.9,
                      }}
                    >
                      {color.hex}
                    </Text>
                  </Pressable>
                ))}
              </DenseRow>
            </View>
          ))}
        </View>
      </Card>
    </View>
  );
}
