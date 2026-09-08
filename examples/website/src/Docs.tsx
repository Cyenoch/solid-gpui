import { PageHeading } from "./PageHeading";
import { openUrl } from "./site";
import { PageActions } from "./PageActions";
import { MarkdownTable } from "./MarkdownTable";
import { MarkdownText } from "./MarkdownText";
import { CodeBlock } from "./CodeBlock";
import { locale, t } from "./i18n";
import { chapterChinese } from "./locale.zh-CN";
import { View, Pressable, TextInput, VirtualList, type VirtualListHandle } from "@solid-gpui/core";
import { createSignal, createMemo } from "@solid-gpui/core/runtime";
import { Copy, Button, colors, panel, Counter, LayoutDemo } from "./ui";
import { examples } from "./examples";
import { markdownBlocks, referenceDocs, type MarkdownBlock } from "./documentation";
const chapters = [
  {
    id: "start",
    title: "Getting started",
    body: "Build a small app with the components you already expect: containers, text, buttons, and inputs. If you know Solid signals and JSX, you are ready to begin.",
    detail:
      "Start by cloning the repository and following the installation guide for your platform. Run the native website, then change a component and see your app update. The example below shows the essentials: a signal, some text, and a click handler.",
    sample: "Counter",
  },
  {
    id: "reactivity",
    title: "Signals & updates",
    body: "Use a signal for values that change, such as a counter, a selected item, or a form field. Read it in JSX and the interface updates when its value changes.",
    detail:
      "Keep state close to the components that use it. Use createMemo for derived values and onCleanup to release timers or subscriptions. Try the counter below to see a signal in action.",
    sample: "Counter",
  },
  {
    id: "layout",
    title: "Layout & styling",
    body: "Group components with View. Arrange them in rows or columns with flexDirection, and add space with gap and padding. Dimensions use pixels.",
    detail:
      "Give scrollable sections a height, allow text to wrap, and test your layout in a smaller window. The example below switches between a row and a column.",
    sample: "Layout",
  },
  {
    id: "input",
    title: "Text & forms",
    body: "Connect TextInput to a signal with value and onChangeText. Use the signal to show a live preview, validate a field, or save the result.",
    detail:
      "Add a placeholder to explain what to enter. Use multiline for longer text and disabled when a field is unavailable. Try changing the name below.",
    sample: "Input",
  },
  {
    id: "web",
    title: "Using the web version",
    body: "Explore the components in your browser without installing the desktop app. This site uses the same Solid component API as the native examples.",
    detail:
      "Web support is experimental. Basic components are available; desktop services and the full component library are not yet supported. Check the Web setup guide before choosing it for a production app.",
    sample: "Counter",
  },
] as const;

const referenceLabels: Record<string, string> = {
  "getting-started": "Installation",
  runtimes: "Choose a runtime",
  "runtime-strategy": "Runtime strategy",
  "gpui-components": "Component library",
  "native-migration": "Native application migration",
  "hot-reload": "Development workflow",
  router: "Router",
  "keyboard-and-menus": "Keyboard & menus",
  "native-composition": "Async & accessibility",
  distribution: "Distribution",
  web: "Web setup",
  "rust-bridge": "Rust integration",
  shiki: "Syntax highlighting",
  protocol: "Protocol",
  "performance-analysis": "Performance analysis",
  "scroll-performance": "Scroll performance",
  troubleshooting: "Troubleshooting",
};

const referenceOrder = new Map(Object.keys(referenceLabels).map((id, index) => [id, index]));
const referenceNavigation = [...referenceDocs].sort(
  (a, b) => (referenceOrder.get(a.id) ?? referenceOrder.size) - (referenceOrder.get(b.id) ?? referenceOrder.size),
);

export function NavItem(props: { label: string; translate?: boolean; active?: boolean; onPress: () => void }) {
  const [hovered, setHovered] = createSignal(false);
  const [focused, setFocused] = createSignal(false);
  return (
    <Pressable
      focusable
      accessibilityRole="link"
      accessibilitySelected={props.active}
      onPress={props.onPress}
      onHoverChange={setHovered}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      style={{
        alignSelf: "flex-start",
        maxWidth: 208,
        flexShrink: 0,
        padding: 5,
        borderRadius: 6,
        borderWidth: 1,
        borderColor: focused() ? colors.muted : "#00000000",
        backgroundColor: props.active ? colors.secondary : "#00000000",
      }}
    >
      <Copy
        translate={props.translate}
        size={13}
        color={props.active || hovered() || focused() ? colors.text : colors.muted}
        style={{ lineHeight: 20 }}
      >
        {props.label}
      </Copy>
    </Pressable>
  );
}

