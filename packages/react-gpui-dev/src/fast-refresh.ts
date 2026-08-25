import { createElement, type FunctionComponent, type ReactNode } from "react";
import { watch, type FSWatcher } from "node:fs";
import { installFastRefreshTransform, performReactRefresh } from "./transform";

export interface RefreshRoot {
  render(node: ReactNode): void;
}

declare global {
  // React Refresh installs these globals before transformed modules execute.
  var RefreshRuntime: RefreshRuntimeLike | undefined;
  var $RefreshReg$: (type: unknown, id: string) => void;
  var $RefreshSig$: (...args: unknown[]) => (...args: unknown[]) => unknown;
}

export interface RefreshModule<Props> {
  readonly default: FunctionComponent<Props>;
  /** Optional only for low-level tests; Babel supplies signatures normally. */
  readonly signature?: string;
}

export type RefreshLoader<Props> = () => Promise<RefreshModule<Props>>;

export type RefreshResult =
  | { readonly kind: "preserved"; readonly signature?: string }
  | { readonly kind: "remounted"; readonly signature?: string }
  | { readonly kind: "failed"; readonly error: unknown };

interface Family<Props> {
  current: FunctionComponent<Props>;
  signature?: string;
}

interface RefreshRuntimeLike {
  register(type: unknown, id: string): void;
  performReactRefresh?(): unknown;
}

function refreshRuntime(): RefreshRuntimeLike | undefined {
  const value = globalThis.RefreshRuntime;
  if (value === undefined || typeof value.register !== "function") return undefined;
  return value;
}

function createFamilyProxy<Props>(family: Family<Props>): FunctionComponent<Props> {
  return function RefreshFamilyProxy(props: Props) {
    return family.current(props);
  };
}

/**
 * Keeps one React component family mounted while its implementation changes.
 *
 * The proxy is the stable Fiber type. When the emitted hook signature is
 * unchanged, the implementation pointer changes and React's existing hook
 * cells remain attached to that Fiber. A signature change creates a new proxy,
 * deliberately remounting only that family. Failed source/eval updates do not
 * call `root.render`, so the last-good native tree remains presented.
 */
export class FastRefreshSession<Props> {
  private family: Family<Props>;
  private proxy: FunctionComponent<Props>;
  private activeType: FunctionComponent<Props>;
  private readonly runtimeManaged: boolean;
  private props: Props;
  private lastGood: RefreshModule<Props>;
  private disposed = false;

  constructor(
    private readonly root: RefreshRoot,
    initial: RefreshModule<Props>,
    props: Props,
    private readonly familyId = "react-gpui-family",
  ) {
    const initialType = initial.default as unknown as {
      readonly __reactGpuiStableFamily?: boolean;
    };
    this.runtimeManaged =
      refreshRuntime() !== undefined &&
      globalThis.__reactGpuiFamily !== undefined &&
      initialType.__reactGpuiStableFamily === true;
    this.family = { current: initial.default, signature: initial.signature };
    this.proxy = createFamilyProxy(this.family);
    this.activeType = this.runtimeManaged ? initial.default : this.proxy;
    this.props = props;
    this.lastGood = initial;
    this.register(initial.default);
    this.root.render(this.element());
  }
  get lastGoodModule(): RefreshModule<Props> {
    return this.lastGood;
  }

  updateProps(props: Props): void {
    if (this.disposed) return;
    this.props = props;
    this.root.render(this.element());
  }

  async refresh(load: RefreshLoader<Props>): Promise<RefreshResult> {
    if (this.disposed) {
      return { kind: "failed", error: new Error("Fast Refresh session is disposed") };
    }

    let next: RefreshModule<Props>;
    try {
      next = await load();
      if (typeof next?.default !== "function") {
        throw new TypeError("refresh module must export a component");
      }
    } catch (error) {
      this.reportFailure(error);
      return { kind: "failed", error };
    }

    this.register(next.default);
    const compatible =
      next.signature === undefined ||
      this.family.signature === undefined ||
      next.signature === this.family.signature;
    if (this.runtimeManaged) {
      this.activeType = next.default;
      this.lastGood = next;
      try {
        this.root.render(this.element());
        performReactRefresh();
        return { kind: compatible ? "preserved" : "remounted", signature: next.signature };
      } catch (error) {
        if (!(error instanceof Error) || !/hook|rendered more/i.test(error.message)) throw error;
        this.family = { current: next.default, signature: next.signature };
        this.proxy = createFamilyProxy(this.family);
        this.activeType = this.proxy;
        this.root.render(this.element());
        performReactRefresh();
        return { kind: "remounted", signature: next.signature };
      }
    }
    if (compatible) {
      this.family.current = next.default;
      this.lastGood = next;
      this.root.render(this.element());
      performReactRefresh();
      return { kind: "preserved", signature: next.signature };
    }
    this.family = { current: next.default, signature: next.signature };
    this.proxy = createFamilyProxy(this.family);
    this.activeType = this.proxy;
    this.lastGood = next;
    this.root.render(this.element());
    performReactRefresh();
    return { kind: "remounted", signature: next.signature };
  }

  dispose(): void {
    this.disposed = true;
  }

  private register(type: FunctionComponent<Props>): void {
    refreshRuntime()?.register(type, `${this.familyId}:implementation`);
  }

  private element() {
    const proxy = this.activeType as unknown as FunctionComponent<unknown>;
    const props = this.props as Record<string, unknown>;
    return createElement(proxy, props);
  }

  private reportFailure(error: unknown): void {
    const detail = error instanceof Error ? `${error.message}\n${error.stack ?? ""}` : String(error);
    console.error(`[react-gpui] Fast Refresh kept the last-good tree: ${detail}`);
  }
}

/** Normal DX entrypoint: Babel injects signatures; callers pass only a module path. */
export async function createFastRefreshSession<Props>(
  root: RefreshRoot,
  path: string,
  props: Props,
  familyId = path,
): Promise<FastRefreshSession<Props>> {
  await installFastRefreshTransform();
  const initial = await importWithRefresh<Props>(path);
  return new FastRefreshSession(root, initial, props, familyId);
}

/**
 * Watch one source module and refresh it with a cache-busting ESM import.
 * Bun's native `fs.watch` runs on the embedded runtime event loop; no child
 * process or GPUI callback is involved.
 */
export function watchModule<Props>(
  path: string,
  session: FastRefreshSession<Props>,
  load: RefreshLoader<Props>,
): FSWatcher {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const watcher = watch(path, { persistent: false }, () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = undefined;
      void session.refresh(load);
    }, 25);
  });
  return watcher;
}

export function importWithRefresh<Props>(path: string): Promise<RefreshModule<Props>> {
  const url = `${path}${path.includes("?") ? "&" : "?"}react_gpui_refresh=${Date.now()}`;
  return import(url) as Promise<RefreshModule<Props>>;
}
