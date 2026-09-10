import { ICON_NAMES } from "../../../packages/solid-gpui/src/index";
import { chinese } from "../src/locale.zh-CN";
import { componentVariants } from "../component-variants";
import { expect, test } from "bun:test";
import ts from "typescript-api";
import { resolve } from "node:path";
import { componentCatalog, componentEntries } from "../component-catalog";
import { componentExamples } from "../component-examples";

test("every exported component has a usage example that type-checks against the SDK", () => {
  const catalog = componentEntries();
  const pages = componentCatalog();
  expect(pages.flatMap((page) => page.members.map((member) => member.name)).sort()).toEqual(
    catalog.map((entry) => entry.name).sort(),
  );
  expect(pages.find((page) => page.name === "Attachment")!.members).toHaveLength(7);
  expect(pages.some((page) => page.name === "AttachmentMedia")).toBe(false);
  expect(new Set(catalog.map((entry) => entry.name)).size).toBe(catalog.length);
  for (const entry of catalog) {
    expect(entry.examples.length).toBeGreaterThanOrEqual(2);
    expect(new Set(entry.examples.map((example) => example.title)).size).toBe(entry.examples.length);
    for (const example of entry.examples) {
      expect(chinese[example.title]).toBeTruthy();
      expect(chinese[example.description]).toBeTruthy();
    }
  }
  for (const page of pages) {
    expect(new Set(page.examples.map((example) => example.title)).size).toBe(page.examples.length);
    expect(new Set(page.examples.map((example) => example.source)).size).toBe(page.examples.length);
  }
  const root = resolve(import.meta.dirname, "..");
  const configPath = resolve(root, "tsconfig.json");
  const config = ts.readConfigFile(configPath, ts.sys.readFile);
  const parsed = ts.parseJsonConfigFileContent(config.config, ts.sys, root, undefined, configPath);
  const sources = new Map(
    [
      ...componentExamples.map((example) => ({ id: example.names[0], source: example.source })),
      ...componentVariants,
    ].map((example) => [resolve(root, `src/__checked_${example.id}.tsx`), example.source]),
  );
  const childContracts = new Map(catalog.map((entry) => [`N.${entry.name}`, entry.children]));
  const invalidChildren: string[] = [];
  for (const [path, source] of sources) {
    const file = ts.createSourceFile(path, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
    const visit = (node: ts.Node) => {
      if (ts.isJsxElement(node) && childContracts.get(node.openingElement.tagName.getText(file)) === false) {
        invalidChildren.push(
          `${path}: ${node.openingElement.tagName.getText(file)} requires named slots or data props`,
        );
      }
      const opening = ts.isJsxElement(node) ? node.openingElement : ts.isJsxSelfClosingElement(node) ? node : undefined;
      if (
        opening &&
        catalog.some(
          (entry) =>
            `N.${entry.name}` === opening.tagName.getText(file) &&
            ["Tooltip", "Popover", "HoverCard", "Collapsible"].includes(entry.name) &&
            entry.slots.includes("trigger"),
        )
      ) {
        const slots = opening.attributes.properties.find(
          (attribute) => ts.isJsxAttribute(attribute) && attribute.name.getText(file) === "slots",
        );
        if (!slots || !/\btrigger\s*:/.test(slots.getText(file))) {
          invalidChildren.push(`${path}: ${opening.tagName.getText(file)} needs a visible trigger slot`);
        }
      }
      if (ts.isJsxElement(node) && node.openingElement.tagName.getText(file) === "N.Bubble") {
        for (const child of node.children) {
          const name = ts.isJsxElement(child)
            ? child.openingElement.tagName.getText(file)
            : ts.isJsxSelfClosingElement(child)
              ? child.tagName.getText(file)
              : "";
          if (["N.BubbleContent", "N.BubbleReactions", "N.Label"].includes(name)) {
            invalidChildren.push(
              `${path}: Bubble owns its content surface; use inheriting Text and the reactions slot`,
            );
          }
        }
      }
      if (
        ts.isJsxAttribute(node) &&
        node.initializer &&
        ts.isStringLiteral(node.initializer) &&
        /\\[nrt]/.test(node.initializer.text)
      ) {
        invalidChildren.push(`${path}: escaped whitespace needs a JSX expression, not a quoted attribute`);
      }
      if (
        ts.isJsxElement(node) &&
        node.openingElement.tagName.getText(file) === "N.Tag" &&
        node.children.some((child) => ts.isJsxSelfClosingElement(child) && child.tagName.getText(file) === "N.Label")
      ) {
        invalidChildren.push(`${path}: Tag content must inherit its variant foreground; use Text`);
      }
      if (ts.isStringLiteral(node) && /^icons\//.test(node.text)) {
        invalidChildren.push(`${path}: application examples cannot use Kit's private icon assets`);
      }
      if (ts.isStringLiteral(node) && /^lucide:/.test(node.text) && !ICON_NAMES.some((name) => name === node.text)) {
        invalidChildren.push(`${path}: application icon is not in the Iconify catalog: ${node.text}`);
      }
      ts.forEachChild(node, visit);
    };
    visit(file);
  }
  expect(invalidChildren).toEqual([]);
  const host = ts.createCompilerHost(parsed.options);
  const readFile = host.readFile;
  const fileExists = host.fileExists;
  host.readFile = (path) => sources.get(path) ?? readFile(path);
  host.fileExists = (path) => sources.has(path) || fileExists(path);
  const program = ts.createProgram([...sources.keys()], parsed.options, host);
  const errors = ts
    .getPreEmitDiagnostics(program)
    .filter((diagnostic) => diagnostic.category === ts.DiagnosticCategory.Error);
  expect(
    errors.map((error) => `${error.file?.fileName}: ${ts.flattenDiagnosticMessageText(error.messageText, "\n")}`),
  ).toEqual([]);
}, 30_000);
