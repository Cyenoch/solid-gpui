import {
  COMMAND_INVOKE_NATIVE,
  MAX_NATIVE_CALL_BYTES,
  encodeFrame,
  type Command,
  type CommandResult,
  type CommandValue,
} from "../protocol";
import { TransportTerminatedError } from "../transport";
import type { PendingCommand } from "./types";

export interface CommandFrameSink {
  getTerminationError(): TransportTerminatedError | undefined;
  submitFrame(frame: Uint8Array): boolean;
}
export class CommandClient {
  private readonly pending = new Map<number, PendingCommand>();
  private nextRequest = 1;

  constructor(private readonly sink: CommandFrameSink) {}

  allocateRequestId(nextU32: (value: number, name: string) => number): number {
    const requestId = this.nextRequest;
    this.nextRequest = nextU32(this.nextRequest, "command request id");
    return requestId;
  }

  submit(command: Command): Promise<CommandValue | null> {
    const requestId = command.requestId;
    return new Promise<CommandValue | null>((resolve, reject) => {
      this.pending.set(requestId, { command: command.command, nodeId: command.nodeId, resolve, reject });
      try {
        if (!this.sink.submitFrame(encodeFrame(command))) {
          this.pending.delete(requestId);
          reject(this.sink.getTerminationError() ?? new TransportTerminatedError("transport is terminated"));
        }
      } catch (error) {
        this.pending.delete(requestId);
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  resolve(result: CommandResult): void {
    const pending = this.pending.get(result.requestId);
    if (pending === undefined) return;
    this.pending.delete(result.requestId);
    if (result.command !== pending.command || result.nodeId !== pending.nodeId) {
      pending.reject(new Error("native command response does not match the pending command"));
      return;
    }
    if (
      (result.value?.type === "bytes" &&
        (pending.command !== COMMAND_INVOKE_NATIVE ||
          !result.success ||
          !(result.value.value instanceof Uint8Array) ||
          result.value.value.byteLength > MAX_NATIVE_CALL_BYTES)) ||
      (pending.command === COMMAND_INVOKE_NATIVE && result.success && result.value?.type !== "bytes")
    ) {
      pending.reject(new Error("native command returned an invalid byte result"));
      return;
    }
    if (result.success) pending.resolve(result.value);
    else pending.reject(new Error(String(result.error ?? "native command failed")));
  }

  rejectAll(error: Error): void {
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
  }
}
