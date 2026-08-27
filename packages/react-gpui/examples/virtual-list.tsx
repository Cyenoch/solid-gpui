import { useMemo, useRef, useState } from "react";
import {
  Pressable,
  StdioTransport,
  Text,
  View,
  VirtualList,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
  type Style,
  type VirtualListHandle,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

type Row = { id: number; label: string };

const rows: Row[] = Array.from({ length: 100_000 }, (_, id) => ({ id, label: `Row ${id}` }));
const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});

function App() {
  const [visible, setVisible] = useState(true);
  const listRef = useRef<VirtualListHandle>(null);
  const theme = useTheme(useAppearance(appearanceStore));
  const style = useMemo<Style>(
    () => ({
      transition: {
        durationMs: 180,
        easing: "easeOut",
        properties: ["opacity"],
      },
      opacity: visible ? 1 : 0.35,
      color: theme.text,
    }),
    [theme, visible],
  );
  const buttonStyle = useMemo<Style>(() => ({ padding: 6, borderRadius: 6, backgroundColor: theme.accent }), [theme]);
  const buttonLabelStyle = useMemo<Style>(() => ({ color: theme.onAccent }), [theme]);
  return (
    <View style={{ flexDirection: "column", gap: 8, backgroundColor: theme.canvas, color: theme.text }}>
      <Text style={style}>{visible ? "Visible" : "Dimmed"}</Text>
      <VirtualList<Row>
        ref={listRef}
        style={{ height: 400, flexGrow: 0 }}
        data={rows}
        itemKey={(row) => row.id}
        renderItem={(row) => <Text style={{ color: theme.text }}>{row.label}</Text>}
        estimatedItemSize={24}
        overscan={3}
        initialNumToRender={16}
        onEndReached={() => console.error("end reached")}
      />
      <View style={{ flexDirection: "row", gap: 8 }}>
        <Pressable style={buttonStyle} onPress={() => void listRef.current?.scrollToIndex(9999)}>
          <Text style={buttonLabelStyle}>Scroll to row 9999</Text>
        </Pressable>
        <Pressable style={buttonStyle} onPress={() => void listRef.current?.scrollToEnd()}>
          <Text style={buttonLabelStyle}>Scroll to end</Text>
        </Pressable>
      </View>
      <Pressable style={buttonStyle} onPress={() => setVisible((current) => !current)}>
        <Text style={buttonLabelStyle}>Toggle opacity</Text>
      </Pressable>
    </View>
  );
}

root.render(<App />);
