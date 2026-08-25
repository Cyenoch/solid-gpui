import { createElement } from "react";
import { expect, it } from "bun:test";
import { decode } from "@msgpack/msgpack";
import type { Root, Transport, TransportChunk, TransportListener, TransportTerminationListener } from "../src/index";
import {
  COMMAND_GET_WINDOW_SIZE,
  EVENT_PRESS,
  EVENT_SCROLL,
  MAX_FRAME_SIZE,
  PROTOCOL_VERSION,
  SCROLL_DELTA_PIXELS,
  encodeFrame,
} from "../src/protocol";
import { MemoryTransport, Pressable, Text, View, createRoot } from "../src/index";
const ITERATIONS = 50_000;
const SURFACES = 10;
const ITERATIONS_PER_SURFACE = ITERATIONS / SURFACES;
const EPOCH = 1;
const FRAME_OVERHEAD = 4;

type WireMessage = readonly unknown[];

class CountingTransport implements Transport {
  readonly inner = new MemoryTransport();
  readonly submitted = this.inner.submitted;
  activeDataListeners = 0;
  activeTerminationListeners = 0;

  submit(frame: Uint8Array): void {
    this.inner.submit(frame);
  }

  onData(listener: TransportListener): () => void {
    this.activeDataListeners += 1;
    const unsubscribe = this.inner.onData(listener);
    let active = true;
    return () => {
      if (!active) return;
      active = false;
      this.activeDataListeners -= 1;
      unsubscribe();
    };
  }

  onTermination(listener: TransportTerminationListener): () => void {
    this.activeTerminationListeners += 1;
    const unsubscribe = this.inner.onTermination(listener);
    let active = true;
    return () => {
      if (!active) return;
      active = false;
      this.activeTerminationListeners -= 1;
      unsubscribe();
    };
  }

  push(chunk: TransportChunk): void {
    this.inner.push(chunk);
  }
}

function message(frame: Uint8Array): WireMessage {
  return decode(frame.subarray(FRAME_OVERHEAD)) as WireMessage;
}

function eventFrame(
  surfaceId: number,
  revision: number,
  sequence: number,
  nodeId: number,
  listenerId: number,
  eventType: number,
  payload: unknown,
): Uint8Array {
  return encodeFrame([
    PROTOCOL_VERSION,
    2,
    surfaceId,
    EPOCH,
    revision,
    sequence,
    nodeId,
    listenerId,
    eventType,
    payload,
  ] as never);
}

function renderTree(value: number, onPress: () => void, onScroll: () => void) {
  return createElement(
    View,
    { onScroll },
    createElement(Pressable, { onPress }, createElement(Text, null, String(value))),
  );
}

