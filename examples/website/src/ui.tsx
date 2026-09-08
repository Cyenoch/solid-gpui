import { translateChild, locale } from "./i18n";
import { View, Text, Pressable, Icon, type IconProps, type SolidChild, type Style } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
export const colors = {
  bg: "#0a0a0a",
  panel: "#111111",
  secondary: "#1c1c1c",
  line: "#292929",
  text: "#fafafa",
  muted: "#a1a1aa",
  accent: "#fafafa",
};
export const panel: Style = {
  flexShrink: 0,
  padding: 24,
  gap: 14,
  borderRadius: 12,
  borderWidth: 1,
  borderColor: colors.line,
  backgroundColor: colors.panel,
};
export function Copy(props: {
  children: SolidChild;
  selectable?: boolean;
  translate?: boolean;
  size?: number;
  color?: string;
  style?: Style;
}) {
  return (
    <Text
      selectable={props.selectable}
      style={{
        flexShrink: 0,
        fontFamily: locale() === "zh-CN" ? "Noto Sans SC" : "Inter Variable",
        fontSize: props.size ?? 14,
        lineHeight: (props.size ?? 14) * 1.55,
        color: props.color ?? colors.text,
        ...props.style,
      }}
    >
      {props.translate === false ? props.children : translateChild(props.children)}
    </Text>
  );
}
export function Button(props: {
  children: SolidChild;
  onPress: () => void;
  primary?: boolean;
  icon?: IconProps["name"];
  iconAfter?: boolean;
  compact?: boolean;
  translate?: boolean;
  disabled?: boolean;
  ghost?: boolean;
  active?: boolean;
  style?: Style;
}) {
  const [hovered, setHovered] = createSignal(false);
  const [focused, setFocused] = createSignal(false);
  return (
    <Pressable
      focusable
      disabled={props.disabled}
      onPress={props.onPress}
      onHoverChange={setHovered}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      style={{
        flexShrink: 0,
        minHeight: props.compact ? 32 : props.ghost ? 36 : 40,
        padding: props.compact ? 5 : 8,
        justifyContent: "center",
        borderRadius: 8,
        borderWidth: 1,
        borderColor: focused() ? colors.muted : props.primary ? colors.text : props.ghost ? "#00000000" : colors.line,
        backgroundColor: props.primary
          ? hovered()
            ? "#d4d4d8"
            : colors.text
          : hovered() || props.active
            ? colors.secondary
            : props.ghost
              ? "#00000000"
              : colors.panel,
        opacity: props.disabled ? 0.5 : 1,
        ...props.style,
      }}
    >
      <View
        style={{
          flexDirection: "row",
          alignItems: "center",
          justifyContent: props.ghost ? "flex-start" : "center",
          marginLeft: props.compact ? 0 : props.ghost ? 2 : 6,
          marginRight: props.compact ? 0 : props.ghost ? 2 : 6,
          gap: 8,
        }}
      >
        {() =>
          props.icon && !props.iconAfter ? (
            <View style={{ width: 16, height: 16, flexShrink: 0 }}>
              <Icon name={props.icon} size={16} color={props.primary ? colors.bg : colors.muted} />
            </View>
          ) : null
        }
        <Text
          style={{
            fontFamily: locale() === "zh-CN" ? "Noto Sans SC" : "Inter Variable",
            fontSize: 13,
            lineHeight: 20,
            minWidth: 0,
            flexShrink: 1,
            color: props.primary ? colors.bg : props.ghost && !props.active ? colors.muted : colors.text,
          }}
        >
          {props.translate === false ? props.children : translateChild(props.children)}
        </Text>
        {() =>
          props.icon && props.iconAfter ? (
            <View style={{ width: 16, height: 16, flexShrink: 0 }}>
              <Icon name={props.icon} size={16} color={props.primary ? colors.bg : colors.muted} />
            </View>
          ) : null
        }
      </View>
    </Pressable>
  );
}
export function Counter() {
  const [count, setCount] = createSignal(0);
  return (
    <View style={panel}>
      <Copy size={15} style={{ fontWeight: "semibold" }}>
        Daily goal
      </Copy>
      <Copy color={colors.muted}>One step at a time. Every little bit counts.</Copy>
      <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "center", gap: 14, padding: 10 }}>
        <Button compact style={{ width: 36, minHeight: 36 }} onPress={() => setCount(Math.max(0, count() - 1))}>
          −
        </Button>
        <Copy size={48} style={{ width: 68, textAlign: "center", fontWeight: "semibold" }}>
          {count()}
        </Copy>
        <Button compact style={{ width: 36, minHeight: 36 }} onPress={() => setCount(count() + 1)}>
          +
        </Button>
      </View>
      <Button icon="lucide:refresh-cw" onPress={() => setCount(0)}>
        Start again
      </Button>
    </View>
  );
}
export function LayoutDemo(props: { compact?: boolean } = {}) {
  const [row, setRow] = createSignal(true);
  return (
    <View style={{ ...panel, padding: props.compact ? 16 : 24 }}>
      <Copy size={15} style={{ fontWeight: "semibold" }}>
        Room to rearrange
      </Copy>
      <Copy color={colors.muted}>Find the layout that works for you.</Copy>
      <View style={{ height: 140, flexDirection: row() ? "row" : "column", gap: 8 }}>
        {(
          [
            { label: "Overview", icon: "lucide:home" },
            { label: "Activity", icon: "lucide:zap" },
            { label: "Settings", icon: "lucide:settings" },
          ] as const
        ).map((item) => (
          <View
            style={{
              padding: props.compact ? 4 : 10,
              flexGrow: 1,
              minWidth: 0,
              minHeight: 0,
              flexDirection: row() ? "column" : "row",
              alignItems: "center",
              justifyContent: "center",
              gap: 6,
              backgroundColor: colors.secondary,
              borderRadius: 8,
            }}
          >
            <Icon name={item.icon} size={16} color={colors.muted} />
            <Copy size={12}>{item.label}</Copy>
          </View>
        ))}
      </View>
      <Button icon="lucide:layout" onPress={() => setRow(!row())}>
        {locale() === "zh-CN" ? `切换为${row() ? "纵向" : "横向"}布局` : `Switch to ${row() ? "column" : "row"}`}
      </Button>
    </View>
  );
}
