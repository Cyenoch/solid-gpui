import { expect, test } from "bun:test";
import { createComponent, createSignal, onCleanup, type Component } from "../src/runtime";
import { createContext, useContext } from "solid-js";
import type { SolidElement } from "../src/index";
import { MemoryTransport, SystemPopover, Text, View, createRoot, mountApplication } from "../src/index";
import { COMMAND_CLOSE_POPUP, COMMAND_OPEN_POPUP, encodeFrame } from "../src/protocol";
import type { GenerationHost, GenerationLifecycle } from "../src/generation";
import { Envelope, type Command } from "../src/protocol/generated/protocol";

const settle = () => new Promise<void>((resolve) => setTimeout(resolve, 0));
const messages = (transport: MemoryTransport) =>
  transport.submitted.map((frame) => Envelope.decode(frame.subarray(4)).body!);
function opening(transport: MemoryTransport): Command {
  const body = messages(transport)
    .reverse()
    .find((body) => body.tag === 4 && body.value.kind === COMMAND_OPEN_POPUP);
  if (body?.tag !== 4) throw new Error("missing popup request");
  return body.value;
}
function acknowledge(transport: MemoryTransport, command: Command, surfaceId: number, sequence = 1) {
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: command.surfaceId!,
      epoch: command.epoch!,
      revision: command.afterRevision!,
      sequence,
      nodeId: 1,
      listenerId: 0,
      payload: {
        type: "command-result",
        result: {
          requestId: command.requestId!,
          command: COMMAND_OPEN_POPUP,
          nodeId: 1,
          success: true,
          error: null,
          value: { type: "number", value: surfaceId },
        },
      },
    }),
  );
}

test("popup content inherits Solid context while commits and cleanup belong to its own Surface", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 1 });
  const Name = createContext<() => string>();
  // Solid's context implementation is universal; its upstream Provider type names DOM children.
  const NameProvider = Name.Provider as unknown as Component<{ value: () => string; children?: SolidElement }>;
  const [name, setName] = createSignal("Ada");
  const [open, setOpen] = createSignal(true);
  let disposed = 0;
  root.render(() =>
    createComponent(NameProvider, {
      value: name,
      get children() {
        return createComponent(View, {
          get children() {
            return [
              createComponent(Text, {
                get children() {
                  return `Owner: ${name()}`;
                },
              }),
              createComponent(SystemPopover, {
                get open() {
                  return open();
                },
                onOpenChange: setOpen,
                width: 320,
                height: 180,
                slots: {
                  get trigger() {
                    return createComponent(Text, { children: "Edit" });
                  },
                },
                content: () => {
                  const value = useContext(Name)!;
                  onCleanup(() => disposed++);
                  return createComponent(Text, {
                    get children() {
                      return `Popup: ${value()}`;
                    },
                  });
                },
              }),
            ];
          },
        });
      },
    }),
  );
  await settle();
  acknowledge(transport, opening(transport), 41);
  await settle();
  const snapshots = messages(transport).filter((body) => body.tag === 1);
  expect(snapshots).toHaveLength(2);
  expect(snapshots[1]!.value.surfaceId).toBe(41);
  expect(snapshots[1]!.value.nodes!.some((node) => node.text === "Popup: Ada")).toBe(true);
  const before = transport.submitted.length;
  setName("Grace");
  await settle();
  const patches = messages(transport)
    .slice(before)
    .filter((body) => body.tag === 3);
  expect(patches.map((body) => body.value.surfaceId).sort()).toEqual([1, 41]);
  setOpen(false);
  await settle();
  expect(disposed).toBe(1);
  const close = messages(transport)
    .reverse()
    .find((body) => body.tag === 4 && body.value.kind === COMMAND_CLOSE_POPUP);
  expect(close?.tag === 4 && close.value.payload).toEqual({
    tag: 16,
    value: { requestId: opening(transport).requestId },
  });
  root.unmount();
  expect(disposed).toBe(1);
});

test("unmount cancels an in-flight open before its native Surface ID arrives", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 7 });
  let mounted = 0;
  root.render(() =>
    createComponent(SystemPopover, {
      open: true,
      onOpenChange: () => {},
      width: 240,
      height: 160,
      slots: {
        get trigger() {
          return createComponent(Text, { children: "Open" });
        },
      },
      content: () => {
        mounted++;
        return createComponent(Text, { children: "Late content" });
      },
    }),
  );
  await settle();
  const request = opening(transport);
  root.render(null);
  await settle();
  const close = messages(transport)
    .reverse()
    .find((body) => body.tag === 4 && body.value.kind === COMMAND_CLOSE_POPUP);
  expect(close?.tag === 4 && close.value.payload).toEqual({ tag: 16, value: { requestId: request.requestId } });
  acknowledge(transport, request, 92);
  await settle();
  expect(mounted).toBe(0);
  expect(messages(transport).some((body) => body.tag === 1 && body.value.surfaceId === 92)).toBe(false);
  root.unmount();
});

