import {
  MAX_EXTENSION_BYTES,
  MAX_EXTENSION_EVENTS,
  MAX_EXTENSION_FIELDS,
  MAX_EXTENSION_TEXT_BYTES,
  type ExtensionField,
  type ExtensionProperties,
  type ExtensionValue,
} from "../protocol";
import type { SolidChild, HostNodeInternal, HostProps } from "./types";
const EXTENSION_ENCODER = new TextEncoder();

/** Provider-neutral descriptor supplied by a generated extension package. */
export interface ExtensionDescriptor<Props extends object = Record<string, unknown>> {
  readonly providerId: Uint8Array;
  readonly catalogDigest: Uint8Array;
  readonly entryId: number;
  readonly entryVersion: number;
  readonly encodeProps: (props: Props) => readonly ExtensionField[];
  readonly eventIds: readonly number[];
}

export type ExtensionEvent = {
  readonly eventId: number;
  readonly fields: readonly ExtensionField[];
  readonly target: HostNodeInternal;
};

export type ExtensionEventHandler = (event: ExtensionEvent) => void;

export type ExtensionProps<Props extends object> = Props & {
  readonly children?: SolidChild;
  readonly onExtensionEvent?: ExtensionEventHandler;
  readonly style?: HostProps["style"];
  readonly onLayout?: HostProps["onLayout"];
};

export type ExtensionComponentProps<Props extends object> = ExtensionProps<Props>;

export function assertExtensionBytes(value: Uint8Array, length: number, name: string): void {
  if (!(value instanceof Uint8Array) || value.byteLength !== length) {
    throw new RangeError(`${name} must be exactly ${length} bytes`);
  }
}

export function assertExtensionId(value: number, name: string): void {
  if (!Number.isInteger(value) || value <= 0 || value > 0xffff_ffff)
    throw new RangeError(`${name} must be a non-zero u32`);
}

function assertExtensionU32(value: number, name: string): void {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) throw new RangeError(`${name} must be a u32`);
}

export function assertExtensionValue(value: ExtensionValue, name: string): void {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${name} is invalid`);
  }
  switch (value.type) {
    case "bool":
      if (typeof value.value !== "boolean") throw new TypeError(`${name} must be a bool`);
      return;
    case "i32":
      if (!Number.isInteger(value.value) || value.value < -0x8000_0000 || value.value > 0x7fff_ffff)
        throw new TypeError(`${name} must be an i32`);
      return;
    case "u32":
      assertExtensionU32(value.value, `${name} value`);
      return;
    case "f32":
      if (!Number.isFinite(value.value) || !Number.isFinite(Math.fround(value.value)))
        throw new TypeError(`${name} must be a finite f32`);
      return;
    case "text":
      if (typeof value.value !== "string") throw new TypeError(`${name} must be text`);
      if (EXTENSION_ENCODER.encode(value.value).byteLength > MAX_EXTENSION_TEXT_BYTES)
        throw new RangeError(`${name} exceeds its byte limit`);
      return;
    case "bytes":
      if (!(value.value instanceof Uint8Array)) throw new TypeError(`${name} must be bytes`);
      if (value.value.byteLength > MAX_EXTENSION_BYTES) throw new RangeError(`${name} exceeds its byte limit`);
      return;
    default:
      throw new TypeError(`${name} has an unknown type`);
  }
}

export function normalizeExtensionProperties<Props extends object>(
  descriptor: ExtensionDescriptor<Props>,
  props: Props,
  eventIds: readonly number[] = descriptor.eventIds,
): ExtensionProperties {
  if (descriptor === null || typeof descriptor !== "object") throw new TypeError("Extension descriptor is invalid");
  assertExtensionBytes(descriptor.providerId, 16, "providerId");
  assertExtensionBytes(descriptor.catalogDigest, 32, "catalogDigest");
  assertExtensionId(descriptor.entryId, "entryId");
  assertExtensionId(descriptor.entryVersion, "entryVersion");
  if (typeof descriptor.encodeProps !== "function") throw new TypeError("Extension encodeProps must be a function");
  if (!Array.isArray(eventIds) || eventIds.length > MAX_EXTENSION_EVENTS) {
    throw new RangeError("eventIds must contain at most 256 IDs");
  }
  assertSortedIds(eventIds, "eventIds");
  const fields = descriptor.encodeProps(props);
  assertSortedExtensionFields(fields);
  return {
    providerId: descriptor.providerId.slice(),
    catalogDigest: descriptor.catalogDigest.slice(),
    entryId: descriptor.entryId,
    entryVersion: descriptor.entryVersion,
    fields: fields.map((field) => ({ id: field.id, value: cloneExtensionValue(field.value) })),
    eventIds: [...eventIds],
  };
}

function cloneExtensionValue(value: ExtensionValue): ExtensionValue {
  return value.type === "bytes" ? { type: "bytes", value: value.value.slice() } : value;
}

export function assertSortedIds(values: readonly number[], name: string): void {
  if (!Array.isArray(values) || values.length > MAX_EXTENSION_EVENTS) throw new RangeError(`${name} is too large`);
  let previous = -1;
  for (const value of values) {
    assertExtensionId(value, `${name} entry`);
    if (value <= previous) throw new RangeError(`${name} must be sorted and unique`);
    previous = value;
  }
}

export function assertSortedExtensionFields(fields: readonly ExtensionField[]): void {
  if (!Array.isArray(fields) || fields.length > MAX_EXTENSION_FIELDS) {
    throw new RangeError("extension fields must contain at most 256 fields");
  }
  let previous = -1;
  let textBytes = 0;
  let bytes = 0;
  const encoder = EXTENSION_ENCODER;
  for (const field of fields) {
    if (field === null || typeof field !== "object") throw new TypeError("extension field is invalid");
    assertExtensionId(field.id, "extension field id");
    if (field.id <= previous) throw new RangeError("extension fields must be sorted and unique");
    assertExtensionValue(field.value, `extension field ${field.id}`);
    if (field.value.type === "text") textBytes += encoder.encode(field.value.value).byteLength;
    if (field.value.type === "bytes") bytes += field.value.value.byteLength;
    if (textBytes > MAX_EXTENSION_TEXT_BYTES || bytes > MAX_EXTENSION_BYTES)
      throw new RangeError("extension field payload exceeds its aggregate byte limit");
    previous = field.id;
  }
}
