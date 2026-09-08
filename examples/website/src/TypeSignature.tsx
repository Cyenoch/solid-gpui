import { Text } from "@solid-gpui/core";
import { createMemo } from "@solid-gpui/core/runtime";
import { highlight } from "./highlight";

export function TypeSignature(props: { source: string }) {
  const tokens = createMemo(() => highlight(props.source, "ts").runs);
  return (
    <Text selectable style={{ fontFamily: "Maple Mono", fontSize: 13, lineHeight: 21, flexShrink: 0 }}>
      {() => tokens().map((token) => <Text style={{ color: token.color }}>{token.text}</Text>)}
    </Text>
  );
}