it("keeps 50k snapshot/patch, event, and command cycles stable", async () => {
  const transport = new CountingTransport();
  let pressCount = 0;
  let scrollCount = 0;
  let lastSurfaceId = 0;
  let settledCommands = 0;
  let inFlightCommands = 0;
  let commitFrames = 0;
  let frameViolations = 0;
  let protocolErrors = 0;
  let sequenceErrors = 0;
  let listenerErrors = 0;
  let globalIteration = 0;
  const started = performance.now();

  for (let surfaceIndex = 0; surfaceIndex < SURFACES; surfaceIndex += 1) {
    const root = createRoot(transport, { epoch: EPOCH });
    expect(transport.activeDataListeners).toBe(1);
    expect(transport.activeTerminationListeners).toBe(1);
    root.render(
      renderTree(
        globalIteration,
        () => {
          pressCount += 1;
        },
        () => {
          scrollCount += 1;
        },
      ),
    );
    commitFrames += 1;
    const snapshot = message(transport.submitted[transport.submitted.length - 1]);
    if (snapshot[0] !== PROTOCOL_VERSION || snapshot[1] !== 1 || snapshot[5] !== 1) protocolErrors += 1;
    const surfaceId = Number(snapshot[2]);
    const nodes = snapshot[6] as readonly (readonly unknown[])[];
    const viewNode = nodes.find((node) => node[3] === 1 && Number(node[6]) > 0);
    const pressableNode = nodes.find((node) => node[3] === 3);
    const viewNodeId = Number(viewNode?.[0]);
    const viewListenerId = Number(viewNode?.[6]);
    const pressableNodeId = Number(pressableNode?.[0]);
    const pressableListenerId = Number(pressableNode?.[6]);
    if (!viewNode || !pressableNode || viewListenerId === 0 || pressableListenerId === 0) protocolErrors += 1;
    if (surfaceId <= lastSurfaceId) sequenceErrors += 1;
    lastSurfaceId = surfaceId;
    let eventSequence = 0;

    for (let offset = 0; offset < ITERATIONS_PER_SURFACE; offset += 1) {
      const submittedBefore = transport.submitted.length;
      globalIteration += 1;
      root.render(
        renderTree(
          globalIteration,
          () => {
            pressCount += 1;
          },
          () => {
            scrollCount += 1;
          },
        ),
      );
      commitFrames += 1;
      if (offset === 0 || offset === ITERATIONS_PER_SURFACE - 1) {
        const patch = message(transport.submitted[transport.submitted.length - 1]);
        if (patch[0] !== PROTOCOL_VERSION || patch[1] !== 3 || patch[2] !== surfaceId) protocolErrors += 1;
      }

      eventSequence += 1;
      transport.push(
        eventFrame(surfaceId, offset + 2, eventSequence, pressableNodeId, pressableListenerId, EVENT_PRESS, null),
      );
      eventSequence += 1;
      transport.push(
        eventFrame(surfaceId, offset + 2, eventSequence, viewNodeId, viewListenerId, EVENT_SCROLL, [
          7,
          SCROLL_DELTA_PIXELS,
          1,
          -1,
          0,
          0,
          [],
        ]),
      );

      inFlightCommands += 1;
      const requestId = offset + 1;
      const sizePromise = root.getWindowSize();
      if (offset === 0) {
        const command = message(transport.submitted[transport.submitted.length - 1]);
        if (
          command[1] !== 4 ||
          command[2] !== surfaceId ||
          command[5] !== requestId ||
          command[7] !== COMMAND_GET_WINDOW_SIZE
        )
          protocolErrors += 1;
      }
      eventSequence += 1;
      transport.push(
        eventFrame(surfaceId, offset + 2, eventSequence, 1, 0, 6, [
          2,
          requestId,
          COMMAND_GET_WINDOW_SIZE,
          1,
          true,
          null,
          [2, [800, 600]],
        ]),
      );
      await expect(sizePromise).resolves.toEqual([800, 600]);
      inFlightCommands -= 1;
      settledCommands += 1;

      for (const frame of transport.submitted.slice(submittedBefore)) {
        if (frame.byteLength > MAX_FRAME_SIZE + FRAME_OVERHEAD) frameViolations += 1;
      }
    }

    root.unmount();
    commitFrames += 1;
    if (transport.activeDataListeners !== 0 || transport.activeTerminationListeners !== 0) listenerErrors += 1;
  }

  const elapsedMs = performance.now() - started;
  expect(elapsedMs).toBeLessThan(30_000);
  expect(settledCommands).toBe(ITERATIONS);
  expect(inFlightCommands).toBe(0);
  expect(pressCount).toBe(ITERATIONS);
  expect(scrollCount).toBe(ITERATIONS);
  expect(commitFrames).toBe(ITERATIONS + SURFACES * 2);
  expect(frameViolations).toBe(0);
  expect(protocolErrors).toBe(0);
  expect(sequenceErrors).toBe(0);
  expect(listenerErrors).toBe(0);
  expect(transport.submitted).toHaveLength(commitFrames + ITERATIONS);
  expect(transport.activeDataListeners).toBe(0);
  expect(transport.activeTerminationListeners).toBe(0);
});
