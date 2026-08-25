import { useState } from "react";
import { StdioTransport, Text, View, createProcessTerminationHandler, createRoot } from "../src/index";

function KeyboardCounter() {
  const [count, setCount] = useState(0);
  const [lastKey, setLastKey] = useState("none");
  return (
    <View
      focusable
      style={{ flexDirection: "column", flexGrow: 1, gap: 8, padding: 16 }}
      onKeyDown={({ key, modifiers, action }) => {
        setCount((current) => current + 1);
        setLastKey(`${action}: ${[...modifiers, key].join("+")}`);
      }}
    >
      <Text>Focus this view and press a key.</Text>
      <Text>Events: {count}</Text>
      <Text>Last key: {lastKey}</Text>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<KeyboardCounter />);
