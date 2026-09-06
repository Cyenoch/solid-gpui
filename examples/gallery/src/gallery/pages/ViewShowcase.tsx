import { Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { contrastingText } from "../theme";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import { Button, Card, CodeSnippet, DenseRow, Divider, PropTable, SectionHeader } from "../components/ui";

export function ViewShowcase(): SolidChild {
  const { theme } = useGallery();

  // Interactive state
  const [padding, setPadding] = createSignal(16);
  const [gap, setGap] = createSignal(12);
  const [borderRadius, setBorderRadius] = createSignal(8);
  const [borderWidth, setBorderWidth] = createSignal(1);
  const [direction, setDirection] = createSignal<"row" | "column">("row");
  const [justify, setJustify] = createSignal<"flex-start" | "center" | "flex-end" | "space-between">("flex-start");
  const [itemCount, setItemCount] = createSignal(3);

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="View Component"
        tag="Core Component"
        description="The fundamental layout primitive in Solid GPUI, backing flexbox containers, surfaces, borders, and interaction handlers."
      />

      {/* Interactive Playground */}
      <Card
        title="Interactive Playground"
        description="Tweak padding, gap, borders, and layout direction live to see real-time updates."
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
          {/* Padding buttons */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Padding:</Text>
            <Button size="sm" variant={padding() === 8 ? "primary" : "secondary"} onPress={() => setPadding(8)}>
              8px
            </Button>
            <Button size="sm" variant={padding() === 16 ? "primary" : "secondary"} onPress={() => setPadding(16)}>
              16px
            </Button>
            <Button size="sm" variant={padding() === 24 ? "primary" : "secondary"} onPress={() => setPadding(24)}>
              24px
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Gap buttons */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Gap:</Text>
            <Button size="sm" variant={gap() === 4 ? "primary" : "secondary"} onPress={() => setGap(4)}>
              4px
            </Button>
            <Button size="sm" variant={gap() === 12 ? "primary" : "secondary"} onPress={() => setGap(12)}>
              12px
            </Button>
            <Button size="sm" variant={gap() === 20 ? "primary" : "secondary"} onPress={() => setGap(20)}>
              20px
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Radius buttons */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Radius:</Text>
            <Button
              size="sm"
              variant={borderRadius() === 0 ? "primary" : "secondary"}
              onPress={() => setBorderRadius(0)}
            >
              0
            </Button>
            <Button
              size="sm"
              variant={borderRadius() === 8 ? "primary" : "secondary"}
              onPress={() => setBorderRadius(8)}
            >
              8px
            </Button>
            <Button
              size="sm"
              variant={borderRadius() === 16 ? "primary" : "secondary"}
              onPress={() => setBorderRadius(16)}
            >
              16px
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Direction */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Direction:</Text>
            <Button
              size="sm"
              variant={direction() === "row" ? "primary" : "secondary"}
              onPress={() => setDirection("row")}
            >
              Row
            </Button>
            <Button
              size="sm"
              variant={direction() === "column" ? "primary" : "secondary"}
              onPress={() => setDirection("column")}
            >
              Column
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Item count */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Items:</Text>
            <Button
              size="sm"
              variant="secondary"
              disabled={itemCount() <= 1}
              onPress={() => setItemCount((c) => Math.max(1, c - 1))}
            >
              -
            </Button>
            <Text
              style={{ color: theme().textPrimary, fontSize: 13, fontWeight: "bold", width: 16, textAlign: "center" }}
            >
              {String(itemCount())}
            </Text>
            <Button
              size="sm"
              variant="secondary"
              disabled={itemCount() >= 6}
              onPress={() => setItemCount((c) => Math.min(6, c + 1))}
            >
              +
            </Button>
          </View>
        </DenseRow>

        {/* Live Preview Stage */}
        <View
          style={{
            backgroundColor: theme().bgActive,
            borderRadius: 8,
            borderWidth: 1,
            borderColor: theme().border,
            padding: 16,
            minHeight: 180,
            minWidth: 0,
            flexShrink: 0,
            overflow: "scroll",
            justifyContent: "center" as const,
          }}
        >
          <View
            style={{
              padding: padding(),
              gap: gap(),
              borderRadius: borderRadius(),
              borderWidth: borderWidth(),
              borderColor: theme().accent,
              backgroundColor: theme().bgCard,
              minWidth:
                direction() === "row"
                  ? itemCount() * 80 + (itemCount() - 1) * gap() + padding() * 2 + borderWidth() * 2
                  : 0,
              flexShrink: 0,
              flexDirection: direction(),
              justifyContent: justify(),
              alignItems: "center" as const,
            }}
          >
            {(() => {
              const boxes: SolidChild[] = [];
              const colors = ["#3B82F6", "#10B981", "#8B5CF6", "#F59E0B", "#EC4899", "#6366F1"];
              for (let i = 0; i < itemCount(); i++) {
                boxes.push(
                  <View
                    style={{
                      width: direction() === "row" ? 80 : undefined,
                      height: direction() === "row" ? 60 : 40,
                      flexShrink: 0,
                      alignSelf: direction() === "column" ? ("stretch" as const) : undefined,
                      backgroundColor: colors[i % colors.length],
                      borderRadius: Math.max(2, borderRadius() / 2),
                      alignItems: "center" as const,
                      justifyContent: "center" as const,
                    }}
                  >
                    <Text
                      style={{ color: contrastingText(colors[i % colors.length]!), fontSize: 13, fontWeight: "bold" }}
                    >{`Box #${i + 1}`}</Text>
                  </View>,
                );
              }
              return boxes;
            })()}
          </View>
        </View>
      </Card>

      {/* Properties & API Reference */}
      <Card title="View Props Reference" description="Summary of supported props on the native View element.">
        <PropTable
          props={[
            {
              name: "style",
              type: "StyleProp",
              default: "undefined",
              description: "Layout, borders, shadows, backgrounds, opacity, cursor, transitions",
            },
            {
              name: "tooltip",
              type: "string",
              default: "undefined",
              description: "Native hover tooltip text displayed after short delay",
            },
            {
              name: "focusable",
              type: "boolean",
              default: "false",
              description: "Whether this View can receive keyboard and native focus",
            },
            {
              name: "onPointerDown",
              type: "PointerHandler",
              default: "undefined",
              description: "Invoked when mouse/pointer button is pressed down on element",
            },
            {
              name: "onPointerUp",
              type: "PointerHandler",
              default: "undefined",
              description: "Invoked when mouse/pointer button is released on element",
            },
            {
              name: "onPointerMove",
              type: "PointerMoveHandler",
              default: "undefined",
              description: "Fired as pointer moves over the element bounds",
            },
            {
              name: "onHoverChange",
              type: "HoverHandler",
              default: "undefined",
              description: "Fired with true when mouse enters, false when mouse leaves",
            },
            {
              name: "onPointerDownOutside",
              type: "PointerDownOutsideHandler",
              default: "undefined",
              description: "Triggered on clicks outside the node (great for popovers / menus)",
            },
            {
              name: "onScroll",
              type: "ScrollHandler",
              default: "undefined",
              description: "Invoked on scroll wheel/trackpad events within the node",
            },
            {
              name: "onLayout",
              type: "LayoutHandler",
              default: "undefined",
              description: "Invoked with { x, y, width, height } after layout computation",
            },
            {
              name: "draggable",
              type: "Draggable",
              default: "undefined",
              description: "Enables drag-and-drop operations starting from this element",
            },
            {
              name: "onDrop",
              type: "DragDropHandler",
              default: "undefined",
              description: "Invoked when a matching drag payload is dropped on this element",
            },
            {
              name: "onExternalFileDrop",
              type: "ExternalFileDropHandler",
              default: "undefined",
              description: "Fired when files from OS finder/explorer are dropped here",
            },
          ]}
        />
      </Card>

      {/* Code Example */}
      <Card title="Code Example">
        <CodeSnippet
          code={`import { View, Text } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

function Card() {
  return createComponent(View, {
    style: {
      padding: 16,
      backgroundColor: "#1E293B",
      borderRadius: 8,
      borderWidth: 1,
      borderColor: "#334155",
      gap: 12,
    },
    get children() {
      return [
        createComponent(Text, {
          style: { color: "#F8FAFC", fontSize: 16, fontWeight: "bold" },
          children: "Native Box Layout",
        }),
      ];
    },
  });
}`}
        />
      </Card>
    </View>
  );
}
