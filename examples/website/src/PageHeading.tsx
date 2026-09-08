import { View, type SolidChild } from "@solid-gpui/core";
import { Copy } from "./ui";

export function PageHeading(props: { title: string; translate?: boolean; width: number; children: SolidChild }) {
  const stacked = () => props.width < 600;
  return (
    <View
      style={{
        width: props.width,
        flexDirection: stacked() ? "column" : "row",
        gap: 16,
        justifyContent: "space-between",
      }}
    >
      <View style={{ width: stacked() ? props.width : props.width - 216, minWidth: 0, flexShrink: 0 }}>
        <Copy translate={props.translate} size={30} style={{ fontWeight: "bold", lineHeight: 38 }}>
          {props.title}
        </Copy>
      </View>
      <View style={{ flexShrink: 0, marginTop: stacked() ? 0 : 5 }}>{props.children}</View>
    </View>
  );
}
