import { Icon, Text, View } from "@solid-gpui/core";
import { createMemo } from "@solid-gpui/core/runtime";
import { Link } from "@solid-gpui/router";
import { useGallery } from "../context";
import { CATEGORIES, PAGES, pagePath } from "../types";

export function Sidebar(props: { readonly compact: boolean }) {
  const { theme, searchQuery } = useGallery();
  const filteredPages = createMemo(() => {
    const q = searchQuery().trim().toLowerCase();
    if (!q) return PAGES;
    return PAGES.filter(
      (page) =>
        page.title.toLowerCase().includes(q) ||
        page.description.toLowerCase().includes(q) ||
        page.keywords.some((keyword) => keyword.toLowerCase().includes(q)),
    );
  });

  return (
    <View
      accessibilityLabel="Component navigation"
      style={{
        width: props.compact ? 52 : 190,
        minWidth: 0,
        minHeight: 0,
        flexShrink: 0,
        backgroundColor: theme().bgSidebar,
        borderWidth: 1,
        borderColor: theme().borderMuted,
        overflow: "scroll",
      }}
    >
      <View style={{ padding: props.compact ? 6 : 10, gap: 18, flexShrink: 0, minWidth: 0 }}>
        {CATEGORIES.map((cat) => {
          const pages = createMemo(() => filteredPages().filter((page) => page.category === cat.id));
          return (
            <>
              {pages().length > 0 ? (
                <View style={{ gap: 3, minWidth: 0, flexShrink: 0 }}>
                  {!props.compact ? (
                    <Text style={{ color: theme().textMuted, fontSize: 10, fontWeight: "semibold", padding: 6 }}>
                      {cat.title.toUpperCase()}
                    </Text>
                  ) : null}
                  {pages().map((page) => (
                    <Link
                      to={pagePath(page.id)}
                      activeOptions={{ exact: true }}
                      accessibilityLabel={page.title}
                      tooltip={page.title}
                      style={{
                        minWidth: 0,
                        flexShrink: 0,
                        padding: 9,
                        borderRadius: 6,
                        cursor: "pointer",
                        backgroundColor: "#00000000",
                      }}
                      activeStyle={{ backgroundColor: theme().accentMuted }}
                    >
                      {(state) => (
                        <View style={{ flexDirection: "row", alignItems: "center", gap: 9, minWidth: 0 }}>
                          <Icon
                            name={page.icon}
                            size={15}
                            color={state.isActive ? theme().accent : theme().textMuted}
                          />
                          {!props.compact ? (
                            <Text
                              style={{
                                color: state.isActive ? theme().accent : theme().textSecondary,
                                fontSize: 12,
                                fontWeight: state.isActive ? "semibold" : "normal",
                                minWidth: 0,
                                flexShrink: 1,
                              }}
                            >
                              {page.title}
                            </Text>
                          ) : null}
                        </View>
                      )}
                    </Link>
                  ))}
                </View>
              ) : null}
            </>
          );
        })}
        {filteredPages().length === 0 ? (
          <Text style={{ color: theme().textMuted, fontSize: 11, minWidth: 0 }}>
            {props.compact ? "—" : "No matching components"}
          </Text>
        ) : null}
      </View>
    </View>
  );
}
