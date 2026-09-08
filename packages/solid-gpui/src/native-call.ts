/** Cancellation belongs to one invocation, independently of its Surface. */
export interface NativeCallOptions {
  readonly signal?: AbortSignal;
  readonly timeoutMs?: number;
}

export function validateCallOptions(options: NativeCallOptions | undefined): void {
  const timeout = options?.timeoutMs;
  if (timeout !== undefined && (!Number.isInteger(timeout) || timeout < 0 || timeout > 2147483647))
    throw new RangeError("Native timeout must be an integer between 0 and 2147483647 milliseconds");
  options?.signal?.throwIfAborted();
}

/** Request identity is safe to record without retaining command arguments. */
export interface NativeCommandIdentity {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly requestId: number;
  readonly nodeId: number;
  readonly command: number;
  readonly functionId?: number;
}

export class NativeCommandError extends Error {
  readonly name = "NativeCommandError";
  constructor(
    message: string,
    readonly identity: NativeCommandIdentity,
  ) {
    super(message);
    Object.freeze(identity);
  }
}
