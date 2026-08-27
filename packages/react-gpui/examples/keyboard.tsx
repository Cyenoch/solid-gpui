import { useMemo, useState } from "react";
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
  type AppearanceStore,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: {
      flexDirection: "column",
      flexGrow: 1,
      gap: 8,
      padding: 16,
      backgroundColor: theme.canvas,
      color: theme.text,
    },
    button: { padding: 8, borderRadius: 6, backgroundColor: theme.accent },
    buttonLabel: { color: theme.onAccent },
  });
}

interface KeyboardCounterProps {
  readonly onNotify: () => void;
  readonly appearanceStore: AppearanceStore;
}

function KeyboardCounter({ onNotify, appearanceStore }: KeyboardCounterProps) {
  const [count, setCount] = useState(0);
  const [lastKey, setLastKey] = useState("none");
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
  return (
    <View
      focusable
      style={styles.root}
      onKeyDown={({ key, modifiers, action }) => {
        setCount((current) => current + 1);
        setLastKey(`${action}: ${[...modifiers, key].join("+")}`);
      }}
    >
      <Text>Focus this view and press a key.</Text>
      <Text>Events: {count}</Text>
      <Text>Last key: {lastKey}</Text>
      <Pressable
        focusable
        style={styles.button}
        onPress={onNotify}
        accessibilityRole="button"
        accessibilityLabel="Send notification"
      >
        <Text style={styles.buttonLabel}>Send a notification</Text>
      </Pressable>
    </View>
  );
}

const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAction: (action) => console.log(`Action: ${action}`),
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(
  <KeyboardCounter
    onNotify={() =>
      void root.showNotification({ title: "React GPUI", body: "Keyboard example notification requested." })
    }
    appearanceStore={appearanceStore}
  />,
);
void root.setKeybindings([{ keystrokes: "cmd-shift-p", actionName: "palette.open" }]);
void root.setMenus([
  {
    title: "Actions",
    items: [{ type: "action", name: "palette.open" }],
  },
]);
