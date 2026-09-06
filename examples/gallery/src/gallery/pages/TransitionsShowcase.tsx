import { Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import type { TransitionEasing } from "@solid-gpui/core";
import type { SolidChild } from "../types";
import { useGallery } from "../context";
import { Badge, Button, Card, CodeSnippet, DenseRow, Divider, SectionHeader } from "../components/ui";

export function TransitionsShowcase(): SolidChild {
  const { theme, showStatus } = useGallery();

  const [expanded, setExpanded] = createSignal(false);
  const [faded, setFaded] = createSignal(false);
  const [colorShift, setColorShift] = createSignal(false);
  const [duration, setDuration] = createSignal(300);
  const [easing, setEasing] = createSignal<TransitionEasing>("easeInOut");
  const [animStatus, setAnimStatus] = createSignal("Idle");

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Transitions & Motion"
        description="Declarative transitions for layout dimensions, background colors, and opacity across signal updates."
        tag="Motion"
      />

      {/* Interactive Transition Box */}
      <Card
        title="Interactive Transition Playground"
        description="Trigger state toggles and observe interpolated transitions across easing curves."
      >
        <View style={{ gap: 16 }}>
          {/* Controls Bar */}
          <View
            style={{
              backgroundColor: theme().bgActive,
              padding: 12,
              borderRadius: 8,
              gap: 12,
            }}
          >
            {/* Easing & Duration */}
            <DenseRow gap={12} style={{ alignItems: "center" }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13, fontWeight: "medium" }}>Easing:</Text>
              {(["linear", "easeIn", "easeOut", "easeInOut"] as const).map((es) => (
                <Button variant={easing() === es ? "primary" : "ghost"} size="sm" onPress={() => setEasing(es)}>
                  {es}
                </Button>
              ))}
              <Divider orientation="vertical" margin={4} />
              <Text style={{ color: theme().textSecondary, fontSize: 13, fontWeight: "medium" }}>Duration:</Text>
              {([150, 300, 600, 1000] as const).map((ms) => (
                <Button variant={duration() === ms ? "primary" : "ghost"} size="sm" onPress={() => setDuration(ms)}>
                  {`${ms}ms`}
                </Button>
              ))}
            </DenseRow>

            {/* Trigger Action Buttons */}
            <DenseRow gap={12} style={{ alignItems: "center" }}>
              <Button
                variant={expanded() ? "primary" : "outline"}
                size="sm"
                onPress={() => {
                  setAnimStatus("Animating Size...");
                  setExpanded(!expanded());
                }}
              >
                {expanded() ? "Shrink Size" : "Expand Size"}
              </Button>
              <Button
                variant={faded() ? "primary" : "outline"}
                size="sm"
                onPress={() => {
                  setAnimStatus("Animating Opacity...");
                  setFaded(!faded());
                }}
              >
                {faded() ? "Fade In (100%)" : "Fade Out (30%)"}
              </Button>
              <Button
                variant={colorShift() ? "primary" : "outline"}
                size="sm"
                onPress={() => {
                  setAnimStatus("Animating Color...");
                  setColorShift(!colorShift());
                }}
              >
                {colorShift() ? "Reset Color" : "Shift Color"}
              </Button>
              <Badge label={`Status: ${animStatus()}`} variant="neutral" />
            </DenseRow>
          </View>

          {/* Animation Canvas */}
          <View
            style={{
              minHeight: 240,
              minWidth: 0,
              flexShrink: 0,
              overflow: "scroll",
              backgroundColor: theme().bgMuted,
              borderRadius: 8,
              borderWidth: 1,
              borderColor: theme().border,
            }}
          >
            <View
              style={{
                alignSelf: "stretch",
                minWidth: (expanded() ? 420 : 200) + 48,
                padding: 24,
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <View
                style={{
                  flexShrink: 0,
                  width: expanded() ? 420 : 200,
                  height: expanded() ? 160 : 90,
                  backgroundColor: colorShift() ? "#8B5CF6" : "#2563EB",
                  opacity: faded() ? 0.3 : 1.0,
                  borderRadius: expanded() ? 20 : 8,
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 16,
                  gap: 8,
                  boxShadow: {
                    color: "#00000044",
                    offsetX: 0,
                    offsetY: expanded() ? 12 : 4,
                    blurRadius: expanded() ? 24 : 8,
                    spreadRadius: 0,
                  },
                  transition: {
                    durationMs: duration(),
                    easing: easing(),
                    properties: ["width", "height", "backgroundColor", "opacity"] as const,
                    onComplete: (gen: number) => {
                      setAnimStatus("Completed");
                      showStatus(`Transition finished (gen: ${gen})`);
                    },
                  },
                }}
              >
                <Text style={{ color: "#FFFFFF", fontSize: 16, fontWeight: "bold" }}>Animated Surface</Text>
                <Text style={{ color: "#FFFFFFCC", fontSize: 12 }}>
                  {() => `Width: ${expanded() ? 420 : 200}px | Height: ${expanded() ? 160 : 90}px`}
                </Text>
              </View>
            </View>
          </View>
        </View>
      </Card>

      {/* Code Example */}
      <Card title="Code Example: Declarative Transitions">
        <CodeSnippet
          code={`import { View } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";

function MorphingBox() {
  const [active, setActive] = createSignal(false);

  return createComponent(View, {
    onPress: () => setActive(!active()),
    get style() {
      return {
        width: active() ? 300 : 120,
        height: active() ? 200 : 80,
        backgroundColor: active() ? "#10B981" : "#3B82F6",
        opacity: active() ? 1.0 : 0.7,
        borderRadius: 8,
        transition: {
          durationMs: 300,
          easing: "easeInOut",
          properties: ["width", "height", "backgroundColor", "opacity"] as const,
          onComplete: () => console.log("Animation complete"),
        },
      };
    },
  });
}`}
        />
      </Card>
    </View>
  );
}
