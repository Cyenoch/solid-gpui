import { describe, expect, test } from "bun:test";
import { ICON_NAMES, MemoryTransport, createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { RouterProvider } from "@solid-gpui/router";
import { createGalleryState } from "../src/gallery/context";
import { createGalleryRouter } from "../src/gallery/routes";
import { CATEGORIES, NAV_TABS, PAGES, pagePath } from "../src/gallery/types";

describe("Solid GPUI Gallery Showcase", () => {
  test("uses available icons in gallery navigation", () => {
    for (const item of [...CATEGORIES, ...NAV_TABS, ...PAGES]) {
      expect(ICON_NAMES.some((name) => name === item.icon)).toBe(true);
    }
  });

  test("routes every gallery page through the native router", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport);
    const gallery = createGalleryState("dark");
    gallery.setRoot(root);
    const router = createGalleryRouter();

    try {
      root.render(() => createComponent(RouterProvider, { router }));
      await router.load();
      await Promise.resolve();
      expect(router.latestLocation.pathname).toBe("/");

      for (const page of PAGES) {
        const path = pagePath(page.id);
        await router.navigate({ to: path });
        await Promise.resolve();
        expect(router.latestLocation.pathname).toBe(path);
      }
    } finally {
      root.unmount();
    }
  });
});
