import { useMemo, useState } from "react";
import {
  Image,
  Pressable,
  StdioTransport,
  StyleSheet,
  Text,
  TextInput,
  View,
  VirtualList,
  createProcessTerminationHandler,
  createRoot,
  type Style,
} from "../src/index";

type Row = { id: number; title: string; detail: string };

const rows: Row[] = Array.from({ length: 32 }, (_, id) => ({
  id,
  title: `Activity ${id + 1}`,
  detail: id % 2 === 0 ? "Synchronized" : "Waiting for input",
}));
const FALLBACK_AVATAR_SOURCE = new URL("./todo-avatar.svg", import.meta.url).pathname;
const MISSING_AVATAR_SOURCE = new URL("./missing-avatar.svg", import.meta.url).pathname;

const styles = StyleSheet.create({
  screen: {
    flexDirection: "column",
    flexGrow: 1,
    gap: 16,
    padding: 24,
    backgroundColor: "#f4f7fb",
  },
  header: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    padding: 16,
    borderWidth: 1,
    borderRadius: 12,
    borderColor: "#d6deea",
    backgroundColor: "#ffffff",
  },
  title: { fontSize: 24, fontWeight: "bold", color: "#14213d" },
  subtitle: { fontSize: 14, color: "#52627a" },
  panels: { flexDirection: "row", flexGrow: 1, gap: 16, alignItems: "stretch" },
  panel: {
    flexDirection: "column",
    flexGrow: 1,
    gap: 12,
    padding: 16,
    borderWidth: 1,
    borderRadius: 12,
    borderColor: "#d6deea",
    backgroundColor: "#ffffff",
  },
  panelTitle: { fontSize: 18, fontWeight: "semibold", color: "#14213d" },
  label: { fontSize: 13, fontWeight: "medium", color: "#52627a" },
  shadowCard: {
    gap: 6,
    padding: 12,
    borderRadius: 10,
    backgroundColor: "#ffffff",
    boxShadow: [
      { offsetX: 0, offsetY: 2, blurRadius: 6, spreadRadius: 0, color: "#14213d26" },
      { offsetX: 0, offsetY: 8, blurRadius: 18, spreadRadius: 0, color: "#14213d18" },
    ],
  },
  showcase: {
    marginTop: 4,
    marginRight: 8,
    marginBottom: 4,
    marginLeft: 8,
    minWidth: 180,
    maxWidth: 320,
    minHeight: 24,
    maxHeight: 56,
    flexShrink: 0,
    alignSelf: "center",
    fontStyle: "italic",
    textDecoration: "underline",
    lineHeight: 20,
    color: "#52627a",
  },
  input: {
    padding: 10,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#b8c5d8",
    fontSize: 16,
    color: "#14213d",
    backgroundColor: "#fbfcfe",
  },
  button: {
    alignItems: "center",
    padding: 10,
    borderRadius: 8,
    backgroundColor: "#2d6cdf",
  },
  buttonLabel: { fontWeight: "bold", color: "#ffffff" },
  badgeAnchor: { position: "relative", minHeight: 100 },
  badge: {
    position: "absolute",
    top: -8,
    right: -8,
    padding: 4,
    borderRadius: 4,
    backgroundColor: "#2d6cdf",
    color: "#ffffff",
    fontSize: 11,
    fontWeight: "bold",
  },
  menuAnchor: { position: "relative", alignSelf: "flex-start" },
  menu: {
    position: "overlay",
    top: 42,
    left: 0,
    width: 180,
    padding: 8,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#b8c5d8",
    backgroundColor: "#ffffff",
  },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    padding: 10,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#e1e7f0",
    backgroundColor: "#fbfcfe",
  },
  rowTitle: { fontSize: 15, fontWeight: "medium", color: "#14213d" },
  rowDetail: { fontSize: 12, color: "#687a93" },
});

