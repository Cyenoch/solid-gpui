import { expect, test } from "bun:test";
import { createSignal, createComponent, batch } from "../src/runtime";
import { createRoot, MemoryTransport, Text } from "../src/index";
import {
  createNativeClient,
  createNativeComponent,
  useNativeClient,
  encodeJson,
  decodeJson,
  type NativeComponentDescriptor,
} from "../src/native";
import { Envelope, type Command } from "../src/protocol/generated/protocol";
import { COMMAND_INVOKE_NATIVE, encodeFrame } from "../src/protocol";

const descriptor: NativeComponentDescriptor = {
  providerId: Array(16).fill(7),
  catalogDigest: Array(32).fill(9),
  entryId: 1,
  entryVersion: 1,
  props: ["label"],
  events: [{ id: 1, name: "press", prop: "onPress" }],
  commands: [{ id: 1, name: "focus" }],
  children: false,
  controlled: null,
};
const clientDescriptor = {
  moduleId: descriptor.providerId,
  moduleDigest: descriptor.catalogDigest,
  commands: [{ id: 1, name: "greet" }],
};
function body(frame: Uint8Array) {
  return Envelope.decode(frame.subarray(4)).body!;
}
function reply(transport: MemoryTransport, command: Command, value: unknown, sequence = 1) {
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: command.surfaceId!,
      epoch: command.epoch!,
      revision: command.afterRevision!,
      sequence,
      nodeId: command.nodeId!,
      listenerId: 0,
      payload: {
        type: "command-result",
        result: {
          requestId: command.requestId!,
          command: COMMAND_INVOKE_NATIVE,
          nodeId: command.nodeId!,
          success: true,
          error: null,
          value: { type: "bytes", value: encodeJson(value) },
        },
      },
    }),
  );
}

test("native JSON rejects lossy values, deep data, cycles and unsafe responses", () => {
  expect(decodeJson(encodeJson({ optional: undefined, text: "中文", nested: [1, null] }))).toEqual({
    text: "中文",
    nested: [1, null],
  });
  const cyclic: { self?: unknown } = {};
  cyclic.self = cyclic;
  let deep: unknown = null;
  for (let i = 0; i < 130; i++) deep = [deep];
  for (const value of [
    NaN,
    Infinity,
    Number.MAX_SAFE_INTEGER + 1,
    1n,
    () => {},
    new Date(),
    cyclic,
    deep,
    [undefined],
    { nested: { no: undefined } },
    { callback: () => {} },
    { [Symbol()]: 1 },
    "x".repeat(1_048_577),
  ])
    expect(() => encodeJson(value)).toThrow();
  for (const value of ["9007199254740992", "1e400", '["unterminated]'])
    expect(() => decodeJson(new TextEncoder().encode(value))).toThrow();
  expect(() => decodeJson(Uint8Array.of(255))).toThrow();
});

test("native components preserve getters and callback generations while subscribing only live handlers", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 201 });
  const calls: string[] = [];
  let label!: (value: string) => void;
  let handler!: (value: (() => void) | undefined) => void;
  const Button = createNativeComponent<{ label: string }, { onPress?: () => void }, {}>(descriptor);
  function App() {
    const [text, setText] = createSignal("before");
    const [onPress, setOnPress] = createSignal<(() => void) | undefined>(() => calls.push("old"));
    label = setText;
    handler = (value) => setOnPress(() => value);
    return Button({
      get label() {
        return text();
      },
      get onPress() {
        return onPress();
      },
    });
  }
  root.render(() => createComponent(App, {}));
  const initial = body(transport.submitted[0]!);
  if (initial.tag !== 1) throw new Error("expected snapshot");
  const node = initial.value.nodes!.find((node) => node.kind === 8)!;
  expect(node.hostProperties?.tag === 5 ? node.hostProperties.value.eventIds : undefined).toEqual([1]);
  batch(() => {
    label("after");
    handler(() => calls.push("new"));
  });
  await Promise.resolve();
  const patch = body(transport.submitted[1]!);
  if (patch.tag !== 3) throw new Error("expected patch");
  const update = patch.value.operations!.find((op) => op.operation?.tag === 2)!.operation!;
  if (update.tag !== 2 || update.value.hostProperties?.tag !== 5) throw new Error("expected extension update");
  const field = update.value.hostProperties.value.fields![0]!.value!;
  if (field.tag !== 6) throw new Error("expected byte field");
  expect(decodeJson(field.value.value!)).toEqual({ label: "after" });
  const event = (revision: number, sequence: number) =>
    transport.push(
      encodeFrame({
        type: "event",
        surfaceId: 201,
        epoch: 1,
        revision,
        sequence,
        nodeId: node.id!,
        listenerId: node.listenerId!,
        payload: {
          type: "extension",
          eventId: 1,
          fields: [{ id: 1, value: { type: "bytes", value: encodeJson(null) } }],
        },
      }),
    );
  event(1, 1);
  event(2, 2);
  expect(calls).toEqual(["old", "new"]);
  handler(undefined);
  await Promise.resolve();
  const removed = body(transport.submitted.at(-1)!);
  if (removed.tag !== 3) throw new Error("expected unsubscribe patch");
  const props = removed.value.operations!.find((op) => op.operation?.tag === 2)!.operation!;
  expect(
    props.tag === 2 && props.value.hostProperties?.tag === 5 ? props.value.hostProperties.value.eventIds : undefined,
  ).toEqual([]);
  root.unmount();
});

