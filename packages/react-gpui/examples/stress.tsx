import { useEffect, useState } from "react";
import { Pressable, StdioTransport, Text, View, createProcessTerminationHandler, createRoot } from "../src/index";

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onTransportTermination: createProcessTerminationHandler(),
});

function StressSurface() {
  const [tick, setTick] = useState(0);
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
    <View style={{ flexDirection: "column", gap: 4, padding: 8 }}>
      <Text>Process soak tick {tick}</Text>
      <Pressable onPress={() => setTick((current) => current + 1)}>
        <Text>Advance</Text>
      </Pressable>
    </View>
  );
}

root.render(<StressSurface />);

process.once("SIGTERM", () => {
  process.exit(0);
});