function Gallery() {
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(false);
  const [activityRows, setActivityRows] = useState(rows);
  const [dragOverId, setDragOverId] = useState<number | null>(null);
  const [layout, setLayout] = useState("not measured");
  const [externalDrop, setExternalDrop] = useState("none");
  const [pointer, setPointer] = useState("none");
  const [hovered, setHovered] = useState(false);
  const [scrollDelta, setScrollDelta] = useState(0);
  const moveActivity = (targetId: number, dragType: string) => {
    const prefix = "activity:";
    if (!dragType.startsWith(prefix)) return;
    const sourceId = Number(dragType.slice(prefix.length));
    if (!Number.isInteger(sourceId) || sourceId === targetId) return;
    setActivityRows((current) => {
      const sourceIndex = current.findIndex((row) => row.id === sourceId);
      const targetIndex = current.findIndex((row) => row.id === targetId);
      if (sourceIndex < 0 || targetIndex < 0) return current;
      const next = [...current];
      const [source] = next.splice(sourceIndex, 1);
      next.splice(targetIndex, 0, source);
      return next;
    });
    setDragOverId(null);
  };
  const [menuOpen, setMenuOpen] = useState(false);
  const [presses, setPresses] = useState(0);
  const activityStyle = useMemo<Style>(
    () => ({
      opacity: active ? 1 : 0.72,
      width: active ? 360 : 260,
      height: active ? 100 : 64,
      borderWidth: 1,
      borderRadius: 12,
      borderColor: active ? "#2d6cdf" : "#d6deea",
      backgroundColor: active ? "#eef4ff" : "#ffffff",
      transition: {
        durationMs: 180,
        easing: "easeOut",
        properties: ["opacity", "backgroundColor", "width", "height"],
      },
    }),
    [active],
  );
  const visibleRows =
    query.trim() === ""
      ? activityRows
      : activityRows.filter((row) => row.title.toLowerCase().includes(query.toLowerCase()));

  return (
    <View style={styles.screen} accessibilityRole="generic" accessibilityLabel="React GPUI capability gallery">
      <View style={styles.header}>
        <View>
          <Text style={styles.title}>React GPUI Gallery</Text>
          <Text style={styles.subtitle}>Native panels, input, lists, events, and transitions.</Text>
        </View>
        <Text style={styles.subtitle}>Presses: {presses}</Text>
        <Text style={styles.subtitle}>
          Layout: {layout}; pointer: {pointer}; hover: {hovered ? "yes" : "no"}; dropped: {externalDrop}
        </Text>
      </View>
      <View style={styles.panels}>
        <View
          style={styles.panel}
          onLayout={(frame) => setLayout(`${Math.round(frame.width)}x${Math.round(frame.height)}`)}
          onExternalFileDrop={(paths) => setExternalDrop(`${paths.length} file(s)`)}
        >
          <Text style={styles.panelTitle}>Compose</Text>
          <Text style={styles.showcase}>Margins, bounds, italic, underline, and line height.</Text>
          <View style={styles.shadowCard}>
            <Text style={styles.panelTitle}>Native card shadow</Text>
            <Text style={styles.subtitle}>One card can carry two GPUI box-shadow layers.</Text>
            <Image
              source={MISSING_AVATAR_SOURCE}
              fallbackSource={FALLBACK_AVATAR_SOURCE}
              objectFit="contain"
              style={{ width: 40, height: 40 }}
              accessibilityLabel="Fallback avatar"
            />
          </View>
          <View
            style={{ height: 72, flexShrink: 0, overflow: "scroll", borderWidth: 1, padding: 8 }}
            onScroll={(event) => setScrollDelta(Math.round(event.dy))}
          >
            <Text>Scroll this panel to exercise View onScroll notifications.</Text>
            <Text>Scroll delta: {scrollDelta}</Text>
            <Text>Native wheel events remain semantic notifications.</Text>
          </View>
          <Text style={styles.label}>Filter activity</Text>
          <TextInput
            style={styles.input}
            value={query}
            placeholder="Search activity"
            onChangeText={setQuery}
            accessibilityLabel="Activity filter"
          />
          <View style={styles.menuAnchor}>
            <Pressable
              style={styles.button}
              onPress={() => setMenuOpen((value) => !value)}
              onPointerDown={() => setPointer("down")}
              onPointerUp={() => setPointer("up")}
              onHoverChange={setHovered}
            >
              <Text style={styles.buttonLabel}>{menuOpen ? "Hide menu" : "Show menu"}</Text>
            </Pressable>
            {menuOpen ? (
              <View style={styles.menu} onPointerDown={() => setPointer("menu-down")}>
                <Text style={styles.rowTitle}>Overlay menu</Text>
                <Text style={styles.rowDetail}>Anchored with top/left offsets.</Text>
              </View>
            ) : null}
          </View>
          <Pressable style={styles.button} onPress={() => setActive((value) => !value)}>
            <Text style={styles.buttonLabel}>{active ? "Deactivate" : "Activate"} transition</Text>
          </Pressable>
          <View style={styles.badgeAnchor}>
            <View style={activityStyle}>
              <Text style={styles.panelTitle}>{active ? "Live panel" : "Standby panel"}</Text>
              <Text style={styles.subtitle}>This panel animates opacity and background color.</Text>
            </View>
            <Text style={styles.badge}>LIVE</Text>
          </View>
        </View>
        <View style={styles.panel}>
          <Text style={styles.panelTitle}>Virtual activity (drag to reorder)</Text>
          <VirtualList<Row>
            style={{ height: 360, flexGrow: 0 }}
            data={visibleRows}
            itemKey={(row) => row.id}
            renderItem={(row) => (
              <View
                style={{
                  ...styles.row,
                  borderColor: dragOverId === row.id ? "#2d6cdf" : "#e1e7f0",
                }}
                draggable={{ type: `activity:${row.id}`, data: row }}
                onDragOver={(type) => {
                  if (type.startsWith("activity:")) setDragOverId(row.id);
                }}
                onDrop={(type) => moveActivity(row.id, type)}
              >
                <Text style={styles.rowTitle}>{row.title}</Text>
                <Text style={styles.rowDetail}>{row.detail}</Text>
              </View>
            )}
            estimatedItemSize={52}
            overscan={3}
            initialNumToRender={8}
          />
          <Pressable style={styles.button} onPress={() => setPresses((value) => value + 1)}>
            <Text style={styles.buttonLabel}>Record press</Text>
          </Pressable>
        </View>
      </View>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<Gallery />);
