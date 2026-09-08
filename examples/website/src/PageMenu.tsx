import { View, Pressable, Icon } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { Copy, colors } from "./ui";
import { t } from "./i18n";
type Menu = { owner: object; x: number; y: number; view: () => void };
const [menu, setMenu] = createSignal<Menu>();
export const pageMenuOpen = (owner: object) => menu()?.owner === owner;
export const closePageMenu = (owner: object) => {
  if (pageMenuOpen(owner)) setMenu(undefined);
};
export const togglePageMenu = (value: Menu) => setMenu(pageMenuOpen(value.owner) ? undefined : value);
export function PageMenu() {
  return (
    <View style={{ position: "absolute", top: 0, left: 0, width: 0, height: 0 }}>
      {() =>
        menu() ? (
          <View
            focusable
            onKeyDown={(event) => {
              if (event.key === "Escape") setMenu(undefined);
            }}
            onPointerDownOutside={() => setMenu(undefined)}
            style={{
              position: "absolute",
              left: menu()!.x,
              top: menu()!.y,
              width: 198,
              padding: 4,
              borderWidth: 1,
              borderColor: colors.line,
              borderRadius: 8,
              backgroundColor: colors.panel,
            }}
          >
            <Pressable
              focusable
              accessibilityLabel={t("View as Markdown")}
              onPress={() => {
                menu()!.view();
                setMenu(undefined);
              }}
              style={{ padding: 7, borderRadius: 5, backgroundColor: "#262626" }}
            >
              <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                <Icon name="lucide:file" size={14} color={colors.muted} />
                <Copy size={13}>View as Markdown</Copy>
              </View>
            </Pressable>
          </View>
        ) : null
      }
    </View>
  );
}
