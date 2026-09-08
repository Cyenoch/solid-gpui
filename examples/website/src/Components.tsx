import { PageHeading } from "./PageHeading";
import { TypeSignature } from "./TypeSignature";
import { MarkdownText } from "./MarkdownText";
import { ComponentPreview } from "./ComponentPreview";
import { openUrl } from "./site";
import { View, Icon, TextInput, VirtualList, type VirtualListHandle } from "@solid-gpui/core";
import { createMemo, createSignal } from "@solid-gpui/core/runtime";
import catalog from "virtual:component-catalog";
import previews from "virtual:component-previews";
import { NavItem } from "./Docs";
import { PageActions } from "./PageActions";
import { MarkdownTable } from "./MarkdownTable";
import { CodeBlock } from "./CodeBlock";
import { importCode, installCode } from "./snippets";
import { Button, Copy, colors } from "./ui";
import { locale, t } from "./i18n";

type Entry = (typeof catalog)[number];
type Field = Entry["properties"][number];
export function Components(props: {
  route: string;
  width: number;
  height: number;
  query: string;
  setQuery: (value: string) => void;
  navigate: (path: string) => void;
}) {
  const [menuOpen, setMenuOpen] = createSignal(false);
  const [error, setError] = createSignal("");
  let article: VirtualListHandle | undefined;
  const mobile = () => props.width < 800;
  const toc = () => props.width >= 1400;
  const sidebar = () => (props.width >= 1800 ? 288 : 232);
  const mainWidth = () => props.width - (mobile() ? 0 : sidebar() + (toc() ? 256 : 0));
  const width = () => Math.min(680, mainWidth() - (mobile() ? 40 : 80));
  const menuHeight = () => (mobile() ? (menuOpen() ? Math.min(420, props.height / 2) : 64) : props.height);
  const current = () =>
    catalog.find((entry) => entry.members.some((member) => `/components/${member.id}` === props.route)) ??
    catalog.find((entry) => entry.name === "Button")!;
  const description = () => (locale() === "en" ? current().description : current().descriptionChinese);
  const index = () => catalog.indexOf(current());
  const filtered = createMemo(() =>
    catalog.filter((entry) =>
      `${entry.members.map((member) => member.name).join(" ")} ${entry.description} ${entry.descriptionChinese}`
        .toLowerCase()
        .includes(props.query.toLowerCase()),
    ),
  );
  const navigate = (entry: Entry) => {
    setMenuOpen(false);
    setError("");
    props.navigate(`/components/${entry.id}`);
  };
  const sections = createMemo(() => [
    "Overview",
    "Installation",
    "Usage",
    ...current().examples.map((example) => example.title),
    "API Reference",
    ...current().members.map((member) => member.name),
    ...(current().definitions.length ? ["Related types"] : []),
    "Next steps",
  ]);
  const source = () =>
    `# ${current().name}\n\n${description()}\n\n## Usage\n\n\`\`\`tsx\n${current().source}\`\`\`\n\n` +
    current()
      .examples.map(
        (example) => `## ${t(example.title)}\n\n${t(example.description)}\n\n\`\`\`tsx\n${example.source}\n\`\`\``,
      )
      .join("\n\n") +
    "\n\n## API Reference\n\n" +
    current()
      .members.map(
        (member) =>
          `### ${member.name}\n\n` +
          [
            ["Properties", member.properties],
            ["Events", member.events],
            ["Commands", member.commands],
          ]
            .map(([label, rows]) => {
              const fields = rows as Field[];
              return fields.length
                ? `#### ${label}\n\n| Name | Type | Required |\n| --- | --- | --- |\n` +
                    fields
                      .map(
                        (field) =>
                          `| ${field.name} | \`${field.type.replaceAll("|", "\\|")}\` | ${field.required ? "Yes" : "No"} |`,
                      )
                      .join("\n")
                : "";
            })
            .filter(Boolean)
            .join("\n\n") +
          `\n\n${member.children ? "Accepts child content." : "Does not accept child content."}\n\n${member.slots.length ? `Named slots: ${member.slots.join(", ")}` : ""}`,
      )
      .join("\n\n");
  const open = (href: string) => openUrl(href);
  const table = (rows: Field[]) => (
    <MarkdownTable
      width={width()}
      open={open}
      renderCell={(cell, column, row) =>
        column === 1 ? <TypeSignature source={rows[row].type} /> : <MarkdownText source={cell} open={open} size={13} />
      }
      block={{
        kind: "table",
        text: "",
        header: [t("Prop"), t("Type"), t("Required")],
        align: ["left", "left", "left"],
        rows: rows.map((field) => [`\`${field.name}\``, `\`${field.type}\``, t(field.required ? "Yes" : "No")]),
      }}
    />
  );
  return (
    <View style={{ height: props.height, flexDirection: mobile() ? "column" : "row" }}>
      <View
        style={{
          width: mobile() ? props.width : sidebar(),
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
              {menuOpen() ? "Close navigation" : "Browse components"}
            </Button>
          ) : (
            <Copy size={12} color={colors.muted}>
              Components
            </Copy>
          )
        }
        {() =>
          !mobile() || menuOpen() ? (
            <View style={{ flexGrow: 1, minHeight: 0, gap: 12 }}>
              {() =>
                mobile() ? (
                  <TextInput
                    value={props.query}
                    onChangeText={props.setQuery}
                    placeholder={t("Search components…")}
                    style={{
                      height: 34,
                      fontSize: 13,
                      padding: 7,
                      backgroundColor: colors.panel,
                      color: colors.text,
                      borderRadius: 6,
                    }}
                  />
                ) : null
              }
              <VirtualList
                data={filtered()}
                itemKey={(entry) => entry.id}
                estimatedItemSize={34}
                style={{
                  width: mobile() ? props.width - 28 : sidebar() - 48,
                  height: menuHeight() - (mobile() ? 112 : 64),
                }}
                renderItem={(entry) => (
                  <NavItem
                    translate={false}
                    label={entry.name}
                    active={entry.id === current().id}
                    onPress={() => navigate(entry)}
                  />
                )}
                emptyState={<Copy>No matching components.</Copy>}
              />
            </View>
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
              data={sections()}
              itemKey={(section) => `${page}:${section}`}
              estimatedItemSize={360}
              initialNumToRender={sections().length}
              overscan={sections().length}
              style={{ width: mainWidth(), height: props.height - (mobile() ? menuHeight() : 0) }}
              renderItem={(section) => (
                <View style={{ alignItems: "center", padding: mobile() ? 20 : 24 }}>
                  <View style={{ width: width(), gap: 24 }}>
                    {() =>
                      section === "Overview" ? (
                        <View style={{ gap: 24, marginTop: 8 }}>
                          <PageHeading translate={false} title={current().name} width={width()}>
                            <PageActions
                              source={source()}
                              previous={index() > 0 ? () => navigate(catalog[index() - 1]) : undefined}
                              next={index() < catalog.length - 1 ? () => navigate(catalog[index() + 1]) : undefined}
                            />
                          </PageHeading>
                          <Copy size={16} color={colors.muted}>
                            {description()}
                          </Copy>
                          <ComponentPreview
                            unavailable={current().previewNote}
                            previewWidth={current().previewWidth}
                            minPreviewWidth={current().previewMinWidth}
                            centered={current().previewCentered}
                            source={current().source}
                            preview={previews[current().name]}
                            width={width()}
                          />
                        </View>
                      ) : section === "Installation" ? (
                        <View style={{ gap: 20 }}>
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            Installation
                          </Copy>
                          <Copy size={15} color={colors.muted}>
                            These components are included in the Solid GPUI SDK. Start from the repository, then import
                            the components you need.
                          </Copy>
                          <CodeBlock source={installCode} language="sh" label="Terminal" />
                        </View>
                      ) : section === "Usage" ? (
                        <View style={{ gap: 20 }}>
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            Usage
                          </Copy>
                          <CodeBlock source={importCode(current().name)} label="Import" />
                          <CodeBlock source={current().source} label="Example.tsx" />
                        </View>
                      ) : current().examples.some((example) => example.title === section) ? (
                        <View style={{ gap: 20 }}>
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            {section}
                          </Copy>
                          {() => {
                            const example = current().examples.find((example) => example.title === section)!;
                            return (
                              <>
                                <Copy color={colors.muted}>{example.description}</Copy>
                                <ComponentPreview
                                  unavailable={current().previewNote}
                                  previewWidth={current().previewWidth}
                                  minPreviewWidth={current().previewMinWidth}
                                  centered={current().previewCentered}
                                  compact
                                  width={width()}
                                  source={example.source}
                                  preview={previews[example.id]}
                                />
                              </>
                            );
                          }}
                        </View>
                      ) : section === "Related types" ? (
                        <View style={{ gap: 20 }}>
                          <Copy size={20} style={{ fontWeight: "bold" }}>
                            Related types
                          </Copy>
                          <CodeBlock source={current().definitions.join("\n\n")} language="ts" label="Types" />
                        </View>
                      ) : section === "Next steps" ? (
                        <View style={{ gap: 20 }}>
                          <View style={{ height: 1, backgroundColor: colors.line }} />
                          <View
                            style={{
                              flexDirection: mobile() ? "column" : "row",
                              gap: 12,
                              justifyContent: "space-between",
                            }}
                          >
                            {() =>
                              index() > 0 ? (
                                <Button
                                  translate={false}
                                  compact
                                  icon="lucide:arrow-left"
                                  onPress={() => navigate(catalog[index() - 1])}
                                >
                                  {catalog[index() - 1].name}
                                </Button>
                              ) : (
                                <View />
                              )
                            }
                            {() =>
                              index() < catalog.length - 1 ? (
                                <Button
                                  translate={false}
                                  compact
                                  icon="lucide:chevron-right"
                                  iconAfter
                                  onPress={() => navigate(catalog[index() + 1])}
                                >
                                  {catalog[index() + 1].name}
                                </Button>
                              ) : null
                            }
                          </View>
                        </View>
                      ) : section === "API Reference" ? (
                        <Copy size={24} style={{ fontWeight: "bold" }}>
                          API Reference
                        </Copy>
                      ) : (
                        <View style={{ gap: 20 }}>
                          <Copy translate={false} size={20} style={{ fontWeight: "bold" }}>
                            {section}
                          </Copy>
                          {() => {
                            const member = current().members.find((member) => member.name === section)!;
                            return (
                              <>
                                {member.properties.length ? (
                                  table(member.properties)
                                ) : (
                                  <Copy color={colors.muted}>This component has no component-specific properties.</Copy>
                                )}
                                {member.events.length ? (
                                  <View style={{ gap: 12 }}>
                                    <Copy>Events</Copy>
                                    {table(member.events)}
                                  </View>
                                ) : null}
                                {member.commands.length ? (
                                  <View style={{ gap: 12 }}>
                                    <Copy>Commands</Copy>
                                    {table(member.commands)}
                                  </View>
                                ) : null}
                                <Copy size={13} color={colors.muted}>
                                  {member.children ? "Accepts child content." : "Does not accept child content."}
                                </Copy>
                                {member.slots.length ? (
                                  <Copy size={13}>
                                    {t("Named slots")}: {member.slots.join(", ")}
                                  </Copy>
                                ) : null}
                              </>
                            );
                          }}
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
        toc() ? (
          <View style={{ width: 256, height: props.height, overflow: "scroll", padding: 24, gap: 4, flexShrink: 0 }}>
            <Copy size={12} color={colors.muted} style={{ marginTop: 12, marginBottom: 12 }}>
              On this page
            </Copy>
            {() =>
              sections().map((section, index) => (
                <NavItem
                  label={section}
                  translate={!current().members.some((member) => member.name === section)}
                  onPress={() => article?.scrollToIndex(index).catch((error) => setError(String(error)))}
                />
              ))
            }
            {() => (error() ? <Copy>{error()}</Copy> : null)}
          </View>
        ) : null
      }
    </View>
  );
}
