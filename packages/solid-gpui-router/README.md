# @solid-gpui/router

TanStack Router Core integration for Solid GPUI, with native links and
memory history for each surface.

## File-based routing

Use `solidGpuiRouter()` from `@solid-gpui/router/vite` before the Solid GPUI JSX
plugin. It uses TanStack's generator to turn `src/routes` into a typed
`src/routeTree.gen.ts`, including updates when route files move. Import runtime
APIs such as `createFileRoute` and `createLazyFileRoute` from `@solid-gpui/router`.
For non-Vite builds and standalone type checking, run `generateRoutes()` from
`@solid-gpui/router/generator` first.

The generator processes file events in order, including deletions arriving
alongside notifications for unchanged generated files.

Follow the [Router guide](../../docs/router.md) for installation, route files,
native navigation, lazy routes, and platform limits. Refer to the
[TanStack Router Solid documentation](https://tanstack.com/router/latest/docs/framework/solid)
for general routing patterns.

## Routing and shared application state

Create one router per native surface. Each router owns its location, history,
params, search state, and route lifecycle. Routers may reuse the same finalized
route tree; do not mutate route options or children after creating the first
router.

Application-global data is separate from routing. Create stores, query clients,
and services once in the application's JavaScript runtime, then pass the same object
references through each router context. Add per-window dependencies such as
`windowId` beside that shared object:

```ts
import { Outlet, createRootRouteWithContext, createRoute, createRouter } from "@solid-gpui/router";

interface RouterContext {
  app: {
    session: { userId: string | undefined };
  };
  windowId: string;
}

const app: RouterContext["app"] = {
  session: { userId: undefined },
};
const rootRoute = createRootRouteWithContext<RouterContext>()({ component: Outlet });
const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
});
const routeTree = rootRoute.addChildren([homeRoute]);

export function createWindowRouter(windowId: string) {
  return createRouter({
    routeTree,
    context: { app, windowId },
  });
}
```

Solid contexts belong to one owner root and therefore do not cross windows.
The shared `app` reference above is the explicit application scope; each
router context remains a distinct window scope. TanStack Query, if used, stays
a separate data-cache concern and can be placed in `app` rather than coupled to
navigation.

See the [shared website router](../../examples/website/src/router.tsx) for a complete
application and the [website guide](../../examples/website/README.md) for browser
hash URLs and native navigation.
