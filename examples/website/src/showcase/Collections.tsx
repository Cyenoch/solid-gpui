import { View, VirtualList } from "@solid-gpui/core";
import * as N from "@solid-gpui/core/components";
import { createMemo, createSignal } from "@solid-gpui/core/runtime";
import { Button, Copy, colors } from "../ui";
import { t } from "../i18n";
const card = {
  padding: 24,
  gap: 20,
  borderWidth: 1,
  borderColor: colors.line,
  borderRadius: 12,
  backgroundColor: colors.panel,
  minWidth: 0,
  flexShrink: 0,
} as const;
export function createCollectionsState() {
  const [query, setQuery] = createSignal("");
  const [selected, setSelected] = createSignal<number>();
  return { query, setQuery, selected, setSelected };
}
export function Collections(props: { state: ReturnType<typeof createCollectionsState> }) {
  const { query, setQuery, selected, setSelected } = props.state;
  const items = Array.from({ length: 10000 }, (_, index) => ({
    id: index + 1,
    title: `Collection ${String(index + 1).padStart(5, "0")}`,
  }));
  const visible = createMemo(() => items.filter((item) => item.title.toLowerCase().includes(query().toLowerCase())));
  return (
    <View style={card}>
      <Copy size={20}>Room for everything.</Copy>
      <Copy color={colors.muted}>Browse ten thousand collections. Find one in a moment.</Copy>
      <N.Input value={query()} onChange={(event) => setQuery(event.value)} placeholder={t("Search collections…")} />
      <Copy size={13} color={colors.muted}>{`${visible().length} ${t("collections")}`}</Copy>
      <VirtualList
        data={visible()}
        itemKey={(item) => item.id}
        estimatedItemSize={48}
        style={{ height: 360 }}
        renderItem={(item) => (
          <Button
            ghost
            icon="lucide:box"
            active={selected() === item.id}
            onPress={() => setSelected(item.id)}
            style={{ height: 48 }}
          >
            {item.title}
          </Button>
        )}
        emptyState={<Copy>No matching collections.</Copy>}
      />
    </View>
  );
}
