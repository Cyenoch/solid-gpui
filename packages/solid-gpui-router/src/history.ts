import {
  parseHref,
  type HistoryLocation,
  type HistoryState,
  type NavigateOptions,
  type NavigationBlocker,
  type RouterHistory,
} from "@tanstack/history";

type Action = Parameters<RouterHistory["notify"]>[0];
type Subscriber = Parameters<RouterHistory["subscribe"]>[0];
type Registration = { blocker: NavigationBlocker };
type PendingNavigation = { cancel: () => void; registrations: Set<Registration> };

/** Native history owns the stack and checks every transition before publishing it. */
export class NativeHistory implements RouterHistory {
  private entries: HistoryLocation[];
  private index: number;
  private registrations = new Set<Registration>();
  private pending?: PendingNavigation;
  private destroyed = false;
  readonly subscribers = new Set<Subscriber>();

  constructor(initialEntries: readonly string[]) {
    if (initialEntries.length === 0) throw new TypeError("initialEntries must contain at least one route");
    this.entries = initialEntries.map((href, index) => this.createLocation(href, undefined, index));
    this.index = this.entries.length - 1;
  }

  get location(): HistoryLocation {
    return this.entries[this.index]!;
  }
  get length(): number {
    return this.entries.length;
  }

  subscribe = (subscriber: Subscriber): (() => void) => {
    this.subscribers.add(subscriber);
    return () => {
      this.subscribers.delete(subscriber);
    };
  };

  block = (blocker: NavigationBlocker): (() => void) => {
    this.assertActive();
    const registration = { blocker };
    this.registrations.add(registration);
    return () => {
      this.registrations.delete(registration);
      if (this.pending?.registrations.has(registration)) this.pending.cancel();
    };
  };

  private assertActive(): void {
    if (this.destroyed) throw new Error("Cannot navigate a destroyed native history");
  }

  private createLocation(href: string, state: HistoryState | undefined, index: number): HistoryLocation {
    const location = parseHref(href, undefined);
    location.state = { ...state, ...location.state, __TSR_index: index };
    return location;
  }

  /** A newer request or removed registration invalidates an outstanding decision. */
  private transition(
    nextLocation: HistoryLocation,
    action: Action,
    options: NavigateOptions | undefined,
    commit: () => void | Promise<void>,
  ): Promise<void> {
    this.assertActive();
    this.pending?.cancel();
    const registrations = options?.ignoreBlocker ? [] : [...this.registrations];
    if (registrations.length === 0) return Promise.resolve(commit());

    const currentLocation = this.location;
    return new Promise<void>((resolve, reject) => {
      const pending: PendingNavigation = {
        registrations: new Set(registrations),
        cancel: () => {
          if (this.pending !== pending) return;
          this.pending = undefined;
          resolve();
        },
      };
      this.pending = pending;
      const check = async () => {
        for (const { blocker } of registrations) {
          if (this.pending !== pending) return;
          const blocked = await blocker.blockerFn({ currentLocation, nextLocation, action: action.type });
          if (this.pending !== pending) return;
          if (blocked) {
            pending.cancel();
            return;
          }
        }
        this.pending = undefined;
        await commit();
        resolve();
      };
      void check().catch((error: unknown) => {
        // An obsolete confirmation must neither commit nor fail a newer request.
        if (this.pending === pending) this.pending = undefined;
        reject(error);
      });
    });
  }

  /** RouterCore's synchronous commit starts only after the native decision settles. */
  commitRouterLocation(
    href: string,
    state: HistoryState,
    replace: boolean,
    options: NavigateOptions,
    commit: () => Promise<void>,
  ): Promise<void> {
    const next = this.createLocation(href, state, this.index + (replace ? 0 : 1));
    return this.transition(next, { type: replace ? "REPLACE" : "PUSH" }, options, commit);
  }

  push = (href: string, state?: HistoryState, options?: NavigateOptions): Promise<void> => {
    const next = this.createLocation(href, state, this.index + 1);
    return this.transition(next, { type: "PUSH" }, options, () => {
      this.entries.splice(this.index + 1, this.entries.length, next);
      this.index++;
      this.notify({ type: "PUSH" });
    });
  };

  replace = (href: string, state?: HistoryState, options?: NavigateOptions): Promise<void> => {
    const next = this.createLocation(href, state, this.index);
    return this.transition(next, { type: "REPLACE" }, options, () => {
      this.entries[this.index] = next;
      this.notify({ type: "REPLACE" });
    });
  };

  private move(delta: number, action: Action, options?: NavigateOptions): Promise<void> {
    if (!Number.isInteger(delta)) throw new TypeError("History delta must be an integer");
    const index = Math.min(Math.max(this.index + delta, 0), this.entries.length - 1);
    if (index === this.index) {
      this.assertActive();
      this.pending?.cancel();
      return Promise.resolve();
    }
    return this.transition(this.entries[index]!, action, options, () => {
      this.index = index;
      this.notify(action);
    });
  }

  go = (delta: number, options?: NavigateOptions): Promise<void> =>
    this.move(delta, { type: "GO", index: delta }, options);
  back = (options?: NavigateOptions): Promise<void> => this.move(-1, { type: "BACK" }, options);
  forward = (options?: NavigateOptions): Promise<void> => this.move(1, { type: "FORWARD" }, options);
  canGoBack = (): boolean => this.index > 0;
  createHref = (href: string): string => href;
  flush = (): void => {};
  notify = (action: Action): void => {
    const location = this.location;
    for (const subscriber of this.subscribers) subscriber({ location, action });
  };
  destroy = (): void => {
    this.destroyed = true;
    this.pending?.cancel();
    this.registrations.clear();
    this.subscribers.clear();
  };
}
