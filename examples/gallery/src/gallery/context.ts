import { createSignal, getOwner, onCleanup, type Accessor } from "solid-js";
import { createWindowSizeStore, type WindowSizeStore } from "@solid-gpui/core";
import type { Root } from "@solid-gpui/core";
import type { ThemeColors, ThemeMode } from "./theme";
import { getTheme } from "./theme";

export interface GalleryState {
  readonly themeMode: Accessor<ThemeMode>;
  readonly setThemeMode: (mode: ThemeMode) => void;
  readonly toggleTheme: () => void;
  readonly theme: Accessor<ThemeColors>;
  readonly searchQuery: Accessor<string>;
  readonly setSearchQuery: (query: string) => void;
  readonly windowSizeStore: WindowSizeStore;
  readonly root: Accessor<Root | null>;
  readonly setRoot: (root: Root | null) => void;
  readonly statusMessage: Accessor<string | null>;
  readonly showStatus: (message: string, variantOrDuration?: string | number) => void;
}

let galleryStateInstance: GalleryState | null = null;

export function createGalleryState(
  initialMode: ThemeMode = "dark",
  windowSizeStore: WindowSizeStore = createWindowSizeStore({ width: 800, height: 600, scaleFactor: 1 }),
): GalleryState {
  const [themeMode, setThemeMode] = createSignal<ThemeMode>(initialMode);
  const [searchQuery, setSearchQuery] = createSignal<string>("");
  const [root, setRoot] = createSignal<Root | null>(null);
  const [statusMessage, setStatusMessage] = createSignal<string | null>(null);

  const theme = () => getTheme(themeMode());

  const toggleTheme = () => {
    setThemeMode((prev) => (prev === "dark" ? "light" : "dark"));
  };

  let statusTimer: ReturnType<typeof setTimeout> | undefined;
  let disposed = false;
  if (getOwner())
    onCleanup(() => {
      disposed = true;
      clearTimeout(statusTimer);
    });
  const showStatus = (message: string, variantOrDuration?: string | number) => {
    if (disposed) return;
    const duration = typeof variantOrDuration === "number" ? variantOrDuration : 3000;
    clearTimeout(statusTimer);
    setStatusMessage(message);
    statusTimer = setTimeout(() => {
      setStatusMessage(null);
      statusTimer = undefined;
    }, duration);
  };

  const galleryState: GalleryState = {
    themeMode,
    setThemeMode,
    toggleTheme,
    theme,
    searchQuery,
    setSearchQuery,
    windowSizeStore,
    root,
    setRoot,
    statusMessage,
    showStatus,
  };
  galleryStateInstance = galleryState;
  return galleryState;
}

export function useGallery(): GalleryState {
  if (!galleryStateInstance) {
    throw new Error("createGalleryState() must run before useGallery()");
  }
  return galleryStateInstance;
}
