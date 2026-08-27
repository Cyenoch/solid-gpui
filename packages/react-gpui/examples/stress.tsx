import { useEffect, useMemo, useState } from "react";
import {
  Pressable,
  StdioTransport,
  StyleSheet,
  Text,
  View,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", gap: 4, padding: 8, backgroundColor: theme.canvas, color: theme.text },
    button: { padding: 6, borderRadius: 5, backgroundColor: theme.accent },
    buttonLabel: { color: theme.onAccent },
  });
}

const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});

function StressSurface() {
  const [tick, setTick] = useState(0);
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
  useEffect(() => {
    const timer = setInterval(() => {
      setTick((current) => current + 1);
    }, 10);
    return () => clearInterval(timer);
  }, []);
  useEffect(() => {
    if (tick === 0 || tick % 10 !== 0) return;
    void root.getWindowSize().catch(() => undefined);
  }, [tick]);
  return (
    <View style={styles.root}>
      <Text>Process soak tick {tick}</Text>
      <Pressable style={styles.button} onPress={() => setTick((current) => current + 1)}>
        <Text style={styles.buttonLabel}>Advance</Text>
      </Pressable>
    </View>
  );
}

root.render(<StressSurface />);

process.once("SIGTERM", () => {
  process.exit(0);
});
