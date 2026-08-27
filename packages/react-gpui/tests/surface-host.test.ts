import { decode } from "@msgpack/msgpack";
import { describe, expect, it } from "bun:test";

import {
  COMMAND_OPEN_SURFACE,
  EVENT_COMMAND_RESULT,
  EVENT_PRESS,
  EVENT_SURFACE_CLOSED,
  PROTOCOL_VERSION,
  encodeFrame,
  framePayload,
} from "../src/protocol";
import { MemoryTransport, TransportTerminatedError, type TransportTerminationListener } from "../src/transport";
import { SurfaceClosedError, SurfaceIdReusedError, createSurfaceHost } from "../src/index";

class TerminatingTransport extends MemoryTransport {
  private readonly terminationListeners = new Set<TransportTerminationListener>();

  override onTermination(listener: TransportTerminationListener): () => void {
    this.terminationListeners.add(listener);
    return () => this.terminationListeners.delete(listener);
  }
  terminate(): void {
    const error = new TransportTerminatedError("test transport terminated", { kind: "eof" });
    const listeners = [...this.terminationListeners];
    this.terminationListeners.clear();
    for (const listener of listeners) listener(error);
  }
}

function commandResult(
  surfaceId: number,
  epoch: number,
  sequence: number,
  requestId: number,
  command: number,
  value: readonly unknown[] | null = null,
): Uint8Array {
  const result: readonly unknown[] =
    value === null ? [2, requestId, command, 1, true, null, null] : [2, requestId, command, 1, true, null, value];
  return encodeFrame([PROTOCOL_VERSION, 2, surfaceId, epoch, 1, sequence, 1, 0, EVENT_COMMAND_RESULT, result] as never);
}

function surfaceClosed(surfaceId: number, epoch: number, sequence: number): Uint8Array {
  return encodeFrame([PROTOCOL_VERSION, 2, surfaceId, epoch, 1, sequence, 0, 0, EVENT_SURFACE_CLOSED, null] as never);
}

function message(transport: MemoryTransport, index: number): readonly unknown[] {
  return decode(transport.submitted[index].slice(4)) as readonly unknown[];
}

