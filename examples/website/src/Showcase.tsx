import { CodeCollapse } from "./CodeCollapse";
import { PageHeading } from "./PageHeading";
import { View, VirtualList, type VirtualListHandle } from "@solid-gpui/core";
import { createMemo, createSignal } from "@solid-gpui/core/runtime";
import { Workspace, createWorkspaceState } from "./showcase/Workspace";
import { Account, createAccountState } from "./showcase/Account";
import { Collections, createCollectionsState } from "./showcase/Collections";
import { showcases } from "./showcase/catalog";
import { NavItem } from "./Docs";
import { CodeBlock } from "./CodeBlock";
import { PageActions } from "./PageActions";
import { Button, Copy, colors } from "./ui";
import { CodeExcerpt } from "./CodeExcerpt";
import { setSiteError } from "./site";
import { t } from "./i18n";

export function Showcase(props: { width: number; height: number; route: string; navigate: (path: string) => void }) {
  const [menuOpen, setMenuOpen] = createSignal(false);
  const [expanded, setExpanded] = createSignal(false);
  const workspace = createWorkspaceState();
  const account = createAccountState();
  const collections = createCollectionsState();
  let article: VirtualListHandle | undefined;
  const current = createMemo(() => showcases.find((entry) => props.route === `/showcase/${entry.id}`) ?? showcases[0]);
  const index = () => showcases.indexOf(current());
  const mobile = () => props.width < 800;
  const hasToc = () => props.width >= 1400;
  const sidebarWidth = () => (props.width >= 1800 ? 288 : 232);
  const menuHeight = () => (mobile() ? (menuOpen() ? 200 : 64) : props.height);
  const mainWidth = () => props.width - (mobile() ? 0 : sidebarWidth() + (hasToc() ? 256 : 0));
  const width = () => Math.min(680, mainWidth() - (mobile() ? 40 : 80));
  const sections = ["Overview", "Try it", "Components", "Next steps"];
  const go = (entry: (typeof showcases)[number]) => {
    setMenuOpen(false);
    setExpanded(false);
    props.navigate(`/showcase/${entry.id}`);
  };
  const source = () => `# ${current().title}\n\n${t(current().description)}\n\n\`\`\`tsx\n${current().source}\n\`\`\``;
  return (
    <View style={{ height: props.height, flexDirection: mobile() ? "column" : "row" }}>
      <View
        style={{
          width: mobile() ? props.width : sidebarWidth(),
          height: menuHeight(),
          padding: mobile() ? 14 : 24,
          gap: 12,
          flexShrink: 0,
        }}
      >
        {() =>
          mobile() ? (
            <Button
              ghost
              compact
              icon="lucide:list"
              style={{ alignSelf: "flex-start" }}
              onPress={() => setMenuOpen(!menuOpen())}
            >
              {menuOpen() ? "Close navigation" : "Browse examples"}
            </Button>
          ) : (
            <Copy size={12} color={colors.muted}>
              Showcase
            </Copy>
          )
        }
        {() =>
          !mobile() || menuOpen() ? (
            <VirtualList
              data={showcases}
              itemKey={(entry) => entry.id}
              estimatedItemSize={34}
              style={{ height: menuHeight() - 64, width: mobile() ? props.width - 28 : sidebarWidth() - 48 }}
              renderItem={(entry) => (
                <NavItem label={entry.title} active={entry.id === current().id} onPress={() => go(entry)} />
              )}
            />
          ) : null
        }
      </View>
      <View style={{ flexGrow: 1, minWidth: 0 }}>
        {() => {
          const page = props.route;
          return (
            <VirtualList
              ref={(handle) => {
                article = handle;
              }}
              data={sections}
              itemKey={(section) => `${page}:${section}`}
              estimatedItemSize={400}
              initialNumToRender={4}
              overscan={4}
              style={{ width: mainWidth(), height: props.height - (mobile() ? menuHeight() : 0) }}
              renderItem={(section) => (
                <View style={{ alignItems: "center", padding: mobile() ? 20 : 24 }}>
                  <View style={{ width: width(), gap: 24 }}>
                    {() =>
                      section === "Overview" ? (
                        <View style={{ gap: 24, marginTop: 8 }}>
                          <PageHeading title={current().title} width={width()}>
                            <PageActions
                              source={source()}
                              previous={index() ? () => go(showcases[index() - 1]!) : undefined}
                              next={index() < showcases.length - 1 ? () => go(showcases[index() + 1]!) : undefined}
                            />
                          </PageHeading>
                          <Copy size={16} color={colors.muted}>
                            {current().description}
                          </Copy>
                          <View
                            style={{ borderWidth: 1, borderColor: colors.line, borderRadius: 14, overflow: "hidden" }}
                          >
                            <View style={{ padding: width() < 440 ? 12 : 28, minWidth: 0 }}>
                              {() =>
                                current().id === "workspace" ? (
                                  <Workspace state={workspace} />
                                ) : current().id === "account" ? (
                                  <Account state={account} />
                                ) : (
                                  <Collections state={collections} />
                                )
                              }
                            </View>
                            <View style={{ height: 1, backgroundColor: colors.line }} />
                            {() =>
                              expanded() ? (
                                <CodeBlock source={current().source} label={`${current().title}.tsx`} />
                              ) : (
                                <CodeExcerpt
                                  source={current().source}
                                  width={width()}
                                  onExpand={() => setExpanded(true)}
                                />
                              )
                            }
                            {() => (expanded() ? <CodeCollapse onPress={() => setExpanded(false)} /> : null)}
                          </View>
                        </View>
                      ) : section === "Try it" ? (
                        <>
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            Try it
                          </Copy>
                          <Copy color={colors.muted}>{current().note}</Copy>
                        </>
                      ) : section === "Components" ? (
                        <>
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            Components
                          </Copy>
                          <Copy color={colors.muted}>Explore the components used in this example.</Copy>
                          {() =>
                            current().components.map((name) => (
                              <Button
                                ghost
                                icon="lucide:box"
                                style={{ alignSelf: "flex-start" }}
                                onPress={() => props.navigate(`/components/${name.toLowerCase()}`)}
                              >
                                {name}
                              </Button>
                            ))
                          }
                        </>
                      ) : (
                        <View style={{ gap: 20 }}>
                          <View style={{ height: 1, backgroundColor: colors.line }} />
                          <View style={{ flexDirection: "row", justifyContent: "space-between", gap: 12 }}>
                            {() =>
                              index() > 0 ? (
                                <Button compact icon="lucide:arrow-left" onPress={() => go(showcases[index() - 1]!)}>
                                  {showcases[index() - 1]!.title}
                                </Button>
                              ) : (
                                <View />
                              )
                            }
                            {() =>
                              index() < showcases.length - 1 ? (
                                <Button
                                  compact
                                  icon="lucide:chevron-right"
                                  iconAfter
                                  onPress={() => go(showcases[index() + 1]!)}
                                >
                                  {showcases[index() + 1]!.title}
                                </Button>
                              ) : null
                            }
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
          <View style={{ width: 256, padding: 24, gap: 4, flexShrink: 0 }}>
            <Copy size={12} color={colors.muted} style={{ marginTop: 12, marginBottom: 12 }}>
              On this page
            </Copy>
            {sections.map((section, index) => (
              <NavItem
                label={section}
                onPress={() => {
                  void article?.scrollToIndex(index).catch((error) => setSiteError(String(error)));
                }}
              />
            ))}
          </View>
        ) : null
      }
    </View>
  );
}
