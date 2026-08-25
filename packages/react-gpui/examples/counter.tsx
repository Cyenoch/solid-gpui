import { useCallback, useState } from "react";
import { Pressable, StyleSheet, StdioTransport, Text, View, createRoot } from "../src/index";

const styles = StyleSheet.create({
  root: { flexDirection: "column", flexGrow: 1, gap: 8, padding: 16, backgroundColor: "#ffffff" },
  count: { color: "#111827" },
  button: { padding: 8, backgroundColor: "#2d6cdf" },
  label: { color: "#ffffff" },
});

function Counter() {
  const [count, setCount] = useState(0);
  const increment = useCallback(() => setCount((current) => current + 1), []);
  return (
    <View style={styles.root}>
      <Text style={styles.count}>Count: {count}</Text>
      <Pressable style={styles.button} onPress={increment}>
        <Text style={styles.label}>Increment</Text>
      </Pressable>
    </View>
  );
}

const root = createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
root.render(<Counter />);
