/*!
 * Adapted from fetch-headers 3.0.1, commit
 * 66d63ac7a67d3b9c863cd80b872f2ea999cbc7a7:
 * https://github.com/jimmywarting/fetch-headers/blob/66d63ac7a67d3b9c863cd80b872f2ea999cbc7a7/headers.js
 * Changes: stateless value validation, ASCII name validation, HTTP whitespace trimming, atomic set,
 * immutable delete validation, and removal of the Node inspection hook and bag export.
 *
 * MIT License
 *
 * Copyright (c) 2021 Jimmy Wärting
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

/** @param {Headers} instance */
function assert(instance, argsCount = 0, requiredArgs = 0, method = "") {
  if (!(instance instanceof Headers)) {
    throw new TypeError("Illegal invocation");
  }
  if (argsCount < requiredArgs) {
    throw new TypeError(
      `"Failed to execute '${method}' on 'Headers'" requires at least ${requiredArgs} argument, but only ${argsCount} were provided.`,
    );
  }
  return /** @type {Bag} */ (wm.get(instance));
}

/**
 * @typedef Bag
 * @property {Object<string, string>} items
 * @property {Array<string>} cookies
 * @property {string} guard
 */

/**
 * @param {Bag} bag
 * @param {HeadersInit} object
 */
function fillHeaders(bag, object) {
  if (object === null) throw new TypeError("HeadersInit can't be null.");

  const iterable = object[Symbol.iterator];

  if (iterable) {
    // @ts-ignore
    for (let header of object) {
      if (typeof header === "string") {
        throw new TypeError("The provided value cannot be converted to a sequence.");
      }

      if (header[Symbol.iterator] && !Array.isArray(header)) {
        header = [...header];
      }

      if (header.length !== 2) {
        throw new TypeError(`Invalid header. Length must be 2, but is ${header.length}`);
      }

      appendHeader(bag, header[0], header[1]);
    }
  } else {
    for (const key of Reflect.ownKeys(object)) {
      const x = Reflect.getOwnPropertyDescriptor(object, key);
      if (x === undefined || !x.enumerable) continue;

      if (typeof key === "symbol") {
        throw new TypeError("Invalid header. Symbol key is not supported.");
      }

      if (!HTTP_TOKEN_CODE_POINT_RE.test(key)) {
        throw new TypeError("Header name is not valid.");
      }

      appendHeader(bag, key, Reflect.get(object, key));
    }
  }
}

const ILLEGAL_VALUE_CHARS = /[\x00\x0A\x0D]/;
const IS_BYTE_STRING = /^[\x00-\xFF]*$/;
const HTTP_TOKEN_CODE_POINT_RE =
  /^[\u0021\u0023\u0024\u0025\u0026\u0027\u002a\u002b\u002d\u002e\u005e\u005f\u0060\u007c\u007e\u0030-\u0039\u0041-\u005a\u0061-\u007a]+$/;
/** @param {string} value */
function normalizeValue(value) {
  value = `${value}`.replace(/^[\r\n\t ]+|[\r\n\t ]+$/g, "");
  if (ILLEGAL_VALUE_CHARS.test(value) || !IS_BYTE_STRING.test(value)) {
    throw new TypeError(`Header value ${JSON.stringify(value)} is not valid.`);
  }
  return value;
}

/**
 * https://fetch.spec.whatwg.org/#concept-headers-append
 * @param {Bag} bag
 * @param {string} name
 * @param {string} value
 */
function appendHeader(bag, name, value) {
  name = normalizeName(name);
  value = normalizeValue(value);

  if (bag.guard === "immutable") {
    throw new TypeError("Headers are immutable.");
  }

  bag.items[name] = name in bag.items ? `${bag.items[name]}, ${value}` : value;

  if (name === "set-cookie") {
    bag.cookies.push(value);
  }
}

/** @param {string} name */
function normalizeName(name) {
  name = `${name}`;
  if (!HTTP_TOKEN_CODE_POINT_RE.test(name)) throw new TypeError("Header name is not valid.");
  return name.toLowerCase();
}

/** @type {WeakMap<Headers, Bag>} */
const wm = new WeakMap();

export class Headers {
  /** @param {HeadersInit} [init] */
  constructor(init = undefined) {
    const bag = {
      items: Object.create(null),
      cookies: [],
      guard: "mutable",
    };

    wm.set(this, bag);

    if (init !== undefined) {
      fillHeaders(bag, init);
    }
  }

  append(name, value) {
    const bag = assert(this, arguments.length, 2, "append");
    appendHeader(bag, name, value);
  }

  delete(name) {
    const bag = assert(this, arguments.length, 1, "delete");
    name = normalizeName(name);
    if (bag.guard === "immutable") throw new TypeError("Headers are immutable.");
    delete bag.items[name];
    if (name === "set-cookie") bag.cookies.length = 0;
  }

  get(name) {
    const bag = assert(this, arguments.length, 1, "get");
    name = normalizeName(name);
    return name in bag.items ? bag.items[name] : null;
  }

  has(name) {
    const bag = assert(this, arguments.length, 1, "has");
    return normalizeName(name) in bag.items;
  }

  set(name, value) {
    const bag = assert(this, arguments.length, 2, "set");
    name = normalizeName(name);
    value = normalizeValue(value);
    this.delete(name);
    appendHeader(bag, name, value);
  }

  forEach(callback, thisArg = globalThis) {
    const bag = assert(this, arguments.length, 1, "forEach");
    if (typeof callback !== "function") {
      throw new TypeError("Failed to execute 'forEach' on 'Headers': parameter 1 is not of type 'Function'.");
    }

    for (const x of this) {
      callback.call(thisArg, x[1], x[0], this);
    }
  }

  toString() {
    return "[object Headers]";
  }

  getSetCookie() {
    const bag = assert(this, 0, 0, "");
    return bag.cookies.slice(0);
  }

  keys() {
    assert(this, 0, 0, "");
    return [...this].map((x) => x[0]).values();
  }

  values() {
    assert(this, 0, 0, "");
    return [...this].map((x) => x[1]).values();
  }

  entries() {
    const bag = assert(this, 0, 0, "");
    /** @type {Array<[string, string]>} */
    const result = [];

    const entries = [...Object.entries(bag.items).sort((a, b) => (a[0] > b[0] ? 1 : -1))];

    for (const [name, value] of entries) {
      if (name === "set-cookie") {
        for (const cookie of bag.cookies) {
          result.push([name, cookie]);
        }
      } else result.push([name, value]);
    }

    return result.values();
  }

  [Symbol.iterator]() {
    return this.entries();
  }
}

const enumerable = { enumerable: true };

Object.defineProperties(Headers.prototype, {
  append: enumerable,
  delete: enumerable,
  entries: enumerable,
  forEach: enumerable,
  get: enumerable,
  getSetCookie: enumerable,
  has: enumerable,
  keys: enumerable,
  set: enumerable,
  values: enumerable,
});
