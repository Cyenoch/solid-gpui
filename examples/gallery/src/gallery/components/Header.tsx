import { Icon, Pressable, Text, TextInput, View, useWindowSize } from "@solid-gpui/core";
import { createMemo } from "@solid-gpui/core/runtime";
import { BackButton, Link, useLocation } from "@solid-gpui/router";
import { useGallery } from "../context";
import { pageFromPathname, pagePath } from "../types";

export function Header() {
  const { theme, themeMode, toggleTheme, searchQuery, setSearchQuery, windowSizeStore } = useGallery();
  const windowSize = useWindowSize(windowSizeStore);
  const compact = createMemo(() => windowSize().width < 720);
  const pathname = useLocation({ select: (location) => location.pathname });
  const currentPage = createMemo(() => pageFromPathname(pathname()));

  return (
    <View
      style={{
        backgroundColor: theme().bgHeader,
        borderWidth: 1,
        borderColor: theme().borderMuted,
        padding: 12,
        gap: 12,
        minWidth: 0,
        flexShrink: 0,
      }}
    >
      <View style={{ flexDirection: "row", alignItems: "center", gap: 10, minWidth: 0 }}>
        <Link to={pagePath("overview")} activeOptions={{ exact: true }} style={{ cursor: "pointer", flexShrink: 0 }}>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 9 }}>
            <View
              style={{
                width: 28,
                height: 28,
                borderRadius: 7,
                backgroundColor: theme().accentMuted,
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Icon name="lucide:layers" size={16} color={theme().accent} />
            </View>
            <Text style={{ color: theme().textPrimary, fontSize: 14, fontWeight: "semibold" }}>Solid GPUI</Text>
          </View>
        </Link>
        {!compact() ? <Text style={{ color: theme().textMuted, fontSize: 12 }}>Component workbench</Text> : null}
        <View style={{ flexGrow: 1, minWidth: 0 }} />
        <BackButton style={{ padding: 7, borderRadius: 5, cursor: "pointer", flexShrink: 0 }}>
          <Icon name="lucide:arrow-left" size={14} color={theme().textSecondary} />
        </BackButton>
        <Pressable
          onPress={toggleTheme}
          accessibilityRole="button"
          accessibilityLabel="Toggle color theme"
          tooltip="Toggle color theme"
          style={{ padding: 7, borderRadius: 5, backgroundColor: theme().bgHover, flexShrink: 0 }}
        >
          <Icon name={themeMode() === "dark" ? "lucide:sun" : "lucide:moon"} size={14} color={theme().textSecondary} />
        </Pressable>
      </View>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 12, minWidth: 0 }}>
        <View style={{ flexDirection: "row", alignItems: "center", gap: 7, minWidth: 0, flexShrink: 1, flexGrow: 1 }}>
          <Icon name={currentPage()?.icon ?? "lucide:home"} size={13} color={theme().textMuted} />
          <Text
            style={{
              color: theme().textSecondary,
              fontSize: 12,
              minWidth: 0,
              flexShrink: 1,
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {currentPage()?.title ?? "Overview"}
          </Text>
        </View>
        <View
          style={{
            width: compact() ? 200 : 260,
            flexShrink: 1,
            minWidth: 0,
            flexDirection: "row",
            alignItems: "center",
            gap: 5,
            backgroundColor: theme().bgInput,
            borderRadius: 6,
            borderWidth: 1,
            borderColor: theme().border,
            padding: 5,
          }}
        >
          <Icon name="lucide:search" size={12} color={theme().textMuted} />
          <TextInput
            placeholder="Find a component…"
            value={searchQuery()}
            onChangeText={setSearchQuery}
            style={{ flexGrow: 1, flexShrink: 1, minWidth: 0, fontSize: 12, color: theme().textPrimary, padding: 2 }}
          />
          {searchQuery() ? (
            <Pressable
              onPress={() => setSearchQuery("")}
              accessibilityLabel="Clear search"
              style={{ flexShrink: 0, padding: 2 }}
            >
              <Icon name="lucide:x" size={12} color={theme().textMuted} />
            </Pressable>
          ) : null}
        </View>
      </View>
    </View>
  );
}
