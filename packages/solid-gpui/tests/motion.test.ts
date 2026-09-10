import { expect, test } from "bun:test";
import { createComponent, createSignal, onCleanup } from "../src/runtime";
import { createRoot, MemoryTransport, Text } from "../src/index";
import { Presence } from "../src/motion";
import { encodeJson } from "../src/native";
import { Envelope } from "../src/protocol/generated/protocol";
import { encodeFrame } from "../src/protocol";

test("Presence retains child resources through exit and ignores completion from a reversed exit", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 211 });
  let show!: (value: boolean) => void;
  let mounts = 0;
  let disposals = 0;
  let completions = 0;
  root.render(() =>
    createComponent(() => {
      const [visible, setVisible] = createSignal(true);
      show = setVisible;
      return Presence({
        get show() {
          return visible();
        },
        children: () => {
          mounts++;
          onCleanup(() => disposals++);
          return Text({ children: "Retained child" });
        },
        onExitComplete: () => completions++,
      });
    }, {}),
  );
  const initial = Envelope.decode(transport.submitted[0]!.subarray(4)).body!;
  if (initial.tag !== 1) throw new Error("expected snapshot");
  const node = initial.value.nodes!.find((node) => node.kind === 8)!;
  let sequence = 0;
  const complete = (generation: number) => {
    const last = Envelope.decode(transport.submitted.at(-1)!.subarray(4)).body!;
    const revision = last.tag === 3 ? last.value.revision! : 1;
    transport.push(
      encodeFrame({
        type: "event",
        surfaceId: 211,
        epoch: 1,
        revision,
        sequence: ++sequence,
        nodeId: node.id!,
        listenerId: node.listenerId!,
        payload: {
          type: "extension",
          eventId: 1,
          fields: [{ id: 1, value: { type: "bytes", value: encodeJson({ present: false, generation }) } }],
        },
      }),
    );
  };
  show(false);
  await Promise.resolve();
  expect(disposals).toBe(0);
  show(true);
  await Promise.resolve();
  complete(1);
  await Promise.resolve();
  expect([mounts, disposals, completions]).toEqual([1, 0, 0]);
  show(false);
  await Promise.resolve();
  complete(3);
  await Promise.resolve();
  expect([mounts, disposals, completions]).toEqual([1, 1, 1]);
  show(true);
  await Promise.resolve();
  expect(mounts).toBe(2);
  root.unmount();
  expect(disposals).toBe(2);
});