test("native refs submit after the complete prop transaction and revoke pending calls on cleanup", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 202 });
  type Ref = { focus(): Promise<string> };
  let ref: Ref | undefined;
  let setLabel!: (value: string) => void;
  const Editor = createNativeComponent<{ label: string }, {}, Ref>({ ...descriptor, events: [] });
  root.render(() => {
    const [label, set] = createSignal("initial");
    setLabel = set;
    return Editor({
      get label() {
        return label();
      },
      ref: (value) => {
        ref = value;
      },
    });
  });
  const mounted = ref!;
  let result!: Promise<string>;
  batch(() => {
    setLabel("updated");
    result = mounted.focus();
    setLabel("final");
  });
  await Promise.resolve();
  const frames = transport.submitted.map(body);
  expect(frames.map((frame) => frame.tag)).toEqual([1, 3, 4]);
  const command = frames[2]!;
  if (command.tag !== 4) throw new Error("expected command");
  expect(command.value.afterRevision).toBe(2);
  expect(command.value.nodeId).toBe(
    frames[0]?.tag === 1 ? frames[0].value.nodes!.find((node) => node.kind === 8)!.id! : -1,
  );
  reply(transport, command.value, "focused");
  await expect(result).resolves.toBe("focused");
  const pending = mounted.focus();
  await Promise.resolve();
  root.render(null);
  await expect(pending).rejects.toThrow("unmounted");
  expect(ref).toBeUndefined();
  await expect(mounted.focus()).rejects.toThrow("unmounted");
  root.unmount();
});

test("controlled native echo commits its value and internal acknowledgement together and ignores stale edits", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 205 });
  const edits: string[] = [];
  const Input = createNativeComponent<
    { value: string; ackEditSeq?: number },
    { onChange: (event: { value: string; editSeq: number }) => void },
    {}
  >({
    ...descriptor,
    props: null,
    commands: [],
    events: [{ id: 1, name: "change", prop: "onChange" }],
    controlled: { eventId: 1, sequenceField: "editSeq", ackProp: "ackEditSeq", valueProp: "value" },
  });
  root.render(() => {
    const [value, setValue] = createSignal("initial");
    return Input({
      get value() {
        return value();
      },
      ackEditSeq: 999,
      onChange(event) {
        edits.push(event.value);
        setValue(event.value);
      },
    });
  });
  const first = body(transport.submitted[0]!);
  if (first.tag !== 1) throw new Error("expected snapshot");
  const node = first.value.nodes!.find((node) => node.kind === 8)!;
  const initial = node.hostProperties?.tag === 5 ? node.hostProperties.value.fields![0]!.value : undefined;
  if (initial?.tag !== 6) throw new Error("expected native JSON");
  expect(decodeJson(initial.value.value!)).toEqual({ value: "initial", ackEditSeq: 0 });
  const emit = (editSeq: number, value: string, revision: number, sequence: number) =>
    transport.push(
      encodeFrame({
        type: "event",
        surfaceId: 205,
        epoch: 1,
        revision,
        sequence,
        nodeId: node.id!,
        listenerId: node.listenerId!,
        payload: {
          type: "extension",
          eventId: 1,
          fields: [{ id: 1, value: { type: "bytes", value: encodeJson({ value, editSeq }) } }],
        },
      }),
    );
  emit(3, "typed", 1, 1);
  expect(transport.submitted).toHaveLength(2);
  const patch = body(transport.submitted[1]!);
  if (patch.tag !== 3) throw new Error("expected acknowledgement patch");
  const update = patch.value.operations!.find((operation) => operation.operation?.tag === 2)!.operation!;
  const bytes =
    update.tag === 2 && update.value.hostProperties?.tag === 5
      ? update.value.hostProperties.value.fields![0]!.value
      : undefined;
  if (bytes?.tag !== 6) throw new Error("expected native JSON update");
  expect(decodeJson(bytes.value.value!)).toEqual({ value: "typed", ackEditSeq: 3 });
  emit(2, "stale", 2, 2);
  expect(edits).toEqual(["typed"]);
  expect(transport.submitted).toHaveLength(2);
  root.unmount();
});

