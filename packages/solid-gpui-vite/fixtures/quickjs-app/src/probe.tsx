import { Text, View } from "@solid-gpui/core";

/** Application TSX: only the application's own transform can compile this for a test. */
export function Probe(props: { label: string }) {
  return (
    <View>
      <Text>{props.label}</Text>
    </View>
  );
}
