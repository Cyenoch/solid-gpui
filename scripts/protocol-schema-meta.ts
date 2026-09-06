import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

export type SchemaType =
  | { readonly kind: "scalar"; readonly name: string }
  | { readonly kind: "array"; readonly element: SchemaType }
  | { readonly kind: "def"; readonly name: string };
export type SchemaField = { readonly id: number; readonly name: string; readonly type: SchemaType };
export type SchemaEnumValue = { readonly name: string; readonly value: number };
export type SchemaDefinition =
  | {
      readonly kind: "message";
      readonly fields: readonly SchemaField[];
    }
  | {
      readonly kind: "union";
      readonly branches: readonly { readonly id: number; readonly type: string }[];
    }
  | {
      readonly kind: "enum";
      readonly base: string;
      readonly values: readonly number[];
      readonly names: readonly string[];
    };
export type Schema = {
  readonly schema: string;
  readonly root: string;
  readonly definitions: Record<string, SchemaDefinition>;
};

type Token = {
  readonly kind: "identifier" | "integer" | "symbol" | "eof";
  readonly value: string;
  readonly offset: number;
};

const SCALARS: Record<string, true> = {
  bool: true,
  byte: true,
  uint8: true,
  uint16: true,
  int16: true,
  uint32: true,
  int32: true,
  uint64: true,
  int64: true,
  float32: true,
  float64: true,
  string: true,
  guid: true,
  date: true,
};
const SINGLE_SYMBOLS: Record<string, true> = {
  "{": true,
  "}": true,
  ":": true,
  "=": true,
  ";": true,
  "[": true,
  "]": true,
};

function syntaxError(source: string, offset: number, message: string): never {
  const line = source.slice(0, offset).split("\n").length;
  throw new Error(`protocol.bop:${line}:${offset}: ${message}`);
}

function tokenize(source: string): readonly Token[] {
  const tokens: Token[] = [];
  let index = 0;
  while (index < source.length) {
    const character = source[index]!;
    if (/\s/u.test(character)) {
      index += 1;
      continue;
    }
    if (character === "/" && source[index + 1] === "/") {
      index += 2;
      while (index < source.length && source[index] !== "\n") index += 1;
      continue;
    }
    if (character === "/" && source[index + 1] === "*") {
      const start = index;
      index += 2;
      while (index + 1 < source.length && !(source[index] === "*" && source[index + 1] === "/")) index += 1;
      if (index + 1 >= source.length) syntaxError(source, start, "unterminated block comment");
      index += 2;
      continue;
    }
    if (character === "-" && source[index + 1] === ">") {
      tokens.push({ kind: "symbol", value: "->", offset: index });
      index += 2;
      continue;
    }
    if (SINGLE_SYMBOLS[character]) {
      tokens.push({ kind: "symbol", value: character, offset: index });
      index += 1;
      continue;
    }
    if (/[A-Za-z_]/u.test(character)) {
      const start = index;
      index += 1;
      while (index < source.length && /[A-Za-z0-9_]/u.test(source[index]!)) index += 1;
      tokens.push({ kind: "identifier", value: source.slice(start, index), offset: start });
      continue;
    }
    if (/[0-9]/u.test(character) || (character === "-" && /[0-9]/u.test(source[index + 1] ?? ""))) {
      const start = index;
      if (character === "-") index += 1;
      while (index < source.length && /[0-9]/u.test(source[index]!)) index += 1;
      if (index < source.length && /[A-Za-z_]/u.test(source[index]!))
        syntaxError(source, index, "integer cannot be followed by an identifier");
      const value = source.slice(start, index);
      const numeric = Number(value);
      if (!Number.isSafeInteger(numeric)) syntaxError(source, start, `integer is outside the safe range: ${value}`);
      tokens.push({ kind: "integer", value, offset: start });
      continue;
    }
    syntaxError(source, index, `unexpected character ${JSON.stringify(character)}`);
  }
  tokens.push({ kind: "eof", value: "", offset: source.length });
  return tokens;
}

class Parser {
  private index = 0;
  private readonly definitions: Record<string, SchemaDefinition> = {};

  constructor(
    private readonly source: string,
    private readonly tokens: readonly Token[],
  ) {}

  private current(): Token {
    return this.tokens[this.index]!;
  }