test("controlled values keep their internal subscription without a handler and unsubscribe in uncontrolled mode", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 206 });
  let setValue!: (value: string | undefined) => void;
  const Input = createNativeComponent<{ value?: string }, {}, {}>({
    ...descriptor,
    props: null,
    commands: [],
    events: [{ id: 1, name: "change", prop: "onChange" }],
    controlled: { eventId: 1, sequenceField: "editSeq", ackProp: "ackEditSeq", valueProp: "value" },
  });
  root.render(() => {
    const [value, set] = createSignal<string | undefined>("locked");
    setValue = set;
    return Input({
      get value() {
        return value();
      },
    });
  });
  const first = body(transport.submitted[0]!);
  if (first.tag !== 1) throw new Error("expected snapshot");
  const node = first.value.nodes!.find((node) => node.kind === 8)!;
  expect(node.hostProperties?.tag === 5 ? node.hostProperties.value.eventIds : undefined).toEqual([1]);
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 206,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: node.id!,
      listenerId: node.listenerId!,
      payload: {
        type: "extension",
        eventId: 1,
        fields: [{ id: 1, value: { type: "bytes", value: encodeJson({ value: "edited", editSeq: 1 }) } }],
      },
    }),
  );
  const latestProperties = () => {
    const frame = body(transport.submitted.at(-1)!);
    if (frame.tag !== 3) throw new Error("expected props patch");
    const operation = frame.value.operations!.find((operation) => operation.operation?.tag === 2)!.operation!;
    if (operation.tag !== 2 || operation.value.hostProperties?.tag !== 5) throw new Error("expected extension props");
    return operation.value.hostProperties.value;
  };
  const acknowledged = latestProperties().fields![0]!.value!;
  if (acknowledged.tag !== 6) throw new Error("expected JSON bytes");
  expect(decodeJson(acknowledged.value.value!)).toEqual({ value: "locked", ackEditSeq: 1 });
  setValue(undefined);
  await Promise.resolve();
  expect(latestProperties().eventIds).toEqual([]);
  setValue("controlled again");
  await Promise.resolve();
  expect(latestProperties().eventIds).toEqual([1]);
  root.unmount();
});

test("native clients bind their setup root across async work and reject after root retirement", async () => {
  const a = new MemoryTransport();
  const b = new MemoryTransport();
  const first = createRoot(a, { surfaceId: 203 });
  const second = createRoot(b, { surfaceId: 204 });
  type Client = { greet(value: { name: string }): Promise<string> };
  let captured!: Client;
  first.render(() => {
    captured = useNativeClient<Client>(clientDescriptor);
    return Text({ children: "first" });
  });
  second.render(() => Text({ children: "second" }));
  const explicit = createNativeClient<Client>(second, clientDescriptor);
  const one = captured.greet({ name: "A" });
  const two = explicit.greet({ name: "B" });
  await Promise.resolve();
  for (const [transport, surfaceId] of [
    [a, 203],
    [b, 204],
  ] as const) {
    const command = body(transport.submitted[1]!);
    if (command.tag !== 4) throw new Error("expected command");
    expect(command.value.surfaceId).toBe(surfaceId);
    reply(transport, command.value, String(surfaceId));
  }
  await expect(one).resolves.toBe("203");
  await expect(two).resolves.toBe("204");
  first.unmount();
  second.unmount();
  await expect(captured.greet({ name: "late" })).rejects.toThrow();
  expect(() => useNativeClient(clientDescriptor)).toThrow("not associated");
});

test("native component contract rejects unknown props and forbidden children before publishing", () => {
  const Component = createNativeComponent<Record<string, unknown>, {}, {}>(descriptor);
  for (const props of [{ typo: 1 }, { label: "ok", children: "forbidden" }, { label: "ok", onPress: 4 }]) {
    const transport = new MemoryTransport();
    const root = createRoot(transport);
    expect(() => root.render(() => Component(props))).toThrow();
    expect(transport.submitted).toHaveLength(0);
    root.unmount();
  }
});
