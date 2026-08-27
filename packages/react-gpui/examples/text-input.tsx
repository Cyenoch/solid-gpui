import { useEffect, useMemo, useRef, useState } from "react";
import {
  Pressable,
  StdioTransport,
  Text,
  TextInput,
  View,
  StyleSheet,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
  type AppearanceStore,
  type TextInputHandle,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", flexGrow: 1, gap: 8, padding: 16, backgroundColor: theme.canvas },
    input: {
      padding: 8,
      borderWidth: 1,
      borderColor: theme.borderInput,
      backgroundColor: theme.input,
      color: theme.text,
    },
    label: { color: theme.text },
    action: { color: theme.accentText, textDecoration: "underline" },
  });
}

function TwoInputs({ appearanceStore }: { readonly appearanceStore: AppearanceStore }) {
  const [first, setFirst] = useState("");
  const [second, setSecond] = useState("Uncontrolled input");
  const [firstStatus, setFirstStatus] = useState("unfocused");
  const [selection, setSelection] = useState("0-0");
  const firstRef = useRef<TextInputHandle>(null);
  const secondRef = useRef<TextInputHandle>(null);
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
  useEffect(() => {
    void firstRef.current?.focus();
  }, []);
  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Text input demo">
      <Text style={styles.label}>First: {first}</Text>
      <TextInput
        ref={firstRef}
        style={styles.input}
        value={first}
        onChangeText={setFirst}
        onFocus={() => setFirstStatus("focused")}
        onBlur={() => setFirstStatus("blurred")}
        onSelectionChange={(value) => setSelection(`${value.start}-${value.end}${value.reversed ? " (reversed)" : ""}`)}
        accessibilityLabel="First name"
        accessibilityDescription="The first controlled text input"
      />
      <Text style={styles.label}>
        First input: {firstStatus}; selection: {selection}
      </Text>
      <Text style={styles.label}>
        In the multiline input, double-click a word or triple-click its line; Cmd-C/Ctrl-C copies the native selection.
      </Text>
      <Pressable onPress={() => void firstRef.current?.setSelection(0, first.length)}>
        <Text style={styles.action}>Select first input</Text>
      </Pressable>
      <Text style={styles.label}>Second: {second}</Text>
      <TextInput
        ref={secondRef}
        style={styles.input}
        defaultValue={second}
        onChangeText={setSecond}
        multiline
        maxLength={64}
        accessibilityLabel="Second name"
        accessibilityDescription="The second uncontrolled multiline text input"
      />
      <Pressable onPress={() => void secondRef.current?.blur()}>
        <Text style={styles.action}>Blur second input</Text>
      </Pressable>
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
root.render(<TwoInputs appearanceStore={appearanceStore} />);