type Section = { id: string; title: string; blocks?: MarkdownBlock[] };

export function Docs(props: {
  route: string;
  width: number;
  height: number;
  navigate: (path: string) => void;
  query: string;
  setQuery: (value: string) => void;
}) {
  const query = () => props.query;
  const setQuery = props.setQuery;
  const [menuOpen, setMenuOpen] = createSignal(false);
  const [showCode, setShowCode] = createSignal(false);
  const [name, setName] = createSignal("World");
  const [jumpError, setJumpError] = createSignal("");
  let article: VirtualListHandle | undefined;
  const mobile = () => props.width < 800;
  const hasToc = () => props.width >= 1400;
  const sidebarWidth = () => (props.width >= 1800 ? 288 : 232);
  const menuHeight = () => (mobile() ? (menuOpen() ? Math.min(420, props.height / 2) : 64) : props.height);
  const articleWidth = () => Math.min(680, props.width - (mobile() ? 40 : sidebarWidth() + (hasToc() ? 256 : 0) + 80));
  const current = () => chapters.find((chapter) => props.route === `/docs/${chapter.id}`) ?? chapters[0];
  const reference = () => referenceDocs.find((doc) => props.route === `/docs/reference/${doc.id}`);
  const pages = [
    ...chapters.map((chapter) => `/docs/${chapter.id}`),
    ...referenceNavigation.map((doc) => `/docs/reference/${doc.id}`),
  ];
  const pageIndex = () => pages.indexOf(props.route);
  const referenceSource = () => (locale() === "zh-CN" ? reference()?.sourceChinese : reference()?.source);
  const pageSource = () =>
    referenceSource() ??
    `# ${t(current().title)}\n\n${locale() === "zh-CN" ? chapterChinese[current().id].body : current().body}\n\n${locale() === "zh-CN" ? chapterChinese[current().id].detail : current().detail}\n\n## ${t("Example")}\n\n\`\`\`tsx\n${examples[current().sample]}\n\`\`\``;
  const actions = () => (
    <PageActions
      source={pageSource()}
      previous={pageIndex() > 0 ? () => navigate(pages[pageIndex() - 1]) : undefined}
      next={pageIndex() < pages.length - 1 ? () => navigate(pages[pageIndex() + 1]) : undefined}
    />
  );
  const title = (text: string) => (
    <PageHeading title={text} width={articleWidth()}>
      {actions()}
    </PageHeading>
  );
  const sections = createMemo<Section[]>(() => {
    if (!reference())
      return [
        { id: "overview", title: "Overview" },
        { id: "example", title: "Example" },
        { id: "next", title: "Next steps" },
      ];
    const result: Section[] = [];
    for (const block of markdownBlocks(referenceSource()!)) {
      if (!result.length || (block.kind === "heading" && (block.level ?? 1) <= 2))
        result.push({
          id: `section-${result.length}`,
          title: block.kind === "heading" ? block.text : "Overview",
          blocks: [],
        });
      result[result.length - 1].blocks!.push(block);
    }
    return result;
  });
  const navigate = (path: string) => {
    setMenuOpen(false);
    setShowCode(false);
    setJumpError("");
    props.navigate(path);
  };
  const openLink = (href: string) => {
    const file = href.split("#")[0].split("/").pop()?.replace(/\.md$/, "");
    if (file && referenceDocs.some((doc) => doc.id === file)) navigate(`/docs/reference/${file}`);
    else {
      const url = new URL(
        href,
        `https://github.com/Cyenoch/solid-gpui/blob/main/docs/${reference()?.id ?? "getting-started"}${locale() === "zh-CN" ? ".zh-CN" : ""}.md`,
      );
      if (url.protocol === "https:" || url.protocol === "http:") openUrl(url.href);
    }
  };
  const filteredChapters = createMemo(() =>
    chapters.filter((chapter) =>
      `${t(chapter.title)} ${chapter.title} ${chapter.body}`.toLowerCase().includes(query().toLowerCase()),
    ),
  );
  const filteredReferences = createMemo(() =>
    referenceNavigation.filter((doc) =>
      `${t(referenceLabels[doc.id] ?? doc.title)} ${doc.title} ${doc.source}`
        .toLowerCase()
        .includes(query().toLowerCase()),
    ),
  );
  const jump = (index: number) => {
    article?.scrollToIndex(index).catch((error) => setJumpError(String(error)));
  };
  return (
    <View style={{ height: props.height, flexDirection: mobile() ? "column" : "row" }}>
      <View
        style={{
          width: mobile() ? props.width : sidebarWidth(),
          height: menuHeight(),
          flexShrink: 0,
          padding: mobile() ? 14 : 24,
          gap: 2,
          overflow: "scroll",
        }}
      >
        {() =>
          mobile() ? (
            <Button
              ghost
              compact
              icon="lucide:list"
              onPress={() => setMenuOpen(!menuOpen())}
              style={{ alignSelf: "flex-start" }}
            >
              {menuOpen() ? "Close navigation" : "Browse documentation"}
            </Button>
          ) : null
        }
        {() =>
          !mobile() || menuOpen() ? (
            <View style={{ gap: 2, flexShrink: 0 }}>
              {() =>
                mobile() ? (
                  <TextInput
                    value={query()}
                    onChangeText={setQuery}
                    placeholder={t("Search topics…")}
                    accessibilityLabel={t("Search topics…")}
                    style={{
                      height: 34,
                      fontSize: 13,
                      lineHeight: 20,
                      borderRadius: 6,
                      padding: 7,
                      marginBottom: 24,
                      flexShrink: 0,
                      color: colors.text,
                      backgroundColor: colors.panel,
                    }}
                  />
                ) : null
              }
              <Copy size={12} color={colors.muted} style={{ marginLeft: 6, marginBottom: 8 }}>
                Guides
              </Copy>
              {() =>
                filteredChapters().map((chapter) => (
                  <NavItem
                    label={chapter.title}
                    active={!reference() && current().id === chapter.id}
                    onPress={() => navigate(`/docs/${chapter.id}`)}
                  />
                ))
              }
              <Copy size={12} color={colors.muted} style={{ marginLeft: 6, marginTop: 28, marginBottom: 8 }}>
                Reference
              </Copy>
              {() =>
                filteredReferences().map((doc) => (
                  <NavItem
                    label={referenceLabels[doc.id] ?? doc.title}
                    active={reference()?.id === doc.id}
                    onPress={() => navigate(`/docs/reference/${doc.id}`)}
                  />
                ))
              }
              {() =>
                !filteredChapters().length && !filteredReferences().length ? <Copy>No matching topics.</Copy> : null
              }
            </View>
          ) : null
        }
      </View>
      <View style={{ flexGrow: 1, minWidth: 0, height: props.height - (mobile() ? menuHeight() : 0) }}>
        {() => {
          const page = props.route;
          return (
            <VirtualList<Section>
              ref={(handle) => {
                article = handle;
              }}
              data={sections()}
              itemKey={(section) => `${page}:${section.id}`}
              estimatedItemSize={400}
              initialNumToRender={sections().length}
              overscan={sections().length}
              style={{
                height: props.height - (mobile() ? menuHeight() : 0),
                width: props.width - (mobile() ? 0 : sidebarWidth() + (hasToc() ? 256 : 0)),
                minWidth: 0,
              }}
              renderItem={(section, index) => (
                <View style={{ alignItems: "center", padding: mobile() ? 20 : 24 }}>
                  <View
                    style={{
                      width: articleWidth(),
                      gap: 24,
                      flexShrink: 0,
                      marginTop: index === 0 && !mobile() ? 8 : 0,
                    }}
                  >
                    {() =>
                      reference() ? (
                        <View style={{ gap: 22 }}>
                          {() =>
                            section.blocks?.map((block) =>
                              block.kind === "table" ? (
                                <MarkdownTable block={block} width={articleWidth()} open={openLink} />
                              ) : block.kind === "code" ? (
                                <CodeBlock source={block.text} language={block.language} label={block.language} />
                              ) : block.kind === "heading" && block.level === 1 ? (
                                title(block.text)
                              ) : block.kind === "heading" ? (
                                <Copy
                                  size={block.level === 1 ? 30 : 20}
                                  style={{ fontWeight: "bold", lineHeight: block.level === 1 ? 38 : 28 }}
                                >
                                  {block.text}
                                </Copy>
                              ) : (
                                <MarkdownText source={block.text} open={openLink} />
                              ),
                            )
                          }
                        </View>
                      ) : section.id === "overview" ? (
                        <View style={{ gap: 24 }}>
                          <View style={{ gap: 10 }}>
                            {title(current().title)}
                            <Copy size={16} color={colors.muted} style={{ lineHeight: 26 }}>
                              {locale() === "zh-CN" ? chapterChinese[current().id].body : current().body}
                            </Copy>
                          </View>
                          <Copy selectable size={15} style={{ lineHeight: 26 }}>
                            {locale() === "zh-CN" ? chapterChinese[current().id].detail : current().detail}
                          </Copy>
                          {() =>
                            current().id === "start" || current().id === "web" ? (
                              <Button
                                compact
                                primary
                                icon="lucide:download"
                                style={{ alignSelf: "flex-start" }}
                                onPress={() =>
                                  navigate(`/docs/reference/${current().id === "web" ? "web" : "getting-started"}`)
                                }
                              >
                                {current().id === "web" ? "Web setup guide" : "Installation guide"}
                              </Button>
                            ) : null
                          }
                        </View>
                      ) : section.id === "example" ? (
                        <View style={{ gap: 20 }}>
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            Example
                          </Copy>
                          <View style={{ flexDirection: "row", gap: 4 }}>
                            <Button
                              compact
                              ghost
                              icon="lucide:monitor"
                              active={!showCode()}
                              onPress={() => setShowCode(false)}
                            >
                              Preview
                            </Button>
                            <Button
                              compact
                              ghost
                              icon="lucide:file"
                              active={showCode()}
                              onPress={() => setShowCode(true)}
                            >
                              Code
                            </Button>
                          </View>
                          {() =>
                            showCode() ? (
                              <CodeBlock source={examples[current().sample]} label="App.tsx" />
                            ) : (
                              <View
                                style={{
                                  borderWidth: 1,
                                  borderColor: colors.line,
                                  borderRadius: 12,
                                  minHeight: 360,
                                  padding: mobile() ? 16 : 32,
                                  alignItems: "center",
                                  justifyContent: "center",
                                }}
                              >
                                <View
                                  style={{ width: Math.min(360, articleWidth() - (mobile() ? 34 : 66)), flexShrink: 0 }}
                                >
                                  {() =>
                                    current().sample === "Input" ? (
                                      <View style={panel}>
                                        <Copy size={24}>
                                          {locale() === "en" ? `Hello, ${name()}!` : `你好，${name()}！`}
                                        </Copy>
                                        <TextInput
                                          value={name()}
                                          onChangeText={setName}
                                          accessibilityLabel={t("Your name")}
                                          style={{
                                            height: 42,
                                            padding: 10,
                                            color: colors.text,
                                            backgroundColor: colors.bg,
                                            borderRadius: 6,
                                          }}
                                        />
                                      </View>
                                    ) : current().sample === "Layout" ? (
                                      <LayoutDemo compact={articleWidth() < 360} />
                                    ) : (
                                      <Counter />
                                    )
                                  }
                                </View>
                              </View>
                            )
                          }
                        </View>
                      ) : (
                        <View style={{ gap: 20 }}>
                          <View style={{ height: 1, backgroundColor: colors.line }} />
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            Next steps
                          </Copy>
                          <View style={{ flexDirection: mobile() ? "column" : "row", gap: 12 }}>
                            <Button
                              compact
                              icon="lucide:book-open"
                              onPress={() =>
                                navigate(
                                  `/docs/${chapters[(chapters.findIndex((c) => c.id === current().id) + 1) % chapters.length].id}`,
                                )
                              }
                            >
                              {chapters[(chapters.findIndex((c) => c.id === current().id) + 1) % chapters.length].title}
                            </Button>
                            <Button
                              compact
                              ghost
                              icon="lucide:external-link"
                              onPress={() => openLink("https://github.com/Cyenoch/solid-gpui/tree/main/docs")}
                            >
                              View on GitHub
                            </Button>
                          </View>
                        </View>
                      )
                    }
                  </View>
                </View>
              )}
            />
          );
        }}
      </View>
      {() =>
        hasToc() ? (
          <View style={{ width: 256, height: props.height, padding: 24, gap: 4, flexShrink: 0, overflow: "scroll" }}>
            <Copy size={12} color={colors.muted} style={{ marginTop: 12, marginBottom: 12, marginLeft: 6 }}>
              On this page
            </Copy>
            {() =>
              sections().map((section, index) => (
                <NavItem label={index === 0 ? "Overview" : section.title} onPress={() => jump(index)} />
              ))
            }
            {() => (jumpError() ? <Copy size={12}>{jumpError()}</Copy> : null)}
            <View style={{ height: 1, backgroundColor: colors.line, marginTop: 24, marginBottom: 16 }} />
            <Button
              ghost
              compact
              icon="lucide:external-link"
              style={{ alignSelf: "flex-start" }}
              onPress={() =>
                openLink(
                  `https://github.com/Cyenoch/solid-gpui/${reference() ? `blob/main/docs/${reference()!.id}${locale() === "zh-CN" ? ".zh-CN" : ""}.md` : "tree/main/examples"}`,
                )
              }
            >
              View on GitHub
            </Button>
          </View>
        ) : null
      }
    </View>
  );
}
