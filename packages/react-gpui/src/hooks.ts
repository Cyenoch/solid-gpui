import { useSyncExternalStore } from "react";

export interface WindowSize {
  readonly width: number;
  readonly height: number;
}

export interface WindowSizeStore {
  getSnapshot(): WindowSize;
  subscribe(listener: () => void): () => void;
  set(width: number, height: number): void;
}

/** Create the explicit store wired to a root's `onWindowResize` callback. */
export function createWindowSizeStore(initial: WindowSize = { width: 1024, height: 720 }): WindowSizeStore {
  let snapshot: WindowSize = { width: initial.width, height: initial.height };
  const listeners = new Set<() => void>();
  return {
    getSnapshot: () => snapshot,
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    set(width, height) {
      if (snapshot.width === width && snapshot.height === height) return;
      snapshot = { width, height };
      for (const listener of listeners) listener();
    },
  };
}

/** Subscribe to an explicit window-size store without an implicit global root. */
export function useWindowSize(store: WindowSizeStore): WindowSize {
  return useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot);
}
