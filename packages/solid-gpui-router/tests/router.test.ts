import { describe, expect, test } from "bun:test";
import { MemoryTransport, Text, View, createRoot } from "@solid-gpui/core";
import { createComponent, createEffect, createRoot as createSolidRoot, onCleanup } from "@solid-gpui/core/runtime";

import {
  DefaultErrorComponent,
  DefaultNotFoundComponent,
  DefaultPendingComponent,
  Outlet,
  RouterProvider,
  createRootRoute,
  createRootRouteWithContext,
  createRoute,
  createRouter,
} from "../src";

function createTestRouteTree(onHomeRender?: () => void, onSettingsRender?: () => void) {
  const rootRoute = createRootRoute({ component: Outlet });
  const homeRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => {
      onHomeRender?.();
      return createComponent(View, {
        children: createComponent(Text, { children: "Home" }),
      });
    },
  });
  const settingsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "settings",
    component: () => {
      onSettingsRender?.();
      return createComponent(View, {
        children: createComponent(Text, { children: "Settings" }),
      });
    },
  });
  return rootRoute.addChildren([homeRoute, settingsRoute]);
}

describe("native router", () => {
  test("keeps history and location independent per window", async () => {
    const routeTree = createTestRouteTree();
    const first = createRouter({ routeTree });
    const second = createRouter({ routeTree });

    expect(first.history).not.toBe(second.history);
    await Promise.all([first.load(), second.load()]);
    await first.navigate({ to: "/settings" });

    expect(first.latestLocation.pathname).toBe("/settings");
    expect(second.latestLocation.pathname).toBe("/");
    expect(second.history.location.pathname).toBe("/");
  });

  test("installs native-safe defaults", () => {
    const router = createRouter({ routeTree: createTestRouteTree() });

    expect(typeof router.options.scrollRestoration).toBe("function");
    expect(
      typeof router.options.scrollRestoration === "function" &&
        router.options.scrollRestoration({ location: router.latestLocation }),
    ).toBe(false);
    expect(router.options.defaultViewTransition).toBe(false);
    expect(router.options.defaultHashScrollIntoView).toBe(false);
    expect(router.options.defaultPreload).toBe(false);
    expect(router.options.defaultPendingComponent).toBe(DefaultPendingComponent);
    expect(router.options.defaultErrorComponent).toBe(DefaultErrorComponent);
    expect(router.options.defaultNotFoundComponent).toBe(DefaultNotFoundComponent);
  });

  test("publishes route matches through Solid stores", async () => {
    const router = createRouter({ routeTree: createTestRouteTree() });
    const observedRouteIds: string[][] = [];
    const dispose = createSolidRoot((disposeRoot) => {
      createEffect(() => observedRouteIds.push(router.stores.__store.get().matches.map((match) => match.routeId)));
      return disposeRoot;
    });

    try {
      await router.load();
      expect(observedRouteIds.at(-1)).toEqual(["__root__", "/"]);
      await router.navigate({ to: "/settings" });
      expect(observedRouteIds.at(-1)).toEqual(["__root__", "/settings"]);
    } finally {
      dispose();
    }
  });

  test("shares explicit app state while keeping each window context distinct", async () => {
    interface AppState {
      readonly loadedBy: string[];
    }
    interface WindowRouterContext {
      readonly app: AppState;
      readonly windowId: string;
    }

    const app: AppState = { loadedBy: [] };
    const rootRoute = createRootRouteWithContext<WindowRouterContext>()({
      component: Outlet,
      loader: ({ context }) => {
        context.app.loadedBy.push(context.windowId);
        return context.windowId;
      },
    });
    const homeRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: "/",
      component: () => createComponent(Text, { children: "Shared app state" }),
    });
    const routeTree = rootRoute.addChildren([homeRoute]);
    const first = createRouter({ routeTree, context: { app, windowId: "first" } });
    const second = createRouter({ routeTree, context: { app, windowId: "second" } });

    await Promise.all([first.load(), second.load()]);

    expect([...app.loadedBy].sort()).toEqual(["first", "second"]);
    expect(first.options.context.app).toBe(app);
    expect(second.options.context.app).toBe(app);
    expect(first.options.context).not.toBe(second.options.context);
    expect(first.stores.getMatchStore("__root__").get()?.loaderData).toBe("first");
    expect(second.stores.getMatchStore("__root__").get()?.loaderData).toBe("second");
  });
  test("preserves the mounted layout and its state across sibling navigation", async () => {
    let mounts = 0;
    let cleanups = 0;
    const layout = createRootRoute({
      component: () => {
        mounts++;
        onCleanup(() => {
          cleanups++;
        });
        return createComponent(View, { children: createComponent(Outlet, {}) });
      },
    });
    const home = createRoute({
      getParentRoute: () => layout,
      path: "/",
      component: () => createComponent(Text, { children: "Home" }),
    });
    const other = createRoute({
      getParentRoute: () => layout,
      path: "other",
      component: () => createComponent(Text, { children: "Other" }),
    });
    const router = createRouter({ routeTree: layout.addChildren([home, other]) });
    const root = createRoot(new MemoryTransport());
    try {
      await router.load();
      root.render(() => createComponent(RouterProvider, { router }));
      await router.load();
      await router.navigate({ to: "/other" });
      await router.navigate({ to: "/" });
      expect(mounts).toBe(1);
      expect(cleanups).toBe(0);
    } finally {
      root.unmount();
    }
    expect(cleanups).toBe(1);
  });

  test("renders and navigates two native roots without DOM hosts", async () => {
    let homeRenders = 0;
    let settingsRenders = 0;
    const routeTree = createTestRouteTree(
      () => {
        homeRenders += 1;
      },
      () => {
        settingsRenders += 1;
      },
    );

    const firstRouter = createRouter({ routeTree });
    const secondRouter = createRouter({ routeTree });
    const firstTransport = new MemoryTransport();
    const secondTransport = new MemoryTransport();
    const firstRoot = createRoot(firstTransport);
    const secondRoot = createRoot(secondTransport);

    try {
      firstRoot.render(() => createComponent(RouterProvider, { router: firstRouter }));
      secondRoot.render(() => createComponent(RouterProvider, { router: secondRouter }));
      const firstPendingFrameCount = firstTransport.submitted.length;
      await Promise.all([firstRouter.load(), secondRouter.load()]);
      await Promise.resolve();
      expect(firstRouter.stores.__store.get().matches.map((match) => [match.routeId, match.status])).toEqual([
        ["__root__", "success"],
        ["/", "success"],
      ]);
      expect(firstTransport.submitted.length).toBeGreaterThan(firstPendingFrameCount);

      const firstFrameCount = firstTransport.submitted.length;
      const secondFrameCount = secondTransport.submitted.length;
      expect(firstFrameCount).toBeGreaterThan(0);
      expect(secondFrameCount).toBeGreaterThan(0);
      expect(homeRenders).toBeGreaterThan(0);

      await firstRouter.navigate({ to: "/settings" });
      await Promise.resolve();
      expect(settingsRenders).toBeGreaterThan(0);

      expect(firstRouter.latestLocation.pathname).toBe("/settings");
      expect(secondRouter.latestLocation.pathname).toBe("/");
      expect(firstTransport.submitted.length).toBeGreaterThan(firstFrameCount);
      expect(secondTransport.submitted.length).toBe(secondFrameCount);
    } finally {
      firstRoot.unmount();
      secondRoot.unmount();
    }
  });
});
