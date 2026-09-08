/** Development generation handoff. Only bounded data crosses VM lifetimes. */
export interface GenerationLifecycle {
  capture(): unknown;
  activate(): void;
  retire(): void;
}
export interface GenerationHost {
  readonly epoch: number;
  readonly state: string;
  register(lifecycle: GenerationLifecycle): void;
}
export function generationHost(): GenerationHost | undefined {
  return (globalThis as { __solidGpuiGeneration?: GenerationHost }).__solidGpuiGeneration;
}

export function encodeGenerationState(value: unknown): string {
  const ancestors = new Set<object>();
  let nodes = 0;
  const check = (value: unknown, depth: number, path: string): void => {
    const reject = (reason: string): never => {
      throw new TypeError(`reload state at ${path}: ${reason}`);
    };
    if (++nodes > 100_000 || depth > 64) throw new RangeError(`reload state at ${path}: exceeds its structure budget`);
    if (value === null || typeof value === "string" || typeof value === "boolean") return;
    if (typeof value === "number") {
      if (!Number.isFinite(value) || Object.is(value, -0)) reject("expected a finite number other than negative zero");
      return;
    }
    if (value === undefined) reject("undefined is not JSON; omit the property or use null");
    if (typeof value !== "object") return reject(`${typeof value} is not JSON`);
    if (ancestors.has(value)) reject("cyclic reference is not JSON");
    const array = Array.isArray(value);
    const prototype = Object.getPrototypeOf(value);
    if (!array && prototype !== Object.prototype && prototype !== null) reject("expected a plain object or array");
    if (Object.getOwnPropertySymbols(value).length) reject("symbol properties are not JSON");
    ancestors.add(value);
    const entries = Object.getOwnPropertyDescriptors(value);
    if (array && Object.keys(entries).length !== value.length + 1)
      reject("expected a dense array without extra properties");
    for (const [key, descriptor] of Object.entries(entries)) {
      if (array && key === "length") continue;
      const index = Number(key);
      if (array && (!Number.isInteger(index) || index < 0 || index >= value.length || String(index) !== key))
        throw new TypeError(`reload state at ${path}[${JSON.stringify(key)}]: extra array properties are not JSON`);
      const childPath = array
        ? `${path}[${key}]`
        : /^[A-Za-z_$][\w$]*$/.test(key)
          ? `${path}.${key}`
          : `${path}[${JSON.stringify(key)}]`;
      if (!descriptor.enumerable || !("value" in descriptor))
        throw new TypeError(`reload state at ${childPath}: accessors and hidden properties are not JSON`);
      check(descriptor.value, depth + 1, childPath);
    }
    ancestors.delete(value);
  };
  check(value, 0, "$");
  const encoded = JSON.stringify(value);
  if (new TextEncoder().encode(encoded).length > 1024 * 1024) throw new RangeError("reload state exceeds 1 MiB");
  return encoded;
}
