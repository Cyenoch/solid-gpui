import { Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import { Button, Card, CodeSnippet, DenseRow, Divider, ResponsiveRow, SectionHeader } from "../components/ui";

export function StylingShowcase(): SolidChild {
  const { theme } = useGallery();

  const [shadowBlur, setShadowBlur] = createSignal(12);
  const [shadowSpread, setShadowSpread] = createSignal(2);
  const [shadowOffsetY, setShadowOffsetY] = createSignal(6);
  const [borderRadius, setBorderRadius] = createSignal(12);
  const [opacity, setOpacity] = createSignal(1);

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Layout & Styling"
        tag="Visual Design"
        description="Declarative style properties including multi-layer box shadows, rounded borders, opacity, colors, and cached StyleSheet objects."
      />

      {/* Box Shadows Interactive Stage */}
      <Card
        title="Interactive Box Shadow Builder"
        description="Adjust blur radius, spread radius, and vertical offset to inspect GPU-accelerated shadow rendering."
      >
        {/* Controls row */}
        <DenseRow
          gap={12}
          style={{
            backgroundColor: theme().bgHover,
            padding: 12,
            borderRadius: 6,
            alignItems: "center",
          }}
        >
          {/* Blur */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Blur:</Text>
            <Button size="sm" variant={shadowBlur() === 4 ? "primary" : "secondary"} onPress={() => setShadowBlur(4)}>
              4px
            </Button>
            <Button size="sm" variant={shadowBlur() === 12 ? "primary" : "secondary"} onPress={() => setShadowBlur(12)}>
              12px
            </Button>
            <Button size="sm" variant={shadowBlur() === 24 ? "primary" : "secondary"} onPress={() => setShadowBlur(24)}>
              24px
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Spread */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Spread:</Text>
            <Button
              size="sm"
              variant={shadowSpread() === 0 ? "primary" : "secondary"}
              onPress={() => setShadowSpread(0)}
            >
              0px
            </Button>
            <Button
              size="sm"
              variant={shadowSpread() === 4 ? "primary" : "secondary"}
              onPress={() => setShadowSpread(4)}
            >
              4px
            </Button>
            <Button
              size="sm"
              variant={shadowSpread() === 8 ? "primary" : "secondary"}
              onPress={() => setShadowSpread(8)}
            >
              8px
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Offset Y */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Offset Y:</Text>
            <Button
              size="sm"
              variant={shadowOffsetY() === 2 ? "primary" : "secondary"}
              onPress={() => setShadowOffsetY(2)}
            >
              2px
            </Button>
            <Button
              size="sm"
              variant={shadowOffsetY() === 8 ? "primary" : "secondary"}
              onPress={() => setShadowOffsetY(8)}
            >
              8px
            </Button>
            <Button
              size="sm"
              variant={shadowOffsetY() === 16 ? "primary" : "secondary"}
              onPress={() => setShadowOffsetY(16)}
            >
              16px
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Border Radius */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Radius:</Text>
            <Button
              size="sm"
              variant={borderRadius() === 4 ? "primary" : "secondary"}
              onPress={() => setBorderRadius(4)}
            >
              4px
            </Button>
            <Button
              size="sm"
              variant={borderRadius() === 12 ? "primary" : "secondary"}
              onPress={() => setBorderRadius(12)}
            >
              12px
            </Button>
            <Button
              size="sm"
              variant={borderRadius() === 24 ? "primary" : "secondary"}
              onPress={() => setBorderRadius(24)}
            >
              24px
            </Button>
          </View>
        </DenseRow>

        {/* Live Elevated Preview Box */}
        <View
          style={{
            backgroundColor: theme().bgActive,
            borderRadius: 8,
            padding: 40,
            minWidth: 0,
            flexShrink: 1,
            overflow: "scroll",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <View
            style={{
              alignSelf: "stretch",
              maxWidth: 280,
              minWidth: 0,
              flexShrink: 0,
              padding: 24,
              backgroundColor: theme().bgCard,
              borderRadius: borderRadius(),
              borderWidth: 1,
              borderColor: theme().border,
              opacity: opacity(),
              boxShadow: {
                color: "#00000044",
                offsetX: 0,
                offsetY: shadowOffsetY(),
                blurRadius: shadowBlur(),
                spreadRadius: shadowSpread(),
              },
              alignItems: "center" as const,
              gap: 8,
            }}
          >
            <Text style={{ color: theme().textPrimary, fontSize: 16, fontWeight: "bold" }}>Elevated Surface</Text>
            <Text style={{ color: theme().textSecondary, fontSize: 12, textAlign: "center" }}>
              {`blur: ${shadowBlur()}px • spread: ${shadowSpread()}px • offset: ${shadowOffsetY()}px`}
            </Text>
          </View>
        </View>
      </Card>

      {/* Elevation Hierarchy */}
      <Card
        title="Elevation Hierarchy"
        description="Predefined shadow presets for multi-tier visual elevation hierarchy."
      >
        {(() => {
          const elevations = [
            { level: "Flat (Elevation 0)", shadow: undefined, desc: "Surface with 1px border" },
            {
              level: "Low (Elevation 1)",
              shadow: { color: "#00000022", offsetX: 0, offsetY: 2, blurRadius: 4, spreadRadius: 0 },
              desc: "Cards and active buttons",
            },
            {
              level: "Medium (Elevation 2)",
              shadow: { color: "#00000033", offsetX: 0, offsetY: 6, blurRadius: 12, spreadRadius: 1 },
              desc: "Dropdowns and toolbars",
            },
            {
              level: "High (Elevation 3)",
              shadow: { color: "#00000055", offsetX: 0, offsetY: 12, blurRadius: 24, spreadRadius: 2 },
              desc: "Dialogs and modals",
            },
          ];

          return (
            <ResponsiveRow gap={16}>
              {elevations.map((elv) => (
                <View
                  style={{
                    flexGrow: 1,
                    minWidth: 0,
                    flexShrink: 1,
                    padding: 16,
                    backgroundColor: theme().bgCard,
                    borderRadius: 8,
                    borderWidth: 1,
                    borderColor: theme().border,
                    boxShadow: elv.shadow,
                    gap: 6,
                  }}
                >
                  <Text
                    style={{
                      color: theme().textPrimary,
                      fontSize: 13,
                      fontWeight: "bold",
                      minWidth: 0,
                      flexShrink: 1,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {elv.level}
                  </Text>
                  <Text style={{ color: theme().textSecondary, fontSize: 11, minWidth: 0, flexShrink: 1 }}>
                    {elv.desc}
                  </Text>
                </View>
              ))}
            </ResponsiveRow>
          );
        })()}
      </Card>

      {/* Code Example */}
      <Card title="Code Example: Box Shadow & StyleSheet">
        <CodeSnippet
          code={`import { StyleSheet, View } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

const styles = StyleSheet.create({
  card: {
    backgroundColor: "#1E293B",
    borderRadius: 12,
    borderWidth: 1,
    borderColor: "#334155",
    padding: 20,
    boxShadow: {
      color: "#00000044",
      offsetX: 0,
      offsetY: 8,
      blurRadius: 16,
      spreadRadius: 1,
    },
  },
});`}
        />
      </Card>
    </View>
  );
}
