import React, { useMemo, useRef, useState } from "react";
import {
  Pressable,
  StdioTransport,
  Text,
  View,
  VirtualList,
  createProcessTerminationHandler,
  createRoot,
  type Style,
  type VirtualListHandle,
} from "../src/index";

type Row = { id: number; label: string };

const rows: Row[] = Array.from({ length: 100_000 }, (_, id) => ({ id, label: `Row ${id}` }));

function App() {
  const [visible, setVisible] = useState(true);
  const listRef = useRef<VirtualListHandle>(null);
  const style = useMemo<Style>(
    () => ({
      transition: {
        durationMs: 180,
        easing: "easeOut",
        properties: ["opacity"],
      },
      opacity: visible ? 1 : 0.35,
    }),
    [visible],
  );
  return (
    <View style={{ flexDirection: "column", gap: 8, backgroundColor: "#ffffff", color: "#111827" }}>
      <Text style={style}>{visible ? "Visible" : "Dimmed"}</Text>
      <VirtualList<Row>
        ref={listRef}
        style={{ height: 400, flexGrow: 0 }}
        data={rows}
        itemKey={(row) => row.id}
        renderItem={(row) => <Text>{row.label}</Text>}
        estimatedItemSize={24}
        overscan={3}
        initialNumToRender={16}
        onEndReached={() => console.error("end reached")}
      />
      <View style={{ flexDirection: "row", gap: 8 }}>
        <Pressable onPress={() => void listRef.current?.scrollToIndex(9999)}>
          <Text>Scroll to row 9999</Text>
        </Pressable>
        <Pressable onPress={() => void listRef.current?.scrollToEnd()}>
          <Text>Scroll to end</Text>
        </Pressable>
      </View>
      <Pressable onPress={() => setVisible((current) => !current)}>
        <Text>Toggle opacity</Text>
      </Pressable>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<App />);
