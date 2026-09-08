import { expect, test } from "bun:test";
import { MemoryTransport, TransportTerminatedError } from "../src/transport";
import { SurfaceRouter } from "../src/renderer/surface-router";
import { encodeFrame, MAX_FRAME_SIZE } from "../src/protocol";
import { mountApplication, Text } from "../src/index";
import { createComponent } from "../src/runtime";
import { encodeGenerationState } from "../src/generation";

class SlowTransport extends MemoryTransport {
  private drain = () => {};
  writable = false;
  override submit(frame: Uint8Array): boolean {
    super.submit(frame);
    return this.writable;
  }
  override onDrain(listener: () => void) {
    this.drain = listener;
    return () => {
      this.drain = () => {};
    };
  }
  resume() {
    this.writable = true;
    this.drain();
  }
}

test("pressure pauses event production, sends accepted frames once, and bounds staging", () => {
  const transport = new SlowTransport();
  let terminated: TransportTerminatedError | undefined;
  const router = new SurfaceRouter(transport, {
    onTermination: (error) => {
      terminated = error;
    },
  });
  let delivered = 0;
  router.register(1, {
    deliver: () => {
      delivered++;
      router.submit(new Uint8Array([3]));
    },
    terminate: () => {},
  });
  router.start();
  router.submit(new Uint8Array([1]));
  router.submit(new Uint8Array([2]));
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 1,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: 1,
      listenerId: 1,
      payload: { type: "press" },
    }),
  );
  expect(delivered).toBe(0);
  expect(transport.submitted.map((f) => [...f])).toEqual([[1]]);
  transport.resume();
  expect(delivered).toBe(1);
  expect(transport.submitted.map((f) => [...f])).toEqual([[1], [2], [3]]);
  transport.writable = false;
  router.submit(new Uint8Array([4]));
  expect(() => router.submit(new Uint8Array(MAX_FRAME_SIZE + 5))).toThrow("capacity");
  expect(terminated).toBeInstanceOf(TransportTerminatedError);
  router.dispose();
});

test("generation handoff rejects values JSON would silently lose or coerce", () => {
  expect(() => encodeGenerationState({ state: [{ session: { userId: undefined } }] })).toThrow(
    "reload state at $.state[0].session.userId: undefined is not JSON; omit the property or use null",
  );
  expect(JSON.parse(encodeGenerationState({ state: [null], text: "你好🌍", count: 3 }))).toEqual({
    state: [null],
    text: "你好🌍",
    count: 3,
  });
  const cycle: unknown[] = [];
  cycle.push(cycle);
  for (const value of [
    undefined,
    NaN,
    Infinity,
    -0,
    new Date(),
    { bad: undefined },
    [, ,],
    Object.assign(new Array(1), { extra: "not an array element" }),
    cycle,
    {
      get value() {
        throw new Error("getter executed");
      },
    },
  ]) {
    expect(() => encodeGenerationState(value)).toThrow();
  }
});

test("application activation drains its staged snapshot before later configuration", () => {
  const transport = new SlowTransport();
  const application = mountApplication({
    transport: () => transport,
    setup: () => ({ render: () => createComponent(Text, { children: "Pressure" }) }),
  });
  expect(transport.submitted.length).toBe(1);
  transport.resume();
  expect(transport.submitted.length).toBe(2);
  application.dispose();
});
