import { StdioTransport, Text, View, createProcessTerminationHandler, createRoot } from "../src/index";

function SelectableTextDemo() {
  return (
    <View style={{ flexDirection: "column", flexGrow: 1, gap: 12, padding: 20 }}>
      <Text style={{ fontSize: 18, fontWeight: "bold" }}>Selectable Text</Text>
      <Text selectable style={{ maxWidth: 520 }}>
        Drag across this paragraph to select text, then press Cmd-C on macOS or Ctrl-C on other platforms to copy it.
      </Text>
      <Text>Selection is owned by the native host, so this example has no JavaScript selection state or callback.</Text>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<SelectableTextDemo />);
