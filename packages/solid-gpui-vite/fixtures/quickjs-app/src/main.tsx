import { Text, View } from "@solid-gpui/core";

/** The application entry the plugin plans for; tests never build or load it. */
export function Main(props: { label: string }) {
  return (
    <View>
      <Text>{props.label}</Text>
    </View>
  );
}
