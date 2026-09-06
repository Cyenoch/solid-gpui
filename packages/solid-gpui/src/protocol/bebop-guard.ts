import { PROTOCOL_SCHEMA } from "./generated/schema-meta";
import {
  MAX_NATIVE_CALL_BYTES,
  MAX_CLIPBOARD_IMAGE_BYTES,
  MAX_CLIPBOARD_TEXT_BYTES,
  MAX_EXTENSION_EVENTS,
  MAX_EXTENSION_FIELDS,
  MAX_FILE_WRITE_BYTES,
  MAX_FRAME_SIZE,
} from "./constants";

type Schema = {
  readonly root: string;
  readonly definitions: Record<string, SchemaDefinition>;
};
type SchemaType = {
  readonly kind: "scalar" | "array" | "def";
  readonly name?: string;
  readonly element?: SchemaType;
};
type SchemaField = { readonly id: number; readonly name: string; readonly type: SchemaType };
type SchemaBranch = { readonly id: number; readonly type: string };
type SchemaDefinition = {
  readonly kind: "message" | "union" | "enum";
  readonly fields?: readonly SchemaField[];
  readonly branches?: readonly SchemaBranch[];
  readonly base?: string;
  readonly values?: readonly number[];
};

export interface BebopDecodeLimits {
  readonly maxFrameBytes: number;
  readonly maxMessageBytes: number;
  readonly maxArrayItems: number;
  readonly maxStringBytes: number;
  readonly maxBytes: number;
  readonly maxNesting: number;
  readonly maxTotalItems: number;
}
export const DEFAULT_BEBOP_LIMITS: BebopDecodeLimits = {
  maxFrameBytes: MAX_FRAME_SIZE,
  maxMessageBytes: MAX_FRAME_SIZE,
  maxArrayItems: 100_000,
  maxStringBytes: MAX_CLIPBOARD_TEXT_BYTES,
  maxBytes: MAX_CLIPBOARD_IMAGE_BYTES,
  maxNesting: 80,
  maxTotalItems: 1_000_000,
};
export class BoundedDecodeError extends Error {
  constructor(
    readonly reason: string,
    readonly offset: number,
  ) {
    super(`${reason} at byte ${offset}`);
    this.name = "BoundedDecodeError";
  }
}

const SCALAR_STRING = 0;
const SCALAR_BOOL = 1;
const SCALAR_BYTE = 2;
const SCALAR_UINT8 = 3;
const SCALAR_UINT16 = 4;
const SCALAR_INT16 = 5;
const SCALAR_UINT32 = 6;
const SCALAR_INT32 = 7;
const SCALAR_FLOAT32 = 8;
const SCALAR_UINT64 = 9;
const SCALAR_INT64 = 10;
const SCALAR_FLOAT64 = 11;
const SCALAR_GUID = 12;
const SCALAR_DATE = 13;
const SCALAR_SIZES = [0, 1, 1, 1, 2, 2, 4, 4, 4, 8, 8, 8, 16, 8] as const;

const STRING_DEFAULT = 0;
const STRING_CONTENT = 1;
const STRING_FONT_FAMILY = 2;
const STRING_PATH = 3;
const STRING_DRAG_TYPE = 4;
const STRING_TAG = 5;
const STRING_NAMED = 6;
const STRING_FILE_TEXT = 7;

const ARRAY_DEFAULT = 0;
const ARRAY_MODIFIERS = 1;
const ARRAY_ACTIONS = 2;
const ARRAY_VALUES = 3;
const ARRAY_ITEMS = 4;
const ARRAY_MENUS = 5;
const ARRAY_PATHS = 6;
const ARRAY_NODES = 7;
const ARRAY_OPERATIONS = 8;
const ARRAY_EXTENSION_FIELDS = 9;
const ARRAY_EXTENSION_EVENTS = 10;
type CompiledType =
  | { readonly kind: "scalar"; readonly code: number; readonly stringLimit: number }
  | {
      readonly kind: "array";
      readonly element: CompiledType;
      readonly arrayLimit: number;
      readonly byteArray: boolean;
      readonly minBytes: number;
      readonly maxBytes: number;
    }
  | { readonly kind: "definition"; readonly index: number };
type CompiledField = { readonly type: CompiledType };
type CompiledDefinition =
  | { readonly kind: "message"; readonly fields: readonly (CompiledField | undefined)[] }
  | { readonly kind: "union"; readonly branches: readonly (number | undefined)[] }
  | { readonly kind: "enum"; readonly base: number; readonly size: number; readonly values: readonly number[] };

