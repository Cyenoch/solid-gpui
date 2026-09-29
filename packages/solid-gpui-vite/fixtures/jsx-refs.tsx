/** @jsxImportSource @solid-gpui/core */
import { MemoryTransport, Text, VirtualList, createRoot, type VirtualListHandle } from "@solid-gpui/core";
import { createSignal, getListener, getOwner, onCleanup, type Owner } from "@solid-gpui/core/runtime";
import { TestHost } from "@solid-gpui/core/testing";

function check(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

export async function verifyJsxRefs(): Promise<void> {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 1 });
  const host = new TestHost(transport);
  const handles: VirtualListHandle[] = [];
  let assigned!: VirtualListHandle;
  let owner: Owner | null = null;
  let cleanups = 0;
  let returnedCleanups = 0;
  const [read, write] = createSignal(0);
  const capture = (handle: VirtualListHandle) => {
    check(getOwner() !== null, "ref lost its Solid owner");
    check(getListener() === null, "ref callback is tracked");
    read();
    handles.push(handle);
    onCleanup(() => cleanups++);
    return () => returnedCleanups++;
  };
  const arrayRefs = [capture, [null, false, undefined, capture]] as const;
  const props = {
    data: [0, 1, 2],
    itemKey: (item: number) => item,
    estimatedItemSize: 20,
    renderItem: (item: number) => <Text>{item}</Text>,
  };
  function Forwarded(input: { capture: typeof capture }) {
    owner = getOwner();
    return <VirtualList {...props} ref={input.capture} />;
  }
  try {
    root.render(() => (
      <>
        <VirtualList {...props} ref={capture} />
        <Forwarded capture={capture} />
        <VirtualList {...props} ref={assigned} />
        <VirtualList {...props} ref={arrayRefs} />
      </>
    ));
    check(owner !== null, "forwarded ref has no owner");
    check(handles.length === 4, "callback refs must run once per mount, including nested arrays");
    check(assigned?.kind === "VirtualList", "assignment ref did not receive a list handle");
    const lists = host.surface(1)?.nodes.filter((node) => node.kind === "VirtualList");
    check(lists?.length === 4, "JSX did not mount four native lists");
    check(handles[2] === handles[3], "array callbacks did not receive the same handle");
    for (const [index, handle] of [handles[0]!, handles[1]!, assigned, handles[2]!].entries()) {
      check(handle.id === lists[index]?.id, "ref does not identify its native list");
      const offset = handle.getScrollOffset();
      await Promise.resolve();
      const command = host.scrollCommands.at(-1);
      check(command?.type === "get-scroll-offset", "list handle did not issue a native command");
      host.replyScroll(command, 40);
      check((await offset) === 40, "list handle did not resolve the native reply");
    }
    write(1);
    await Promise.resolve();
    check(handles.length === 4, "signal reads in refs caused repeated callbacks");
  } finally {
    root.unmount();
  }
  check(cleanups === 4, "ref onCleanup did not follow its owner");
  check(handles.length === 4, "VirtualList unexpectedly called refs on unmount");
  check(returnedCleanups === 0, "ref return values must not register cleanup");
}

await verifyJsxRefs();
