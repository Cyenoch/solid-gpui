import { useCallback, useMemo, useState } from "react";
import {
  Pressable,
  StyleSheet,
  StdioTransport,
  Text,
  View,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
  type AppearanceStore,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", flexGrow: 1, gap: 8, padding: 16, backgroundColor: theme.canvas },
    count: { color: theme.text },
    button: { padding: 8, backgroundColor: theme.accent },
    label: { color: theme.onAccent },
  });
}

function Counter({ appearanceStore }: { readonly appearanceStore: AppearanceStore }) {
  const [count, setCount] = useState(0);
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
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

const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<Counter appearanceStore={appearanceStore} />);
