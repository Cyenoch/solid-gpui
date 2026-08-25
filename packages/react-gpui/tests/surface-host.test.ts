import { decode } from "@msgpack/msgpack";
import { describe, expect, it } from "bun:test";

import {
  COMMAND_OPEN_SURFACE,
  EVENT_COMMAND_RESULT,
  EVENT_SURFACE_CLOSED,
  PROTOCOL_VERSION,
  encodeFrame,
} from "../src/protocol";
import { MemoryTransport, TransportTerminatedError, type TransportTerminationListener } from "../src/transport";
import { createSurfaceHost } from "../src/surface-host";

class TerminatingTransport extends MemoryTransport {
  private readonly terminationListeners = new Set<TransportTerminationListener>();

  override onTermination(listener: TransportTerminationListener): () => void {
    this.terminationListeners.add(listener);
    return () => this.terminationListeners.delete(listener);
  }

  terminate(): void {
    const error = new TransportTerminatedError("test transport terminated");
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
    value === null ? [2, requestId, command, 1, true, null] : [2, requestId, command, 1, true, null, value];
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

    const root = host.createRoot({ surfaceId: 41 });
    root.render(null);
    expect(message(transport, 2)[2]).toBe(41);
    root.unmount();
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
    expect(() => first.render(null)).toThrow("unmounted root");
    expect(() => second.render(null)).not.toThrow();
    second.unmount();
    host.dispose();
  });
  it("rejects pending commands when one surface closes", async () => {
    const transport = new MemoryTransport();
    const host = createSurfaceHost(transport);
    const root = host.createRoot({ surfaceId: 23 });
    root.render(null);
    const pending = root.pickFiles().catch((error: unknown) => error);

    transport.push(surfaceClosed(23, 1, 1));

    await expect(pending).resolves.toMatchObject({ message: "root is unmounted" });
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
