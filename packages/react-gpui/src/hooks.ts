import { useSyncExternalStore } from "react";

export interface WindowSize {
  readonly width: number;
  readonly height: number;
  readonly scaleFactor: number;
}

type WindowSizeInput = Omit<WindowSize, "scaleFactor"> & { readonly scaleFactor?: number };

export interface WindowSizeStore {
  getSnapshot(): WindowSize;
  subscribe(listener: () => void): () => void;
  set(width: number, height: number, scaleFactor?: number): void;
}

/** Create the explicit store wired to a root's `onWindowResize` callback. */
export function createWindowSizeStore(
  initial: WindowSizeInput = { width: 1024, height: 720, scaleFactor: 1 },
): WindowSizeStore {
  let snapshot: WindowSize = {
    width: initial.width,
    height: initial.height,
    scaleFactor: initial.scaleFactor ?? 1,
  };
  const listeners = new Set<() => void>();
  return {
    getSnapshot: () => snapshot,
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    set(width, height, scaleFactor = 1) {
      if (snapshot.width === width && snapshot.height === height && snapshot.scaleFactor === scaleFactor) return;
      snapshot = { width, height, scaleFactor };
      for (const listener of listeners) listener();
    },
  };
}

/** Subscribe to an explicit window-size store without an implicit global root. */
export function useWindowSize(store: WindowSizeStore): WindowSize {
  return useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot);
}

export type Appearance = "light" | "dark";

export interface AppearanceStore {
  getSnapshot(): Appearance;
  subscribe(listener: () => void): () => void;
  set(appearance: Appearance): void;
}

/** Create the explicit store wired to a root's `onAppearance` callback. */
export function createAppearanceStore(initial: Appearance = "light"): AppearanceStore {
  let snapshot = initial;
  const listeners = new Set<() => void>();
  return {
    getSnapshot: () => snapshot,
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    set(appearance) {
      if (snapshot === appearance) return;
      snapshot = appearance;
      for (const listener of listeners) listener();
    },
  };
}

/** Subscribe to an explicit appearance store without an implicit global root. */
export function useAppearance(store: AppearanceStore): Appearance {
  return useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot);
}
