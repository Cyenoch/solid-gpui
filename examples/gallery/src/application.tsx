import { mountApplication, type DisposableTransport, type WindowSize } from "@solid-gpui/core";
import { RouterProvider } from "@solid-gpui/router";
import { createGalleryState } from "./gallery/context";
import { createGalleryRouter } from "./gallery/routes";
import type { ThemeMode } from "./gallery/theme";

interface GalleryReloadState {
  readonly path: string;
  readonly theme: ThemeMode;
  readonly search: string;
  readonly window: WindowSize;
}

export function mountGallery(transport: () => DisposableTransport, hotKey?: string) {
  return mountApplication<GalleryReloadState>({
    hotKey,
    transport,
    setup(previous) {
      const gallery = createGalleryState(previous?.theme ?? "dark");
      const router = createGalleryRouter(previous ? [previous.path] : undefined);
      if (previous) {
        gallery.setSearchQuery(previous.search);
        gallery.windowSizeStore.set(previous.window.width, previous.window.height, previous.window.scaleFactor);
      }
      return {
        render: () => <RouterProvider router={router} />,
        rootOptions: {
          onAppearance: (appearance) => gallery.setThemeMode(appearance),
          onWindowResize: (width, height, scale) => gallery.windowSizeStore.set(width, height, scale),
        },
        onMount(root) {
          gallery.setRoot(root);
          root.setTitle("Solid GPUI — Component Workbench").catch((error: unknown) => {
            gallery.showStatus(`Unable to set window title: ${String(error)}`, "warning");
          });
        },
        captureState: () => ({
          path: router.stores.location.get().href,
          theme: gallery.themeMode(),
          search: gallery.searchQuery(),
          window: gallery.windowSizeStore.getSnapshot(),
        }),
      };
    },
  });
}
