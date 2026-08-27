import { useState } from "react";
import { Pressable, StdioTransport, Text, View, createProcessTerminationHandler, createRoot } from "../src/index";

interface KeyboardCounterProps {
  readonly onNotify: () => void;
}

function KeyboardCounter({ onNotify }: KeyboardCounterProps) {
  const [count, setCount] = useState(0);
  const [lastKey, setLastKey] = useState("none");
  return (
    <View
      focusable
      style={{
        flexDirection: "column",
        flexGrow: 1,
        gap: 8,
        padding: 16,
        backgroundColor: "#ffffff",
        color: "#111827",
      }}
      onKeyDown={({ key, modifiers, action }) => {
        setCount((current) => current + 1);
        setLastKey(`${action}: ${[...modifiers, key].join("+")}`);
      }}
    >
      <Text>Focus this view and press a key.</Text>
      <Text>Events: {count}</Text>
      <Text>Last key: {lastKey}</Text>
      <Pressable focusable onPress={onNotify} accessibilityRole="button" accessibilityLabel="Send notification">
        <Text>Send a notification</Text>
      </Pressable>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAction: (action) => console.log(`Action: ${action}`),
  onTransportTermination: createProcessTerminationHandler(),
});
void root.setKeybindings([{ keystrokes: "cmd-shift-p", actionName: "palette.open" }]);
root.render(
  <KeyboardCounter
    onNotify={() => {
      void root.showNotification({ title: "React GPUI", body: "Keyboard example notification requested." });
    }}
  />,
);
void root.setMenus([
  {
    title: "Actions",
    items: [{ type: "action", name: "palette.open" }],
  },
]);
