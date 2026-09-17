/**
 * Ambient declarations for the QuickJS application runtime.
 *
 * A QuickJS application has no DOM, no Node and no Bun: the declarations here
 * describe exactly the host facilities the runtime installs, and nothing else.
 * Reference this file from a DOM-free application instead of the `dom` lib,
 * `vite/client` or `bun-types`:
 *
 * ```json
 * { "compilerOptions": { "lib": ["ES2024"], "types": ["@solid-gpui/core/quickjs"] } }
 * ```
 *
 * The globals come from two places, both always present in a QuickJS build:
 *
 * - the VM bootstrap (timers, microtasks, `performance`, `console`, UTF-8
 *   `TextEncoder`/`TextDecoder`) in `crates/solid-gpui/src/runtime/quickjs-bootstrap.js`;
 * - the platform shim every QuickJS bundle imports first
 *   (`@solid-gpui/vite/quickjs-platform`): `self` plus the URL, event, abort,
 *   header and redirect-response classes the router needs. There is no `fetch`,
 *   `Request`, filesystem, socket, `requestAnimationFrame` or `import.meta.hot`;
 *   use native commands for host services.
 *
 * This file must stay a global script (no top-level `import`/`export`), so both
 * its globals and its ambient asset modules reach the program that includes it.
 */

declare function setTimeout(handler: () => void, timeout?: number, ...args: unknown[]): number;
declare function setInterval(handler: () => void, timeout?: number, ...args: unknown[]): number;
declare function clearTimeout(id?: number): void;
declare function clearInterval(id?: number): void;
declare function queueMicrotask(callback: () => void): void;

/** Monotonic milliseconds since an unspecified origin; the VM exposes `now` only. */
declare var performance: { readonly now: () => number };

/** Every level routes to the host's log sink; arguments are formatted, not object-inspected. */
declare var console: {
  readonly log: (...args: unknown[]) => void;
  readonly info: (...args: unknown[]) => void;
  readonly debug: (...args: unknown[]) => void;
  readonly warn: (...args: unknown[]) => void;
  readonly error: (...args: unknown[]) => void;
};

/** UTF-8 only, no legacy labels. */
declare class TextEncoder {
  constructor();
  readonly encoding: "utf-8";
  encode(value?: string): Uint8Array;
  encodeInto(value: string, target: Uint8Array): { read: number; written: number };
}

/** UTF-8 only: any other label throws `RangeError`. */
declare class TextDecoder {
  constructor(label?: string, options?: { fatal?: boolean; ignoreBOM?: boolean });
  readonly encoding: "utf-8";
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;
  decode(input?: ArrayBuffer | ArrayBufferView, options?: { stream?: boolean }): string;
}

/** The platform shim points `self` at the VM global object. */
declare var self: typeof globalThis;

/** Single ESM module loaded from the entry path the host was started with. */
interface ImportMeta {
  url: string;
}

/**
 * An asset emitted by the build as an inlined data URL. A QuickJS build produces
 * one self-contained module, so only `?inline` imports are supported; importing
 * an asset path without the suffix would emit a second file.
 */
declare module "*?inline" {
  const source: string;
  export default source;
}

declare class URLSearchParams {
  constructor(init?: string | Record<string, string> | readonly (readonly [string, string])[] | URLSearchParams);
  readonly size: number;
  append(name: string, value: string): void;
  delete(name: string, value?: string): void;
  get(name: string): string | null;
  getAll(name: string): string[];
  has(name: string, value?: string): boolean;
  set(name: string, value: string): void;
  sort(): void;
  toString(): string;
  forEach(callback: (value: string, name: string, params: URLSearchParams) => void): void;
  entries(): IterableIterator<[string, string]>;
  keys(): IterableIterator<string>;
  values(): IterableIterator<string>;
  [Symbol.iterator](): IterableIterator<[string, string]>;
}

