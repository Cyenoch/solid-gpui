import { Icon, Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import type { SolidChild } from "../types";
import type { CursorStyle } from "@solid-gpui/core";
import { useGallery } from "../context";
import { Badge, Card, DenseRow, Divider, SectionHeader } from "../components/ui";

export function CursorsShowcase(): SolidChild {
  const { theme } = useGallery();
  const [activeCursor, setActiveCursor] = createSignal<CursorStyle>("default");

  const cursorList: {
    readonly style: CursorStyle;
    readonly label: string;
    readonly icon:
      | "lucide:arrow-left"
      | "lucide:mouse-pointer-click"
      | "lucide:type"
      | "lucide:move"
      | "lucide:triangle-alert"
      | "lucide:file"
      | "lucide:plus"
      | "lucide:copy"
      | "lucide:external-link";
    readonly category: string;
  }[] = [
    { style: "default", label: "Default", icon: "lucide:arrow-left", category: "Standard" },
    { style: "pointer", label: "Pointer (Hand)", icon: "lucide:mouse-pointer-click", category: "Interactive" },
    { style: "text", label: "I-Beam Text", icon: "lucide:type", category: "Editing" },
    { style: "vertical-text", label: "Vertical Text", icon: "lucide:type", category: "Editing" },
    { style: "grab", label: "Grab (Open Hand)", icon: "lucide:move", category: "Drag & Pan" },
    { style: "grabbing", label: "Grabbing (Closed)", icon: "lucide:move", category: "Drag & Pan" },
    { style: "move", label: "Move (4-Way)", icon: "lucide:move", category: "Drag & Pan" },
    { style: "not-allowed", label: "Not Allowed", icon: "lucide:triangle-alert", category: "Status" },
    { style: "no-drop", label: "No Drop", icon: "lucide:triangle-alert", category: "Status" },
    { style: "context-menu", label: "Context Menu", icon: "lucide:file", category: "Status" },
    { style: "crosshair", label: "Crosshair", icon: "lucide:plus", category: "Precision" },
    { style: "copy", label: "Copy", icon: "lucide:copy", category: "Drag Feedback" },
    { style: "alias", label: "Alias (Link)", icon: "lucide:external-link", category: "Drag Feedback" },
    { style: "col-resize", label: "Col Resize", icon: "lucide:move", category: "Resize" },
    { style: "row-resize", label: "Row Resize", icon: "lucide:move", category: "Resize" },
    { style: "ew-resize", label: "EW Resize", icon: "lucide:move", category: "Resize" },
    { style: "ns-resize", label: "NS Resize", icon: "lucide:move", category: "Resize" },
    { style: "nesw-resize", label: "NESW Resize", icon: "lucide:move", category: "Resize" },
    { style: "nwse-resize", label: "NWSE Resize", icon: "lucide:move", category: "Resize" },
  ];

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Native Desktop Cursor Styles"
        description="GPUI sets the native host operating system mouse pointer directly via the style.cursor property. Hover over the cards below to test all supported cursor styles."
        tag="Cursors"
      />

      {/* Active Cursor Status Banner */}
      <DenseRow
        gap={12}
        style={{
          minWidth: 0,
          flexShrink: 0,
          alignItems: "center",
          padding: 12,
          backgroundColor: theme().bgHover,
          borderRadius: 6,
        }}
      >
        <View
          style={{
            width: 10,
            height: 10,
            borderRadius: 5,
            backgroundColor: theme().accent,
            flexShrink: 0,
          }}
        />
        <Text style={{ color: theme().textSecondary, fontSize: 13, minWidth: 0, flexShrink: 1 }}>
          Hover over elements below to preview cursor styles. Active cursor:
        </Text>
        <Text
          style={{
            color: theme().accent,
            fontSize: 13,
            fontWeight: "bold",
            minWidth: 0,
            flexShrink: 1,
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {activeCursor()}
        </Text>
      </DenseRow>

      {/* Cursors Grid */}
      <Card
        title="Cursor Styles Catalog"
        description="Hover over each tile to activate the corresponding OS cursor style."
      >
        <DenseRow gap={12}>
          {cursorList.map((cursor) => (
            <View
              onHoverChange={() => setActiveCursor(cursor.style)}
              style={{
                flexGrow: 1,
                flexShrink: 0,
                width: 170,
                padding: 14,
                borderRadius: 8,
                borderWidth: 1,
                borderColor: activeCursor() === cursor.style ? theme().accent : theme().border,
                backgroundColor: theme().bgCard,
                gap: 10,
                cursor: cursor.style,
              }}
            >
              <View style={{ flexDirection: "column", gap: 6, alignItems: "flex-start", minWidth: 0, flexShrink: 0 }}>
                <View style={{ flexDirection: "row", alignItems: "center", gap: 6, minWidth: 0, flexShrink: 1 }}>
                  <Icon name={cursor.icon} size={14} color={theme().textPrimary} />
                  <Text
                    style={{
                      color: theme().textPrimary,
                      fontWeight: "semibold",
                      fontSize: 13,
                      minWidth: 0,
                      flexShrink: 1,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {cursor.label}
                  </Text>
                </View>
                <View style={{ flexShrink: 0 }}>
                  <Badge
                    label={cursor.category}
                    variant={activeCursor() === cursor.style ? "accent" : "neutral"}
                    size="sm"
                  />
                </View>
              </View>
              <Text style={{ color: theme().textMuted, fontSize: 11 }}>{cursor.style}</Text>
              <View
                style={{
                  height: 50,
                  borderRadius: 4,
                  backgroundColor: theme().bgHover,
                  alignItems: "center",
                  justifyContent: "center",
                  cursor: cursor.style,
                }}
              >
                <Text style={{ color: theme().textSecondary, fontSize: 11 }}>Hover me</Text>
              </View>
            </View>
          ))}
        </DenseRow>
      </Card>
    </View>
  );
}
