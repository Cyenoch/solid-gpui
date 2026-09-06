import { Icon, Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import type { SolidChild } from "../types";
import { useGallery } from "../context";
import { Badge, Button, Card, CodeSnippet, DenseRow, ResponsiveRow, SectionHeader } from "../components/ui";

interface DragCardItem {
  readonly id: string;
  readonly title: string;
  readonly column: "todo" | "inprogress" | "done";
}

export function DragDropShowcase(): SolidChild {
  const { theme, showStatus } = useGallery();

  const [items, setItems] = createSignal<DragCardItem[]>([
    { id: "1", title: "Implement Bebop v5 Codec", column: "done" },
    { id: "2", title: "Add Drag & Drop Host Callbacks", column: "done" },
    { id: "3", title: "Build Gallery Navigation Bar", column: "inprogress" },
    { id: "4", title: "Add Native Menu Keybindings", column: "todo" },
    { id: "5", title: "Benchmark Signal Batching", column: "todo" },
  ]);

  const [activeDragId, setActiveDragId] = createSignal<string | null>(null);
  const [hoveredColumn, setHoveredColumn] = createSignal<string | null>(null);
  const [droppedFiles, setDroppedFiles] = createSignal<string[]>([]);

  const moveItem = (id: string, targetCol: "todo" | "inprogress" | "done") => {
    setItems((prev) => prev.map((item) => (item.id === id ? { ...item, column: targetCol } : item)));
    showStatus(`Moved item to ${targetCol.toUpperCase()}`);
  };

  const columns: {
    id: "todo" | "inprogress" | "done";
    title: string;
    badge: "default" | "accent" | "success";
  }[] = [
    { id: "todo", title: "To Do", badge: "default" },
    { id: "inprogress", title: "In Progress", badge: "accent" },
    { id: "done", title: "Completed", badge: "success" },
  ];

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Drag & Drop Operations"
        description="Desktop drag-and-drop primitives for draggable payloads, drop targets, and external file paths."
        tag="Interaction"
      />

      {/* Interactive Kanban Board */}
      <Card
        title="1. Interactive Kanban Task Board"
        description="Drag tasks between columns or click the column action buttons to reassign them."
      >
        <View style={{ gap: 16 }}>
          <ResponsiveRow gap={16}>
            {columns.map((col) => {
              const colItems = () => items().filter((it) => it.column === col.id);
              const isHovered = () => hoveredColumn() === col.id;

              return (
                <View
                  onDragOver={(dragType) => {
                    setHoveredColumn(col.id);
                  }}
                  onDrop={(dragType) => {
                    const dragId = activeDragId();
                    if (dragId) moveItem(dragId, col.id);
                    setHoveredColumn(null);
                  }}
                  style={{
                    flexGrow: 1,
                    minWidth: 0,
                    flexShrink: 1,
                    minHeight: 280,
                    backgroundColor: isHovered() ? theme().bgActive : theme().bgMuted,
                    borderWidth: 2,
                    borderColor: isHovered() ? theme().accent : theme().border,
                    borderRadius: 8,
                    padding: 12,
                    gap: 10,
                    transition: { durationMs: 150, properties: ["backgroundColor"] as const },
                  }}
                >
                  {/* Column Header */}
                  <View
                    style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center", minWidth: 0 }}
                  >
                    <Text
                      style={{
                        color: theme().textPrimary,
                        fontSize: 14,
                        fontWeight: "bold",
                        minWidth: 0,
                        flexShrink: 1,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                      }}
                    >
                      {col.title}
                    </Text>
                    <Badge label={`${colItems().length}`} variant={col.badge} size="sm" />
                  </View>

                  {/* Cards List */}
                  <View style={{ gap: 8, flexGrow: 1 }}>
                    {colItems().map((card) => (
                      <View
                        draggable={{
                          type: "kanban-card",
                          data: card.id,
                        }}
                        onPointerDown={() => setActiveDragId(card.id)}
                        style={{
                          minWidth: 0,
                          flexShrink: 1,
                          backgroundColor: theme().bgCard,
                          borderRadius: 6,
                          padding: 12,
                          borderWidth: 1,
                          borderColor: theme().border,
                          gap: 8,
                          cursor: "grab" as const,
                          boxShadow: {
                            color: "#00000033",
                            offsetX: 0,
                            offsetY: 2,
                            blurRadius: 4,
                            spreadRadius: 0,
                          },
                        }}
                      >
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
                          {card.title}
                        </Text>
                        {/* Quick move buttons */}
                        <DenseRow gap={4}>
                          {col.id !== "todo" ? (
                            <Button
                              size="sm"
                              variant="ghost"
                              icon="lucide:arrow-left"
                              onPress={() => moveItem(card.id, "todo")}
                            >
                              Todo
                            </Button>
                          ) : null}
                          {col.id !== "inprogress" ? (
                            <Button size="sm" variant="ghost" onPress={() => moveItem(card.id, "inprogress")}>
                              Progress
                            </Button>
                          ) : null}
                          {col.id !== "done" ? (
                            <Button
                              size="sm"
                              variant="ghost"
                              icon="lucide:chevron-right"
                              iconPosition="end"
                              onPress={() => moveItem(card.id, "done")}
                            >
                              Done
                            </Button>
                          ) : null}
                        </DenseRow>
                      </View>
                    ))}
                  </View>
                </View>
              );
            })}
          </ResponsiveRow>
        </View>
      </Card>

      {/* External OS File Drop Zone */}
      <Card
        title="2. External OS File Drop Zone"
        description="Drag and drop any files from Finder / File Explorer into this area to capture absolute native paths."
      >
        <View
          onExternalFileDrop={(paths) => {
            setDroppedFiles((prev) => [...paths, ...prev]);
            showStatus(`Received ${paths.length} external dropped files`);
          }}
          style={{
            minWidth: 0,
            flexShrink: 1,
            backgroundColor: theme().bgActive,
            borderWidth: 2,
            borderColor: theme().accent,
            borderRadius: 8,
            padding: 24,
            alignItems: "center" as const,
            justifyContent: "center" as const,
            gap: 12,
            minHeight: 140,
          }}
        >
          <Icon name="lucide:folder-open" size={32} color={theme().accent} />
          <Text style={{ color: theme().textPrimary, fontSize: 15, fontWeight: "bold" }}>Drop OS Files Here</Text>
          <Text style={{ color: theme().textSecondary, fontSize: 13 }}>
            Listens to native onExternalFileDrop events directly from GPUI platform window.
          </Text>
          {droppedFiles().length > 0 ? (
            <View style={{ alignSelf: "stretch", minWidth: 0, flexShrink: 0, marginTop: 8, gap: 4 }}>
              <Text style={{ color: theme().accent, fontSize: 12, fontWeight: "bold" }}>
                {`DROPPED FILES (${droppedFiles().length}):`}
              </Text>
              {droppedFiles().map((p) => (
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
                  {`• ${p}`}
                </Text>
              ))}
            </View>
          ) : null}
        </View>
      </Card>

      {/* Code Example */}
      <Card title="Code Example: Drag & Drop">
        <CodeSnippet
          code={`import { View, Text } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

// 1. Draggable Item
createComponent(View, {
  draggable: { type: "custom-token", data: { id: 42 } },
  children: createComponent(Text, { children: "Drag Me" }),
});

// 2. Drop Target
createComponent(View, {
  onDragOver: (dragType) => console.log("Dragging over with type:", dragType),
  onDrop: (dragType) => console.log("Dropped payload:", dragType),
  onExternalFileDrop: (paths) => console.log("Dropped native OS files:", paths),
  children: createComponent(Text, { children: "Drop Area" }),
});`}
        />
      </Card>
    </View>
  );
}
