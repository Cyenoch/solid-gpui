import { Outlet, useLocation, useNavigate } from "@solid-gpui/router";
import {
  size,
  docsQuery,
  setDocsQuery,
  headerHeight,
  openUrl,
  setSiteError,
  sourceDocument,
  setSourceDocument,
  siteError,
} from "./site";
import { locale, toggleLocale, t } from "./i18n";
import { View, Pressable, Text, TextInput, Icon, Image } from "@solid-gpui/core";
import projectIcon from "../../../assets/branding/icon-64.png?inline";
import { createMemo } from "@solid-gpui/core/runtime";
import { Button, colors } from "./ui";
import { PageMenu } from "./PageMenu";
export function App() {
  const route = useLocation({ select: (location) => location.pathname });
  const go = useNavigate();
  const navigate = (to: string) => {
    void go({ to }).catch((error) => setSiteError(String(error)));
  };
  const section = createMemo(() =>
    route().startsWith("/components")
      ? "components"
      : route().startsWith("/docs")
        ? "docs"
        : route().startsWith("/showcase")
          ? "showcase"
          : "landing",
  );
  const isLanding = createMemo(() => section() === "landing");
  return (
    <View
      style={{
        width: size().width,
        height: size().height,
        backgroundColor: route() !== "/" ? colors.bg : "#00000000",
        fontFamily: "Inter Variable",
        flexDirection: "column",
      }}
    >
      <View
        style={{
          height: 64,
          width: route() !== "/" ? size().width : Math.min(1320, size().width),
          alignSelf: "center",
          backgroundColor: colors.bg,
          flexShrink: 0,
          padding: size().width < 600 ? 12 : 24,
          flexDirection: "row",
          alignItems: "center",
          gap: size().width < 600 ? 2 : 16,
        }}
      >
        <Pressable focusable onPress={() => navigate("/")} accessibilityLabel="Solid GPUI home">
          <View style={{ flexDirection: "row", alignItems: "center", gap: 9 }}>
            <Image source={projectIcon} objectFit="contain" style={{ width: 28, height: 28, borderRadius: 6 }} />
            <Text style={{ fontSize: 18, fontWeight: "bold", color: colors.text }}>Solid GPUI</Text>
          </View>
        </Pressable>
        {() =>
          size().width >= 500 ? (
            <View style={{ flexDirection: "row", gap: 8 }}>
              <Button
                ghost
                icon="lucide:book-open"
                active={route().startsWith("/docs")}
                onPress={() => {
                  setDocsQuery("");
                  navigate("/docs/start");
                }}
              >
                Docs
              </Button>
              <Button
                ghost
                active={route().startsWith("/components")}
                onPress={() => {
                  setDocsQuery("");
                  navigate("/components/button");
                }}
              >
                Components
              </Button>
              <Button ghost active={section() === "showcase"} onPress={() => navigate("/showcase/workspace")}>
                Showcase
              </Button>
            </View>
          ) : null
        }
        <View style={{ flexGrow: 1, minWidth: 0 }} />
        <Button ghost icon="lucide:globe" onPress={toggleLocale}>
          {locale() === "en" ? "中文" : "EN"}
        </Button>
        {() =>
          size().width >= 1100 && section() !== "showcase" ? (
            !isLanding() ? (
              <View
                style={{
                  width: 240,
                  height: 34,
                  padding: 7,
                  gap: 8,
                  flexDirection: "row",
                  alignItems: "center",
                  backgroundColor: colors.panel,
                  borderRadius: 6,
                }}
              >
                <Icon name="lucide:search" size={14} color={colors.muted} />
                <TextInput
                  value={docsQuery()}
                  onChangeText={setDocsQuery}
                  placeholder={t(route().startsWith("/components") ? "Search components…" : "Search documentation…")}
                  accessibilityLabel={t("Search documentation…")}
                  style={{ flexGrow: 1, minWidth: 0, height: 20, fontSize: 13, lineHeight: 20, color: colors.text }}
                />
              </View>
            ) : (
              <Button
                icon="lucide:search"
                onPress={() => navigate("/docs/start")}
                style={{ width: 220, backgroundColor: colors.secondary, borderWidth: 0 }}
              >
                Search documentation…
              </Button>
            )
          ) : null
        }
        {() =>
          size().width > 900 ? (
            <Button ghost icon="lucide:external-link" onPress={() => openUrl("https://github.com/Cyenoch/solid-gpui")}>
              GitHub ↗
            </Button>
          ) : null
        }
      </View>
      {() =>
        size().width < 500 ? (
          <View
            style={{ height: 40, flexShrink: 0, flexDirection: "row", gap: 8, padding: 4, backgroundColor: colors.bg }}
          >
            <Button
              ghost
              compact
              active={route().startsWith("/docs")}
              onPress={() => {
                setDocsQuery("");
                navigate("/docs/start");
              }}
            >
              Docs
            </Button>
            <Button
              ghost
              compact
              active={route().startsWith("/components")}
              onPress={() => {
                setDocsQuery("");
                navigate("/components/button");
              }}
            >
              Components
            </Button>
            <Button ghost compact active={section() === "showcase"} onPress={() => navigate("/showcase/workspace")}>
              Showcase
            </Button>
          </View>
        ) : null
      }
      <View style={{ height: 1, flexShrink: 0, backgroundColor: colors.line }} />
      <Outlet />
      {() =>
        sourceDocument() !== undefined ? (
          <View
            style={{
              position: "absolute",
              top: headerHeight(),
              left: 0,
              width: size().width,
              height: size().height - headerHeight(),
              padding: 24,
              gap: 12,
              backgroundColor: colors.bg,
              overflow: "scroll",
            }}
          >
            <Button icon="lucide:arrow-left" onPress={() => setSourceDocument(undefined)}>
              Close Markdown
            </Button>
            <Text selectable style={{ color: colors.text, fontFamily: "Maple Mono", fontSize: 13, lineHeight: 22 }}>
              {sourceDocument()}
            </Text>
          </View>
        ) : null
      }
      {() => (siteError() ? <Text style={{ color: colors.text }}>{siteError()}</Text> : null)}
      <PageMenu />
    </View>
  );
}
