// This closure retains all VM callbacks. Only its byte bridge reaches Rust.
(function (submit, now, log, decodeUtf8) {
  let subscription;
  let ended = false;
  const rejections = new Map();
  const timers = new Map();
  const heap = [];
  let nextTimerId = 1;

  function less(a, b) {
    return a.due < b.due || (a.due === b.due && a.id < b.id);
  }
  function swap(a, b) {
    [heap[a], heap[b]] = [heap[b], heap[a]];
    heap[a].index = a;
    heap[b].index = b;
  }
  function repair(index) {
    while (index > 0 && less(heap[index], heap[(index - 1) >> 1])) {
      const parent = (index - 1) >> 1;
      swap(index, parent);
      index = parent;
    }
    for (;;) {
      let child = index * 2 + 1;
      if (child >= heap.length) break;
      if (child + 1 < heap.length && less(heap[child + 1], heap[child])) child++;
      if (!less(heap[child], heap[index])) break;
      swap(index, child);
      index = child;
    }
  }
  function removeTimer(timer) {
    const index = timer.index;
    const last = heap.pop();
    timers.delete(timer.id);
    if (index < heap.length) {
      heap[index] = last;
      last.index = index;
      repair(index);
    }
  }
  function addTimer(callback, delay, interval, args) {
    if (typeof callback !== 'function') throw new TypeError('timer callback must be a function');
    if (timers.size >= 4096) throw new RangeError('QuickJS timer capacity exceeded');
    delay = Number(delay);
    delay = Number.isFinite(delay) ? Math.max(0, Math.min(2147483647, delay)) : 0;
    if (interval) delay = Math.max(1, delay);
    if (!Number.isSafeInteger(nextTimerId)) throw new RangeError('QuickJS timer ID exhausted');
    const timer = { id: nextTimerId++, due: now() + delay, delay, interval, callback, args, index: heap.length };
    timers.set(timer.id, timer);
    heap.push(timer);
    repair(timer.index);
    return timer.id;
  }
  function clearTimer(id) {
    const timer = timers.get(Number(id));
    if (timer) removeTimer(timer);
  }
  globalThis.setTimeout = (callback, delay = 0, ...args) => addTimer(callback, delay, false, args);
  globalThis.setInterval = (callback, delay = 0, ...args) => addTimer(callback, delay, true, args);
  globalThis.clearTimeout = globalThis.clearInterval = clearTimer;
  globalThis.queueMicrotask = callback => {
    if (typeof callback !== 'function') throw new TypeError('microtask callback must be a function');
    Promise.resolve().then(callback);
  };
  globalThis.performance = Object.freeze({ now });
  const format = value => {
    if (typeof value === 'string') return value;
    try { return JSON.stringify(value) ?? String(value); } catch { return String(value); }
  };
  globalThis.console = Object.freeze(Object.fromEntries(
    ['log', 'info', 'debug', 'warn', 'error'].map(level => [level, (...args) => log(args.map(format).join(' '))]),
  ));

  // Iterate Unicode scalars while reporting UTF-16 input consumption. A lone
  // surrogate is replaced, and encodeInto never writes part of a UTF-8 scalar.
  class TextEncoder {
    get encoding() { return 'utf-8'; }
    encode(value = '') {
      value = String(value);
      const target = new Uint8Array(value.length * 3);
      return target.slice(0, this.encodeInto(value, target).written);
    }
    encodeInto(value, target) {
      if (!(target instanceof Uint8Array)) throw new TypeError('encodeInto requires Uint8Array');
      value = String(value);
      let read = 0, written = 0;
      for (const scalar of value) {
        let code = scalar.codePointAt(0);
        if (code >= 0xd800 && code <= 0xdfff) code = 0xfffd;
        const count = code < 0x80 ? 1 : code < 0x800 ? 2 : code < 0x10000 ? 3 : 4;
        if (written + count > target.length) break;
        if (count === 1) target[written++] = code;
        else {
          target[written++] = (count === 2 ? 0xc0 : count === 3 ? 0xe0 : 0xf0) | (code >> (6 * (count - 1)));
          for (let shift = 6 * (count - 2); shift >= 0; shift -= 6) target[written++] = 0x80 | ((code >> shift) & 0x3f);
        }
        read += scalar.length;
      }
      return { read, written };
    }
  }
  class TextDecoder {
    #fatal; #ignoreBOM; #pending = new Uint8Array(); #bomSeen = false;
    constructor(label = 'utf-8', options = {}) {
      if (!['utf-8', 'utf8', 'unicode-1-1-utf-8'].includes(String(label).trim().toLowerCase()))
        throw new RangeError('QuickJS TextDecoder supports UTF-8 only');
      this.#fatal = Boolean(options.fatal);
      this.#ignoreBOM = Boolean(options.ignoreBOM);
    }
    get encoding() { return 'utf-8'; }
    get fatal() { return this.#fatal; }
    get ignoreBOM() { return this.#ignoreBOM; }
    decode(input = new Uint8Array(), options = {}) {
      let bytes;
      if (input instanceof ArrayBuffer) bytes = new Uint8Array(input);
      else if (ArrayBuffer.isView(input)) bytes = new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
      else throw new TypeError('decode requires an ArrayBuffer or view');
      if (this.#pending.length) {
        const joined = new Uint8Array(this.#pending.length + bytes.length);
        joined.set(this.#pending); joined.set(bytes, this.#pending.length); bytes = joined;
      }
      this.#pending = new Uint8Array();
      let end = bytes.length;
      if (options.stream && end) {
        let start = end - 1;
        while (start > 0 && end - start < 4 && (bytes[start] & 0xc0) === 0x80) start--;
        const lead = bytes[start];
        const need = lead >= 0xc2 && lead <= 0xdf ? 2 : lead >= 0xe0 && lead <= 0xef ? 3 : lead >= 0xf0 && lead <= 0xf4 ? 4 : 0;
        const second = bytes[start + 1];
        const validSecond = second === undefined || ((second & 0xc0) === 0x80 &&
          !(lead === 0xe0 && second < 0xa0) && !(lead === 0xed && second >= 0xa0) &&
          !(lead === 0xf0 && second < 0x90) && !(lead === 0xf4 && second >= 0x90));
        if (need > end - start && validSecond) {
          this.#pending = bytes.slice(start); end = start;
        }
      }
      let result;
      try { result = decodeUtf8(bytes.subarray(0, end), this.#fatal); }
      catch (error) {
        this.#pending = new Uint8Array();
        // A failed streaming chunk does not start a new BOM scope.
        if (!options.stream) this.#bomSeen = false;
        throw error;
      }
      if (!this.#bomSeen && result.length) {
        this.#bomSeen = true;
        if (!this.#ignoreBOM && result.charCodeAt(0) === 0xfeff) result = result.slice(1);
      }
      if (!options.stream) this.#bomSeen = false;
      return result;
    }
  }
  globalThis.TextEncoder = TextEncoder;
  globalThis.TextDecoder = TextDecoder;
  globalThis.__solidGpuiHost = Object.freeze({
    submit(frame) {
      if (ended) throw new Error('QuickJS transport is closed');
      submit(frame);
    },
    subscribe(onData, onTermination) {
      if (ended) throw new Error('QuickJS transport is closed');
      if (subscription) throw new Error('QuickJS supports one active transport per VM');
      if (typeof onData !== 'function' || typeof onTermination !== 'function') throw new TypeError('transport callbacks must be functions');
      const current = { onData, onTermination };
      subscription = current;
      return () => { if (subscription === current) subscription = undefined; };
    },
  });
  return {
    subscribed: () => Boolean(subscription),
    dispatch: frame => subscription.onData(frame),
    nextTimer: () => heap.length ? Math.max(0, heap[0].due - now()) : -1,
    runTimer() {
      const timer = heap[0];
      if (!timer || timer.due > now()) return false;
      removeTimer(timer);
      if (timer.interval) {
        timer.due = now() + timer.delay; timer.index = heap.length;
        timers.set(timer.id, timer); heap.push(timer); repair(timer.index);
      }
      timer.callback(...timer.args);
      return true;
    },
    trackRejection(promise, reason, handled) {
      if (handled) rejections.delete(promise); else rejections.set(promise, reason);
    },
    checkRejections() {
      if (rejections.size) throw rejections.values().next().value;
    },
    terminate(message) {
      ended = true;
      const current = subscription; subscription = undefined;
      timers.clear(); heap.length = 0; rejections.clear();
      current?.onTermination(message);
    },
  };
})
