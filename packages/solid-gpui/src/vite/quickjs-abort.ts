/*!
 * Adapted from @edge-runtime/primitives 6.0.0, commit
 * 440c123a37284d6a852ce453af810ad484ecfc01:
 * https://github.com/vercel/edge-runtime/blob/440c123a37284d6a852ce453af810ad484ecfc01/packages/primitives/src/primitives/abort-controller.js
 * Changes: explicit platform imports and types; use the upstream event shim's
 * event attribute handling and core-js DOMException; validate timer bounds.
 *
 * The MIT License (MIT)
 * Copyright (c) 2024 Vercel, Inc.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
import DOMException from "core-js-pure/actual/dom-exception";
import { Event, EventTarget, getEventAttributeValue, setEventAttributeValue } from "event-target-shim";

const kSignal = Symbol("kSignal");
const kAborted = Symbol("kAborted");
const kReason = Symbol("kReason");

function createAbortSignal(): AbortSignal {
  const signal = Object.setPrototypeOf(new EventTarget(), AbortSignal.prototype) as AbortSignal;
  signal[kAborted] = false;
  signal[kReason] = undefined;
  return signal;
}

function abortSignalAbort(signal: AbortSignal, reason?: unknown): void {
  if (signal.aborted) return;
  signal[kReason] = reason === undefined ? new DOMException("This operation was aborted", "AbortError") : reason;
  signal[kAborted] = true;
  signal.dispatchEvent(new Event("abort"));
}

export class AbortController {
  readonly [kSignal] = createAbortSignal();

  get signal(): AbortSignal {
    return this[kSignal];
  }

  abort(reason?: unknown): void {
    abortSignalAbort(this.signal, reason);
  }
}

export class AbortSignal extends EventTarget {
  declare [kAborted]: boolean;
  declare [kReason]: unknown;

  constructor() {
    super();
    throw new TypeError("Illegal constructor");
  }

  get aborted(): boolean {
    return this[kAborted];
  }

  get reason(): unknown {
    return this[kReason];
  }

  get onabort(): ((event: Event) => void) | null {
    return getEventAttributeValue(this, "abort");
  }

  set onabort(value: ((event: Event) => void) | null) {
    setEventAttributeValue(this, "abort", value);
  }

  throwIfAborted(): void {
    if (this[kAborted]) throw this[kReason];
  }

  static abort(reason?: unknown): AbortSignal {
    const signal = createAbortSignal();
    abortSignalAbort(signal, reason);
    return signal;
  }

  static timeout(milliseconds: number): AbortSignal {
    if (!Number.isSafeInteger(milliseconds) || milliseconds < 0 || milliseconds > 2 ** 31 - 1) {
      throw new RangeError("AbortSignal.timeout requires an integer between 0 and 2147483647 milliseconds");
    }
    const signal = createAbortSignal();
    setTimeout(() => {
      abortSignalAbort(signal, new DOMException("The operation was aborted due to timeout", "TimeoutError"));
    }, milliseconds);
    return signal;
  }
}
