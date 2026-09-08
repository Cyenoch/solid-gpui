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
import { NativeCommandError, validateCallOptions, type NativeCallOptions } from "../native-call";

export interface CommandFrameSink {
  getTerminationError(): TransportTerminatedError | undefined;
  submitFrame(frame: Uint8Array): boolean;
  cancelNative(requestId: number): void;
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

  submit(command: Command, options?: NativeCallOptions): Promise<CommandValue | null> {
    const requestId = command.requestId;
    return new Promise<CommandValue | null>((resolve, reject) => {
      validateCallOptions(options);
      let timeout: ReturnType<typeof setTimeout> | undefined;
      let submitted = false;
      let cancelled = false;
      const signal = options?.signal;
      const cleanup = () => {
        if (timeout !== undefined) clearTimeout(timeout);
        signal?.removeEventListener("abort", onAbort);
      };
      const cancel = (reason: unknown) => {
        if (!this.pending.delete(requestId)) return;
        cancelled = true;
        cleanup();
        reject(reason);
        if (submitted) this.sink.cancelNative(requestId);
      };
      const onAbort = () => cancel(signal!.reason);
      this.pending.set(requestId, {
        command: command.command,
        nodeId: command.nodeId,
        resolve,
        reject,
        cleanup,
        identity: {
          surfaceId: command.surfaceId,
          epoch: command.epoch,
          requestId,
          nodeId: command.nodeId,
          command: command.command,
          ...(command.payload?.type === "invoke-native" ? { functionId: command.payload.functionId } : {}),
        },
      });
      signal?.addEventListener("abort", onAbort, { once: true });
      if (options?.timeoutMs !== undefined)
        timeout = setTimeout(
          () => cancel(new DOMException("Native command timed out", "TimeoutError")),
          options.timeoutMs,
        );
      try {
        submitted = this.sink.submitFrame(encodeFrame(command));
        if (!submitted) {
          this.pending.delete(requestId);
          cleanup();
          reject(this.sink.getTerminationError() ?? new TransportTerminatedError("transport is terminated"));
        } else if (cancelled) {
          // Never let reentrant cancellation overtake its original request.
          this.sink.cancelNative(requestId);
        }
      } catch (error) {
        this.pending.delete(requestId);
        cleanup();
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  resolve(result: CommandResult): void {
    const pending = this.pending.get(result.requestId);
    if (pending === undefined) return;
    this.pending.delete(result.requestId);
    pending.cleanup();
    if (result.command !== pending.command || result.nodeId !== pending.nodeId) {
      pending.reject(
        new NativeCommandError("native command response does not match the pending command", pending.identity),
      );
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
      pending.reject(new NativeCommandError("native command returned an invalid byte result", pending.identity));
      return;
    }
    if (result.success) pending.resolve(result.value);
    else pending.reject(new NativeCommandError(String(result.error ?? "native command failed"), pending.identity));
  }

  rejectAll(error: Error): void {
    for (const pending of this.pending.values()) {
      pending.cleanup();
      pending.reject(error);
    }
    this.pending.clear();
  }
}
