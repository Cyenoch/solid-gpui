import { expect, test } from "bun:test";
import { MemoryTransport, Text, createRoot } from "@solid-gpui/core";
import { createComponent, onCleanup } from "@solid-gpui/core/runtime";
import { Outlet, RouterProvider, createRootRoute, createRoute, createRouter } from "../src";

function setup(mounted = true) {
  let mounts = 0;
  let cleanups = 0;
  let loads = 0;
  const layout = createRootRoute({ component: Outlet });
  const routes = ["/a", "/b", "/c"].map((path) =>
    createRoute({
      getParentRoute: () => layout,
      path,
      loader: () => {
        loads++;
      },
      component: () => {
        mounts++;
        onCleanup(() => cleanups++);
        return createComponent(Text, { children: path });
      },
    }),
  );
  const router = createRouter({ routeTree: layout.addChildren(routes), initialEntries: ["/a", "/b", "/c"] });
  const root = createRoot(new MemoryTransport());
  return {
    router,
    counts: () => ({ mounts, cleanups, loads }),
    async start() {
      await router.load();
      if (mounted) {
        root.render(() => createComponent(RouterProvider, { router }));
        await router.load();
      }
      router.history.back({ ignoreBlocker: true });
      await router.load();
    },
    dispose: () => root.unmount(),
  };
}

for (const action of ["push", "replace", "back", "forward", "go"] as const) {
  test(`${action}: cancellation preserves history, route state and mounted owner`, async () => {
    expect(typeof document).toBe("undefined");
    const app = setup();
    const { router } = app;
    try {
      await app.start();
      const location = router.history.location;
      const state = router.state;
      const counts = app.counts();
      let calls = 0;
      let decide!: (blocked: boolean) => void;
      const unblock = router.history.block({
        blockerFn: ({ currentLocation, nextLocation, action: kind }) => {
          calls++;
          expect(currentLocation).toBe(location);
          expect(nextLocation.pathname).toBe(action === "forward" ? "/c" : "/a");
          expect(String(kind)).toBe(action.toUpperCase());
          return new Promise<boolean>((resolve) => {
            decide = resolve;
          });
        },
        enableBeforeUnload: false,
      });
      const navigate = () =>
        action === "push" || action === "replace"
          ? router.navigate({ to: "/a", replace: action === "replace" })
          : action === "go"
            ? router.history.go(-1)
            : router.history[action]();
      const pending = navigate();
      expect(calls).toBe(1);
      expect(router.history.location).toBe(location);
      expect(router.state).toBe(state);
      decide(true);
      await pending;
      expect(router.history.location).toBe(location);
      expect(router.history.length).toBe(3);
      expect(router.state).toBe(state);
      expect(app.counts()).toEqual(counts);

      const notifications: string[] = [];
      const unsubscribe = router.history.subscribe(({ location }) => notifications.push(location.pathname));
      const approved = navigate();
      decide(false);
      await approved;
      await router.load();
      expect(calls).toBe(2);
      expect(notifications).toEqual([action === "forward" ? "/c" : "/a"]);
      expect(router.state.location.pathname).toBe(notifications[0]);
      expect(router.history.location.state.__TSR_index).toBe(
        action === "replace" ? 1 : action === "back" || action === "go" ? 0 : 2,
      );
      unsubscribe();
      unblock();
    } finally {
      app.dispose();
    }
  });
}

test("unmounted typed navigation waits for confirmation and rejects blocker failures", async () => {
  const app = setup(false);
  const { router } = app;
  try {
    await app.start();
    let decide!: (value: boolean) => void;
    const unblock = router.history.block({
      blockerFn: () =>
        new Promise<boolean>((resolve) => {
          decide = resolve;
        }),
    });
    let settled = false;
    const pending = router.navigate({ to: "/a" }).then(() => {
      settled = true;
    });
    await Promise.resolve();
    expect(settled).toBe(false);
    expect(router.state.location.pathname).toBe("/b");
    decide(false);
    await pending;
    expect(router.state.location.pathname).toBe("/a");
    unblock();
    router.history.block({
      blockerFn: () => {
        throw new Error("confirmation failed");
      },
    });
    await expect(router.navigate({ to: "/c" })).rejects.toThrow("confirmation failed");
    expect(router.state.location.pathname).toBe("/a");
  } finally {
    app.dispose();
  }
});

