import { Text, View } from "@solid-gpui/core";
import type { AlignItems, FlexDirection, JustifyContent } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { contrastingText } from "../theme";
import { useGallery } from "../context";
import { Badge, Button, Card, DenseRow, Divider, ResponsiveRow, SectionHeader } from "../components/ui";
import type { SolidChild } from "../types";

export function FlexboxShowcase(): SolidChild {
  const { theme } = useGallery();

  const [direction, setDirection] = createSignal<FlexDirection>("row");
  const [justify, setJustify] = createSignal<JustifyContent>("space-between");
  const [align, setAlign] = createSignal<AlignItems>("center");
  const [gap, setGap] = createSignal(12);
  const [boxCount, setBoxCount] = createSignal(4);

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Flexbox Layout Engine"
        description="Solid GPUI uses Taffy for native flexbox layout. Direction, justification, alignment, gap, grow, and shrink are supported."
        tag="Layout"
      />

      {/* Visual Playground */}
      <Card
        title="Interactive Flexbox Playground"
        description="Adjust flex container properties and observe real-time layout changes."
      >
        <View style={{ gap: 16 }}>
          {/* Control Panel */}
          <View
            style={{
              backgroundColor: theme().bgActive,
              padding: 14,
              borderRadius: 8,
              gap: 12,
            }}
          >
            {/* Flex Direction */}
            <DenseRow gap={10} style={{ alignItems: "center" }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13, width: 120, fontWeight: "semibold" }}>
                flexDirection:
              </Text>
              {(["row", "column", "row-reverse", "column-reverse"] as const).map((dir) => (
                <Button variant={direction() === dir ? "primary" : "ghost"} size="sm" onPress={() => setDirection(dir)}>
                  {dir}
                </Button>
              ))}
            </DenseRow>

            {/* Justify Content */}
            <DenseRow gap={10} style={{ alignItems: "center" }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13, width: 120, fontWeight: "semibold" }}>
                justifyContent:
              </Text>
              {(["flex-start", "center", "flex-end", "space-between", "space-around", "space-evenly"] as const).map(
                (j) => (
                  <Button variant={justify() === j ? "primary" : "ghost"} size="sm" onPress={() => setJustify(j)}>
                    {j}
                  </Button>
                ),
              )}
            </DenseRow>

            {/* Align Items */}
            <DenseRow gap={10} style={{ alignItems: "center" }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13, width: 120, fontWeight: "semibold" }}>
                alignItems:
              </Text>
              {(["flex-start", "center", "flex-end", "stretch"] as const).map((a) => (
                <Button variant={align() === a ? "primary" : "ghost"} size="sm" onPress={() => setAlign(a)}>
                  {a}
                </Button>
              ))}
            </DenseRow>

            {/* Gap & Boxes */}
            <DenseRow gap={16} style={{ alignItems: "center" }}>
              <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13, width: 120, fontWeight: "semibold" }}>
                  gap: {gap()}px
                </Text>
                <Button size="sm" variant="secondary" onPress={() => setGap((g) => Math.max(0, g - 4))}>
                  -4px
                </Button>
                <Button size="sm" variant="secondary" onPress={() => setGap((g) => Math.min(48, g + 4))}>
                  +4px
                </Button>
              </View>
              <Divider orientation="vertical" margin={4} />
              <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13 }}>Boxes: {boxCount()}</Text>
                <Button size="sm" variant="secondary" onPress={() => setBoxCount((c) => Math.max(2, c - 1))}>
                  -
                </Button>
                <Button size="sm" variant="secondary" onPress={() => setBoxCount((c) => Math.min(8, c + 1))}>
                  +
                </Button>
              </View>
            </DenseRow>
          </View>

          {/* Layout Preview Area */}
          <View
            style={{
              minHeight: 240,
              minWidth: 0,
              flexShrink: 0,
              overflow: "scroll",
              backgroundColor: theme().bgMuted,
              borderRadius: 8,
              borderWidth: 1,
              borderColor: theme().accent,
            }}
          >
            <View
              style={{
                alignSelf: "stretch",
                minWidth:
                  direction() === "row" || direction() === "row-reverse"
                    ? boxCount() * 90 + (boxCount() - 1) * gap() + 32
                    : 0,
                minHeight: 240,
                flexShrink: 0,
                padding: 16,
                gap: gap(),
                flexDirection: direction(),
                justifyContent: justify(),
                alignItems: align(),
              }}
            >
              {(() => {
                const count = boxCount();
                const boxes: SolidChild[] = [];
                const palettes = [
                  { bg: "#3B82F6", label: "Item #1" },
                  { bg: "#10B981", label: "Item #2" },
                  { bg: "#8B5CF6", label: "Item #3" },
                  { bg: "#F59E0B", label: "Item #4" },
                  { bg: "#EF4444", label: "Item #5" },
                  { bg: "#06B6D4", label: "Item #6" },
                  { bg: "#EC4899", label: "Item #7" },
                  { bg: "#84CC16", label: "Item #8" },
                ];

                for (let i = 0; i < count; i++) {
                  const item = palettes[i % palettes.length]!;
                  const variedHeight = 48 + ((i * 18) % 40);

                  boxes.push(
                    <View
                      style={{
                        backgroundColor: item.bg,
                        borderRadius: 6,
                        padding: 12,
                        minWidth: 90,
                        flexShrink: 0,
                        height: align() === "stretch" ? undefined : variedHeight,
                        alignItems: "center" as const,
                        justifyContent: "center" as const,
                      }}
                    >
                      <Text style={{ color: contrastingText(item.bg), fontSize: 13, fontWeight: "bold" }}>
                        {item.label}
                      </Text>
                      <Text style={{ color: contrastingText(item.bg), fontSize: 10 }}>
                        {align() === "stretch" ? "stretch" : `${variedHeight}px`}
                      </Text>
                    </View>,
                  );
                }

                return boxes;
              })()}
            </View>
          </View>
        </View>
      </Card>

      {/* Real-World Multi-Column Grid Layout Example */}
      <Card title="Responsive Multi-Column Card Grid">
        <ResponsiveRow gap={12}>
          {[
            { title: "Cloud Instance", metric: "4 vCPU / 16GB", status: "Running", color: "#10B981" },
            { title: "PostgreSQL DB", metric: "248.5 GB Stored", status: "Healthy", color: "#3B82F6" },
            { title: "Cache Cluster", metric: "99.4% Hit Ratio", status: "Active", color: "#8B5CF6" },
          ].map((c) => (
            <View
              style={{
                flexGrow: 1,
                minWidth: 0,
                flexShrink: 1,
                backgroundColor: theme().bgActive,
                borderRadius: 8,
                padding: 14,
                gap: 8,
                borderWidth: 1,
                borderColor: theme().border,
              }}
            >
              <View
                style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center", minWidth: 0 }}
              >
                <Text
                  style={{
                    color: theme().textPrimary,
                    fontSize: 14,
                    fontWeight: "semibold",
                    minWidth: 0,
                    flexShrink: 1,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}
                >
                  {c.title}
                </Text>
                <Badge label={c.status} variant="success" size="sm" />
              </View>
              <Text style={{ color: theme().textSecondary, fontSize: 12, minWidth: 0, flexShrink: 1 }}>{c.metric}</Text>
            </View>
          ))}
        </ResponsiveRow>
      </Card>
    </View>
  );
}
