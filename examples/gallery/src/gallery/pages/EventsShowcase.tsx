import { Text, TextInput, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import { Badge, Button, Card, CodeSnippet, DenseRow, ResponsiveRow, SectionHeader } from "../components/ui";

interface EventEntry {
  readonly time: string;
  readonly type: string;
  readonly details: string;
}

export function EventsShowcase(): SolidChild {
  const { theme, showStatus } = useGallery();

  const [logs, setLogs] = createSignal<EventEntry[]>([]);
  const [popoverOpen, setPopoverOpen] = createSignal(false);
  const [pointerPos, setPointerPos] = createSignal({ x: 0, y: 0 });
  const [focusName, setFocusName] = createSignal<string>("None");
  const [lastKeyPressed, setLastKeyPressed] = createSignal<string>("None");

  const addLog = (type: string, details: string) => {
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [{ time, type, details }, ...prev.slice(0, 7)]);
  };

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Interactions & Events"
        tag="Events Architecture"
        description="Pointer, keyboard, focus, outside-click, and scroll event handling."
      />

      {/* Pointer Events Pad & Popover Area */}
      <Card
        title="Pointer Tracking & Outside Click Popover"
        description="Move, click, or scroll in the pointer pad; open the popover and click outside to observe dismissal."
      >
        <ResponsiveRow gap={16}>
          {/* Pointer Pad */}
          <View
            style={{
              flexGrow: 1,
              minWidth: 0,
              flexShrink: 1,
              minHeight: 180,
              backgroundColor: theme().bgActive,
              borderRadius: 8,
              borderWidth: 1,
              borderColor: theme().border,
              alignItems: "center",
              justifyContent: "center",
              gap: 8,
            }}
            onPointerMove={(e) => {
              setPointerPos({ x: Math.round(e.x), y: Math.round(e.y) });
              addLog("pointerMove", `x: ${Math.round(e.x)}, y: ${Math.round(e.y)}`);
            }}
            onPointerDown={(e) => {
              addLog("pointerDown", `button: ${e.button}, clicks: ${e.clickCount}`);
              showStatus(`Pointer down (button ${e.button})`, "info");
            }}
            onPointerUp={(e) => {
              addLog("pointerUp", `button: ${e.button}`);
            }}
            onHoverChange={(isHover) => {
              addLog("hoverChange", `hovered: ${isHover}`);
            }}
            onScroll={(e) => {
              addLog("scroll", `dx: ${e.dx.toFixed(1)}, dy: ${e.dy.toFixed(1)}`);
            }}
          >
            <Text style={{ color: theme().accent, fontSize: 14, fontWeight: "bold" }}>Interactive Pointer Pad</Text>
            <Text style={{ color: theme().textSecondary, fontSize: 12 }}>
              Coordinates: ({pointerPos().x}, {pointerPos().y})
            </Text>
            <Text style={{ color: theme().textMuted, fontSize: 11 }}>Move mouse, click, or scroll here</Text>
          </View>

          {/* Outside Click Popover Section */}
          <View
            style={{
              flexGrow: 1,
              minWidth: 0,
              flexShrink: 1,
              minHeight: 180,
              backgroundColor: theme().bgActive,
              borderRadius: 8,
              borderWidth: 1,
              borderColor: theme().border,
              padding: 16,
              alignItems: "center",
              justifyContent: "center",
              gap: 12,
            }}
          >
            <Button
              variant={popoverOpen() ? "primary" : "outline"}
              onPress={() => {
                setPopoverOpen(!popoverOpen());
                addLog("popoverToggle", `open: ${!popoverOpen()}`);
              }}
            >
              {popoverOpen() ? "Close Popover" : "Open Popover (Outside Click)"}
            </Button>
            {popoverOpen() ? (
              <View
                style={{
                  padding: 14,
                  backgroundColor: theme().bgCard,
                  borderRadius: 8,
                  borderWidth: 1,
                  borderColor: theme().accent,
                  gap: 6,
                  boxShadow: { color: "#00000044", offsetX: 0, offsetY: 4, blurRadius: 12, spreadRadius: 0 },
                }}
                onPointerDownOutside={() => {
                  setPopoverOpen(false);
                  addLog("pointerDownOutside", "Clicked outside popover -> closed");
                  showStatus("Dismissed via onPointerDownOutside", "warning");
                }}
              >
                <Text style={{ color: theme().textPrimary, fontSize: 13, fontWeight: "bold" }}>Popover Menu</Text>
                <Text style={{ color: theme().textSecondary, fontSize: 11 }}>
                  Click anywhere outside this box to automatically close it.
                </Text>
              </View>
            ) : (
              <Text style={{ color: theme().textMuted, fontSize: 12 }}>Popover is currently closed</Text>
            )}
          </View>
        </ResponsiveRow>

        {/* Focus & Keyboard Area */}
        <ResponsiveRow
          gap={16}
          grow={[1, 0]}
          style={{ padding: 14, backgroundColor: theme().bgHover, borderRadius: 8, alignItems: "center" }}
        >
          <View style={{ flexGrow: 1, minWidth: 0, flexShrink: 1, gap: 6 }}>
            <Text style={{ color: theme().textPrimary, fontSize: 13, fontWeight: "bold" }}>
              Keyboard & Focus Ring Test
            </Text>
            <Text style={{ color: theme().textSecondary, fontSize: 12 }}>
              Focused: {focusName()} • Last Key: {lastKeyPressed()}
            </Text>
          </View>
          <DenseRow gap={8}>
            <TextInput
              style={{
                width: 140,
                flexShrink: 0,
                backgroundColor: theme().bgInput,
                borderWidth: 1,
                borderColor: theme().border,
                borderRadius: 6,
                padding: 6,
                color: theme().textPrimary,
                fontSize: 12,
              }}
              placeholder="Focus me..."
              onFocus={() => {
                setFocusName("Input #1");
                addLog("focus", "Input #1 gained focus");
              }}
              onBlur={() => {
                setFocusName("None");
                addLog("blur", "Input #1 lost focus");
              }}
              onKeyDown={(e) => {
                setLastKeyPressed(e.key);
                addLog("keyDown", `key: ${e.key}, action: ${e.action}`);
              }}
            />
            <TextInput
              style={{
                width: 140,
                flexShrink: 0,
                backgroundColor: theme().bgInput,
                borderWidth: 1,
                borderColor: theme().border,
                borderRadius: 6,
                padding: 6,
                color: theme().textPrimary,
                fontSize: 12,
              }}
              placeholder="Focus me too..."
              onFocus={() => {
                setFocusName("Input #2");
                addLog("focus", "Input #2 gained focus");
              }}
              onBlur={() => {
                setFocusName("None");
                addLog("blur", "Input #2 lost focus");
              }}
              onKeyDown={(e) => {
                setLastKeyPressed(e.key);
                addLog("keyDown", `key: ${e.key}, action: ${e.action}`);
              }}
            />
          </DenseRow>
        </ResponsiveRow>

        {/* Live Event Stream */}
        <View
          style={{
            backgroundColor: theme().bgMuted,
            borderRadius: 8,
            padding: 12,
            gap: 6,
            borderWidth: 1,
            borderColor: theme().border,
          }}
        >
          <DenseRow gap={8} style={{ justifyContent: "flex-start", alignItems: "center" }}>
            <Text style={{ color: theme().textMuted, fontSize: 12, fontWeight: "bold" }}>REAL-TIME EVENT LOG</Text>
            <Button size="sm" variant="ghost" onPress={() => setLogs([])}>
              Clear Logs
            </Button>
          </DenseRow>
          {logs().length === 0 ? (
            <Text style={{ color: theme().textMuted, fontSize: 12, fontStyle: "italic" }}>
              Interact with the areas above to see live event payloads.
            </Text>
          ) : (
            <View style={{ gap: 4 }}>
              {logs().map((l) => (
                <View style={{ flexDirection: "row", gap: 8, alignItems: "center", minWidth: 0, flexShrink: 1 }}>
                  <Text style={{ color: theme().textMuted, fontSize: 11, flexShrink: 0 }}>[{l.time}]</Text>
                  <Badge label={l.type} variant="accent" size="sm" />
                  <Text
                    style={{
                      color: theme().textPrimary,
                      fontSize: 12,
                      minWidth: 0,
                      flexShrink: 1,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {l.details}
                  </Text>
                </View>
              ))}
            </View>
          )}
        </View>
      </Card>

      {/* Code Example */}
      <Card title="Code Example: Event Listeners">
        <CodeSnippet
          code={`import { View } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

createComponent(View, {
  onPointerMove: (e) => console.log(e.position.x, e.position.y),
  onPointerDownOutside: () => closeDropdown(),
  onScroll: (e) => console.log(e.delta.x, e.delta.y),
  onKeyDown: (e) => {
    if (e.key === "Escape") closeDropdown();
  },
})`}
        />
      </Card>
    </View>
  );
}
