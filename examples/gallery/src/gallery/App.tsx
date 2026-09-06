import { Icon, Text, View, useWindowSize } from "@solid-gpui/core";
import { createMemo } from "@solid-gpui/core/runtime";
import { Outlet } from "@solid-gpui/router";
import { Header } from "./components/Header";
import { Sidebar } from "./components/Sidebar";
import { useGallery } from "./context";

export function App() {
  const { theme, statusMessage, windowSizeStore } = useGallery();
  const windowSize = useWindowSize(windowSizeStore);
  const compact = createMemo(() => windowSize().width < 720);

  // These panes consume remaining window space. Zero main-axis bases avoid
  // measuring the entire routed page to derive an intrinsic flex basis.
  return (
    <View
      style={{
        height: 0,
        flexGrow: 1,
        flexShrink: 1,
        alignSelf: "stretch",
        minWidth: 0,
        minHeight: 0,
        backgroundColor: theme().bgApp,
        flexDirection: "column",
        overflow: "hidden",
        position: "relative",
      }}
    >
      <Header />
      <View
        style={{
          height: 0,
          flexDirection: "row",
          flexGrow: 1,
          flexShrink: 1,
          minWidth: 0,
          minHeight: 0,
          overflow: "hidden",
        }}
      >
        <Sidebar compact={compact()} />
        <View
          accessibilityLabel="Showcase content"
          style={{ width: 0, flexGrow: 1, flexShrink: 1, minWidth: 0, minHeight: 0, overflow: "scroll" }}
        >
          <View style={{ padding: compact() ? 16 : 28, gap: 24, minWidth: 0, flexShrink: 0 }}>
            <Outlet />
          </View>
        </View>
      </View>
      <View
        style={{
          height: 26,
          flexShrink: 0,
          padding: 6,
          flexDirection: "row",
          alignItems: "center",
          gap: 6,
          backgroundColor: theme().bgHeader,
          borderWidth: 1,
          borderColor: theme().borderMuted,
          minWidth: 0,
        }}
      >
        <Icon name="lucide:check" size={11} color={theme().success} />
        <Text style={{ color: theme().textMuted, fontSize: 10 }}>Native surface</Text>
        <View style={{ flexGrow: 1 }} />
        <Text style={{ color: theme().textMuted, fontSize: 10 }}>SolidJS / GPUI</Text>
      </View>
      {statusMessage() ? (
        <View
          style={{
            position: "absolute",
            bottom: 38,
            left: compact() ? 68 : 218,
            right: 16,
            minWidth: 0,
            backgroundColor: theme().bgCard,
            borderWidth: 1,
            borderColor: theme().borderFocus,
            borderRadius: 8,
            padding: 12,
            flexDirection: "row",
            alignItems: "center",
            gap: 10,
          }}
        >
          <Icon name="lucide:bell" size={16} color={theme().accent} />
          <Text style={{ color: theme().textPrimary, fontSize: 12, minWidth: 0, flexShrink: 1 }}>
            {statusMessage()}
          </Text>
        </View>
      ) : null}
    </View>
  );
}
