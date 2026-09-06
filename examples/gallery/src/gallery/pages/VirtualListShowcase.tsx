import { Pressable, Text, View, VirtualList, type VirtualListHandle } from "@solid-gpui/core";
import { createMemo, createSignal } from "@solid-gpui/core/runtime";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import { Badge, Button, Card, CodeSnippet, DenseRow, Input, PropTable, SectionHeader } from "../components/ui";

interface ListItem {
  readonly id: number;
  readonly title: string;
  readonly status: "Active" | "Pending" | "Completed" | "Archived";
  readonly category: string;
  readonly latency: number;
}

export function VirtualListShowcase(): SolidChild {
  const { theme, showStatus } = useGallery();
  let listHandle: VirtualListHandle | undefined;

  const [totalCount, setTotalCount] = createSignal(1000);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [selectedId, setSelectedId] = createSignal<number | null>(null);

  const statuses: ("Active" | "Pending" | "Completed" | "Archived")[] = ["Active", "Pending", "Completed", "Archived"];
  const categories = ["Network", "Database", "Renderer", "Layout", "Audio", "Graphics"];

  // Generate dataset
  const allItems = createMemo<ListItem[]>(() => {
    const count = totalCount();
    return Array.from({ length: count }, (_, i) => ({
      id: i + 1,
      title: `Virtual Dataset Item #${i + 1} (${categories[i % categories.length]})`,
      status: statuses[i % statuses.length],
      category: categories[i % categories.length],
      latency: Math.floor((i * 13) % 150) + 5,
    }));
  });

  const filteredItems = createMemo<ListItem[]>(() => {
    const q = searchQuery().toLowerCase().trim();
    if (!q) return allItems();
    return allItems().filter((item) => item.title.toLowerCase().includes(q) || item.status.toLowerCase().includes(q));
  });

  const handleScrollToTop = () => {
    listHandle?.scrollToIndex(0);
    showStatus("Scrolled to top", "info");
  };

  const handleScrollToMiddle = () => {
    const mid = Math.floor(filteredItems().length / 2);
    listHandle?.scrollToIndex(mid);
    showStatus(`Scrolled to item #${mid}`, "info");
  };

  const handleScrollToBottom = () => {
    listHandle?.scrollToEnd();
    showStatus("Scrolled to end of list", "info");
  };

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="VirtualList Component"
        tag="Virtualization"
        description="Windowed rendering for large datasets. Visible rows and overscan items stay mounted while scrolling."
      />

      {/* Virtual List Playground Card */}
      <Card
        title="Large Dataset Virtualization"
        description="Inspect scrolling, item selection, filter queries, and navigation across thousands of items."
      >
        {/* Top Bar Controls */}
        <DenseRow
          gap={12}
          style={{
            justifyContent: "flex-start",
            alignItems: "center",
            backgroundColor: theme().bgHover,
            padding: 12,
            borderRadius: 6,
          }}
        >
          {/* Dataset size buttons */}
          <DenseRow gap={6} style={{ alignItems: "center" }}>
            <Text style={{ color: theme().textSecondary, fontSize: 12, fontWeight: "semibold" }}>Dataset Size:</Text>
            <Button
              size="sm"
              variant={totalCount() === 100 ? "primary" : "secondary"}
              onPress={() => setTotalCount(100)}
            >
              100
            </Button>
            <Button
              size="sm"
              variant={totalCount() === 1000 ? "primary" : "secondary"}
              onPress={() => setTotalCount(1000)}
            >
              1,000
            </Button>
            <Button
              size="sm"
              variant={totalCount() === 5000 ? "primary" : "secondary"}
              onPress={() => setTotalCount(5000)}
            >
              5,000
            </Button>
            <Button
              size="sm"
              variant={totalCount() === 20000 ? "primary" : "secondary"}
              onPress={() => setTotalCount(20000)}
            >
              20,000
            </Button>
          </DenseRow>

          {/* Scroll Navigation Buttons */}
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Button size="sm" variant="outline" onPress={handleScrollToTop}>
              Top
            </Button>
            <Button size="sm" variant="outline" onPress={handleScrollToMiddle}>
              Middle
            </Button>
            <Button size="sm" variant="outline" onPress={handleScrollToBottom}>
              End
            </Button>
          </View>
        </DenseRow>

        {/* Filter Search Input */}
        <Input
          placeholder="Filter items by name, category, or status (e.g. 'Network', 'Completed')..."
          value={searchQuery()}
          onInput={setSearchQuery}
        />

        {/* Virtual List Box */}
        <View
          style={{
            height: 320,
            flexDirection: "column",
            minWidth: 0,
            flexShrink: 0,
            backgroundColor: theme().bgActive,
            borderRadius: 8,
            borderWidth: 1,
            borderColor: theme().border,
            overflow: "hidden",
          }}
        >
          <VirtualList<ListItem>
            ref={(handle: VirtualListHandle) => {
              listHandle = handle;
            }}
            data={filteredItems()}
            itemKey={(item) => item.id}
            estimatedItemSize={48}
            overscan={4}
            style={{ flexGrow: 1, minWidth: 0, flexShrink: 1 }}
            emptyState={
              <View style={{ padding: 32, alignItems: "center", justifyContent: "center", gap: 8 }}>
                <Text style={{ color: theme().textMuted, fontSize: 14, fontWeight: "medium" }}>
                  No items match your search filter
                </Text>
              </View>
            }
            renderItem={(item) => {
              const isSelected = () => selectedId() === item.id;
              let statusVariant: "accent" | "success" | "warning" | "neutral" = "neutral";
              if (item.status === "Active") statusVariant = "accent";
              else if (item.status === "Completed") statusVariant = "success";
              else if (item.status === "Pending") statusVariant = "warning";

              return (
                <Pressable
                  style={{
                    height: 48,
                    flexDirection: "row",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: 8,
                    minWidth: 0,
                    flexShrink: 1,
                    backgroundColor: isSelected() ? theme().bgHover : "#00000000",
                    borderWidth: 1,
                    borderColor: isSelected() ? theme().borderFocus : "#00000000",
                  }}
                  onPress={() => {
                    setSelectedId(item.id);
                    showStatus(`Selected row #${item.id}: ${item.title}`, "info");
                  }}
                >
                  <View
                    style={{
                      flexDirection: "row",
                      alignItems: "center",
                      gap: 12,
                      minWidth: 0,
                      flexGrow: 1,
                      flexShrink: 1,
                    }}
                  >
                    <View
                      style={{
                        width: 44,
                        flexShrink: 0,
                        height: 28,
                        borderRadius: 14,
                        backgroundColor: theme().bgActive,
                        alignItems: "center",
                        justifyContent: "center",
                      }}
                    >
                      <Text style={{ color: theme().accent, fontSize: 11, fontWeight: "bold" }}>{`#${item.id}`}</Text>
                    </View>
                    <View style={{ gap: 2, minWidth: 0, flexGrow: 1, flexShrink: 1 }}>
                      <Text
                        style={{
                          color: isSelected() ? theme().accent : theme().textPrimary,
                          fontSize: 13,
                          fontWeight: "medium",
                          minWidth: 0,
                          flexShrink: 1,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        {item.title}
                      </Text>
                      <Text
                        style={{
                          color: theme().textMuted,
                          fontSize: 11,
                          minWidth: 0,
                          flexShrink: 1,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        {`Category: ${item.category} • Latency: ${item.latency}ms`}
                      </Text>
                    </View>
                  </View>
                  <Badge label={item.status} variant={statusVariant} size="sm" />
                </Pressable>
              );
            }}
          />
        </View>
      </Card>

      {/* Props Reference */}
      <Card
        title="VirtualList Props Reference"
        description="All supported properties and callbacks on the VirtualList component."
      >
        <PropTable
          props={[
            {
              name: "data",
              type: "readonly T[]",
              default: "required",
              description: "Array of data items to virtualize",
            },
            {
              name: "itemKey",
              type: "(item: T, idx: number) => string|number",
              default: "required",
              description: "Unique identifier for each item",
            },
            {
              name: "renderItem",
              type: "(item: T, idx: number) => SolidChild",
              default: "required",
              description: "Render function producing child elements",
            },
            {
              name: "estimatedItemSize",
              type: "number",
              default: "required",
              description: "Estimated height in pixels of each item for scroll offset computation",
            },
            {
              name: "overscan",
              type: "number",
              default: "2",
              description: "Number of additional items to render above and below visible viewport",
            },
            {
              name: "emptyState",
              type: "SolidChild",
              default: "null",
              description: "Element rendered when data array is empty",
            },
            {
              name: "onEndReached",
              type: "() => void",
              default: "undefined",
              description: "Fired when user scrolls near the end of the list",
            },
            {
              name: "onScroll",
              type: "ScrollHandler",
              default: "undefined",
              description: "Fired during scrolling with offset metrics",
            },
          ]}
        />
      </Card>

      {/* Code Example */}
      <Card title="Code Example">
        <CodeSnippet
          code={`import { VirtualList, View, Text } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

createComponent(VirtualList, {
  data: items(),
  itemKey: (item) => item.id,
  estimatedItemSize: 48,
  overscan: 4,
  renderItem: (item) =>
    createComponent(View, {
      style: { height: 48, padding: 8 },
      children: item.title,
    }),
  emptyState: createComponent(Text, { children: "No items found" }),
  onEndReached: () => loadMoreItems(),
});`}
        />
      </Card>
    </View>
  );
}
