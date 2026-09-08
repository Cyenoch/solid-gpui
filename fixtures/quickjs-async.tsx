import { createRoot, MemoryTransport, Text, TextInput, View } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import {
  createResource,
  createSignal,
  ErrorBoundary,
  Suspense,
  lazy,
  onCleanup,
  useTransition,
} from "@solid-gpui/core/runtime";
import { Envelope } from "../packages/solid-gpui/src/protocol/generated/protocol";
import { COMMAND_INVOKE_NATIVE, encodeFrame } from "../packages/solid-gpui/src/protocol";

function assert(value: unknown, message: string): void {
  if (!value) throw new Error(message);
}
const tick = async () => {
  for (let i = 0; i < 12; i++) await Promise.resolve();
};
const transport = new MemoryTransport();
const root = createRoot(transport, { surfaceId: 7, epoch: 1 });
let sequence = 0;
let frameCursor = 0;
const waiting: { requestId: number; revision: number }[] = [];
function calls() {
  for (const frame of transport.submitted.slice(frameCursor)) {
    const body = Envelope.decode(frame.subarray(4)).body;
    if (body?.tag === 4 && body.value.kind === COMMAND_INVOKE_NATIVE)
      waiting.push({ requestId: body.value.requestId!, revision: body.value.afterRevision! });
  }
  frameCursor = transport.submitted.length;
}
function reply(value: string, success = true) {
  calls();
  const call = waiting.shift();
  assert(call, "resource submitted a native request");
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 7,
      epoch: 1,
      revision: call!.revision,
      sequence: ++sequence,
      nodeId: 1,
      listenerId: 0,
      payload: {
        type: "command-result",
        result: {
          requestId: call!.requestId,
          command: COMMAND_INVOKE_NATIVE,
          nodeId: 1,
          success,
          error: success ? null : value,
          value: success ? { type: "bytes", value: new TextEncoder().encode(value) } : null,
        },
      },
    }),
  );
}
function published(text: string) {
  return transport.submitted.some((frame) => {
    const body = Envelope.decode(frame.subarray(4)).body;
    if (body?.tag === 1) return body.value.nodes?.some((node) => node.text === text);
    if (body?.tag !== 3) return false;
    return body.value.operations?.some(({ operation }) =>
      operation?.tag === 1
        ? operation.value.node?.text === text
        : operation?.tag === 2 && operation.value.text === text,
    );
  });
}
let change!: (value: number) => void;
let recover!: () => void;
let start!: (work: () => void) => Promise<void>;
let pending!: () => boolean;
let cleanups = 0;
let fallbackMounts = 0;
let resolveLazy!: (value: { default: () => import("@solid-gpui/core/jsx-runtime").JSX.Element }) => void;
const Lazy = lazy(
  () =>
    new Promise<{ default: () => import("@solid-gpui/core/jsx-runtime").JSX.Element }>((resolve) => {
      resolveLazy = resolve;
    }),
);
function Loading() {
  fallbackMounts++;
  return <Text>Loading</Text>;
}
function Data() {
  const [key, setKey] = createSignal(1);
  change = setKey;
  [pending, start] = useTransition();
  const [value] = createResource(key, () =>
    root.invokeNative(new Uint8Array(16), new Uint8Array(32), 1, new Uint8Array()),
  );
  onCleanup(() => cleanups++);
  return <Text>{value() ? `Value: ${new TextDecoder().decode(value()!)}` : ""}</Text>;
}
root.render(() => (
  <View>
    <Text>Persistent shell</Text>
    <TextInput defaultValue="Retained draft" />
    <ErrorBoundary
      fallback={(_error, reset) => {
        recover = reset;
        return <Text>Recoverable error</Text>;
      }}
    >
      <Suspense fallback={<Loading />}>
        <Data />
        <Suspense fallback={<Text>Loading module</Text>}>
          <Lazy />
        </Suspense>
      </Suspense>
    </ErrorBoundary>
  </View>
));
await tick();
assert(published("Loading"), "Suspense fallback reached the native tree");
reply("first");
resolveLazy({ default: () => <Text>Lazy ready</Text> });
await tick();
assert(published("Value: first") && published("Lazy ready"), "resource and lazy content reached the native tree");
const firstFallbacks = fallbackMounts;
const transition = start(() => change(2));
await tick();
assert(pending(), "transition remains pending for native work");
assert(fallbackMounts === firstFallbacks, "transition retains resolved content");
reply("second");
await transition;
await tick();
assert(!pending() && published("Value: second"), "transition commits the native result");
change(3);
await tick();
reply("expected native failure", false);
await tick();
assert(published("Recoverable error") && cleanups === 1, "native failure reaches ErrorBoundary and disposes children");
recover();
await tick();
reply("recovered");
await tick();
assert(published("Value: recovered"), "ErrorBoundary retry remounts a working resource");
change(4);
await tick();
calls();
root.unmount();
const count = transport.submitted.length;
reply("late");
await tick();
assert(
  transport.submitted.length === count && cleanups === 2,
  "unmount discards pending native output and disposes owners",
);
createRoot(new EmbeddedTransport()).render(() => <Text>Async: passed</Text>);
