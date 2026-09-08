import { copyText } from "./site";
import { View, Pressable, Icon } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { CodeLines } from "./CodeLines";
import { Copy, colors } from "./ui";
import { t } from "./i18n";

export function CodeBlock(props: { source: string; language?: string; label?: string }) {
  const [copied, setCopied] = createSignal(false);
  const [hovered, setHovered] = createSignal(false);
  const [focused, setFocused] = createSignal(false);
  const [error, setError] = createSignal("");
  return (
    <View
      style={{
        flexShrink: 0,
        minWidth: 0,
        padding: 16,
        borderRadius: 16,
        backgroundColor: "#161616",
        position: "relative",
        gap: 8,
      }}
    >
      <CodeLines source={props.source} language={props.language} />
      <View style={{ position: "absolute", right: 12, top: 12 }}>
        <Pressable
          focusable
          accessibilityLabel={t(copied() ? "Copied" : "Copy")}
          onHoverChange={setHovered}
          onFocus={() => setFocused(true)}
          onBlur={() => setFocused(false)}
          onPress={() =>
            copyText(props.source)
              .then(() => setCopied(true))
              .catch((error) => setError(String(error)))
          }
          style={{
            width: 28,
            height: 28,
            justifyContent: "center",
            alignItems: "center",
            borderRadius: 6,
            backgroundColor: hovered() ? colors.secondary : "#161616",
            borderWidth: 1,
            borderColor: focused() ? colors.muted : "#00000000",
          }}
        >
          <Icon
            name={copied() ? "lucide:check" : "lucide:copy"}
            size={14}
            color={hovered() ? colors.text : colors.muted}
          />
        </Pressable>
      </View>
      {() => (error() ? <Copy size={12}>{error()}</Copy> : null)}
    </View>
  );
}
