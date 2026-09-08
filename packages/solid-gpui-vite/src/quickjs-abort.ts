/*!
 * Adapted from @edge-runtime/primitives 6.0.0, commit
 * 440c123a37284d6a852ce453af810ad484ecfc01:
 * https://github.com/vercel/edge-runtime/blob/440c123a37284d6a852ce453af810ad484ecfc01/packages/primitives/src/primitives/abort-controller.js
 * Changes: explicit platform imports and types; use the upstream event shim's
 * event attribute handling and core-js DOMException; validate timer bounds;
 * compose cancellation through an internal dependency graph.
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

interface SignalState {
  aborted: boolean;
  reason: unknown;
  sources?: Set<WeakRef<AbortSignal>>;
  dependents: Set<AbortSignal>;
}

const states = new WeakMap<AbortSignal, SignalState>();

function stateOf(signal: AbortSignal): SignalState {
  const state = states.get(signal);
  if (!state) throw new TypeError("Expected a QuickJS AbortSignal");
  return state;
}

function createAbortSignal(): AbortSignal {
  const signal = Object.setPrototypeOf(new EventTarget(), AbortSignal.prototype) as AbortSignal;
  states.set(signal, { aborted: false, reason: undefined, dependents: new Set() });
  return signal;
}

function abortSignalAbort(signal: AbortSignal, reason?: unknown): void {
  const state = stateOf(signal);
  if (state.aborted) return;
  reason = reason === undefined ? new DOMException("This operation was aborted", "AbortError") : reason;
  const pending = [signal, ...state.dependents];
  // All combinations point directly to their original sources. Commit every
  // reason before user callbacks can abort another source reentrantly.
  for (const target of pending) {
    const targetState = stateOf(target);
    targetState.reason = reason;
    targetState.aborted = true;
    for (const reference of targetState.sources ?? []) {
      const source = reference.deref();
      if (source) stateOf(source).dependents.delete(target);
    }
    targetState.sources?.clear();
    targetState.dependents.clear();
  }
  for (const target of pending) EventTarget.prototype.dispatchEvent.call(target, new Event("abort"));
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
  constructor() {
    super();
    throw new TypeError("Illegal constructor");
  }

  get aborted(): boolean {
    return stateOf(this).aborted;
  }

  get reason(): unknown {
    return stateOf(this).reason;
  }

  get onabort(): ((event: Event) => void) | null {
    return getEventAttributeValue(this, "abort");
  }

  set onabort(value: ((event: Event) => void) | null) {
    setEventAttributeValue(this, "abort", value);
  }

  throwIfAborted(): void {
    const state = stateOf(this);
    if (state.aborted) throw state.reason;
  }

  static abort(reason?: unknown): AbortSignal {
    const signal = createAbortSignal();
    abortSignalAbort(signal, reason);
    return signal;
  }

  static any(signals: Iterable<AbortSignal>): AbortSignal {
    // Consume and validate the complete sequence before observing cancellation.
    const inputs: AbortSignal[] = [];
    for (const signal of signals) {
      stateOf(signal);
      inputs.push(signal);
    }
    const result = createAbortSignal();
    for (const signal of inputs) {
      const state = stateOf(signal);
      if (state.aborted) {
        abortSignalAbort(result, state.reason);
        return result;
      }
    }
    const sources = new Set<AbortSignal>();
    for (const signal of inputs) {
      const state = stateOf(signal);
      if (state.sources === undefined) sources.add(signal);
      else {
        for (const reference of state.sources) {
          const source = reference.deref();
          if (source) sources.add(source);
        }
      }
    }
    const references = new Set<WeakRef<AbortSignal>>();
    stateOf(result).sources = references;
    for (const source of sources) {
      // Sources own pending combinations; cancellation unlinks them from every
      // source. Reverse links must not keep an otherwise dead controller alive.
      stateOf(source).dependents.add(result);
      references.add(new WeakRef(source));
    }
    return result;
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