function definitionMap(schema: Schema): Record<string, SchemaDefinition> {
  return schema.definitions as unknown as Record<string, SchemaDefinition>;
}
const SCHEMA_DEFINITIONS = definitionMap(PROTOCOL_SCHEMA);
// Compile canonical metadata once so the hot path uses numeric plans rather than schema maps or names.
const DEFINITION_NAMES = Object.keys(SCHEMA_DEFINITIONS);
const DEFINITION_IDS: Record<string, number> = {};
for (let index = 0; index < DEFINITION_NAMES.length; index += 1) DEFINITION_IDS[DEFINITION_NAMES[index]!] = index;

function scalarCode(name: string): number {
  switch (name) {
    case "string":
      return SCALAR_STRING;
    case "bool":
      return SCALAR_BOOL;
    case "byte":
      return SCALAR_BYTE;
    case "uint8":
      return SCALAR_UINT8;
    case "uint16":
      return SCALAR_UINT16;
    case "int16":
      return SCALAR_INT16;
    case "uint32":
      return SCALAR_UINT32;
    case "int32":
      return SCALAR_INT32;
    case "float32":
      return SCALAR_FLOAT32;
    case "uint64":
      return SCALAR_UINT64;
    case "int64":
      return SCALAR_INT64;
    case "float64":
      return SCALAR_FLOAT64;
    case "guid":
      return SCALAR_GUID;
    case "date":
      return SCALAR_DATE;
    default:
      throw new Error(`unknown schema scalar ${name}`);
  }
}
function stringLimitKind(parent: string, field: string): number {
  if (field === "content") return STRING_CONTENT;
  if (field === "fontFamily") return STRING_FONT_FAMILY;
  if (field === "path" || field === "source" || field === "fallbackSource") return STRING_PATH;
  if (field === "dragType") return STRING_DRAG_TYPE;
  if (field === "tag") return STRING_TAG;
  if (field === "title" || field === "action" || field === "name" || field === "label" || field === "key")
    return STRING_NAMED;
  if (parent === "FileTextValue") return STRING_FILE_TEXT;
  return STRING_DEFAULT;
}
function arrayLimitKind(field: string): number {
  if (field === "modifiers") return ARRAY_MODIFIERS;
  if (field === "actions" || field === "bindings") return ARRAY_ACTIONS;
  if (field === "values") return ARRAY_VALUES;
  if (field === "items") return ARRAY_ITEMS;
  if (field === "menus") return ARRAY_MENUS;
  if (field === "paths" || field === "exportFiles") return ARRAY_PATHS;
  if (field === "nodes") return ARRAY_NODES;
  if (field === "operations") return ARRAY_OPERATIONS;
  if (field === "fields") return ARRAY_EXTENSION_FIELDS;
  if (field === "eventIds") return ARRAY_EXTENSION_EVENTS;
  return ARRAY_DEFAULT;
}
function compileType(type: SchemaType, parent: string, field: string): CompiledType {
  if (type.kind === "scalar") {
    return { kind: "scalar", code: scalarCode(type.name!), stringLimit: stringLimitKind(parent, field) };
  }
  if (type.kind === "array") {
    const element = compileType(type.element!, parent, field);
    return {
      kind: "array",
      element,
      arrayLimit: arrayLimitKind(field),
      byteArray: element.kind === "scalar" && element.code === SCALAR_BYTE,
      minBytes:
        parent === "InvokeNativeCommand" && field === "moduleId"
          ? 16
          : parent === "InvokeNativeCommand" && field === "moduleDigest"
            ? 32
            : 0,
      maxBytes:
        parent === "InvokeNativeCommand"
          ? field === "moduleId"
            ? 16
            : field === "moduleDigest"
              ? 32
              : MAX_NATIVE_CALL_BYTES
          : parent === "BytesValue"
            ? MAX_NATIVE_CALL_BYTES
            : MAX_FRAME_SIZE,
    };
  }
  const index = DEFINITION_IDS[type.name!];
  if (index === undefined) throw new Error(`unknown schema definition ${type.name}`);
  return { kind: "definition", index };
}
function compileDefinition(name: string): CompiledDefinition {
  const definition = SCHEMA_DEFINITIONS[name]!;
  if (definition.kind === "enum") {
    const base = scalarCode(definition.base!);
    return { kind: "enum", base, size: SCALAR_SIZES[base]!, values: definition.values ?? [] };
  }
  if (definition.kind === "union") {
    const branches: (number | undefined)[] = [];
    for (const branch of definition.branches ?? []) {
      const index = DEFINITION_IDS[branch.type];
      if (index === undefined) throw new Error(`unknown union schema definition ${branch.type}`);
      branches[branch.id] = index;
    }
    return { kind: "union", branches };
  }
  const fields: (CompiledField | undefined)[] = [];
  for (const field of definition.fields ?? []) fields[field.id] = { type: compileType(field.type, name, field.name) };
  return { kind: "message", fields };
}
const COMPILED_DEFINITIONS = DEFINITION_NAMES.map(compileDefinition);
const ROOT_DEFINITION = DEFINITION_IDS[PROTOCOL_SCHEMA.root]!;

