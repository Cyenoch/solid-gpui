import { createMemo, createSignal, onCleanup } from "solid-js";
import { createHostElement, type SolidElement } from "./renderer";
import { afterRootCommit, resolveTree } from "./renderer/host-config";
import type { ExtensionDescriptor, ExtensionEvent } from "./renderer/extension";
import { assertExtensionId } from "./renderer/extension";
import type { HostProps, SolidChild } from "./renderer/types";
import { COMMAND_INVOKE_NATIVE, MAX_NATIVE_CALL_BYTES } from "./protocol";

export interface NativeInvoker {
  invokeNative(
    moduleId: Uint8Array,
    moduleDigest: Uint8Array,
    functionId: number,
    args: Uint8Array,
  ): Promise<Uint8Array>;
}
export interface NativeCommandDescriptor {
  readonly id: number;
  readonly name: string;
}
export interface NativeClientDescriptor {
  readonly moduleId: readonly number[];
  readonly moduleDigest: readonly number[];
  readonly commands: readonly NativeCommandDescriptor[];
}
export interface NativeComponentDescriptor {
  readonly providerId: readonly number[];
  readonly catalogDigest: readonly number[];
  readonly entryId: number;
  readonly entryVersion: 1;
  readonly props: readonly string[] | null;
  readonly events: readonly { readonly id: number; readonly name: string; readonly prop: string }[];
  readonly commands: readonly NativeCommandDescriptor[];
  readonly children: boolean;
  readonly slots: readonly string[];
  readonly controlled: {
    readonly eventId: number;
    readonly sequenceField: string;
    readonly ackProp: string;
    readonly valueProp: string;
  } | null;
}
export type NativeComponentProps<P, E, R, S extends string = never> = P & {
  readonly slots?: { readonly [K in S]?: SolidChild };
} & E & {
    readonly style?: HostProps["style"];
    readonly children?: SolidChild;
    readonly onLayout?: HostProps["onLayout"];
    readonly ref?: (value: R | undefined) => void;
  };

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });
const MAX_DEPTH = 128;

function jsonValue(value: unknown, depth: number, ancestors: Set<object>, omitUndefined: boolean): unknown {
  if (depth > MAX_DEPTH) throw new RangeError("Native JSON exceeds depth 128");
  if (value === null || typeof value === "boolean") return value;
  if (typeof value === "number") {
    if (!Number.isFinite(value) || Math.abs(value) > Number.MAX_SAFE_INTEGER)
      throw new TypeError("Native JSON numbers must be finite and safe");
    return value;
  }
  if (typeof value === "string") {
    if (value.length > MAX_NATIVE_CALL_BYTES) throw new RangeError("Native JSON exceeds 1 MiB");
    return value;
  }
  if (typeof value !== "object") throw new TypeError("Native JSON contains an unsupported value");
  if (ancestors.has(value)) throw new TypeError("Native JSON contains a cycle");
  const prototype = Object.getPrototypeOf(value);
  if (Array.isArray(value) ? prototype !== Array.prototype : prototype !== Object.prototype && prototype !== null)
    throw new TypeError("Native JSON requires plain objects");
  if (Object.getOwnPropertySymbols(value).length !== 0) throw new TypeError("Native JSON does not support symbol keys");
  ancestors.add(value);
  try {
    if (Array.isArray(value)) {
      if (Object.keys(value).some((key) => !/^(0|[1-9][0-9]*)$/.test(key) || Number(key) >= value.length))
        throw new TypeError("Native JSON arrays cannot have named properties");
      return Array.from(value, (item) => jsonValue(item, depth + 1, ancestors, false));
    }
    const result: Record<string, unknown> = Object.create(null);
    for (const key of Object.keys(value)) {
      const item = (value as Record<string, unknown>)[key];
      if (item === undefined && omitUndefined) continue;
      result[key] = jsonValue(item, depth + 1, ancestors, false);
    }
    return result;
  } finally {
    ancestors.delete(value);
  }
}

/** Only the outer DTO may omit optional fields; nested data remains strict JSON. */
export function encodeJson(value: unknown): Uint8Array {
  const bytes = encoder.encode(JSON.stringify(jsonValue(value, 0, new Set(), true)));
  if (bytes.byteLength > MAX_NATIVE_CALL_BYTES) throw new RangeError("Native JSON exceeds 1 MiB");
  return bytes;
}

export function decodeJson(bytes: Uint8Array): unknown {
  if (!(bytes instanceof Uint8Array)) throw new TypeError("Native JSON requires bytes");
  if (bytes.byteLength > MAX_NATIVE_CALL_BYTES) throw new RangeError("Native JSON exceeds 1 MiB");
  const value: unknown = JSON.parse(decoder.decode(bytes));
  jsonValue(value, 0, new Set(), false);
  return value;
}

