import { decodeJson, encodeJson } from "./native";
import {
  COMMAND_INVOKE_NATIVE,
  FrameDecoder,
  PROTOCOL_VERSION,
  encodeFrame,
  utf8ByteLength,
  type EventPayload,
} from "./protocol";
import { boundedBebopDecode } from "./protocol/bebop-guard";
import { Envelope, NodeKind, type Command, type Node as WireNode } from "./protocol/generated/protocol";
import { MemoryTransport } from "./transport";
import { TestTree, required } from "./testing/tree";
import type { TestCommit, TestEvent, TestNativeCall, TestNode, TestSurface } from "./testing/types";

export type { TestCommit, TestEvent, TestNativeCall, TestNode, TestSurface } from "./testing/types";

interface Target {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
  readonly node: WireNode;
}

/**
 * Inspect real renderer commits and deliver events through a MemoryTransport.
 * Reads consume already-submitted frames; they do not advance timers or wait for application work.
 * This is not a GPUI layout, paint or platform-service emulator. Unmount your roots after each test.
 */
export class TestHost {
  private readonly decoder = new FrameDecoder();
  private readonly trees = new Map<number, TestTree>();
  private readonly targets = new WeakMap<TestNode, Target>();
  private readonly requests = new WeakMap<TestNativeCall, Command>();
  private readonly sequences = new Map<string, number>();
  private readonly edits = new Map<string, number>();
  private readonly history: TestCommit[] = [];
  private readonly calls: TestNativeCall[] = [];
  private cursor = 0;

  constructor(readonly transport: MemoryTransport = new MemoryTransport()) {}

  /** Snapshot/Patch history in submission order; reading also replays pending commits. */
  get commits(): readonly TestCommit[] {
    this.read();
    return Object.freeze([...this.history]);
  }

  /** All observed InvokeNative requests, including already answered/cancelled calls. */
  get nativeCalls(): readonly TestNativeCall[] {
    this.read();
    return Object.freeze([...this.calls]);
  }

  /** A detached tree view, or undefined before this Surface's first Snapshot. */
  surface(surfaceId: number): TestSurface | undefined {
    this.read();
    const tree = this.trees.get(surfaceId);
    return tree?.view((view, node) => {
      this.targets.set(view, { surfaceId, epoch: tree.epoch, revision: tree.revision, node });
    });
  }

  /** Decode the JSON props of a component created by createNativeComponent. */
  nativeProps(node: TestNode): unknown {
    const properties = this.target(node).node.hostProperties;
    const field = properties?.tag === 5 ? properties.value.fields?.find((entry) => entry.id === 1) : undefined;
    if (field?.value?.tag !== 6) throw new TypeError("TestNode does not contain native JSON props");
    return decodeJson(required(field.value.value.value, "native props bytes"));
  }

  /** Deliver to the captured revision, preserving the renderer's stale-listener/epoch checks. */
  dispatch(node: TestNode, event: TestEvent): void {
    const target = this.target(node);
    const listenerId = target.node.listenerId ?? 0;
    if (listenerId === 0) throw new Error("TestNode has no event listener");
    let payload: EventPayload;
    switch (event.type) {
      case "input": {
        if (target.node.kind !== NodeKind.TextInput) throw new TypeError("Input events require a TextInput node");
        const key = `${target.surfaceId}:${target.epoch}:${node.id}`;
        const input = target.node.hostProperties;
        const acknowledged = input?.tag === 1 ? (input.value.ackEditSeq ?? 0) : 0;
        const editSeq = Math.max(this.edits.get(key) ?? 0, acknowledged) + 1;
        this.edits.set(key, editSeq);
        const start = event.selectionStart ?? utf8ByteLength(event.text);
        payload = {
          type: "change",
          data: {
            text: event.text,
            selectionStart: start,
            selectionEnd: event.selectionEnd ?? start,
            markedStart: null,
            markedEnd: null,
            editSeq,
            reversed: false,
          },
        };
        break;
      }
      case "native": {
        const properties = target.node.hostProperties;
        if (properties?.tag !== 5 || !properties.value.eventIds?.includes(event.eventId))
          throw new Error("TestNode is not subscribed to this native event");
        payload = {
          type: "extension",
          eventId: event.eventId,
          fields: [{ id: 1, value: { type: "bytes", value: encodeJson(event.value) } }],
        };
        break;
      }
      default:
        payload = { type: event.type };
    }
    this.send(target.surfaceId, target.epoch, target.revision, node.id, listenerId, payload);
  }

