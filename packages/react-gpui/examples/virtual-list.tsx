import React, { useMemo, useState } from "react";
import { Pressable, Text, View, VirtualList, createRoot, type Style } from "../src/index";
import { StdioTransport } from "../src/transport";

type Row = { id: number; label: string };

const rows: Row[] = Array.from({ length: 100_000 }, (_, id) => ({ id, label: `Row ${id}` }));

function App() {
  const [visible, setVisible] = useState(true);
  const style = useMemo<Style>(() => ({
    transition: {
      durationMs: 180,
      easing: "easeOut",
      properties: ["opacity"],
    },
    opacity: visible ? 1 : 0.35,
  }), [visible]);
  return (
    <View style={{ flexDirection: "column", gap: 8 }}>
      <Text style={style}>{visible ? "Visible" : "Dimmed"}</Text>
      <VirtualList<Row>
        style={{ height: 400, flexGrow: 0 }}
        data={rows}
        itemKey={(row) => row.id}
        renderItem={(row) => <Text>{row.label}</Text>}
        estimatedItemSize={24}
        overscan={3}
        initialNumToRender={16}
        onEndReached={() => console.error("end reached")}
      />
      <Pressable onPress={() => setVisible((current) => !current)}><Text>Toggle opacity</Text></Pressable>
    </View>
  );
}

const root = createRoot(new StdioTransport());
root.render(<App />);
