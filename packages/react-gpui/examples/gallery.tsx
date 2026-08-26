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
  createWindowSizeStore,
  useWindowSize,
  type Style,
  type WindowSizeStore,
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
    width: 800,
    height: 600,
    flexDirection: "column",
    flexGrow: 1,
    gap: 12,
    padding: 20,
    overflow: "scroll",
    backgroundColor: "#f7f8fa",
  },
  header: {
    height: 64,
    flexShrink: 0,
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    gap: 12,
    padding: 12,
    borderWidth: 1,
    borderRadius: 12,
    borderColor: "#d8e0ea",
    backgroundColor: "#ffffff",
  },
  headerCopy: { flexGrow: 1, minWidth: 0, gap: 2 },
  title: { fontSize: 20, lineHeight: 24, fontWeight: "bold", color: "#172033" },
  subtitle: { fontSize: 12, lineHeight: 16, color: "#5b6b7f", minWidth: 0 },
  headerMeta: { flexDirection: "row", alignItems: "center", gap: 8, flexShrink: 0 },
  chip: {
    padding: 6,
    borderRadius: 8,
    backgroundColor: "#eaf1ff",
    color: "#2458b8",
    fontSize: 11,
    fontWeight: "semibold",
  },
  chipMuted: {
    padding: 6,
    borderRadius: 8,
    backgroundColor: "#f1f3f6",
    color: "#5b6b7f",
    fontSize: 11,
    fontWeight: "medium",
  },
  body: {
    width: 0,
    minWidth: 0,
    minHeight: 0,
    flexGrow: 1,
    flexShrink: 1,
    flexDirection: "row",
    alignItems: "stretch",
    gap: 12,
  },
  bodyCompact: {
    minWidth: 0,
    flexDirection: "column",
    alignItems: "stretch",
    gap: 12,
    flexGrow: 0,
    flexShrink: 0,
  },
  panelCompact: {
    minWidth: 0,
    minHeight: 0,
    flexShrink: 0,
    flexDirection: "column",
    gap: 8,
    padding: 12,
    borderWidth: 1,
    borderRadius: 12,
    borderColor: "#d8e0ea",
    backgroundColor: "#ffffff",
  },
  panel: {
    width: 0,
    minWidth: 0,
    minHeight: 0,
    flexGrow: 1,
    flexShrink: 1,
    flexDirection: "column",
    gap: 8,
    padding: 12,
    borderWidth: 1,
    borderRadius: 12,
    borderColor: "#d8e0ea",
    backgroundColor: "#ffffff",
  },
  panelHeader: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    gap: 8,
    minWidth: 0,
    flexShrink: 0,
  },
  panelTitle: { fontSize: 16, lineHeight: 20, fontWeight: "semibold", color: "#172033" },
  panelKicker: { fontSize: 11, lineHeight: 16, color: "#5b6b7f" },
  helper: { fontSize: 12, lineHeight: 18, color: "#5b6b7f", minWidth: 0, lineClamp: 2, textOverflow: "ellipsis" },
  showcase: {
    fontSize: 12,
    lineHeight: 18,
    fontStyle: "italic",
    textDecoration: "underline",
    color: "#5b6b7f",
    minWidth: 0,
    lineClamp: 1,
    textOverflow: "ellipsis",
  },
  card: {
    flexDirection: "column",
    gap: 8,
    padding: 10,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#e2e7ee",
    backgroundColor: "#fbfcfe",
    boxShadow: [
      { offsetX: 0, offsetY: 2, blurRadius: 8, spreadRadius: 0, color: "#17203318" },
      { offsetX: 0, offsetY: 5, blurRadius: 14, spreadRadius: 0, color: "#17203310" },
    ],
  },
  cardRow: { flexDirection: "row", alignItems: "center", gap: 8, minWidth: 0 },
  cardCopy: { flexGrow: 1, minWidth: 0, gap: 2 },
  cardTitle: { fontSize: 13, lineHeight: 18, fontWeight: "semibold", color: "#172033" },
  metadata: { fontSize: 11, lineHeight: 16, color: "#5b6b7f", minWidth: 0, lineClamp: 1, textOverflow: "ellipsis" },
  avatar: { width: 32, height: 32, flexShrink: 0 },
  scrollDemo: {
    height: 72,
    flexShrink: 0,
    flexDirection: "column",
    gap: 4,
    padding: 8,
    overflow: "scroll",
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#e2e7ee",
    backgroundColor: "#ffffff",
  },
  scrollLabel: { fontSize: 11, lineHeight: 16, fontWeight: "semibold", color: "#172033" },
  fieldGroup: { flexDirection: "column", gap: 4, flexShrink: 0 },
  label: { fontSize: 12, lineHeight: 16, fontWeight: "medium", color: "#172033" },
  input: {
    height: 34,
    padding: 8,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#b9c6d6",
    fontSize: 13,
    color: "#172033",
    backgroundColor: "#ffffff",
  },
  actionRow: { flexDirection: "row", alignItems: "center", gap: 8, flexShrink: 0 },
  menuAnchor: { position: "relative", flexShrink: 0 },
  button: {
    minWidth: 96,
    height: 32,
    alignItems: "center",
    justifyContent: "center",
    padding: 8,
    borderRadius: 8,
    backgroundColor: "#2d6cdf",
    cursor: "pointer",
  },
  buttonHover: { backgroundColor: "#2458b8" },
  buttonActive: { backgroundColor: "#1e4a9b" },
  buttonSecondary: {
    minWidth: 96,
    height: 32,
    alignItems: "center",
    justifyContent: "center",
    padding: 8,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#b9c6d6",
    backgroundColor: "#ffffff",
    cursor: "pointer",
  },
  buttonSecondaryHover: { borderColor: "#2d6cdf", backgroundColor: "#f4f7fb" },
  buttonSecondaryActive: { borderColor: "#2458b8", backgroundColor: "#eaf1ff" },
  buttonDisabled: { opacity: 0.45, backgroundColor: "#e7ebf0", cursor: "not-allowed" },
  buttonLabel: { fontSize: 12, lineHeight: 16, fontWeight: "semibold", color: "#ffffff" },
  buttonSecondaryLabel: { fontSize: 12, lineHeight: 16, fontWeight: "semibold", color: "#2458b8" },
  menu: {
    position: "overlay",
    width: 200,
    left: 0,
    top: 36,
    flexDirection: "column",
    gap: 4,
    padding: 8,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#b9c6d6",
    backgroundColor: "#ffffff",
    boxShadow: [
      { offsetX: 0, offsetY: 4, blurRadius: 12, spreadRadius: 0, color: "#17203324" },
      { offsetX: 0, offsetY: 8, blurRadius: 20, spreadRadius: 0, color: "#17203312" },
    ],
  },
  menuTitle: { padding: 4, fontSize: 11, lineHeight: 16, fontWeight: "semibold", color: "#5b6b7f" },
  menuItem: {
    height: 30,
    flexDirection: "row",
    alignItems: "center",
    padding: 6,
    borderRadius: 6,
    backgroundColor: "#ffffff",
    cursor: "pointer",
  },
  menuItemHover: { backgroundColor: "#eaf1ff" },
  menuItemDisabled: { opacity: 0.45, cursor: "not-allowed" },
  menuItemLabel: { fontSize: 12, lineHeight: 16, color: "#172033" },
  activityCardAnchor: { position: "relative", minWidth: 0, flexShrink: 0 },
  activityCard: {
    width: 280,
    maxWidth: 320,
    height: 64,
    flexDirection: "column",
    gap: 4,
    padding: 8,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#d8e0ea",
    backgroundColor: "#ffffff",
  },
  activityCardActive: { width: 320, height: 80, borderColor: "#2d6cdf", backgroundColor: "#eaf1ff" },
  badge: {
    position: "absolute",
    top: -6,
    right: 4,
    padding: 4,
    borderRadius: 6,
    backgroundColor: "#0f8a5f",
    color: "#ffffff",
    fontSize: 10,
    lineHeight: 14,
    fontWeight: "bold",
  },
  list: { height: 320, minWidth: 0, flexShrink: 0 },
  row: {
    minWidth: 0,
    height: 44,
    flexGrow: 1,
    flexShrink: 0,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 8,
    padding: 8,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#e2e7ee",
    backgroundColor: "#fbfcfe",
    cursor: "grab",
  },
  rowDragOver: { borderColor: "#2d6cdf", backgroundColor: "#eaf1ff" },
  rowCopy: { flexGrow: 1, minWidth: 0, gap: 2 },
  rowTitle: { fontSize: 13, lineHeight: 18, fontWeight: "medium", color: "#172033", minWidth: 0 },
  rowDetail: { fontSize: 11, lineHeight: 16, color: "#5b6b7f", minWidth: 0 },
  rowStatus: { padding: 4, borderRadius: 6, backgroundColor: "#eaf6f1", color: "#0f8a5f", fontSize: 10 },
  rowStatusWaiting: { backgroundColor: "#f1f3f6", color: "#5b6b7f" },
  listActions: { flexDirection: "row", alignItems: "center", gap: 8, flexShrink: 0 },
  emptyState: { padding: 12, fontSize: 12, lineHeight: 18, color: "#5b6b7f" },
});
function Gallery({ windowSizeStore }: { windowSizeStore: WindowSizeStore }) {
  const { width, height } = useWindowSize(windowSizeStore);
  const compact = width < 1100;
  const screenStyle = { ...styles.screen, width, height };
  const bodyStyle = compact ? styles.bodyCompact : styles.body;
  const panelStyle = compact ? styles.panelCompact : styles.panel;

  const [query, setQuery] = useState("");
  const [active, setActive] = useState(false);
  const [activityRows, setActivityRows] = useState(rows);
  const [dragOverId, setDragOverId] = useState<number | null>(null);
  const [layout, setLayout] = useState("800×600");
  const [externalDrop, setExternalDrop] = useState("none");
  const [pointer, setPointer] = useState("none");
  const [menuHovered, setMenuHovered] = useState(false);
  const [activateHovered, setActivateHovered] = useState(false);
  const [recordHovered, setRecordHovered] = useState(false);
  const [scrollDelta, setScrollDelta] = useState(0);
  const [menuOpen, setMenuOpen] = useState(false);
  const [menuHover, setMenuHover] = useState(false);
  const [presses, setPresses] = useState(0);
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
  const activityStyle = useMemo<Style>(
    () => ({
      ...styles.activityCard,
      ...(active ? styles.activityCardActive : {}),
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
  const menuButtonStyle = {
    ...styles.button,
    ...(menuHovered ? styles.buttonHover : {}),
    ...(pointer === "menu-down" ? styles.buttonActive : {}),
  };
  const activateButtonStyle = {
    ...styles.button,
    ...(activateHovered ? styles.buttonHover : {}),
    ...(pointer === "activate-down" ? styles.buttonActive : {}),
  };
  const recordButtonStyle = {
    ...styles.buttonSecondary,
    ...(recordHovered ? styles.buttonSecondaryHover : {}),
    ...(pointer === "record-down" ? styles.buttonSecondaryActive : {}),
  };
  const menuItemStyle = { ...styles.menuItem, ...(menuHover ? styles.menuItemHover : {}) };

  return (
    <View
      style={screenStyle}
      accessibilityRole="generic"
      accessibilityLabel="React GPUI capability gallery"
      onLayout={(frame) => setLayout(`${Math.round(frame.width)}×${Math.round(frame.height)}`)}
    >
      <View style={styles.header}>
        <View style={styles.headerCopy}>
          <Text style={styles.title}>React GPUI Gallery</Text>
          <Text style={styles.subtitle}>Native controls, scroll, overlays, drag reorder, and text rendering.</Text>
        </View>
        <View style={styles.headerMeta}>
          <Text style={styles.chip}>Presses {presses}</Text>
          <Text style={styles.chipMuted}>{layout}</Text>
          <Text style={styles.chipMuted}>{pointer === "none" ? "Idle" : pointer}</Text>
        </View>
      </View>
      <View
        style={bodyStyle}
        onExternalFileDrop={(paths) => setExternalDrop(`${paths.length} file${paths.length === 1 ? "" : "s"}`)}
      >
        <View style={panelStyle}>
          <View style={styles.panelHeader}>
            <Text style={styles.panelTitle}>Compose</Text>
            <Text style={styles.panelKicker}>Inputs & surfaces</Text>
          </View>
          <Text style={styles.helper}>
            A compact surface for native text, fallback images, wheel input, and overlays.
          </Text>
          <Text style={styles.showcase}>Margins, bounds, italic, underline, and line height.</Text>
          <View style={styles.card}>
            <View style={styles.cardRow}>
              <View style={styles.cardCopy}>
                <Text style={styles.cardTitle}>Fallback avatar</Text>
                <Text style={styles.metadata}>Missing source resolves to a local fallback.</Text>
              </View>
              <Image
                source={MISSING_AVATAR_SOURCE}
                fallbackSource={FALLBACK_AVATAR_SOURCE}
                objectFit="contain"
                style={styles.avatar}
                accessibilityLabel="Fallback avatar"
              />
            </View>
            <Text style={styles.metadata}>Two low-noise shadow layers on one card.</Text>
          </View>
          <View style={styles.scrollDemo} onScroll={(event) => setScrollDelta(Math.round(event.dy))}>
            <Text style={styles.scrollLabel}>Wheel region</Text>
            <Text style={styles.metadata}>Scroll delta: {scrollDelta}</Text>
            <Text style={styles.metadata}>Native wheel events remain semantic notifications.</Text>
          </View>
          <View style={styles.fieldGroup}>
            <Text style={styles.label}>Filter activity</Text>
            <TextInput
              style={styles.input}
              value={query}
              placeholder="Search activities…"
              onChangeText={setQuery}
              accessibilityLabel="Activity filter"
              accessibilityRole="textbox"
            />
          </View>
          <View style={styles.actionRow}>
            <View style={styles.menuAnchor}>
              <Pressable
                style={menuButtonStyle}
                focusable
                accessibilityRole="button"
                accessibilityLabel={menuOpen ? "Hide activity menu" : "Show activity menu"}
                onPress={() => setMenuOpen((value) => !value)}
                onPointerDown={() => setPointer("menu-down")}
                onPointerUp={() => setPointer("menu-up")}
                onHoverChange={setMenuHovered}
                onKeyDown={({ key, action }) => {
                  if (key === "escape" && action === "down") setMenuOpen(false);
                }}
              >
                <Text style={styles.buttonLabel}>{menuOpen ? "Hide menu" : "Show menu"}</Text>
              </Pressable>
              {menuOpen ? (
                <View style={styles.menu} onPointerDown={() => setPointer("menu-down")}>
                  <Text style={styles.menuTitle}>Activity actions</Text>
                  <Pressable
                    style={menuItemStyle}
                    focusable
                    accessibilityRole="button"
                    accessibilityLabel="Refresh activity"
                    onPress={() => {
                      setMenuOpen(false);
                      setPointer("refresh");
                    }}
                    onHoverChange={setMenuHover}
                  >
                    <Text style={styles.menuItemLabel}>Refresh activity</Text>
                  </Pressable>
                  <Pressable
                    style={{ ...styles.menuItem, ...styles.menuItemDisabled }}
                    focusable
                    disabled
                    accessibilityRole="button"
                    accessibilityLabel="Export activity disabled"
                    accessibilityDisabled
                  >
                    <Text style={styles.menuItemLabel}>Export (disabled)</Text>
                  </Pressable>
                </View>
              ) : null}
            </View>
            <Pressable
              style={activateButtonStyle}
              focusable
              accessibilityRole="button"
              accessibilityLabel={active ? "Deactivate live panel" : "Activate live panel"}
              onPress={() => setActive((value) => !value)}
              onHoverChange={setActivateHovered}
              onPointerDown={() => setPointer("activate-down")}
              onPointerUp={() => setPointer("activate-up")}
            >
              <Text style={styles.buttonLabel}>{active ? "Deactivate" : "Activate"}</Text>
            </Pressable>
          </View>
          <View style={styles.activityCardAnchor}>
            <View style={activityStyle}>
              <Text style={styles.cardTitle}>{active ? "Live panel" : "Standby panel"}</Text>
              <Text style={styles.metadata}>Opacity, color, width, and height transition together.</Text>
            </View>
            <Text style={styles.badge}>LIVE</Text>
          </View>
          <Text style={styles.metadata}>Drop: {externalDrop}</Text>
        </View>
        <View style={panelStyle}>
          <View style={styles.panelHeader}>
            <Text style={styles.panelTitle}>Activity</Text>
            <Text style={styles.panelKicker}>
              {visibleRows.length} of {activityRows.length}
            </Text>
          </View>
          <VirtualList<Row>
            style={styles.list}
            data={visibleRows}
            itemKey={(row) => row.id}
            emptyState={<Text style={styles.emptyState}>No activities match this filter.</Text>}
            renderItem={(row) => (
              <View
                style={{ ...styles.row, ...(dragOverId === row.id ? styles.rowDragOver : {}) }}
                draggable={{ type: `activity:${row.id}`, data: row }}
                onDragOver={(type) => {
                  if (type.startsWith("activity:")) setDragOverId(row.id);
                }}
                onDrop={(type) => moveActivity(row.id, type)}
              >
                <View style={styles.rowCopy}>
                  <Text style={styles.rowTitle}>{row.title}</Text>
                  <Text style={styles.rowDetail}>{row.detail}</Text>
                </View>
                <Text style={{ ...styles.rowStatus, ...(row.id % 2 === 0 ? {} : styles.rowStatusWaiting) }}>
                  {row.id % 2 === 0 ? "Ready" : "Waiting"}
                </Text>
              </View>
            )}
            estimatedItemSize={48}
            overscan={2}
            initialNumToRender={6}
          />
          <View style={styles.listActions}>
            <Pressable
              style={recordButtonStyle}
              focusable
              accessibilityRole="button"
              accessibilityLabel="Record press"
              onHoverChange={setRecordHovered}
              onPointerDown={() => setPointer("record-down")}
              onPointerUp={() => setPointer("record-up")}
              onPress={() => setPresses((value) => value + 1)}
            >
              <Text style={styles.buttonSecondaryLabel}>Record press</Text>
            </Pressable>
            <Text style={styles.metadata}>Drag rows to reorder</Text>
          </View>
        </View>
      </View>
    </View>
  );
}

const windowSizeStore = createWindowSizeStore({ width: 800, height: 600 });
const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onWindowResize: (width, height, scaleFactor) => windowSizeStore.set(width, height, scaleFactor),
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<Gallery windowSizeStore={windowSizeStore} />);