function identity(values: readonly number[], length: number, name: string): Uint8Array {
  if (
    !Array.isArray(values) ||
    values.length !== length ||
    values.some((v) => !Number.isInteger(v) || v < 0 || v > 255)
  )
    throw new TypeError(`${name} must contain exactly ${length} bytes`);
  return Uint8Array.from(values);
}

function commandProxy<T>(
  commands: readonly NativeCommandDescriptor[],
  invoke: (id: number, request: unknown) => Promise<unknown>,
): T {
  const proxy: Record<string, (request?: unknown) => Promise<unknown>> = Object.create(null);
  const ids = new Set<number>();
  for (const { id, name } of commands) {
    assertExtensionId(id, "native command id");
    if (!name || name === "then" || Object.hasOwn(proxy, name) || ids.has(id))
      throw new TypeError("Native commands must have unique IDs and non-then names");
    ids.add(id);
    proxy[name] = async (request) => invoke(id, request === undefined ? null : request);
  }
  return Object.freeze(proxy) as T;
}

export function createNativeClient<T>(invoker: NativeInvoker, descriptor: NativeClientDescriptor): T {
  const moduleId = identity(descriptor.moduleId, 16, "moduleId");
  const digest = identity(descriptor.moduleDigest, 32, "moduleDigest");
  return commandProxy<T>(descriptor.commands, async (id, request) => {
    const bytes = encodeJson(request);
    // Finish Solid's synchronous batch before asking the bound root to flush.
    await Promise.resolve();
    return decodeJson(await invoker.invokeNative(moduleId.slice(), digest.slice(), id, bytes));
  });
}

export function useNativeClient<T>(descriptor: NativeClientDescriptor): T {
  return createNativeClient<T>(resolveTree(), descriptor);
}

const HOST_PROPS = new Set(["style", "children", "slots", "onLayout", "ref"]);