  private take(kind: Token["kind"], value?: string): Token {
    const token = this.current();
    if (token.kind !== kind || (value !== undefined && token.value !== value)) {
      syntaxError(
        this.source,
        token.offset,
        `expected ${value ?? kind}, found ${token.value === "" ? "end of file" : token.value}`,
      );
    }
    this.index += 1;
    return token;
  }

  private accept(value: string): boolean {
    const token = this.current();
    if (token.value !== value) return false;
    this.index += 1;
    return true;
  }

  private identifier(): string {
    return this.take("identifier").value;
  }

  private integer(): number {
    return Number(this.take("integer").value);
  }

  private id(): number {
    const token = this.current();
    const value = this.integer();
    if (value < 1 || value > 255) syntaxError(this.source, token.offset, `ID must be between 1 and 255, got ${value}`);
    return value;
  }

  private parseType(): SchemaType {
    const name = this.identifier();
    const base: SchemaType = SCALARS[name] ? { kind: "scalar", name } : { kind: "def", name };
    if (!this.accept("[")) return base;
    this.take("symbol", "]");
    return { kind: "array", element: base };
  }

  private parseFields(): readonly SchemaField[] {
    this.take("symbol", "{");
    const fields: SchemaField[] = [];
    let previous = 0;
    while (!this.accept("}")) {
      const id = this.id();
      if (id <= previous)
        syntaxError(this.source, this.current().offset, "message field IDs must be strictly increasing");
      previous = id;
      this.take("symbol", "->");
      const type = this.parseType();
      const name = this.identifier();
      this.take("symbol", ";");
      fields.push({ id, name, type });
    }
    return fields;
  }

  private parseEnum(name: string): SchemaDefinition {
    this.take("symbol", ":");
    const base = this.identifier();
    if (!SCALARS[base]) syntaxError(this.source, this.current().offset, `unknown enum base ${base}`);
    this.take("symbol", "{");
    const values: number[] = [];
    const names: string[] = [];
    const namesSeen = new Set<string>();
    const valuesSeen = new Set<number>();
    while (!this.accept("}")) {
      const valueName = this.identifier();
      if (namesSeen.has(valueName))
        syntaxError(this.source, this.current().offset, `duplicate enum member ${valueName}`);
      this.take("symbol", "=");
      const value = Number(this.take("integer").value);
      if (valuesSeen.has(value)) syntaxError(this.source, this.current().offset, `duplicate enum value ${value}`);
      this.take("symbol", ";");
      namesSeen.add(valueName);
      valuesSeen.add(value);
      names.push(valueName);
      values.push(value);
    }
    if (values.length === 0) syntaxError(this.source, this.current().offset, `enum ${name} must not be empty`);
    return { kind: "enum", base, values, names };
  }

  private parseUnion(name: string): SchemaDefinition {
    this.definitions[name] = { kind: "union", branches: [] };
    this.take("symbol", "{");
    const branches: { id: number; type: string }[] = [];
    let previous = 0;
    const ids = new Set<number>();
    while (!this.accept("}")) {
      const id = this.id();
      if (id <= previous)
        syntaxError(this.source, this.current().offset, "union branch IDs must be strictly increasing");
      if (ids.has(id)) syntaxError(this.source, this.current().offset, `duplicate union branch ID ${id}`);
      previous = id;
      ids.add(id);
      this.take("symbol", "->");
      const inlineMessage = this.accept("message");
      const type = this.identifier();
      if (inlineMessage) {
        if (this.definitions[type] !== undefined)
          syntaxError(this.source, this.current().offset, `duplicate definition ${type}`);
        this.definitions[type] = { kind: "message", fields: this.parseFields() };
      } else {
        this.take("symbol", ";");
      }
      branches.push({ id, type });
    }
    const definition: SchemaDefinition = { kind: "union", branches };
    this.definitions[name] = definition;
    return definition;
  }

  private parseDefinition(): void {
    const kind = this.identifier();
    if (kind !== "enum" && kind !== "message" && kind !== "union")
      syntaxError(this.source, this.current().offset, `unknown top-level declaration ${kind}`);
    const name = this.identifier();
    if (this.definitions[name] !== undefined)
      syntaxError(this.source, this.current().offset, `duplicate definition ${name}`);
    if (kind === "enum") {
      this.definitions[name] = this.parseEnum(name);
      return;
    }
    if (kind === "message") {
      this.definitions[name] = { kind: "message", fields: this.parseFields() };
      return;
    }
    this.parseUnion(name);
  }

