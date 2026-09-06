import { Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import { Badge, Button, Card, CodeSnippet, DenseRow, PropTable, ResponsiveRow, SectionHeader } from "../components/ui";

export function PressableShowcase(): SolidChild {
  const { theme, showStatus } = useGallery();

  const [clickCount, setClickCount] = createSignal(0);
  const [lastAction, setLastAction] = createSignal<string>("None");
  const [hoveredBtn, setHoveredBtn] = createSignal<string | null>(null);
  const [disabledToggle, setDisabledToggle] = createSignal(false);
  const [eventLogs, setEventLogs] = createSignal<{ time: string; text: string }[]>([]);

  const logEvent = (text: string) => {
    const time = new Date().toLocaleTimeString();
    setEventLogs((prev) => [{ time, text }, ...prev.slice(0, 7)]);
  };

  const handleButtonClick = (name: string) => {
    setClickCount((c) => c + 1);
    setLastAction(name);
    logEvent(`Clicked ${name}`);
    showStatus(`Button "${name}" activated`, "info");
  };

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Pressable Component"
        tag="User Interaction"
        description="Interactive pressable container with hover tracking, mouse buttons, keyboard activation, accessibility labels, and tooltips."
      />

      {/* Interactive Button Gallery */}
      <Card
        title="Interactive Button Gallery"
        description="Click various button variants to inspect events, hover states, and disabled handling."
        actions={
          <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
            <Button
              size="sm"
              variant={disabledToggle() ? "danger" : "secondary"}
              onPress={() => setDisabledToggle(!disabledToggle())}
            >
              {disabledToggle() ? "Enable All" : "Disable All"}
            </Button>
          </View>
        }
      >
        {/* Button row */}
        <DenseRow
          gap={12}
          style={{
            alignItems: "center",
            padding: 16,
            backgroundColor: theme().bgHover,
            borderRadius: 8,
          }}
        >
          <Button
            variant="primary"
            disabled={disabledToggle()}
            onPress={() => handleButtonClick("Primary")}
            tooltip="Primary Action Button"
          >
            Primary Button
          </Button>
          <Button
            variant="secondary"
            disabled={disabledToggle()}
            onPress={() => handleButtonClick("Secondary")}
            tooltip="Secondary Action"
          >
            Secondary Button
          </Button>
          <Button
            variant="outline"
            disabled={disabledToggle()}
            onPress={() => handleButtonClick("Outline")}
            tooltip="Outlined Surface"
          >
            Outline Button
          </Button>
          <Button
            variant="ghost"
            disabled={disabledToggle()}
            onPress={() => handleButtonClick("Ghost")}
            tooltip="Ghost Button"
          >
            Ghost Button
          </Button>
          <Button
            variant="danger"
            disabled={disabledToggle()}
            onPress={() => handleButtonClick("Danger")}
            tooltip="Destructive Action"
          >
            Danger Button
          </Button>
          <Button
            variant="success"
            disabled={disabledToggle()}
            onPress={() => handleButtonClick("Success")}
            tooltip="Success Action"
          >
            Success Button
          </Button>
        </DenseRow>

        {/* Live Stats & Event Log */}
        <ResponsiveRow gap={16} grow={[1, 2]}>
          {/* Stats Box */}
          <View
            style={{
              flexGrow: 1,
              padding: 14,
              backgroundColor: theme().bgActive,
              borderRadius: 8,
              gap: 8,
            }}
          >
            <Text style={{ color: theme().textMuted, fontSize: 12, fontWeight: "bold" }}>INTERACTION STATS</Text>
            <View style={{ flexDirection: "row", justifyContent: "space-between", minWidth: 0 }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13 }}>Total Clicks:</Text>
              <Text style={{ color: theme().accent, fontSize: 13, fontWeight: "bold" }}>{String(clickCount())}</Text>
            </View>
            <View style={{ flexDirection: "row", justifyContent: "space-between", minWidth: 0 }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13 }}>Last Clicked:</Text>
              <Text
                style={{
                  color: theme().textPrimary,
                  fontSize: 13,
                  fontWeight: "medium",
                  minWidth: 0,
                  flexShrink: 1,
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
              >
                {lastAction()}
              </Text>
            </View>
            <View style={{ flexDirection: "row", justifyContent: "space-between", minWidth: 0 }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13 }}>Disabled Mode:</Text>
              <Badge
                label={disabledToggle() ? "Disabled" : "Active"}
                variant={disabledToggle() ? "danger" : "success"}
                size="sm"
              />
            </View>
          </View>

          {/* Real-time Event Log */}
          <View
            style={{
              flexGrow: 2,
              padding: 14,
              backgroundColor: theme().bgActive,
              borderRadius: 8,
              gap: 8,
            }}
          >
            <View style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center", minWidth: 0 }}>
              <Text
                style={{
                  color: theme().textMuted,
                  fontSize: 12,
                  fontWeight: "bold",
                  minWidth: 0,
                  flexShrink: 1,
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
              >
                RECENT EVENTS (REAL-TIME)
              </Text>
              <Button size="sm" variant="ghost" onPress={() => setEventLogs([])}>
                Clear Logs
              </Button>
            </View>
            {eventLogs().length === 0 ? (
              <Text style={{ color: theme().textMuted, fontSize: 12, fontStyle: "italic" }}>
                Click any button above to see events logged in real time.
              </Text>
            ) : (
              <View style={{ gap: 4 }}>
                {eventLogs().map((log) => (
                  <View style={{ flexDirection: "row", gap: 8, alignItems: "center", minWidth: 0 }}>
                    <Text style={{ color: theme().textMuted, fontSize: 11, flexShrink: 0 }}>{`[${log.time}]`}</Text>
                    <Text
                      style={{
                        color: "#38BDF8",
                        fontSize: 12,
                        minWidth: 0,
                        flexShrink: 1,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                      }}
                    >
                      {log.text}
                    </Text>
                  </View>
                ))}
              </View>
            )}
          </View>
        </ResponsiveRow>
      </Card>

      {/* Props Reference */}
      <Card
        title="Pressable Props Reference"
        description="All supported properties and callbacks on the native Pressable component."
      >
        <PropTable
          props={[
            {
              name: "onPress",
              type: "PressHandler",
              default: "undefined",
              description: "Fired when element is clicked or triggered via keyboard Space/Enter",
            },
            {
              name: "onHoverChange",
              type: "(hover: boolean) => void",
              default: "undefined",
              description: "Triggered whenever cursor enters or leaves element bounds",
            },
            {
              name: "disabled",
              type: "boolean",
              default: "false",
              description: "Disables click events, visual focus rings, and darkens opacity",
            },
            {
              name: "tooltip",
              type: "string",
              default: "undefined",
              description: "Native hover tooltip displayed after cursor dwell delay",
            },
            {
              name: "accessibilityRole",
              type: "string",
              default: "'button'",
              description: "Accessible element role for screen readers and OS accessibility",
            },
            {
              name: "accessibilityLabel",
              type: "string",
              default: "undefined",
              description: "Accessibility announcement label",
            },
          ]}
        />
      </Card>

      {/* Code Example */}
      <Card title="Code Example">
        <CodeSnippet
          code={`import { Pressable, Text } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";

function HoverButton() {
  const [hovered, setHovered] = createSignal(false);

  return createComponent(Pressable, {
    style: {
      padding: 8,
      backgroundColor: hovered() ? "#1D4ED8" : "#2563EB",
      borderRadius: 6,
    },
    onHoverChange: setHovered,
    onPress: () => console.log("Clicked"),
    children: createComponent(Text, {
      style: { color: "#FFFFFF", fontWeight: "medium" },
      children: "Interactive Pressable",
    }),
  });
}`}
        />
      </Card>
    </View>
  );
}