export function createNativeComponent<P extends object, E extends object, R, S extends string = never>(
  descriptor: NativeComponentDescriptor,
): (props: NativeComponentProps<P, E, R, S>) => SolidElement {
  const providerId = identity(descriptor.providerId, 16, "providerId");
  const catalogDigest = identity(descriptor.catalogDigest, 32, "catalogDigest");
  assertExtensionId(descriptor.entryId, "entryId");
  if (descriptor.entryVersion !== 1) throw new TypeError("Native component entryVersion must be 1");
  const eventProps = new Set<string>();
  const eventIds = new Set<number>();
  const events = descriptor.events
    .map((event) => {
      assertExtensionId(event.id, "event id");
      if (eventIds.has(event.id) || eventProps.has(event.prop) || HOST_PROPS.has(event.prop))
        throw new TypeError("Native event IDs and props must be unique");
      eventIds.add(event.id);
      eventProps.add(event.prop);
      return { ...event };
    })
    .sort((a, b) => a.id - b.id);
  const slots = new Set(descriptor.slots);
  if (slots.size !== descriptor.slots.length || [...slots].some((name) => !name || name === "children"))
    throw new TypeError("Native slots must have unique nonempty names separate from default children");
  const allowedProps = descriptor.props === null ? null : new Set(descriptor.props);
  if (allowedProps && [...allowedProps].some((name) => HOST_PROPS.has(name) || eventProps.has(name)))
    throw new TypeError("Native data props conflict with framework props");
  const controlled = descriptor.controlled;
  if (
    controlled &&
    (!eventIds.has(controlled.eventId) ||
      !controlled.sequenceField ||
      !controlled.ackProp ||
      !controlled.valueProp ||
      HOST_PROPS.has(controlled.ackProp) ||
      eventProps.has(controlled.ackProp) ||
      HOST_PROPS.has(controlled.valueProp) ||
      eventProps.has(controlled.valueProp))
  )
    throw new TypeError("Native controlled config must name a declared event and a data acknowledgement prop");

  return (props) => {
    let disposed = false;
    const [acknowledged, setAcknowledged] = createSignal(0);
    const values = props as Record<string, unknown>;
    const state = createMemo(() => {
      const dto: Record<string, unknown> = Object.create(null);
      for (const key of Object.keys(props)) {
        if (HOST_PROPS.has(key) || eventProps.has(key) || key === controlled?.ackProp) continue;
        if (allowedProps !== null && !allowedProps.has(key)) throw new TypeError(`Unknown native prop: ${key}`);
        dto[key] = values[key];
      }
      if (controlled) dto[controlled.ackProp] = acknowledged();
      const internalEvent =
        controlled && dto[controlled.valueProp] !== undefined && dto[controlled.valueProp] !== null
          ? controlled.eventId
          : undefined;
      const callbacks = new Map<number, (...args: unknown[]) => unknown>();
      for (const event of events) {
        const handler = values[event.prop];
        if (handler === undefined) continue;
        if (typeof handler !== "function") throw new TypeError(`${event.prop} must be a function`);
        callbacks.set(event.id, handler as (...args: unknown[]) => unknown);
      }
      const subscribed = events
        .filter((event) => callbacks.has(event.id) || event.id === internalEvent)
        .map((event) => event.id);
      const bytes = encodeJson(dto);
      const extension: ExtensionDescriptor = {
        providerId,
        catalogDigest,
        entryId: descriptor.entryId,
        entryVersion: descriptor.entryVersion,
        eventIds: subscribed,
        encodeProps: () => [{ id: 1, value: { type: "bytes", value: bytes } }],
      };
      const onEvent =
        subscribed.length === 0
          ? undefined
          : (event: ExtensionEvent) => {
              const handler = callbacks.get(event.eventId);
              if ((!handler && event.eventId !== internalEvent) || disposed) return;
              const field = event.fields[0];
              if (event.fields.length !== 1 || field?.id !== 1 || field.value.type !== "bytes")
                throw new TypeError("Native event must contain one JSON byte field");
              const payload = decodeJson(field.value.value);
              if (controlled && event.eventId === controlled.eventId) {
                const sequence =
                  payload !== null && typeof payload === "object"
                    ? (payload as Record<string, unknown>)[controlled.sequenceField]
                    : undefined;
                if (typeof sequence !== "number")
                  throw new TypeError("Native controlled event requires an edit sequence");
                assertExtensionId(sequence, "native edit sequence");
                if (sequence <= acknowledged()) return;
                setAcknowledged(sequence);
              }
              if (payload === null) handler?.();
              else handler?.(payload);
            };
      return { extension, onEvent };
    });
    const defaultChildren = createMemo(() => {
      const children = props.children;
      if (!descriptor.children && children !== undefined && children !== null && children !== false)
        throw new TypeError("This native component does not accept JS children");
      return children;
    });
    // Stable groups preserve Solid owners when a named slot changes. Native
    // projects their children where the GPUI component places each slot.
    const slotValues = createMemo(() => {
      const value = props.slots;
      if (value !== undefined && (value === null || typeof value !== "object" || Array.isArray(value)))
        throw new TypeError("Native slots must be an object");
      for (const name of Object.keys(value ?? {})) {
        if (!slots.has(name)) throw new TypeError(`Unknown native slot: ${name}`);
      }
      return value;
    });
    const slotGroups =
      descriptor.slots.length === 0
        ? undefined
        : ["children", ...descriptor.slots].map((name) =>
            createHostElement("View", {
              get children() {
                return name === "children" ? defaultChildren() : slotValues()?.[name as S];
              },
            }),
          );
    const node = createHostElement("Extension", {
      get __extensionDescriptor() {
        return state().extension;
      },
      get onExtensionEvent() {
        return state().onEvent;
      },
      get style() {
        return props.style;
      },
      get onLayout() {
        return props.onLayout;
      },
      get children() {
        return slotGroups ?? defaultChildren();
      },
    });
    const tree = resolveTree(node);
    const pending = new Set<(error: Error) => void>();
    const ref = commandProxy<R>(descriptor.commands, async (id, request) => {
      const args = encodeJson(request);
      await Promise.resolve();
      return afterRootCommit(tree, () => {
        if (disposed || tree.isDisposed() || !node.attached)
          return Promise.reject(new Error("Native component is unmounted"));
        return new Promise((resolve, reject) => {
          pending.add(reject);
          void tree
            .submitCommandValue(node, COMMAND_INVOKE_NATIVE, {
              type: "invoke-native",
              moduleId: providerId.slice(),
              moduleDigest: catalogDigest.slice(),
              functionId: id,
              args,
            })
            .then((result) => {
              if (disposed || !node.attached) throw new Error("Native component is unmounted");
              if (result?.type !== "bytes") throw new TypeError("Native command returned invalid bytes");
              resolve(decodeJson(result.value));
            })
            .catch(reject)
            .finally(() => pending.delete(reject));
        });
      });
    });
    const refCallback = props.ref;
    onCleanup(() => {
      disposed = true;
      for (const reject of pending) reject(new Error("Native component is unmounted"));
      pending.clear();
      refCallback?.(undefined);
    });
    refCallback?.(ref);
    return node;
  };
}
