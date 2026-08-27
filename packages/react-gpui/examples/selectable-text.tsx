import {
  StdioTransport,
  Text,
  View,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
} from "../src/index";
import { useTheme } from "./theme";

const appearanceStore = createAppearanceStore();

function SelectableTextDemo() {
  const theme = useTheme(useAppearance(appearanceStore));
  return (
    <View
      style={{
        flexDirection: "column",
        flexGrow: 1,
        gap: 12,
        padding: 20,
        backgroundColor: theme.canvas,
        color: theme.text,
      }}
    >
      <Text style={{ fontSize: 18, fontWeight: "bold", color: theme.text }}>Selectable Text</Text>
      <Text selectable style={{ maxWidth: 520, color: theme.text }}>
        Drag across this paragraph to select text, then press Cmd-C on macOS or Ctrl-C on other platforms to copy it.
      </Text>
      <Text style={{ color: theme.text }}>
        Selection is owned by the native host, so this example has no JavaScript selection state or callback.
      </Text>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<SelectableTextDemo />);
