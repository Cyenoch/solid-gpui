import { createElement, useState, type ReactNode } from "../../react-gpui/node_modules/react";
import {
  Pressable,
  StdioTransport,
  Text,
  View,
  createRoot,
  type Root,
} from "@react-gpui/core";
import { installFastRefreshTransform } from "../src/transform";

await installFastRefreshTransform(new URL(".", import.meta.url).pathname);

type RefreshHost = {
  root?: Root;
  implementation?: () => ReactNode;
  stable?: () => ReactNode;
};
const host = (globalThis as { __reactGpuiRefresh?: RefreshHost }).__reactGpuiRefresh ??= {};

function Counter() {
  const [count, setCount] = useState(0);
  return (
    <View>
      <Text>Live refresh counter: {count}</Text>
      <Pressable onPress={() => setCount((value) => value + 1)}>
        <Text>Increment</Text>
      </Pressable>
    </View>
  );
}

host.implementation = Counter;
host.stable ??= () => host.implementation?.() ?? null;
host.root ??= createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
host.root.render(createElement(host.stable));
