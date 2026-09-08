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
  const check = (value: unknown, depth: number): void => {
    if (++nodes > 100_000 || depth > 64) throw new RangeError("reload state exceeds its structure budget");
    if (value === null || typeof value === "string" || typeof value === "boolean") return;
    if (typeof value === "number" && Number.isFinite(value) && !Object.is(value, -0)) return;
    if (typeof value !== "object" || ancestors.has(value))
      throw new TypeError("reload state must be acyclic JSON data");
    const prototype = Object.getPrototypeOf(value);
    if (!Array.isArray(value) && prototype !== Object.prototype && prototype !== null)
      throw new TypeError("reload state must contain plain objects and arrays");
    if (Object.getOwnPropertySymbols(value).length) throw new TypeError("reload state cannot contain symbols");
    ancestors.add(value);
    const entries = Object.getOwnPropertyDescriptors(value);
    if (Array.isArray(value) && Object.keys(entries).length !== value.length + 1)
      throw new TypeError("reload state cannot contain sparse arrays or array properties");
    for (const [key, descriptor] of Object.entries(entries)) {
      if (Array.isArray(value) && key === "length") continue;
      if (!descriptor.enumerable || !("value" in descriptor))
        throw new TypeError("reload state cannot contain accessors or hidden properties");
      check(descriptor.value, depth + 1);
    }
    ancestors.delete(value);
  };
  check(value, 0);
  const encoded = JSON.stringify(value);
  if (new TextEncoder().encode(encoded).length > 1024 * 1024) throw new RangeError("reload state exceeds 1 MiB");
  return encoded;
}
