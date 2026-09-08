import { View, type SolidChild } from "@solid-gpui/core";
/** Desktop landing content shares the page; browser GPU artwork stays in the Web host. */
export function HeroVisual(props: { width: number; height: number; children: SolidChild }) {
  return (
    <View
      style={{
        width: props.width,
        minHeight: props.height,
        alignItems: "center",
        justifyContent: "center",
        padding: props.width < 600 ? 24 : 60,
        gap: 20,
      }}
    >
      {props.children}
    </View>
  );
}
