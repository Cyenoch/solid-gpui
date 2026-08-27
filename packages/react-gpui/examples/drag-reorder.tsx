import { useMemo, useState } from "react";
import {
  StdioTransport,
  StyleSheet,
  Text,
  View,
  VirtualList,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
  type AppearanceStore,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

type Activity = {
  readonly id: number;
  readonly title: string;
  readonly detail: string;
};

const INITIAL_ACTIVITIES: Activity[] = [
  { id: 1, title: "Prepare release notes", detail: "Documentation" },
  { id: 2, title: "Review pull requests", detail: "Engineering" },
  { id: 3, title: "Run accessibility pass", detail: "Quality" },
  { id: 4, title: "Publish preview build", detail: "Release" },
  { id: 5, title: "Send team update", detail: "Communication" },
];

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", flexGrow: 1, gap: 12, padding: 20, backgroundColor: theme.canvas },
    title: { fontSize: 18, lineHeight: 24, fontWeight: "bold", color: theme.text },
    helper: { fontSize: 12, lineHeight: 18, color: theme.textMuted },
    list: { height: 320, flexShrink: 0 },
    rowSlot: { height: 56, flexShrink: 0 },
    row: {
      height: 48,
      flexDirection: "column",
      justifyContent: "center",
      gap: 2,
      padding: 8,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.borderSubtle,
      backgroundColor: theme.surface,
      cursor: "grab",
    },
    rowDragOver: { borderColor: theme.focusRing, backgroundColor: theme.accentSoft },
    rowTitle: { fontSize: 13, lineHeight: 18, fontWeight: "semibold", color: theme.text },
    rowDetail: { fontSize: 11, lineHeight: 16, color: theme.textMuted },
    footer: { fontSize: 12, lineHeight: 18, color: theme.textMuted },
  });
}

function dragSourceId(dragType: string): number | null {
  const prefix = "activity:";
  if (!dragType.startsWith(prefix)) return null;
  const id = Number(dragType.slice(prefix.length));
  return Number.isInteger(id) ? id : null;
}

function DragReorder({ appearanceStore }: { readonly appearanceStore: AppearanceStore }) {
  const [activities, setActivities] = useState(INITIAL_ACTIVITIES);
  const [dragOverId, setDragOverId] = useState<number | null>(null);
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);

  const moveActivity = (targetId: number, dragType: string) => {
    const sourceId = dragSourceId(dragType);
    if (sourceId === null || sourceId === targetId) {
      setDragOverId(null);
      return;
    }
    setActivities((current) => {
      const sourceIndex = current.findIndex((activity) => activity.id === sourceId);
      const targetIndex = current.findIndex((activity) => activity.id === targetId);
      if (sourceIndex < 0 || targetIndex < 0) return current;
      const next = [...current];
      const [source] = next.splice(sourceIndex, 1);
      if (source === undefined) return current;
      next.splice(targetIndex, 0, source);
      return next;
    });
    setDragOverId(null);
  };

  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Drag reorder example">
      <Text style={styles.title}>Drag to reorder activities</Text>
      <Text style={styles.helper}>
        Drag a row over another row, then release. The native host supplies a compact neutral “Moving item” preview.
      </Text>
      <VirtualList<Activity>
        style={styles.list}
        data={activities}
        itemKey={(activity) => activity.id}
        estimatedItemSize={56}
        initialNumToRender={activities.length}
        renderItem={(activity) => (
          <View style={styles.rowSlot}>
            <View
              style={{ ...styles.row, ...(dragOverId === activity.id ? styles.rowDragOver : {}) }}
              draggable={{ type: `activity:${activity.id}`, data: activity }}
              onDragOver={(dragType) => {
                const sourceId = dragSourceId(dragType);
                if (sourceId !== null && sourceId !== activity.id) setDragOverId(activity.id);
              }}
              onDrop={(dragType) => moveActivity(activity.id, dragType)}
            >
              <Text style={styles.rowTitle}>{activity.title}</Text>
              <Text style={styles.rowDetail}>{activity.detail}</Text>
            </View>
          </View>
        )}
      />
      <Text style={styles.footer}>Drop target: {dragOverId === null ? "none" : `Activity ${dragOverId}`}</Text>
    </View>
  );
}

const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<DragReorder appearanceStore={appearanceStore} />);
