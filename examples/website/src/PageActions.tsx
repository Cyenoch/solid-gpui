import { copyText, setSourceDocument } from "./site";
import { View, Pressable, Icon, type LayoutFrame } from "@solid-gpui/core";
import { createSignal, onCleanup } from "@solid-gpui/core/runtime";
import { Copy, colors } from "./ui";
import { pageMenuOpen, closePageMenu, togglePageMenu } from "./PageMenu";
import { t } from "./i18n";
export function PageActions(props: { source: string; previous?: () => void; next?: () => void }) {
  const owner = {};
  let frame: LayoutFrame | undefined;
  const [copied, setCopied] = createSignal(false);
  const [error, setError] = createSignal("");
  onCleanup(() => {
    closePageMenu(owner);
  });
  const actionStyle = {
    height: 28,
    padding: 5,
    borderRadius: 7,
    backgroundColor: "#262626",
    justifyContent: "center" as const,
    flexShrink: 0,
  };
  return (
    <View
      focusable
      onLayout={(value) => {
        if (frame && (frame.x !== value.x || frame.y !== value.y)) closePageMenu(owner);
        frame = value;
      }}
      style={{ gap: 6, flexShrink: 0, alignSelf: "flex-start" }}
      onKeyDown={(event) => {
        if (event.key === "Escape") closePageMenu(owner);
      }}
    >
      <View style={{ flexDirection: "row", gap: 8 }}>
        <View style={{ ...actionStyle, padding: 0, flexDirection: "row", alignItems: "center" }}>
          <Pressable
            focusable
            accessibilityLabel={t("Copy Page")}
            onPress={() => {
              copyText(props.source)
                .then(() => setCopied(true))
                .catch((error) => setError(String(error)));
            }}
            style={{ padding: 5 }}
          >
            <View style={{ flexDirection: "row", alignItems: "center", gap: 6, marginLeft: 4, marginRight: 4 }}>
              <Icon name={copied() ? "lucide:check" : "lucide:copy"} size={14} color={colors.text} />
              <Copy size={12} style={{ lineHeight: 18 }}>
                {copied() ? "Copied" : "Copy Page"}
              </Copy>
            </View>
          </Pressable>
          <View style={{ width: 1, height: 16, backgroundColor: colors.line }} />
          <Pressable
            focusable
            accessibilityLabel={t("Page options")}
            accessibilityExpanded={pageMenuOpen(owner)}
            onPress={() => {
              if (!frame) return;
              togglePageMenu({
                owner,
                x: frame.x,
                y: frame.y + 34,
                view: () => {
                  setSourceDocument(props.source);
                },
              });
            }}
            style={{ padding: 6 }}
          >
            <Icon name="lucide:chevron-down" size={14} color={colors.text} />
          </Pressable>
        </View>
        <Pressable
          focusable
          disabled={!props.previous}
          accessibilityLabel={t("Previous page")}
          onPress={() => props.previous?.()}
          style={{ ...actionStyle, width: 28, opacity: props.previous ? 1 : 0.35, alignItems: "center" }}
        >
          <Icon name="lucide:arrow-left" size={14} color={colors.text} />
        </Pressable>
        <Pressable
          focusable
          disabled={!props.next}
          accessibilityLabel={t("Next page")}
          onPress={() => props.next?.()}
          style={{ ...actionStyle, width: 28, opacity: props.next ? 1 : 0.35, alignItems: "center" }}
        >
          <Copy size={16} style={{ lineHeight: 18 }}>
            →
          </Copy>
        </Pressable>
      </View>
      {() => (error() ? <Copy size={12}>{error()}</Copy> : null)}
    </View>
  );
}