test("native dismissal disposes the child once and a later open gets a fresh Surface", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 1 });
  const [open, setOpen] = createSignal(true);
  let disposed = 0;
  root.render(() =>
    createComponent(SystemPopover, {
      get open() {
        return open();
      },
      onOpenChange: setOpen,
      width: 240,
      height: 160,
      slots: {
        get trigger() {
          return createComponent(Text, { children: "Open" });
        },
      },
      content: () => {
        onCleanup(() => disposed++);
        return createComponent(Text, { children: "Form" });
      },
    }),
  );
  await settle();
  acknowledge(transport, opening(transport), 50);
  await settle();
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 50,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: 0,
      listenerId: 0,
      payload: { type: "surface-closed" },
    }),
  );
  await settle();
  expect(open()).toBe(false);
  expect(disposed).toBe(1);
  setOpen(true);
  await settle();
  acknowledge(transport, opening(transport), 51, 2);
  await settle();
  expect(
    messages(transport)
      .filter((body) => body.tag === 1)
      .map((body) => body.value.surfaceId),
  ).toEqual([1, 50, 51]);
  root.unmount();
  expect(disposed).toBe(2);
  const close = messages(transport)
    .reverse()
    .find((body) => body.tag === 4 && body.value.kind === COMMAND_CLOSE_POPUP);
  expect(close?.tag === 4 && close.value.payload).toEqual({
    tag: 16,
    value: { requestId: opening(transport).requestId },
  });
});

test("unmounting and remounting a nested trigger retains the parent's Host Tree", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 1 });
  const [mounted, setMounted] = createSignal(true);
  let disposed = 0;
  root.render(() =>
    createComponent(View, {
      get children() {
        return mounted()
          ? createComponent(SystemPopover, {
              open: true,
              onOpenChange: () => {},
              width: 300,
              height: 180,
              slots: {
                get trigger() {
                  return createComponent(Text, { children: "Outer" });
                },
              },
              content: () => {
                onCleanup(() => disposed++);
                return createComponent(SystemPopover, {
                  open: true,
                  onOpenChange: () => {},
                  width: 200,
                  height: 100,
                  slots: {
                    get trigger() {
                      return createComponent(Text, { children: "Inner" });
                    },
                  },
                  content: () => createComponent(Text, { children: "Nested" }),
                });
              },
            })
          : null;
      },
    }),
  );
  await settle();
  acknowledge(transport, opening(transport), 40);
  await settle();
  expect(opening(transport).surfaceId).toBe(40);
  acknowledge(transport, opening(transport), 41);
  await settle();
  setMounted(false);
  await settle();
  expect(disposed).toBe(1);
  setMounted(true);
  await settle();
  expect(opening(transport).surfaceId).toBe(1);
  acknowledge(transport, opening(transport), 42, 2);
  await settle();
  expect(opening(transport).surfaceId).toBe(42);
  root.unmount();
  expect(disposed).toBe(2);
});

test("a staged QuickJS generation opens controlled popovers only after activation", async () => {
  const scope = globalThis as { __solidGpuiGeneration?: GenerationHost };
  const previous = scope.__solidGpuiGeneration;
  let lifecycle: GenerationLifecycle | undefined;
  const transport = new MemoryTransport();
  scope.__solidGpuiGeneration = {
    epoch: 2,
    state: JSON.stringify({ state: [true], surfaceId: 1, open: true, activationSequence: 0 }),
    register(value) {
      lifecycle = value;
    },
  };
  try {
    mountApplication<boolean>({
      transport: () => transport,
      setup: (open = true) => ({
        captureState: () => open,
        render: () =>
          createComponent(SystemPopover, {
            open,
            onOpenChange: () => {},
            width: 240,
            height: 160,
            slots: {
              get trigger() {
                return createComponent(Text, { children: "Open" });
              },
            },
            content: () => createComponent(Text, { children: "Restored content" }),
          }),
      }),
    });
    await settle();
    expect(messages(transport).some((body) => body.tag === 1)).toBe(true);
    expect(messages(transport).some((body) => body.tag === 4 && body.value.kind === COMMAND_OPEN_POPUP)).toBe(false);
    lifecycle!.activate();
    await settle();
    const request = opening(transport);
    expect(request.epoch).toBe(2);
    acknowledge(transport, request, 70);
    await settle();
    expect(
      messages(transport).some((body) => body.tag === 1 && body.value.surfaceId === 70 && body.value.epoch === 2),
    ).toBe(true);
  } finally {
    lifecycle?.retire();
    if (previous) scope.__solidGpuiGeneration = previous;
    else delete scope.__solidGpuiGeneration;
  }
});
