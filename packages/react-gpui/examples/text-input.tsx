import { useEffect, useRef, useState } from "react";
import { StdioTransport, Text, TextInput, View, createRoot, StyleSheet, type TextInputHandle } from "../src/index";

const styles = StyleSheet.create({
  root: { flexDirection: "column", flexGrow: 1, gap: 8, padding: 16, backgroundColor: "#ffffff" },
  input: { padding: 8, backgroundColor: "#f2f4f7", color: "#111827" },
  label: { color: "#111827" },
});

function TwoInputs() {
  const [first, setFirst] = useState("");
  const [second, setSecond] = useState("");
  const firstRef = useRef<TextInputHandle>(null);
  useEffect(() => {
    void firstRef.current?.focus();
  }, []);
  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Text input demo">
      <Text style={styles.label}>First: {first}</Text>
      <TextInput ref={firstRef} style={styles.input} value={first} onChangeText={setFirst} accessibilityLabel="First name" accessibilityDescription="The first controlled text input" />
      <Text style={styles.label}>Second: {second}</Text>
      <TextInput style={styles.input} value={second} onChangeText={setSecond} accessibilityLabel="Second name" accessibilityDescription="The second controlled text input" />
    </View>
  );
}

const root = createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
root.render(<TwoInputs />);
