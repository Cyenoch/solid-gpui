import { untrack } from "solid-js";

export type RefCallback<T> = ((value: T) => void) | readonly RefCallback<T>[] | null | undefined | false;
/** A JSX assignment target, callback, or nested callback array. */
export type Ref<T> = T | RefCallback<T>;

/** Compiler ref ABI: preserve the owner, suppress tracking, and ignore callback return values. */
export function applyRef<T>(ref: RefCallback<T>, value: T): void {
  untrack(() => {
    if (Array.isArray(ref)) {
      for (const callback of ref) applyRef(callback, value);
    } else if (ref) {
      (ref as (value: T) => void)(value);
    }
  });
}