  /** Reply with opaque bytes; generated JSON DTO clients use encodeJson(value). */
  reply(call: TestNativeCall, value: Uint8Array): void {
    this.respond(call, { type: "bytes", value }, null);
  }

  /** Reject one request with the host's domain-error message. */
  reject(call: TestNativeCall, error: string): void {
    this.respond(call, null, error);
  }

  private target(node: TestNode): Target {
    const target = this.targets.get(node);
    if (!target) throw new TypeError("Use a TestNode obtained from this TestHost.surface()");
    return target;
  }

  private respond(
    call: TestNativeCall,
    value: { type: "bytes"; value: Uint8Array } | null,
    error: string | null,
  ): void {
    const command = this.requests.get(call);
    if (!command) throw new Error("Native call belongs to another TestHost or has already been answered");
    this.send(call.surfaceId, call.epoch, required(command.afterRevision, "command revision"), call.nodeId, 0, {
      type: "command-result",
      result: {
        requestId: call.requestId,
        command: COMMAND_INVOKE_NATIVE,
        nodeId: call.nodeId,
        success: error === null,
        error,
        value,
      },
    });
    this.requests.delete(call);
  }

  private send(
    surfaceId: number,
    epoch: number,
    revision: number,
    nodeId: number,
    listenerId: number,
    payload: EventPayload,
  ): void {
    const key = `${surfaceId}:${epoch}`;
    const sequence = (this.sequences.get(key) ?? 0) + 1;
    const frame = encodeFrame({ type: "event", surfaceId, epoch, revision, sequence, nodeId, listenerId, payload });
    this.sequences.set(key, sequence);
    this.transport.push(frame);
  }

  private read(): void {
    if (this.cursor > this.transport.submitted.length) throw new Error("Do not clear TestHost.transport.submitted");
    while (this.cursor < this.transport.submitted.length) {
      for (const payload of this.decoder.push(this.transport.submitted[this.cursor]!)) {
        boundedBebopDecode(payload);
        const envelope = Envelope.decode(payload);
        if (envelope.protocolVersion !== PROTOCOL_VERSION) throw new Error("TestHost protocol version mismatch");
        const body = required(envelope.body, "body");
        if (body.tag === 1 || body.tag === 3) {
          const surfaceId = required(body.value.surfaceId, "surfaceId");
          if (body.tag === 1) this.trees.set(surfaceId, new TestTree(body.value));
          else required(this.trees.get(surfaceId), "Snapshot before Patch").apply(body.value);
          const tree = this.trees.get(surfaceId)!;
          this.history.push(
            Object.freeze({
              type: body.tag === 1 ? "snapshot" : "patch",
              surfaceId,
              epoch: tree.epoch,
              revision: tree.revision,
            }),
          );
        } else if (body.tag === 4) {
          const command = body.value;
          if (command.kind !== COMMAND_INVOKE_NATIVE) continue;
          if (command.payload?.tag !== 12) throw new Error("Native call is missing its payload");
          const value = command.payload.value;
          const call: TestNativeCall = Object.freeze({
            surfaceId: required(command.surfaceId, "surfaceId"),
            epoch: required(command.epoch, "epoch"),
            nodeId: required(command.nodeId, "nodeId"),
            requestId: required(command.requestId, "requestId"),
            moduleId: Object.freeze([...required(value.moduleId, "moduleId")]),
            moduleDigest: Object.freeze([...required(value.moduleDigest, "moduleDigest")]),
            functionId: required(value.functionId, "functionId"),
            args: required(value.args, "native arguments").slice(),
          });
          this.calls.push(call);
          this.requests.set(call, command);
        } else {
          throw new Error("TestHost received an event on the renderer's outbound stream");
        }
      }
      this.cursor++;
    }
  }
}
