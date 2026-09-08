/// <reference types="vite/client" />
import { Icon, Image, Text, View, mountApplication, type Root } from "@solid-gpui/core";
import { Button, Input, Tag, TitleBar, createClient, type ApplicationTheme } from "@solid-gpui/core/components";
import { createSignal, onCleanup } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { createRootRoute, createRoute, createRouter, RouterProvider, Outlet, Link } from "@solid-gpui/router";
import { useNative, applicationIcons } from "./native";

const cover = "assets/cover.png";

const [brand] = applicationIcons;
const palette = {
  background: "#131217",
  sidebar: "#0F0E12",
  surface: "#1B1A20",
  raised: "#222127",
  border: "#2C2B33",
  foreground: "#ECEAF1",
  muted: "#A09DA9",
  primary: "#D4688C",
  primaryHover: "#E07B9E",
  primaryForeground: "#241219",
  danger: "#C4574E",
};
const theme: ApplicationTheme = {
  inputBackground: palette.surface,
  components: {
    button: { height: 32, fontSize: 14, lineHeight: 20, paddingX: 12 },
    input: { height: 32, fontSize: 14, lineHeight: 20 },
    select: { height: 32, fontSize: 14, lineHeight: 20 },
    tag: { fontSize: 12, lineHeight: 16, paddingY: 2 },
    menu: { height: 30, fontSize: 14, lineHeight: 20 },
  },
  colors: {
    background: palette.background,
    foreground: palette.foreground,
    border: palette.border,
    sidebar: palette.sidebar,
    sidebarForeground: palette.foreground,
    popover: palette.raised,
    popoverForeground: palette.foreground,
    input: palette.surface,
    muted: palette.raised,
    mutedForeground: palette.muted,
    primary: palette.primary,
    primaryHover: palette.primaryHover,
    primaryActive: palette.primary,
    primaryForeground: palette.primaryForeground,
    ring: palette.primary,
    button: palette.surface,
    buttonHover: palette.raised,
    buttonActive: palette.raised,
    buttonForeground: palette.foreground,
    buttonPrimary: palette.primary,
    buttonPrimaryHover: palette.primaryHover,
    buttonPrimaryActive: palette.primary,
    buttonPrimaryForeground: palette.primaryForeground,
    danger: palette.danger,
    buttonDanger: palette.danger,
    titleBar: palette.background,
    titleBarBorder: palette.border,
  },
  fontFamily: ".SystemUIFont",
  fontSize: 14,
  lineHeight: 20,
  radius: 6,
  radiusLg: 10,
};

function Home() {
  const native = useNative();
  const [calls, setCalls] = createSignal(0);
  return (
    <View style={{ padding: 24, gap: 20 }}>
      <View style={{ height: 230, position: "relative", borderRadius: 12, overflow: "hidden" }}>
        <Image source={cover} objectFit="cover" style={{ widthPercent: 100, heightPercent: 100 }} />
        <View
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            linearGradient: {
              angle: 180,
              stops: [
                { color: "#13121700", position: 0 },
                { color: "#131217FF", position: 1 },
              ],
            },
            padding: 20,
            justifyContent: "flex-end",
            gap: 8,
          }}
        >
          <Text style={{ color: "#FFFFFF", fontSize: 24 }}>Native migration fixture</Text>
          <Tag>
            <Text>Offline resources</Text>
          </Tag>
        </View>
      </View>
      <View style={{ flexDirection: "row", gap: 0 }}>
        <Button
          variant="primary"
          style={{ borderTopRightRadius: 0, borderBottomRightRadius: 0 }}
          onPress={async () => setCalls(await native.serviceCount())}
        >
          <Text>Start game</Text>
        </Button>
        <Button
          variant="primary"
          style={{ borderTopLeftRadius: 0, borderBottomLeftRadius: 0, borderLeftWidth: 1, borderColor: palette.border }}
        >
          <Text>⌄</Text>
        </Button>
      </View>
      <Text>Rust service calls: {calls()}</Text>
      <Input placeholder="Native input" />
      <View style={{ gap: 16 }}>
        {[
          ["Runtime: Bun", "Renderer: GPUI"],
          ["Navigation: router", "Assets: embedded"],
        ].map((row) => (
          <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 16 }}>
            {row.map((label) => (
              <View style={{ width: 0, minWidth: 300, flexGrow: 1, padding: 12, backgroundColor: palette.surface }}>
                <Text>{label}</Text>
              </View>
            ))}
          </View>
        ))}
      </View>
    </View>
  );
}
function Shell(props: { onFullscreen: () => void }) {
  return (
    <View
      style={{
        heightPercent: 100,
        flexDirection: "column",
        backgroundColor: palette.background,
        color: palette.foreground,
        fontFamily: ".SystemUIFont",
        fontSize: 14,
        lineHeight: 20,
      }}
    >
      <TitleBar
        style={{
          height: 48,
          padding: 0,
          paddingLeft: 88,
          paddingRight: 16,
          backgroundColor: palette.background,
          borderWidth: 0,
          borderBottomWidth: 1,
          borderColor: palette.border,
        }}
      >
        <View style={{ flexDirection: "row", flexGrow: 1, alignItems: "center", gap: 16 }}>
          <Icon name={brand} color={palette.primary} size={22} />
          <Text>Migration fixture</Text>
          <Link to="/">
            <Text>Home</Text>
          </Link>
          <Link to="/settings">
            <Text>Settings</Text>
          </Link>
          <Input placeholder="Title bar input" style={{ width: 160 }} />
          <Button label="Fullscreen" onPress={props.onFullscreen} />
        </View>
      </TitleBar>
      <View style={{ flexGrow: 1, minHeight: 0, overflow: "scroll" }}>
        <Outlet />
      </View>
      <View
        style={{
          height: 40,
          paddingLeft: 24,
          justifyContent: "center",
          borderTopWidth: 1,
          borderColor: palette.border,
        }}
      >
        <Text>SolidJS pages · Rust services</Text>
      </View>
    </View>
  );
}
mountApplication<string>({
  transport: () => new StdioTransport(),
  hotKey: import.meta.hot ? import.meta.url : undefined,
  setup(previous = "/") {
    let mountedRoot: Root | undefined;
    let active = true;
    onCleanup(() => {
      active = false;
      mountedRoot = undefined;
    });
    const rootRoute = createRootRoute({
      component: () => <Shell onFullscreen={() => void mountedRoot?.toggleFullscreen()} />,
    });
    const home = createRoute({ getParentRoute: () => rootRoute, path: "/", component: Home });
    const settings = createRoute({
      getParentRoute: () => rootRoute,
      path: "/settings",
      component: () => <Text style={{ padding: 24 }}>Settings fixture</Text>,
    });
    const router = createRouter({ routeTree: rootRoute.addChildren([home, settings]), initialEntries: [previous] });
    return {
      render: () => <RouterProvider router={router} />,
      onMount(root) {
        mountedRoot = root;
        const native = createClient(root);
        void native
          .setTheme("dark")
          .then(() => (active ? native.setApplicationTheme(theme) : undefined))
          .catch((error: unknown) => {
            if (active) console.error("Theme configuration failed:", error);
          });
      },
      captureState: () => router.stores.location.get().href,
    };
  },
});