  private validateType(type: SchemaType): void {
    if (type.kind === "def" && this.definitions[type.name] === undefined)
      throw new Error(`protocol.bop references unknown definition ${type.name}`);
    if (type.kind === "array") this.validateType(type.element);
  }

  private validateReferences(): void {
    for (const definition of Object.values(this.definitions)) {
      if (definition.kind === "message") {
        for (const field of definition.fields) this.validateType(field.type);
      } else if (definition.kind === "union") {
        for (const branch of definition.branches) {
          if (this.definitions[branch.type] === undefined)
            throw new Error(`protocol.bop references unknown definition ${branch.type}`);
        }
      }
    }
  }

  parse(): Schema {
    while (this.current().kind !== "eof") this.parseDefinition();
    this.validateReferences();
    const envelope = this.definitions.Envelope;
    if (envelope?.kind !== "message") throw new Error("protocol.bop must define message Envelope");
    if (
      envelope.fields.length !== 2 ||
      envelope.fields[0]?.name !== "protocolVersion" ||
      envelope.fields[1]?.name !== "body"
    )
      throw new Error("Envelope must contain protocolVersion and body as fields 1 and 2");
    return { schema: "protocol.bop", root: "Envelope", definitions: this.definitions };
  }
}

export function parseSchema(source: string): Schema {
  return new Parser(source, tokenize(source)).parse();
}

