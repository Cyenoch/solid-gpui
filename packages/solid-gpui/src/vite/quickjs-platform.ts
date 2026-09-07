import URL from "core-js-pure/actual/url";
import URLSearchParams from "core-js-pure/actual/url-search-params";
import DOMException from "core-js-pure/actual/dom-exception";
import { Event, EventTarget } from "event-target-shim";
import { Headers } from "./quickjs-headers.js";
import { AbortController, AbortSignal } from "./quickjs-abort";

/** TanStack redirects need Response identity and metadata, but no network or body services. */
class NativeRouterResponse {
  readonly #status: number;
  readonly #statusText: string;
  readonly #headers: Headers;

  constructor(body: unknown = null, init: ResponseInit = {}) {
    if (body !== null) {
      throw new TypeError("QuickJS Response supports bodyless native router redirects only");
    }
    const status = init.status === undefined ? 200 : Number(init.status);
    if (!Number.isInteger(status) || status < 200 || status > 599) {
      throw new RangeError("Response status must be an integer between 200 and 599");
    }
    const statusText = init.statusText === undefined ? "" : String(init.statusText);
    if (/[^\t\x20-\x7e\x80-\xff]/.test(statusText)) {
      throw new TypeError("Response statusText must be a valid HTTP reason phrase");
    }
    this.#status = status;
    this.#statusText = statusText;
    this.#headers = new Headers(init.headers);
  }

  get status(): number {
    return this.#status;
  }

  get statusText(): string {
    return this.#statusText;
  }

  get headers(): Headers {
    return this.#headers;
  }

  get ok(): boolean {
    return this.#status >= 200 && this.#status < 300;
  }

  get body(): null {
    return null;
  }

  get bodyUsed(): boolean {
    return false;
  }
}

// This module is an explicit QuickJS build entry dependency. Bun keeps its own
// platform, and no unavailable fetch, filesystem, or socket services are advertised.
Object.assign(globalThis, {
  self: globalThis,
  URL,
  URLSearchParams,
  DOMException,
  Event,
  EventTarget,
  AbortController,
  AbortSignal,
  Headers,
  Response: NativeRouterResponse,
});