function validUtf8(data: Uint8Array, offset: number, length: number): boolean {
  const end = offset + length;
  while (offset < end) {
    const first = data[offset++]!;
    if (first <= 0x7f) continue;
    if (first >= 0xc2 && first <= 0xdf) {
      if (offset >= end || (data[offset++]! & 0xc0) !== 0x80) return false;
      continue;
    }
    if (first >= 0xe0 && first <= 0xef) {
      if (offset + 1 >= end) return false;
      const second = data[offset++]!;
      if ((second & 0xc0) !== 0x80 || (first === 0xe0 && second < 0xa0) || (first === 0xed && second >= 0xa0))
        return false;
      if ((data[offset++]! & 0xc0) !== 0x80) return false;
      continue;
    }
    if (first >= 0xf0 && first <= 0xf4) {
      if (offset + 2 >= end) return false;
      const second = data[offset++]!;
      if ((second & 0xc0) !== 0x80 || (first === 0xf0 && second < 0x90) || (first === 0xf4 && second >= 0x90))
        return false;
      if ((data[offset++]! & 0xc0) !== 0x80 || (data[offset++]! & 0xc0) !== 0x80) return false;
      continue;
    }
    return false;
  }
  return true;
}

class Guard {
  private offset = 0;
  private depth = 0;
  private totalItems = 0;
  private readonly bounds: number[] = [];
  private readonly view: DataView;
  constructor(
    private readonly data: Uint8Array,
    private readonly limits: BebopDecodeLimits,
  ) {
    this.view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  }
  private end(): number {
    return this.bounds[this.bounds.length - 1] ?? this.data.length;
  }
  private fail(reason: string): never {
    throw new BoundedDecodeError(reason, this.offset);
  }
  private ensure(size: number): void {
    if (!Number.isSafeInteger(size) || size < 0 || this.offset + size > this.end())
      this.fail("remaining-byte budget exceeded");
  }
  private u8(): number {
    this.ensure(1);
    return this.data[this.offset++]!;
  }
  private u32(): number {
    this.ensure(4);
    const value = this.view.getUint32(this.offset, true);
    this.offset += 4;
    return value;
  }
  private blob(max: number): void {
    const lengthOffset = this.offset;
    const length = this.u32();
    if (length > max) throw new BoundedDecodeError(`string budget exceeded (${length} > ${max})`, lengthOffset);
    this.ensure(length);
    if (!validUtf8(this.data, this.offset, length)) throw new BoundedDecodeError("invalid UTF-8 string", this.offset);
    this.offset += length;
  }
  private stringLimit(kind: number): number {
    switch (kind) {
      case STRING_CONTENT:
        return MAX_FILE_WRITE_BYTES;
      case STRING_FONT_FAMILY:
        return 256;
      case STRING_PATH:
        return 1024;
      case STRING_DRAG_TYPE:
        return 512;
      case STRING_TAG:
        return 1024;
      case STRING_NAMED:
        return 1024;
      case STRING_FILE_TEXT:
        return MAX_FILE_WRITE_BYTES;
      default:
        return this.limits.maxStringBytes;
    }
  }
  private arrayLimit(kind: number): number {
    switch (kind) {
      case ARRAY_MODIFIERS:
        return 5;
      case ARRAY_ACTIONS:
        return 64;
      case ARRAY_VALUES:
        return 2;
      case ARRAY_ITEMS:
        return 1024;
      case ARRAY_MENUS:
        return 64;
      case ARRAY_PATHS:
      case ARRAY_EXTENSION_FIELDS:
        return MAX_EXTENSION_FIELDS;
      case ARRAY_EXTENSION_EVENTS:
        return MAX_EXTENSION_EVENTS;
      case ARRAY_NODES:
      case ARRAY_OPERATIONS:
      case ARRAY_DEFAULT:
      default:
        return this.limits.maxArrayItems;
    }
  }
  private scalar(code: number, stringLimit: number): void {
    if (code === SCALAR_STRING) return this.blob(this.stringLimit(stringLimit));
    if (code === SCALAR_BOOL) {
      const value = this.u8();
      if (value > 1) throw new BoundedDecodeError("boolean must be 0 or 1", this.offset - 1);
      return;
    }
    const size = SCALAR_SIZES[code] ?? 0;
    if (size === 0) this.fail(`unknown scalar ${code}`);
    this.ensure(size);
    this.offset += size;
  }
  private type(type: CompiledType): void {
    if (type.kind === "scalar") return this.scalar(type.code, type.stringLimit);
    if (type.kind === "array") {
      const lengthOffset = this.offset;
      const length = this.u32();
      if (type.byteArray) {
        const limit = Math.min(this.limits.maxBytes, type.maxBytes);
        if (length < type.minBytes)
          throw new BoundedDecodeError(`byte length below minimum (${length} < ${type.minBytes})`, lengthOffset);
        if (length > limit) throw new BoundedDecodeError(`bytes budget exceeded (${length} > ${limit})`, lengthOffset);
        this.ensure(length);
        this.offset += length;
        return;
      }
      const limit = this.arrayLimit(type.arrayLimit);
      if (length > limit)
        throw new BoundedDecodeError(`array item budget exceeded (${length} > ${limit})`, lengthOffset);
      this.totalItems += length;
      if (this.totalItems > this.limits.maxTotalItems)
        throw new BoundedDecodeError("total array item budget exceeded", lengthOffset);
      for (let index = 0; index < length; index += 1) this.type(type.element);
      return;
    }
    this.definition(type.index);
  }
  private enumValue(definition: Extract<CompiledDefinition, { readonly kind: "enum" }>): void {
    const valueOffset = this.offset;
    this.ensure(definition.size);
    const value =
      definition.size === 1
        ? this.data[this.offset]!
        : definition.base === SCALAR_UINT16
          ? this.view.getUint16(this.offset, true)
          : definition.base === SCALAR_INT16
            ? this.view.getInt16(this.offset, true)
            : definition.base === SCALAR_INT32
              ? this.view.getInt32(this.offset, true)
              : this.view.getUint32(this.offset, true);
    this.offset += definition.size;
    if (!definition.values.includes(value)) throw new BoundedDecodeError(`unknown enum value ${value}`, valueOffset);
  }
  private definition(index: number): void {
    this.depth += 1;
    if (this.depth > this.limits.maxNesting) this.fail(`nesting budget exceeded (${this.limits.maxNesting})`);
    const definition = COMPILED_DEFINITIONS[index];
    if (definition === undefined) this.fail(`unknown definition ${index}`);
    if (definition.kind === "enum") {
      this.enumValue(definition);
      this.depth -= 1;
      return;
    }
    const lengthOffset = this.offset;
    const length = this.u32();
    if (length > this.limits.maxMessageBytes)
      throw new BoundedDecodeError(
        `message budget exceeded (${length} > ${this.limits.maxMessageBytes})`,
        lengthOffset,
      );
    const union = definition.kind === "union";
    const contentLength = union ? length + 1 : length;
    if (!Number.isSafeInteger(contentLength)) this.fail("message length overflow");
    const end = this.offset + contentLength;
    this.ensure(contentLength);
    this.bounds.push(end);
    if (union) {
      const tagOffset = this.offset;
      const tag = this.u8();
      const branch = definition.branches[tag];
      if (branch === undefined) throw new BoundedDecodeError(`unknown union discriminator ${tag}`, tagOffset);
      this.definition(branch);
      if (this.offset !== end) this.fail("union payload has trailing bytes");
      this.bounds.pop();
      this.depth -= 1;
      return;
    }
    let last = 0;
    let terminated = false;
    while (this.offset < end) {
      const idOffset = this.offset;
      const id = this.u8();
      if (id === 0) {
        terminated = true;
        if (this.offset !== end) throw new BoundedDecodeError("message bytes after terminator", this.offset);
        break;
      }
      if (id <= last)
        throw new BoundedDecodeError(id === last ? "duplicate message field" : "message fields out of order", idOffset);
      last = id;
      const field = definition.fields[id];
      if (field === undefined) throw new BoundedDecodeError(`unknown message field ${id}`, idOffset);
      this.type(field.type);
    }
    if (!terminated || this.offset !== end) this.fail("message terminator or consumed length missing");
    this.bounds.pop();
    this.depth -= 1;
  }
  run(): void {
    if (this.data.byteLength > this.limits.maxFrameBytes)
      throw new BoundedDecodeError(`frame budget exceeded (${this.data.byteLength} > ${this.limits.maxFrameBytes})`, 0);
    this.definition(ROOT_DEFINITION);
    if (this.offset !== this.data.byteLength) this.fail("trailing bytes");
  }
}
export function boundedBebopDecode(payload: Uint8Array, limits: BebopDecodeLimits = DEFAULT_BEBOP_LIMITS): void {
  new Guard(payload, limits).run();
}
