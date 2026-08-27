import { useState } from "react";
import {
  Pressable,
  StdioTransport,
  StyleSheet,
  Text,
  View,
  createProcessTerminationHandler,
  createRoot,
} from "../src/index";

const styles = StyleSheet.create({
  root: { flexDirection: "column", flexGrow: 1, gap: 12, padding: 20, backgroundColor: "#f7f8fa" },
  title: { fontSize: 18, lineHeight: 24, fontWeight: "bold", color: "#172033" },
  helper: { fontSize: 12, lineHeight: 18, color: "#5b6b7f" },
  anchor: { position: "relative", alignSelf: "flex-start" },
  trigger: {
    minWidth: 180,
    alignItems: "center",
    justifyContent: "center",
    padding: 10,
    borderRadius: 8,
    backgroundColor: "#2d6cdf",
    cursor: "pointer",
  },
  triggerLabel: { fontSize: 13, lineHeight: 18, fontWeight: "semibold", color: "#ffffff" },
  menu: {
    position: "overlay",
    width: 220,
    left: 0,
    top: 42,
    flexDirection: "column",
    gap: 4,
    padding: 8,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: "#b9c6d6",
    backgroundColor: "#ffffff",
    boxShadow: { offsetX: 0, offsetY: 4, blurRadius: 12, spreadRadius: 0, color: "#17203324" },
  },
  menuTitle: { padding: 4, fontSize: 11, lineHeight: 16, fontWeight: "semibold", color: "#5b6b7f" },
  item: { padding: 8, borderRadius: 6, backgroundColor: "#ffffff", cursor: "pointer" },
  itemLabel: { fontSize: 13, lineHeight: 18, color: "#172033" },
  itemDisabled: { opacity: 0.45, cursor: "not-allowed" },
  status: { fontSize: 12, lineHeight: 18, color: "#5b6b7f" },
});

function Dropdown() {
  const [open, setOpen] = useState(false);
  const [selection, setSelection] = useState("none");
  const [dismissal, setDismissal] = useState("none");

  const closeOnEscape = ({ key, action }: { key: string; action: string }) => {
    if (key === "escape" && action === "down") {
      setOpen(false);
      setDismissal("Escape");
    }
  };

  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Dropdown example">
      <Text style={styles.title}>Dropdown with outside dismissal</Text>
      <Text style={styles.helper}>
        The overlay closes on an outside pointer down or Escape. One item stays disabled to show native interaction
        state.
      </Text>
      <View style={styles.anchor}>
        <Pressable
          focusable
          style={styles.trigger}
          accessibilityRole="button"
          accessibilityLabel={open ? "Close actions" : "Open actions"}
          onPress={() => setOpen((current) => !current)}
          onKeyDown={closeOnEscape}
        >
          <Text style={styles.triggerLabel}>{open ? "Close actions" : "Open actions"}</Text>
        </Pressable>
        {open ? (
          <View
            style={styles.menu}
            onPointerDownOutside={({ x, y }) => {
              setOpen(false);
              setDismissal(`outside ${Math.round(x)},${Math.round(y)}`);
            }}
          >
            <Text style={styles.menuTitle}>Actions</Text>
            <Pressable
              focusable
              style={styles.item}
              accessibilityRole="button"
              accessibilityLabel="Refresh data"
              onKeyDown={closeOnEscape}
              onPress={() => {
                setSelection("Refresh data");
                setOpen(false);
              }}
            >
              <Text style={styles.itemLabel}>Refresh data</Text>
            </Pressable>
            <Pressable
              focusable
              disabled
              style={{ ...styles.item, ...styles.itemDisabled }}
              accessibilityRole="button"
              accessibilityLabel="Export data disabled"
              accessibilityDisabled
            >
              <Text style={styles.itemLabel}>Export data (disabled)</Text>
            </Pressable>
          </View>
        ) : null}
      </View>
      <Text style={styles.status}>
        Selection: {selection}; dismissed by: {dismissal}
      </Text>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<Dropdown />);