test("new navigation supersedes pending decisions across router and history", async () => {
  const app = setup();
  const { router } = app;
  try {
    await app.start();
    const decisions: ((blocked: boolean) => void)[] = [];
    router.history.block({ blockerFn: () => new Promise<boolean>((resolve) => decisions.push(resolve)) });
    const paths: string[] = [];
    router.history.subscribe(({ location }) => paths.push(location.pathname));
    const first = router.navigate({ to: "/a" });
    const second = router.history.forward();
    await first;
    expect(router.history.location.pathname).toBe("/b");
    decisions[1]!(false);
    await second;
    decisions[0]!(false);
    await Promise.resolve();
    expect(paths).toEqual(["/c"]);

    const third = router.history.back();
    await router.navigate({ to: "/a", ignoreBlocker: true });
    await third;
    decisions[2]!(false);
    await Promise.resolve();
    expect(paths).toEqual(["/c", "/a"]);
    expect(decisions).toHaveLength(3);
    expect(router.state.location.pathname).toBe("/a");
  } finally {
    app.dispose();
  }
});

test("unregister and destroy settle pending navigation without waiting for the dialog", async () => {
  const app = setup();
  const { router } = app;
  try {
    await app.start();
    let decide!: (blocked: boolean) => void;
    const unblock = router.history.block({
      blockerFn: () =>
        new Promise<boolean>((resolve) => {
          decide = resolve;
        }),
    });
    const location = router.history.location;
    const pending = router.navigate({ to: "/a" });
    unblock();
    await pending;
    decide(false);
    await Promise.resolve();
    expect(router.history.location).toBe(location);
    await router.navigate({ to: "/a" });
    expect(router.state.location.pathname).toBe("/a");

    router.history.block({ blockerFn: () => new Promise<boolean>(() => {}) });
    const destroyed = router.navigate({ to: "/c" });
    app.dispose();
    await destroyed;
    expect(router.history.location.pathname).toBe("/a");
    await expect(router.navigate({ to: "/c" })).rejects.toThrow("destroyed native history");
  } finally {
    app.dispose();
  }
});

test("ordered blockers, direct writes, errors and stack boundaries share one transaction", async () => {
  const app = setup(false);
  const { history } = app.router;
  try {
    await app.start();
    const order: number[] = [];
    const first = history.block({
      blockerFn: () => {
        order.push(1);
        return false;
      },
    });
    const second = history.block({
      blockerFn: () => {
        order.push(2);
        return true;
      },
    });
    const third = history.block({
      blockerFn: () => {
        order.push(3);
        return false;
      },
    });
    const location = history.location;
    await history.push("/blocked");
    await history.replace("/blocked");
    expect(order).toEqual([1, 2, 1, 2]);
    expect(history.location).toBe(location);
    second();
    await history.push("/branch?q=1#part", { custom: "kept" });
    expect(order).toEqual([1, 2, 1, 2, 1, 3]);
    expect(history.location).toMatchObject({
      pathname: "/branch",
      search: "?q=1",
      hash: "#part",
      state: { custom: "kept", __TSR_index: 2 },
    });
    expect(history.length).toBe(3);
    const branch = history.location;
    await history.forward();
    expect(history.location).toBe(branch);
    expect(order).toHaveLength(6);
    first();
    third();
    const failure = history.block({
      blockerFn: async () => {
        throw new Error("dialog failed");
      },
    });
    await expect(history.back()).rejects.toThrow("dialog failed");
    expect(history.location).toBe(branch);
    await history.go(-20, { ignoreBlocker: true });
    expect(history.location.pathname).toBe("/a");
    expect(history.canGoBack()).toBe(false);
    await history.go(20, { ignoreBlocker: true });
    expect(history.location).toBe(branch);
    failure();
  } finally {
    app.dispose();
  }
});
