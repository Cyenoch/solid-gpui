import { Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import {
  Badge,
  Button,
  Card,
  CodeSnippet,
  DenseRow,
  Divider,
  PropTable,
  ResponsiveRow,
  SectionHeader,
} from "../components/ui";

export function TextShowcase(): SolidChild {
  const { theme } = useGallery();

  const [fontSize, setFontSize] = createSignal(16);
  const [fontWeight, setFontWeight] = createSignal<"normal" | "medium" | "semibold" | "bold" | "heavy">("medium");
  const [fontStyle, setFontStyle] = createSignal<"normal" | "italic">("normal");
  const [decoration, setDecoration] = createSignal<"none" | "underline" | "lineThrough">("none");
  const [textAlign, setTextAlign] = createSignal<"left" | "center" | "right">("left");
  const [selectable, setSelectable] = createSignal(true);
  const [sampleText, setSampleText] = createSignal("The quick brown fox jumps over the lazy dog");

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Text Component"
        tag="Typography"
        description="Native text rendering with subpixel anti-aliasing, layout shaping, font weights, line clamping, and selection."
      />

      {/* Interactive Typography Controls */}
      <Card
        title="Interactive Typography Controls"
        description="Modify font parameters to inspect layout and rendering behavior in real time."
      >
        <DenseRow
          style={{
            backgroundColor: theme().bgHover,
            padding: 12,
            borderRadius: 6,
            alignItems: "center",
          }}
        >
          {/* Font Size */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Size:</Text>
            <Button size="sm" variant={fontSize() === 14 ? "primary" : "secondary"} onPress={() => setFontSize(14)}>
              14px
            </Button>
            <Button size="sm" variant={fontSize() === 18 ? "primary" : "secondary"} onPress={() => setFontSize(18)}>
              18px
            </Button>
            <Button size="sm" variant={fontSize() === 24 ? "primary" : "secondary"} onPress={() => setFontSize(24)}>
              24px
            </Button>
            <Button size="sm" variant={fontSize() === 32 ? "primary" : "secondary"} onPress={() => setFontSize(32)}>
              32px
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Weight */}
          <DenseRow gap={6} style={{ alignItems: "center" }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Weight:</Text>
            <Button
              size="sm"
              variant={fontWeight() === "normal" ? "primary" : "secondary"}
              onPress={() => setFontWeight("normal")}
            >
              Normal
            </Button>
            <Button
              size="sm"
              variant={fontWeight() === "medium" ? "primary" : "secondary"}
              onPress={() => setFontWeight("medium")}
            >
              Medium
            </Button>
            <Button
              size="sm"
              variant={fontWeight() === "semibold" ? "primary" : "secondary"}
              onPress={() => setFontWeight("semibold")}
            >
              Semibold
            </Button>
            <Button
              size="sm"
              variant={fontWeight() === "bold" ? "primary" : "secondary"}
              onPress={() => setFontWeight("bold")}
            >
              Bold
            </Button>
          </DenseRow>

          <Divider orientation="vertical" margin={4} />

          {/* Style */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Style:</Text>
            <Button
              size="sm"
              variant={fontStyle() === "normal" ? "primary" : "secondary"}
              onPress={() => setFontStyle("normal")}
            >
              Regular
            </Button>
            <Button
              size="sm"
              variant={fontStyle() === "italic" ? "primary" : "secondary"}
              onPress={() => setFontStyle("italic")}
            >
              Italic
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Align */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Align:</Text>
            <Button
              size="sm"
              variant={textAlign() === "left" ? "primary" : "secondary"}
              onPress={() => setTextAlign("left")}
            >
              Left
            </Button>
            <Button
              size="sm"
              variant={textAlign() === "center" ? "primary" : "secondary"}
              onPress={() => setTextAlign("center")}
            >
              Center
            </Button>
            <Button
              size="sm"
              variant={textAlign() === "right" ? "primary" : "secondary"}
              onPress={() => setTextAlign("right")}
            >
              Right
            </Button>
          </View>

          <Divider orientation="vertical" margin={4} />

          {/* Selectable */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Button
              size="sm"
              variant={selectable() ? "primary" : "secondary"}
              onPress={() => setSelectable(!selectable())}
            >
              {selectable() ? "Selectable: On" : "Selectable: Off"}
            </Button>
          </View>
        </DenseRow>

        {/* Live Text Preview Box */}
        <View
          style={{
            backgroundColor: theme().bgActive,
            borderRadius: 8,
            padding: 20,
            minHeight: 120,
            minWidth: 0,
            flexShrink: 1,
            justifyContent: "center",
          }}
        >
          <Text
            style={{
              color: theme().textPrimary,
              fontSize: fontSize(),
              fontWeight: fontWeight(),
              fontStyle: fontStyle(),
              textDecoration: decoration(),
              textAlign: textAlign(),
              lineHeight: fontSize() * 1.4,
              minWidth: 0,
              flexShrink: 1,
            }}
            selectable={selectable()}
          >
            {sampleText()}
          </Text>
        </View>
      </Card>

      {/* Line Clamp Showcase */}
      <Card
        title="Multi-line Clamping & Overflow"
        description="Native text truncation using lineClamp and textOverflow ellipsis."
      >
        <ResponsiveRow gap={16}>
          <View style={{ padding: 14, backgroundColor: theme().bgActive, borderRadius: 8, gap: 6 }}>
            <Badge label="lineClamp: 2" variant="accent" size="sm" />
            <Text
              style={{
                color: theme().textPrimary,
                fontSize: 13,
                lineHeight: 18,
                lineClamp: 2,
                textOverflow: "ellipsis",
                minWidth: 0,
                flexShrink: 1,
              }}
            >
              This is a long paragraph that demonstrates multi-line clamping. When the text exceeds the configured
              number of lines, it is automatically truncated with an ellipsis, preventing overflow while maintaining
              layout integrity.
            </Text>
          </View>
          <View style={{ padding: 14, backgroundColor: theme().bgActive, borderRadius: 8, gap: 6 }}>
            <Badge label="lineClamp: 3" variant="accent" size="sm" />
            <Text
              style={{
                color: theme().textPrimary,
                fontSize: 13,
                lineHeight: 18,
                lineClamp: 3,
                textOverflow: "ellipsis",
                minWidth: 0,
                flexShrink: 1,
              }}
            >
              Solid GPUI integrates deeply with GPUI's text shaping and layout engine. It accurately computes line
              heights, glyph metrics, and truncation boundaries natively, providing desktop-class typography performance
              across macOS, Linux, and Windows.
            </Text>
          </View>
        </ResponsiveRow>
      </Card>

      {/* Properties & API Reference */}
      <Card title="Text Props Reference" description="All properties supported on the native Text component.">
        <PropTable
          props={[
            { name: "style.fontSize", type: "number", default: "14", description: "Font size in pixels" },
            {
              name: "style.fontWeight",
              type: "'normal'|'medium'|'semibold'|'bold'|'heavy'",
              default: "'normal'",
              description: "Font weight mapping",
            },
            {
              name: "style.fontStyle",
              type: "'normal'|'italic'",
              default: "'normal'",
              description: "Italic or normal font style",
            },
            {
              name: "style.color",
              type: "string (hex)",
              default: "inherit",
              description: "Text color formatted as #RRGGBB or #RRGGBBAA",
            },
            {
              name: "style.textAlign",
              type: "'left'|'center'|'right'",
              default: "'left'",
              description: "Horizontal text alignment",
            },
            {
              name: "style.textDecoration",
              type: "'none'|'underline'|'lineThrough'",
              default: "'none'",
              description: "Text decoration styling",
            },
            {
              name: "style.lineHeight",
              type: "number",
              default: "fontSize * 1.3",
              description: "Explicit line height in pixels",
            },
            {
              name: "style.lineClamp",
              type: "number",
              default: "undefined",
              description: "Maximum number of lines rendered before truncation",
            },
            {
              name: "style.textOverflow",
              type: "'clip'|'ellipsis'",
              default: "'clip'",
              description: "Truncation style when text overflows container",
            },
            {
              name: "selectable",
              type: "boolean",
              default: "false",
              description: "Whether the text can be selected and copied by the user",
            },
            {
              name: "onPress",
              type: "PressHandler",
              default: "undefined",
              description: "Callback when the text element itself is clicked",
            },
          ]}
        />
      </Card>

      {/* Code Example */}
      <Card title="Code Example">
        <CodeSnippet
          code={`import { Text } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

createComponent(Text, {
  style: {
    color: "#60A5FA",
    fontSize: 20,
    fontWeight: "bold",
    textDecoration: "underline",
  },
  selectable: true,
  children: "Native High-DPI Typography",
})`}
        />
      </Card>
    </View>
  );
}
