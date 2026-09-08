import { View, Pressable, Icon } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { Copy, colors } from "./ui";

export function CodeCollapse(props: { onPress: () => void }) {
  const [hovered, setHovered] = createSignal(false);
  const [focused, setFocused] = createSignal(false);
  return (
    <View>
      <View style={{ height: 1, backgroundColor: colors.line }} />
      <Pressable
        focusable
        onPress={props.onPress}
        onHoverChange={setHovered}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        style={{
          height: 42,
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "center",
          gap: 8,
          backgroundColor: hovered() ? colors.secondary : "#161616",
          borderWidth: 1,
          borderColor: focused() ? colors.muted : "#00000000",
        }}
      >
        <Icon name="lucide:minus" size={14} color={hovered() ? colors.text : colors.muted} />
        <Copy size={13} color={hovered() ? colors.text : colors.muted}>
          Hide Code
        </Copy>
      </Pressable>
    </View>
  );
}
