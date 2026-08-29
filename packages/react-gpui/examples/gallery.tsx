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
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  createWindowSizeStore,
  useAppearance,
  useWindowSize,
  type AppearanceStore,
  type Style,
  type WindowSizeStore,
} from "../src/index";
import { useTheme, type Theme } from "./theme";
type Row = { id: number; title: string; detail: string };

const rows: Row[] = Array.from({ length: 32 }, (_, id) => ({
  id,
  title: `Activity ${id + 1}`,
  detail: id % 2 === 0 ? "Synchronized" : "Waiting for input",
}));
const FALLBACK_AVATAR_SOURCE = new URL("./todo-avatar.svg", import.meta.url).pathname;
const MISSING_AVATAR_SOURCE = new URL("./missing-avatar.svg", import.meta.url).pathname;

function createStyles(theme: Theme) {
  return StyleSheet.create({
    screen: {
      width: 800,
      height: 600,
      flexDirection: "column",
      flexGrow: 1,
      gap: 12,
      padding: 20,
      overflow: "scroll",
      backgroundColor: theme.canvas,
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
      borderColor: theme.border,
      backgroundColor: theme.surface,
    },
    headerCopy: { flexGrow: 1, minWidth: 0, gap: 2 },
    title: { fontSize: 20, lineHeight: 24, fontWeight: "bold", color: theme.text },
    subtitle: { fontSize: 12, lineHeight: 16, color: theme.textMuted, minWidth: 0 },
    headerMeta: { flexDirection: "row", alignItems: "center", gap: 8, flexShrink: 0 },
    chip: {
      padding: 6,
      borderRadius: 8,
      backgroundColor: theme.accentSoft,
      color: theme.accentText,
      fontSize: 11,
      fontWeight: "semibold",
    },
    chipMuted: {
      padding: 6,
      borderRadius: 8,
      backgroundColor: theme.surfaceSubtle,
      color: theme.textMuted,
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
      borderColor: theme.border,
      backgroundColor: theme.surface,
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
      borderColor: theme.border,
      backgroundColor: theme.surface,
    },
    panelHeader: {
      flexDirection: "row",
      justifyContent: "space-between",
      alignItems: "center",
      gap: 8,
      minWidth: 0,
      flexShrink: 0,
    },
    panelTitle: { fontSize: 16, lineHeight: 20, fontWeight: "semibold", color: theme.text },
    panelKicker: { fontSize: 11, lineHeight: 16, color: theme.textMuted },
    helper: {
      fontSize: 12,
      lineHeight: 18,
      color: theme.textMuted,
      minWidth: 0,
      lineClamp: 2,
      textOverflow: "ellipsis",
    },
    showcase: {
      fontSize: 12,
      lineHeight: 18,
      fontStyle: "italic",
      textDecoration: "underline",
      color: theme.textMuted,
      minWidth: 0,
      lineClamp: 1,
      textOverflow: "ellipsis",
    },
    richTextParagraph: { fontSize: 13, lineHeight: 19, color: theme.text, minWidth: 0 },
    richTextBold: { color: theme.accentText, fontWeight: "bold" },
    richTextItalic: { color: theme.success, fontStyle: "italic" },
    richTextStrike: { color: theme.danger, textDecoration: "lineThrough" },
    richTextUnderline: { color: theme.accentText, textDecoration: "underline" },
    richTextLink: { color: theme.accentText, fontWeight: "semibold", textDecoration: "underline" },
    richTextSelectable: { fontSize: 12, lineHeight: 18, color: theme.text, minWidth: 0 },
    richTextSelectableAccent: { color: theme.accentText, fontWeight: "semibold" },
    richTextSelectableMuted: { color: theme.textMuted, fontStyle: "italic" },
    richTextCaption: { fontSize: 11, lineHeight: 16, color: theme.textSubtle },
    card: {
      flexDirection: "column",
      gap: 8,
      padding: 10,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.borderSubtle,
      backgroundColor: theme.surfaceRaised,
      boxShadow: [
        { offsetX: 0, offsetY: 2, blurRadius: 8, spreadRadius: 0, color: theme.shadow },
        { offsetX: 0, offsetY: 5, blurRadius: 14, spreadRadius: 0, color: theme.shadowSoft },
      ],
    },
    cardRow: { flexDirection: "row", alignItems: "center", gap: 8, minWidth: 0 },
    cardCopy: { flexGrow: 1, minWidth: 0, gap: 2 },
    cardTitle: { fontSize: 13, lineHeight: 18, fontWeight: "semibold", color: theme.text },
    metadata: {
      fontSize: 11,
      lineHeight: 16,
      color: theme.textMuted,
      minWidth: 0,
      lineClamp: 1,
      textOverflow: "ellipsis",
    },
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
      borderColor: theme.borderSubtle,
      backgroundColor: theme.surface,
    },
    scrollLabel: { fontSize: 11, lineHeight: 16, fontWeight: "semibold", color: theme.text },
    fieldGroup: { flexDirection: "column", gap: 4, flexShrink: 0 },
    label: { fontSize: 12, lineHeight: 16, fontWeight: "medium", color: theme.text },
    input: {
      height: 34,
      padding: 8,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.borderInput,
      fontSize: 13,
      color: theme.text,
      backgroundColor: theme.input,
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
      backgroundColor: theme.accent,
      cursor: "pointer",
    },
    buttonHover: { backgroundColor: theme.accentHover },
    buttonActive: { backgroundColor: theme.accentPressed },
    buttonSecondary: {
      minWidth: 96,
      height: 32,
      alignItems: "center",
      justifyContent: "center",
      padding: 8,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.borderInput,
      backgroundColor: theme.surface,
      cursor: "pointer",
    },
    buttonSecondaryHover: { borderColor: theme.focusRing, backgroundColor: theme.accentSoftHover },
    buttonSecondaryActive: { borderColor: theme.accentText, backgroundColor: theme.accentSoftPressed },
    buttonDisabled: { opacity: 0.45, backgroundColor: theme.disabled, cursor: "not-allowed" },
    buttonLabel: { fontSize: 12, lineHeight: 16, fontWeight: "semibold", color: theme.onAccent },
    buttonSecondaryLabel: { fontSize: 12, lineHeight: 16, fontWeight: "semibold", color: theme.accentText },
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
      borderColor: theme.borderInput,
      backgroundColor: theme.surface,
      boxShadow: [
        { offsetX: 0, offsetY: 4, blurRadius: 12, spreadRadius: 0, color: theme.shadowStrong },
        { offsetX: 0, offsetY: 8, blurRadius: 20, spreadRadius: 0, color: theme.shadowSoft },
      ],
    },
    menuTitle: { padding: 4, fontSize: 11, lineHeight: 16, fontWeight: "semibold", color: theme.textMuted },
    menuItem: {
      height: 30,
      flexDirection: "row",
      alignItems: "center",
      padding: 6,
      borderRadius: 6,
      backgroundColor: theme.surface,
      cursor: "pointer",
    },
    menuItemHover: { backgroundColor: theme.accentSoft },
    menuItemDisabled: { opacity: 0.45, backgroundColor: theme.disabled, cursor: "not-allowed" },
    menuItemLabel: { fontSize: 12, lineHeight: 16, color: theme.text },
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
      borderColor: theme.border,
      backgroundColor: theme.surface,
    },
    activityCardActive: {
      width: 320,
      height: 80,
      borderColor: theme.focusRing,
      backgroundColor: theme.accentSoft,
    },
    badge: {
      position: "absolute",
      top: -6,
      right: 4,
      padding: 4,
      borderRadius: 6,
      backgroundColor: theme.success,
      color: theme.onAccent,
      fontSize: 10,
      lineHeight: 14,
      fontWeight: "bold",
    },
    list: { height: 320, minWidth: 0, flexShrink: 0 },
    activityRowSlot: {
      height: 52,
      minWidth: 0,
      flexShrink: 0,
      flexDirection: "column",
    },
    row: {
      minWidth: 0,
      height: 44,
      flexGrow: 0,
      flexDirection: "row",
      alignItems: "center",
      justifyContent: "space-between",
      gap: 8,
      padding: 8,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.borderSubtle,
      backgroundColor: theme.surfaceRaised,
      cursor: "grab",
    },
    rowDragOver: { borderColor: theme.focusRing, backgroundColor: theme.accentSoft },
    rowCopy: { flexGrow: 1, minWidth: 0, gap: 2 },
    rowTitle: { fontSize: 13, lineHeight: 18, fontWeight: "medium", color: theme.text, minWidth: 0 },
    rowDetail: { fontSize: 11, lineHeight: 16, color: theme.textMuted, minWidth: 0 },
    rowStatus: { padding: 4, borderRadius: 6, backgroundColor: theme.successSoft, color: theme.success, fontSize: 10 },
    rowStatusWaiting: { backgroundColor: theme.surfaceSubtle, color: theme.textMuted },
    listActions: { flexDirection: "row", alignItems: "center", gap: 8, flexShrink: 0 },
    emptyState: { padding: 12, fontSize: 12, lineHeight: 18, color: theme.textMuted },
  });
}
function Gallery({
  windowSizeStore,
  appearanceStore,
}: {
  readonly windowSizeStore: WindowSizeStore;
  readonly appearanceStore: AppearanceStore;
}) {
  const { width, height } = useWindowSize(windowSizeStore);
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
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
  const [richTextLinkActivations, setRichTextLinkActivations] = useState(0);
  const [menuFocused, setMenuFocused] = useState(false);
  const [activateFocused, setActivateFocused] = useState(false);
  const [recordFocused, setRecordFocused] = useState(false);
  const [menuItemFocused, setMenuItemFocused] = useState(false);
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
    ...(menuHovered || menuFocused ? styles.buttonHover : {}),
    ...(pointer === "menu-down" ? styles.buttonActive : {}),
  };
  const activateButtonStyle = {
    ...styles.button,
    ...(activateHovered || activateFocused ? styles.buttonHover : {}),
    ...(pointer === "activate-down" ? styles.buttonActive : {}),
  };
  const recordButtonStyle = {
    ...styles.buttonSecondary,
    ...(recordHovered || recordFocused ? styles.buttonSecondaryHover : {}),
    ...(pointer === "record-down" ? styles.buttonSecondaryActive : {}),
  };
  const menuItemStyle = {
    ...styles.menuItem,
    ...(menuHover || menuItemFocused ? styles.menuItemHover : {}),
  };

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
                accessibilityExpanded={menuOpen}
                onPress={() => setMenuOpen((value) => !value)}
                onPointerDown={() => setPointer("menu-down")}
                onPointerUp={() => setPointer("menu-up")}
                onHoverChange={setMenuHovered}
                onFocus={() => setMenuFocused(true)}
                onBlur={() => setMenuFocused(false)}
                onKeyDown={({ key, action }) => {
                  if (key === "escape" && action === "down") setMenuOpen(false);
                }}
              >
                <Text style={styles.buttonLabel}>{menuOpen ? "Hide menu" : "Show menu"}</Text>
              </Pressable>
              {menuOpen ? (
                <View
                  style={styles.menu}
                  onPointerDownOutside={() => setMenuOpen(false)}
                  onPointerDown={() => setPointer("menu-down")}
                >
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
                    onFocus={() => setMenuItemFocused(true)}
                    onBlur={() => setMenuItemFocused(false)}
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
              onFocus={() => setActivateFocused(true)}
              onBlur={() => setActivateFocused(false)}
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
              <View style={styles.activityRowSlot}>
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
              </View>
            )}
            estimatedItemSize={52}
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
              onFocus={() => setRecordFocused(true)}
              onBlur={() => setRecordFocused(false)}
              onPointerDown={() => setPointer("record-down")}
              onPointerUp={() => setPointer("record-up")}
              onPress={() => setPresses((value) => value + 1)}
            >
              <Text style={styles.buttonSecondaryLabel}>Record press</Text>
            </Pressable>
            <Text style={styles.metadata}>Drag rows to reorder</Text>
          </View>
          <View style={styles.card}>
            <Text style={styles.cardTitle}>Rich text</Text>
            <Text style={styles.metadata}>Inline styling, keyboard links, and native selection across runs.</Text>
            <Text style={styles.richTextParagraph}>
              Mix <Text style={styles.richTextBold}>bold</Text>, <Text style={styles.richTextItalic}>italic</Text>,{" "}
              <Text style={styles.richTextStrike}>struck</Text>, and{" "}
              <Text style={styles.richTextUnderline}>underlined</Text> runs.
            </Text>
            <Text style={styles.richTextParagraph}>
              Read the{" "}
              <Text
                style={styles.richTextLink}
                accessibilityRole="link"
                accessibilityLabel="Rich text guide"
                onPress={() => setRichTextLinkActivations((value) => value + 1)}
              >
                rich text guide
              </Text>{" "}
              to explore the same native run semantics.
            </Text>
            <Text style={styles.richTextCaption}>Tab to focus, Enter to activate</Text>
            <Text style={styles.metadata}>Link activations: {richTextLinkActivations}</Text>
            <Text selectable style={styles.richTextSelectable}>
              Select across <Text style={styles.richTextSelectableAccent}>colored runs</Text> and{" "}
              <Text style={styles.richTextSelectableMuted}>copy the paragraph natively</Text>.
            </Text>
          </View>
        </View>
      </View>
    </View>
  );
}

const windowSizeStore = createWindowSizeStore({ width: 800, height: 600 });
const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onWindowResize: (width, height, scaleFactor) => windowSizeStore.set(width, height, scaleFactor),
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<Gallery windowSizeStore={windowSizeStore} appearanceStore={appearanceStore} />);