export function schemaDigest(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

function json(value: unknown): string {
  return JSON.stringify(value, null, 2);
}

function enumEntries(definition: SchemaDefinition): readonly SchemaEnumValue[] {
  if (definition.kind !== "enum") throw new Error("expected enum definition");
  return definition.names.map((name, index) => ({ name, value: definition.values[index]! }));
}
function unionEntries(definition: SchemaDefinition): readonly { readonly name: string; readonly value: number }[] {
  if (definition.kind !== "union") throw new Error("expected union definition");
  return definition.branches.map(({ type, id }) => ({ name: type, value: id }));
}
function requiredDefinition(schema: Schema, name: string): SchemaDefinition {
  const definition = schema.definitions[name];
  if (definition === undefined) throw new Error(`protocol.bop is missing ${name}`);
  return definition;
}
function objectFromEntries(
  entries: readonly { readonly name: string; readonly value: number }[],
): Record<string, number> {
  return Object.fromEntries(entries.map(({ name, value }) => [name, value]));
}

function renderFacts(schema: Schema): string {
  const body = objectFromEntries(unionEntries(requiredDefinition(schema, "Body")));
  const host = objectFromEntries(unionEntries(requiredDefinition(schema, "HostProperties")));
  const patch = objectFromEntries(unionEntries(requiredDefinition(schema, "PatchOperationValue")));
  const menu = objectFromEntries(unionEntries(requiredDefinition(schema, "MenuItemValue")));
  const commandPayload = objectFromEntries(unionEntries(requiredDefinition(schema, "CommandPayload")));
  const commandValue = objectFromEntries(unionEntries(requiredDefinition(schema, "CommandValue")));
  const eventPayload = objectFromEntries(unionEntries(requiredDefinition(schema, "EventPayload")));
  const node = objectFromEntries(enumEntries(requiredDefinition(schema, "NodeKind")));
  const appearance = objectFromEntries(enumEntries(requiredDefinition(schema, "WindowAppearance")));
  const event = objectFromEntries(enumEntries(requiredDefinition(schema, "EventKind")));
  const command = objectFromEntries(enumEntries(requiredDefinition(schema, "CommandKind")));
  const commandKinds = enumEntries(requiredDefinition(schema, "CommandKind"))
    .filter(({ name }) => name !== "Unknown")
    .map(({ value }) => value);
  return `// Generated from protocol.bop; do not edit.\nexport const BODY_TAGS = ${json(body)} as const;\nexport const HOST_PROPERTIES_TAGS = ${json(host)} as const;\nexport const PATCH_OPERATION_TAGS = ${json(patch)} as const;\nexport const MENU_ITEM_TAGS = ${json(menu)} as const;\nexport const COMMAND_PAYLOAD_TAGS = ${json(commandPayload)} as const;\nexport const COMMAND_VALUE_TAGS = ${json(commandValue)} as const;\nexport const EVENT_PAYLOAD_TAGS = ${json(eventPayload)} as const;\nexport const NODE_KIND_CODES = ${json(node)} as const;\nexport const WINDOW_APPEARANCE_CODES = ${json(appearance)} as const;\nexport const EVENT_KIND_CODES = ${json(event)} as const;\nexport const COMMAND_KIND_CODES = ${json(command)} as const;\nexport const COMMAND_KINDS = ${json(commandKinds)} as const;\n`;
}

function rustName(value: string): string {
  return value.replace(/[A-Z]/gu, (character, index) =>
    index === 0 ? character.toLowerCase() : `_${character.toLowerCase()}`,
  );
}
function rustConst(value: string): string {
  return rustName(value).toUpperCase();
}
function renderRustFacts(schema: Schema): string {
  const lines = ["// Generated from protocol.bop; do not edit.", "#![allow(dead_code)]", ""];
  const addConstants = (prefix: string, entries: readonly { readonly name: string; readonly value: number }[]) => {
    for (const { name, value } of entries) lines.push(`pub const ${prefix}_${rustConst(name)}: u32 = ${value};`);
    lines.push("");
  };
  addConstants("BODY", unionEntries(requiredDefinition(schema, "Body")));
  addConstants("HOST_PROPERTIES", unionEntries(requiredDefinition(schema, "HostProperties")));
  addConstants("PATCH_OPERATION", unionEntries(requiredDefinition(schema, "PatchOperationValue")));
  addConstants("MENU_ITEM", unionEntries(requiredDefinition(schema, "MenuItemValue")));
  addConstants("COMMAND_PAYLOAD", unionEntries(requiredDefinition(schema, "CommandPayload")));
  addConstants("COMMAND_VALUE", unionEntries(requiredDefinition(schema, "CommandValue")));
  addConstants("EVENT_PAYLOAD", unionEntries(requiredDefinition(schema, "EventPayload")));
  addConstants("NODE_KIND", enumEntries(requiredDefinition(schema, "NodeKind")));
  addConstants("EVENT", enumEntries(requiredDefinition(schema, "EventKind")));
  addConstants("COMMAND", enumEntries(requiredDefinition(schema, "CommandKind")));
  const renderConversion = (functionName: string, typeName: string, entries: readonly SchemaEnumValue[]) => {
    lines.push(`pub(crate) const fn ${functionName}(value: u32) -> Option<super::generated::${typeName}> {`);
    lines.push("    match value {");
    for (const { name, value } of entries)
      lines.push(`        ${value} => Some(super::generated::${typeName}::${name}),`);
    lines.push("        _ => None,");
    lines.push("    }");
    lines.push("}", "");
  };
  renderConversion("node_kind", "NodeKind", enumEntries(requiredDefinition(schema, "NodeKind")));
  renderConversion(
    "window_appearance",
    "WindowAppearance",
    enumEntries(requiredDefinition(schema, "WindowAppearance")),
  );
  renderConversion("event_kind", "EventKind", enumEntries(requiredDefinition(schema, "EventKind")));
  renderConversion("command_kind", "CommandKind", enumEntries(requiredDefinition(schema, "CommandKind")));
  const commandKinds = enumEntries(requiredDefinition(schema, "CommandKind"))
    .filter(({ name }) => name !== "Unknown")
    .map(({ value }) => value);
  lines.push(`pub const COMMAND_KINDS: &[u32] = &[${commandKinds.join(", ")}];`, "");
  return `${lines.join("\n")}\n`;
}

function schemaMetadata(schema: Schema, digest: string): Record<string, unknown> {
  return { schema: schema.schema, root: schema.root, digest, definitions: schema.definitions };
}

function escaped(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
function snakeCase(value: string): string {
  return value.replace(/[A-Z]/gu, (character) => `_${character.toLowerCase()}`);
}
function assertGeneratedParity(schema: Schema, tsSource: string, rustSource: string): void {
  const sources = [
    ["TypeScript", tsSource],
    ["Rust", rustSource],
  ] as const;
  for (const [label, source] of sources) {
    for (const [name, definition] of Object.entries(schema.definitions)) {
      const declaration =
        label === "TypeScript"
          ? new RegExp(`\\b(?:interface|enum|type)\\s+${escaped(name)}\\b`, "u")
          : new RegExp(`\\b(?:struct|enum)\\s+${escaped(name)}\\b|\\n\\s+${escaped(name)}\\s*\\{`, "u");
      if (!declaration.test(source)) throw new Error(`${label} generated protocol is missing definition ${name}`);
      if (definition.kind === "enum") {
        for (const entry of enumEntries(definition)) {
          if (!new RegExp(`\\b${escaped(entry.name)}\\s*=\\s*${entry.value}\\b`, "u").test(source))
            throw new Error(`${label} generated protocol is missing enum ${name}.${entry.name}=${entry.value}`);
        }
      } else if (definition.kind === "message") {
        for (const field of definition.fields) {
          const fieldName = label === "TypeScript" ? field.name : snakeCase(field.name);
          if (!new RegExp(`\\b${escaped(fieldName)}\\b`, "u").test(source))
            throw new Error(`${label} generated protocol is missing ${name}.${field.name}`);
        }
      } else {
        for (const branch of definition.branches) {
          if (!source.includes(branch.type))
            throw new Error(`${label} generated protocol is missing ${name} branch ${branch.type}`);
        }
      }
    }
  }
}

function metadataFromTypeScript(source: string): unknown {
  const prefix = "export const PROTOCOL_SCHEMA = ";
  const start = source.indexOf(prefix);
  const suffix = " as const;";
  const end = source.lastIndexOf(suffix);
  if (start < 0 || end < start + prefix.length)
    throw new Error("generated TypeScript schema metadata has no PROTOCOL_SCHEMA export");
  return JSON.parse(source.slice(start + prefix.length, end));
}

const schemaPath = resolve(process.argv[2] ?? "packages/solid-gpui/src/protocol/protocol.bop");

export async function generateProtocolArtifacts(
  options: {
    readonly schemaPath?: string;
    readonly acceptSchemaDigest?: boolean;
  } = {},
): Promise<void> {
  const sourcePath = resolve(options.schemaPath ?? schemaPath);
  const source = await readFile(sourcePath, "utf8");
  const schema = parseSchema(source);
  const digest = schemaDigest(source);
  const lockPath = resolve("packages/solid-gpui/src/protocol/schema-lock.json");
  const lock = JSON.parse(await readFile(lockPath, "utf8")) as {
    readonly protocolVersion?: number;
    readonly schema?: string;
    readonly sha256?: string;
  };
  if (lock.protocolVersion !== 5 || lock.schema !== schema.schema)
    throw new Error("schema lock must explicitly identify protocol v5 and protocol.bop");
  if (lock.sha256 !== digest) {
    if (!options.acceptSchemaDigest) {
      throw new Error(
        `protocol.bop digest ${digest} is not accepted for v5; update schema-lock.json deliberately or pass --accept-schema-digest`,
      );
    }
    await writeFile(
      lockPath,
      `${JSON.stringify({ protocolVersion: 5, schema: schema.schema, sha256: digest }, null, 2)}\n`,
    );
  }
  const metadata = schemaMetadata(schema, digest);
  const tsPath = resolve("packages/solid-gpui/src/protocol/generated/schema-meta.ts");
  const rustPath = resolve("crates/solid-gpui/src/protocol/generated/schema-meta.json");
  const factsTsPath = resolve("packages/solid-gpui/src/protocol/generated/schema-facts.ts");
  const factsRustPath = resolve("crates/solid-gpui/src/protocol/generated/schema_facts.rs");
  const guardRustPath = resolve("crates/solid-gpui/src/protocol/generated/schema_guard.rs");
  const generatedTsPath = resolve("packages/solid-gpui/src/protocol/generated/protocol.ts");
  const generatedRustPath = resolve("crates/solid-gpui/src/protocol/generated/protocol.rs");
  const generatedTs = await readFile(generatedTsPath, "utf8");
  const generatedRust = await readFile(generatedRustPath, "utf8");
  assertGeneratedParity(schema, generatedTs, generatedRust);
  await writeFile(
    tsPath,
    `// Generated from protocol.bop; do not edit.\nexport const PROTOCOL_SCHEMA = ${json(metadata)} as const;\nexport type ProtocolSchema = typeof PROTOCOL_SCHEMA;\n`,
  );
  await writeFile(rustPath, `${JSON.stringify(metadata, null, 2)}\n`);
  await writeFile(factsTsPath, renderFacts(schema));
  await writeFile(factsRustPath, renderRustFacts(schema));
  await writeFile(guardRustPath, renderRustGuard(schema));
  const generatedMetadata = metadataFromTypeScript(await readFile(tsPath, "utf8"));
  if (JSON.stringify(generatedMetadata) !== JSON.stringify(metadata))
    throw new Error("generated TypeScript schema metadata is not self-consistent");
  console.log(`wrote ${tsPath}, ${rustPath}, ${factsTsPath}, ${factsRustPath}, and ${guardRustPath}`);
}

function renderRustGuard(schema: Schema): string {
  const lines = [
    "// Generated from protocol.bop; do not edit.",
    "",
    "#[derive(Clone, Copy, Debug)]",
    "pub(crate) enum TypeSpec {",
    "    Scalar(&'static str),",
    "    Array { element: usize },",
    "    Definition { definition: usize },",
    "}",
    "",
    "#[derive(Clone, Copy, Debug)]",
    "pub(crate) struct Field {",
    "    pub(crate) id: u8,",
    "    pub(crate) name: &'static str,",
    "    pub(crate) type_id: usize,",
    "}",
    "",
    "#[derive(Clone, Copy, Debug)]",
    "pub(crate) struct Branch {",
    "    pub(crate) id: u8,",
    "    pub(crate) definition: usize,",
    "}",
    "",
    "#[derive(Clone, Copy, Debug)]",
    "pub(crate) enum Definition {",
    "    Message { name: &'static str, fields: &'static [Field] },",
    "    Union { branches: &'static [Branch] },",
    "    Enum { name: &'static str, base: &'static str, values: &'static [i64] },",
    "}",
    "",
  ];
  const definitionNames = Object.keys(schema.definitions);
  const definitionIds = new Map(definitionNames.map((name, index) => [name, index]));
  const typeSpecs: {
    readonly kind: "scalar" | "array" | "definition";
    readonly name?: string;
    readonly index?: number;
  }[] = [];
  const typeId = (type: SchemaType): number => {
    if (type.kind === "scalar") {
      const id = typeSpecs.length;
      typeSpecs.push({ kind: "scalar", name: type.name });
      return id;
    }
    if (type.kind === "array") {
      const element = typeId(type.element);
      const id = typeSpecs.length;
      typeSpecs.push({ kind: "array", index: element });
      return id;
    }
    const definition = definitionIds.get(type.name);
    if (definition === undefined) throw new Error(`unknown definition ${type.name}`);
    const id = typeSpecs.length;
    typeSpecs.push({ kind: "definition", index: definition });
    return id;
  };
  const fieldSets: string[] = [];
  const branchSets: string[] = [];
  const definitionLines: string[] = [];
  for (const name of definitionNames) {
    const definition = schema.definitions[name]!;
    if (definition.kind === "message") {
      const setName = `FIELDS_${definitionLines.length}`;
      const fields = definition.fields.map(
        (field) =>
          `    Field { id: ${field.id}, name: ${JSON.stringify(field.name)}, type_id: ${typeId(field.type)} },`,
      );
      fieldSets.push(`const ${setName}: &[Field] = &[\n${fields.join("\n")}\n];`);
      definitionLines.push(`    Definition::Message { name: ${JSON.stringify(name)}, fields: ${setName} },`);
    } else if (definition.kind === "union") {
      const setName = `BRANCHES_${definitionLines.length}`;
      const branches = definition.branches.map((branch) => {
        const index = definitionIds.get(branch.type);
        if (index === undefined) throw new Error(`unknown union definition ${branch.type}`);
        return `    Branch { id: ${branch.id}, definition: ${index} },`;
      });
      branchSets.push(`const ${setName}: &[Branch] = &[\n${branches.join("\n")}\n];`);
      definitionLines.push(`    Definition::Union { branches: ${setName} },`);
    } else {
      definitionLines.push(
        `    Definition::Enum { name: ${JSON.stringify(name)}, base: ${JSON.stringify(definition.base)}, values: &[${definition.values.join(", ")}] },`,
      );
    }
  }
  lines.push(
    ...fieldSets,
    "",
    ...branchSets,
    "",
    "pub(crate) static TYPES: &[TypeSpec] = &[",
    ...typeSpecs.map((type) => {
      if (type.kind === "scalar") return `    TypeSpec::Scalar(${JSON.stringify(type.name)}),`;
      if (type.kind === "array") return `    TypeSpec::Array { element: ${type.index} },`;
      return `    TypeSpec::Definition { definition: ${type.index} },`;
    }),
    "];",
    "",
    "pub(crate) static DEFINITIONS: &[Definition] = &[",
    ...definitionLines,
    "];",
    `pub(crate) const ROOT_DEFINITION: usize = ${definitionIds.get(schema.root)};`,
    "",
  );
  return lines.join("\n");
}

if (import.meta.main) {
  await generateProtocolArtifacts({ acceptSchemaDigest: process.argv.includes("--accept-schema-digest") });
}
