import { componentFamilies } from "./component-families";
import { componentGroups } from "./component-groups";
import { componentDocumentation, isNewComponent } from "./component-introduced";
import { previewNotes } from "./component-previews";
import { componentVariants } from "./component-variants";
import ts from "typescript-api";
import { resolve } from "node:path";
import { componentExamples } from "./component-examples";

export function componentEntries() {
  const path = resolve(import.meta.dirname, "../../packages/solid-gpui/src/components.ts");
  const program = ts.createProgram([path], { target: ts.ScriptTarget.ES2022, skipLibCheck: true, strict: true });
  const checker = program.getTypeChecker();
  const file = program.getSourceFile(path)!;
  const fields = (node: ts.TypeNode) =>
    checker.getPropertiesOfType(checker.getTypeFromTypeNode(node)).map((symbol) => {
      const declaration = symbol.valueDeclaration ?? symbol.declarations![0];
      return {
        name: symbol.name,
        type: checker.typeToString(
          checker.getTypeOfSymbolAtLocation(symbol, declaration),
          undefined,
          ts.TypeFormatFlags.NoTruncation,
        ),
        required: !(symbol.flags & ts.SymbolFlags.Optional),
      };
    });
  const result = [];
  for (const statement of file.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      const call = declaration.initializer;
      if (!call || !ts.isCallExpression(call) || call.expression.getText(file) !== "createNativeComponent") continue;
      const name = declaration.name.getText(file);
      const example = componentExamples.find((example) => example.names.includes(name));
      if (!example?.descriptionChinese) throw new Error(`Missing component documentation: ${name}`);
      const documentation = componentDocumentation[name];
      if (!documentation) throw new Error(`Missing component documentation dates: ${name}`);
      const properties = fields(call.typeArguments![0]);
      const events = fields(call.typeArguments![1]);
      const commands = fields(call.typeArguments![2]);
      const metadata = call.arguments[0] as ts.ObjectLiteralExpression;
      const value = (key: string) =>
        metadata.properties.find((prop) => prop.name?.getText(file) === key) as ts.PropertyAssignment;
      const slots = JSON.parse(value("slots").initializer.getText(file)) as string[];
      const children = value("children").initializer.kind === ts.SyntaxKind.TrueKeyword;
      result.push({
        name,
        id: name.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase(),
        documentation,
        isNew: isNewComponent(documentation.created),
        description: example.description,
        previewNote: previewNotes[name],
        previewWidth: /^(Table|DataTable|DockArea|Settings|Setting|Resizable|TextView|Editor|Message)/.test(name)
          ? 560
          : 360,
        previewMinWidth: /^(Settings|Setting|DockArea)/.test(name) ? 480 : name === "Pagination" ? 340 : 0,
        previewCentered: /^(Avatar|Calendar|Caret|Icon|Rating|Spinner|ProgressCircle|Kbd)$/.test(name),
        descriptionChinese: example.descriptionChinese,
        source: example.source,
        examples: componentVariants.filter((variant) => variant.component === name).map((variant) => ({ ...variant })),
        properties,
        events,
        commands,
        slots,
        children,
        definitions: [...new Set(properties.flatMap((prop) => prop.type.match(/\b[A-Z]\w+\b/g) ?? []))].flatMap(
          (name) => {
            const declaration = file.statements.find(
              (node) => ts.isTypeAliasDeclaration(node) && node.name.text === name,
            );
            return declaration ? [declaration.getText(file)] : [];
          },
        ),
      });
    }
  }
  const dated = new Set(result.map((entry) => entry.name));
  const stale = Object.keys(componentDocumentation).filter((name) => !dated.has(name));
  if (stale.length) throw new Error(`Documentation dates without a component: ${stale.join(", ")}`);
  return result;
}

/** Place every catalog page in its navigation group, rejecting gaps and duplicates. */
function groupPages(pages: string[]): Map<string, { label: string; order: number }> {
  const result = new Map<string, { label: string; order: number }>();
  componentGroups.forEach((group, order) => {
    for (const member of group.members) {
      if (result.has(member)) throw new Error(`Component listed in two groups: ${member}`);
      if (!pages.includes(member)) throw new Error(`Component group member without a page: ${member}`);
      result.set(member, { label: group.label, order });
    }
  });
  const ungrouped = pages.filter((page) => !result.has(page));
  if (ungrouped.length) throw new Error(`Components without a navigation group: ${ungrouped.join(", ")}`);
  return result;
}

export function componentCatalog() {
  const entries = componentEntries();
  const owners = new Map(componentFamilies.flatMap(([owner, ...parts]) => parts.map((part) => [part, owner])));
  const pages = entries.filter((entry) => !owners.has(entry.name));
  const groups = groupPages(pages.map((page) => page.name));
  return pages
    .map((entry) => {
      const names = componentFamilies.find(([owner]) => owner === entry.name) ?? [entry.name];
      const members = names.map((name) => {
        const member = entries.find((item) => item.name === name);
        if (!member) throw new Error(`Unknown component family member: ${name}`);
        return member;
      });
      return {
        ...entry,
        group: groups.get(entry.name)!.label,
        members: members.map(({ name, id, documentation, isNew, properties, events, commands, slots, children }) => ({
          name,
          id,
          documentation,
          isNew,
          properties,
          events,
          commands,
          slots,
          children,
        })),
        documentation: {
          created: members
            .map((member) => member.documentation.created)
            .sort()
            .at(0)!,
          updated: members
            .map((member) => member.documentation.updated)
            .sort()
            .at(-1)!,
        },
        isNew: members.some((member) => member.isNew),
        examples: [
          ...new Map(members.flatMap((member) => member.examples).map((example) => [example.source, example])).values(),
        ],
        definitions: [...new Set(members.flatMap((member) => member.definitions))],
      };
    })
    .sort(
      (left, right) =>
        groups.get(left.name)!.order - groups.get(right.name)!.order ||
        (left.name < right.name ? -1 : left.name > right.name ? 1 : 0),
    );
}