describe("SurfaceHost", () => {
  it("opens a surface and returns the native surface id value", async () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const source = host.createRoot({ surfaceId: 1 });
    source.render(null);

    const opened = source.openSurface({ title: "child", width: 640, height: 480 });
    expect(message(transport, 1)).toEqual([
      PROTOCOL_VERSION,
      4,
      1,
      1,
      1,
      1,
      1,
      COMMAND_OPEN_SURFACE,
      ["child", [640, 480]],
    ]);

    transport.push(commandResult(1, 1, 1, 1, COMMAND_OPEN_SURFACE, [1, 41]));
    await expect(opened).resolves.toBe(41);
    const malformed = source.openSurface();
    expect(message(transport, 2)[7]).toBe(COMMAND_OPEN_SURFACE);
    transport.push(commandResult(1, 1, 2, 2, COMMAND_OPEN_SURFACE, [2, [41, 42]]));
    await expect(malformed).rejects.toThrow("invalid surface id");

    const root = host.createRoot({ surfaceId: 41 });
    root.render(null);
    expect(message(transport, 3)[2]).toBe(41);
    root.unmount();
    source.unmount();
    host.dispose();
  });
  it("encodes creation options as an appended compatible payload", async () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const source = host.createRoot({ surfaceId: 2 });
    source.render(null);

    const opened = source.openSurface({
      title: "Inspector",
      width: 640,
      height: 480,
      kind: "floating",
      resizable: false,
      minSize: [320, 240],
    });
    expect(message(transport, 1)[8]).toEqual(["Inspector", [640, 480], [1, false, 320, 240]]);
    transport.push(commandResult(2, 1, 1, 1, COMMAND_OPEN_SURFACE, [1, 9]));
    await expect(opened).resolves.toBe(9);

    const before = transport.submitted.length;
    await expect(source.openSurface({ minSize: [0, 240] })).rejects.toThrow("surface minSize");
    await expect(source.openSurface({ kind: "popup" as never })).rejects.toThrow("surface kind");
    expect(transport.submitted).toHaveLength(before);
    source.unmount();
    host.dispose();
  });

  it("uses empty title and zero size defaults for root surface creation", async () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const source = host.createRoot({ surfaceId: 3 });
    source.render(null);

    const opened = source.openSurface();
    expect(message(transport, 1)[8]).toEqual(["", [0, 0]]);
    transport.push(commandResult(3, 1, 1, 1, COMMAND_OPEN_SURFACE, [1, 4]));
    await expect(opened).resolves.toBe(4);
    source.unmount();
    host.dispose();
  });

  it("demultiplexes command results to isolated roots", async () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const first = host.createRoot({ surfaceId: 11 });
    const second = host.createRoot({ surfaceId: 12 });
    first.render(null);
    second.render(null);

    const firstResult = first.setTitle("first");
    const secondResult = second.setTitle("second");
    const firstEvent = commandResult(11, 1, 1, 1, 6);
    const secondEvent = commandResult(12, 1, 1, 1, 6);
    transport.push(new Uint8Array([...secondEvent, ...firstEvent]));

    await expect(firstResult).resolves.toBeUndefined();
    await expect(secondResult).resolves.toBeUndefined();
    first.unmount();
    second.unmount();
    host.dispose();
  });
  it("routes surface closure to only the matching root", () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const closed: number[] = [];
    const first = host.createRoot({ surfaceId: 21, onClose: () => closed.push(21) });
    const second = host.createRoot({ surfaceId: 22, onClose: () => closed.push(22) });
    first.render(null);
    second.render(null);

    transport.push(surfaceClosed(21, 1, 1));

    expect(closed).toEqual([21]);
    expect(() => first.render(null)).toThrow(SurfaceClosedError);
    expect(() => second.render(null)).not.toThrow();
    second.unmount();
    host.dispose();
  });
  it("rejects explicit registration of a natively closed surface id", () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const root = host.createRoot({ surfaceId: 61 });
    root.render(null);

    transport.push(surfaceClosed(61, 1, 1));

    expect(() => host.createRoot({ surfaceId: 61, epoch: 2 })).toThrow(SurfaceIdReusedError);
    expect(() => host.createRoot({ surfaceId: 61 })).toThrow("surface 61 was already closed");
    const replacement = host.createRoot();
    expect(replacement).toBeDefined();
    replacement.unmount();
    host.dispose();
  });

  it("retires explicitly unmounted surface ids", () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const root = host.createRoot({ surfaceId: 62 });
    root.render(null);
    root.unmount();

    expect(() => host.createRoot({ surfaceId: 62, epoch: 2 })).toThrow(SurfaceIdReusedError);
    host.dispose();
  });
  it("rejects pending commands when one surface closes", async () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const root = host.createRoot({ surfaceId: 23 });
    root.render(null);
    const pending = root.pickFiles().catch((error: unknown) => error);

    transport.push(surfaceClosed(23, 1, 1));

    await expect(pending).resolves.toBeInstanceOf(SurfaceClosedError);
    await expect(pending).resolves.toMatchObject({ message: "surface 23 is closed", surfaceId: 23 });
    host.dispose();
  });

  it("rejects pending root commands on shared termination", async () => {
    const transport = new TerminatingTransport();
    const host = createSurfaceHost(transport);
    const first = host.createRoot({ surfaceId: 31 });
    const second = host.createRoot({ surfaceId: 32 });
    first.render(null);
    second.render(null);
    const firstPending = first.setTitle("first").catch((error: unknown) => error);
    const secondPending = second.pickSavePath({ defaultName: "second.txt" }).catch((error: unknown) => error);

    transport.terminate();

    await expect(firstPending).resolves.toBeInstanceOf(TransportTerminatedError);
    await expect(secondPending).resolves.toBeInstanceOf(TransportTerminatedError);
    host.dispose();
  });
  it("fails the shared host on a malformed event frame", async () => {
    const transport = new MemoryTransport();
    const terminations: TransportTerminatedError[] = [];
    const host = createSurfaceHost(transport, {
      onTransportTermination: (error) => terminations.push(error),
    });
    const first = host.createRoot({ surfaceId: 51 });
    const second = host.createRoot({ surfaceId: 52 });
    first.render(null);
    second.render(null);
    const pending = [first.setTitle("first"), second.pickSavePath({ defaultName: "second.txt" })];
    const valid = encodeFrame([PROTOCOL_VERSION, 2, 51, 1, 1, 1, 0, 0, EVENT_PRESS, null]);
    const malformed = framePayload(new Uint8Array([0xd9, 1, 0xff]));
    transport.push(new Uint8Array([...valid, ...malformed, ...valid]));

    const errors = await Promise.all(pending.map((command) => command.catch((error: unknown) => error)));
    expect(terminations).toHaveLength(1);
    const termination = terminations[0]!;
    expect(termination.cause).toEqual({ kind: "protocol", detail: "received malformed event frame" });
    expect(errors).toEqual([termination, termination]);
    await expect(first.setTitle("after")).rejects.toBe(termination);
    host.dispose();
  });

  it("fails the shared host on a version mismatch with actionable diagnostics", async () => {
    const transport = new MemoryTransport();
    const terminations: TransportTerminatedError[] = [];
    const host = createSurfaceHost(transport, {
      onTransportTermination: (error) => terminations.push(error),
    });
    const root = host.createRoot({ surfaceId: 53 });
    root.render(null);
    const pending = root.setTitle("pending");
    const version = PROTOCOL_VERSION + 1;
    const mismatched = encodeFrame([version, 2, 53, 1, 1, 1, 0, 0, EVENT_PRESS, null] as never);
    transport.push(mismatched);

    const termination = await pending.catch((error: unknown) => {
      if (!(error instanceof TransportTerminatedError)) throw error;
      return error;
    });
    if (!(termination instanceof TransportTerminatedError)) throw new Error("expected transport termination");
    expect(terminations).toEqual([termination]);
    const detail = termination.cause?.kind === "protocol" ? termination.cause.detail : "";
    expect(detail).toContain(`protocol v${version}`);
    expect(detail).toContain(`protocol v${PROTOCOL_VERSION}`);
    root.unmount();
    host.dispose();
  });
  it("rejects every root's pending command on host disposal", async () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const first = host.createRoot({ surfaceId: 41 });
    const second = host.createRoot({ surfaceId: 42 });
    first.render(null);
    second.render(null);
    const firstPending = first.pickFiles().catch((error: unknown) => error);
    const secondPending = second.pickSavePath().catch((error: unknown) => error);

    host.dispose();

    await expect(firstPending).resolves.toMatchObject({ message: "SurfaceHost is disposed" });
    await expect(secondPending).resolves.toMatchObject({ message: "SurfaceHost is disposed" });
  });
});
