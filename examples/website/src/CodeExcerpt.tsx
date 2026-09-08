import { View } from "@solid-gpui/core";
import { CodeLines } from "./CodeLines";
import { excerptCode } from "./snippets";
import { Button } from "./ui";

export function CodeExcerpt(props: { source: string; width: number; onExpand: () => void }) {
  return (
    <View style={{ height: 108, overflow: "hidden", position: "relative", padding: 16, backgroundColor: "#161616" }}>
      <CodeLines source={excerptCode(props.source)} faded />
      <View style={{ position: "absolute", left: (props.width - 100) / 2, top: 30, width: 100 }}>
        <Button compact onPress={props.onExpand} style={{ backgroundColor: "#0a0a0a" }}>
          View Code
        </Button>
      </View>
    </View>
  );
}