declare class URL {
  constructor(url: string | URL, base?: string | URL);
  static canParse(url: string | URL, base?: string | URL): boolean;
  static parse(url: string | URL, base?: string | URL): URL | null;
  href: string;
  readonly origin: string;
  protocol: string;
  username: string;
  password: string;
  host: string;
  hostname: string;
  port: string;
  pathname: string;
  search: string;
  readonly searchParams: URLSearchParams;
  hash: string;
  toString(): string;
  toJSON(): string;
}

/** `name` and `code` are the only state the platform shim preserves. */
declare class DOMException extends Error {
  constructor(message?: string, name?: string);
  readonly name: string;
  readonly code: number;
}

interface QuickJsEventInit {
  bubbles?: boolean;
  cancelable?: boolean;
  composed?: boolean;
}

/** The event shim implements the `Event` interface; it has no node tree. */
declare class Event {
  constructor(type: string, init?: QuickJsEventInit);
  static readonly NONE: number;
  static readonly CAPTURING_PHASE: number;
  static readonly AT_TARGET: number;
  static readonly BUBBLING_PHASE: number;
  readonly type: string;
  readonly target: EventTarget | null;
  readonly srcElement: EventTarget | null;
  readonly currentTarget: EventTarget | null;
  readonly eventPhase: number;
  readonly bubbles: boolean;
  readonly cancelable: boolean;
  readonly composed: boolean;
  readonly defaultPrevented: boolean;
  readonly isTrusted: boolean;
  readonly timeStamp: number;
  readonly NONE: number;
  readonly CAPTURING_PHASE: number;
  readonly AT_TARGET: number;
  readonly BUBBLING_PHASE: number;
  cancelBubble: boolean;
  composedPath(): EventTarget[];
  initEvent(type: string, bubbles?: boolean, cancelable?: boolean): void;
  preventDefault(): void;
  stopImmediatePropagation(): void;
  stopPropagation(): void;
}

declare class EventTarget {
  constructor();
  addEventListener(
    type: string,
    listener: ((event: Event) => void) | { handleEvent(event: Event): void } | null,
    options?: { capture?: boolean; passive?: boolean; once?: boolean; signal?: AbortSignal | null },
  ): void;
  addEventListener(
    type: string,
    listener: ((event: Event) => void) | { handleEvent(event: Event): void } | null,
    capture: boolean,
  ): void;
  removeEventListener(
    type: string,
    listener: ((event: Event) => void) | { handleEvent(event: Event): void } | null,
    options?: { capture?: boolean },
  ): void;
  removeEventListener(
    type: string,
    listener: ((event: Event) => void) | { handleEvent(event: Event): void } | null,
    capture: boolean,
  ): void;
  dispatchEvent(event: Event): boolean;
}

declare class AbortController {
  constructor();
  readonly signal: AbortSignal;
  abort(reason?: unknown): void;
}

declare class AbortSignal extends EventTarget {
  readonly aborted: boolean;
  readonly reason: unknown;
  onabort: ((event: Event) => void) | null;
  throwIfAborted(): void;
  static abort(reason?: unknown): AbortSignal;
  static any(signals: Iterable<AbortSignal>): AbortSignal;
  static timeout(milliseconds: number): AbortSignal;
}

declare class Headers {
  constructor(init?: Headers | readonly (readonly [string, string])[] | Record<string, string>);
  append(name: string, value: string): void;
  delete(name: string): void;
  get(name: string): string | null;
  getSetCookie(): string[];
  has(name: string): boolean;
  set(name: string, value: string): void;
  toString(): string;
  forEach(callback: (value: string, name: string, headers: Headers) => void): void;
  entries(): IterableIterator<[string, string]>;
  keys(): IterableIterator<string>;
  values(): IterableIterator<string>;
  [Symbol.iterator](): IterableIterator<[string, string]>;
}

/** Bodyless redirect response: the shim rejects any other use. */
declare class Response {
  constructor(body?: null, init?: { status?: number; statusText?: string; headers?: Headers });
  readonly status: number;
  readonly statusText: string;
  readonly headers: Headers;
  readonly ok: boolean;
  readonly body: null;
  readonly bodyUsed: false;
}
