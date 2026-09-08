import { componentFamilies } from "./component-families";
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
  return result;
}

export function componentCatalog() {
  const entries = componentEntries();
  const owners = new Map(componentFamilies.flatMap(([owner, ...parts]) => parts.map((part) => [part, owner])));
  return entries
    .filter((entry) => !owners.has(entry.name))
    .map((entry) => {
      const names = componentFamilies.find(([owner]) => owner === entry.name) ?? [entry.name];
      const members = names.map((name) => {
        const member = entries.find((item) => item.name === name);
        if (!member) throw new Error(`Unknown component family member: ${name}`);
        return member;
      });
      return {
        ...entry,
        members: members.map(({ name, id, properties, events, commands, slots, children }) => ({
          name,
          id,
          properties,
          events,
          commands,
          slots,
          children,
        })),
        examples: [
          ...new Map(members.flatMap((member) => member.examples).map((example) => [example.source, example])).values(),
        ],
        definitions: [...new Set(members.flatMap((member) => member.definitions))],
      };
    });
}
