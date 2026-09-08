# Router

`@solid-gpui/router` brings TanStack Router Core to Solid GPUI. Use file-based
routing for new applications: route files define the hierarchy, and the build
step generates the route tree and its TypeScript relationships.

Start here for Solid GPUI setup and native behavior. For routing concepts and
application patterns, use the [TanStack Router Solid documentation](https://tanstack.com/router/latest/docs/framework/solid).
In particular, read [file-based routing](https://tanstack.com/router/latest/docs/framework/solid/routing/file-based-routing),
[file naming conventions](https://tanstack.com/router/latest/docs/framework/solid/routing/file-naming-conventions),
[data loading](https://tanstack.com/router/latest/docs/framework/solid/guide/data-loading),
[search parameters](https://tanstack.com/router/latest/docs/framework/solid/guide/search-params),
and [router context](https://tanstack.com/router/latest/docs/framework/solid/guide/router-context).
Use `@solid-gpui/router` imports and native components when adapting those examples.

## Configure generation

Install the router alongside your existing Solid GPUI application:

```sh
bun add @solid-gpui/router
```

Add `solidGpuiRouter()` before the universal JSX plugin in your Vite configuration:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";
import { solidGpuiRouter } from "@solid-gpui/router/vite";

export default defineConfig({
  plugins: [
    solidGpuiRouter(),
    solidGpui({ entry: "src/app.tsx" }),
  ],
});
```

The defaults are `src/routes` and `src/routeTree.gen.ts`, relative to Vite's root.
The plugin generates before module resolution, follows file additions, edits,
renames, and deletions during development, and runs during production builds.
File events are processed in order; a notification for an unchanged generated
tree cannot swallow a route deletion arriving at the same time.
The same plugin works with the browser GPUI host's universal JSX setup.
Do not also install TanStack's web-framework router plugin for the same directory.

To change the paths, pass `routesDirectory` and `generatedRouteTree` to
`solidGpuiRouter`. Other options include `routeFilePrefix`, `routeFileIgnorePrefix`,
`routeFileIgnorePattern`, `indexToken`, `routeToken`, `quoteStyle`, and `semicolons`.
The naming tokens are strings. Prefer the defaults unless your project needs a
different convention.

Generation is also available before standalone type checks or Vite builds targeting Bun or QuickJS:

```ts
// generate-routes.ts
import { generateRoutes } from "@solid-gpui/router/generator";

await generateRoutes({ root: import.meta.dirname });
```

```sh
bun generate-routes.ts
bunx tsc --noEmit
```

Commit `routeTree.gen.ts` so editors and clean checkouts have route types. Exclude
it from your formatter and linter, and regenerate it before checking types in CI.
The generator manages route IDs inside `createFileRoute(...)` when files move.
Keep the named `Route` export. Empty route files receive a native-safe scaffold.
Build tooling stays in the `/vite` and `/generator` entry points; applications
only import the runtime entry point.

## Define routes

A small application can use this structure:

```text
src/
  app.tsx
  router.ts
  routeTree.gen.ts
  routes/
    __root.tsx
    index.tsx
    posts.tsx
    posts.$postId.tsx
```

Keep the shared shell in the root route and render children through `Outlet`:

```tsx
// src/routes/__root.tsx
import { View } from "@solid-gpui/core";
import { Link, Outlet, createRootRoute } from "@solid-gpui/router";

export const Route = createRootRoute({
  component: () => (
    <View style={{ flexGrow: 1, minHeight: 0, gap: 12 }}>
      <Link to="/">Home</Link>
      <Outlet />
    </View>
  ),
});
```

```tsx
// src/routes/index.tsx
import { Text } from "@solid-gpui/core";
import { createFileRoute } from "@solid-gpui/router";

export const Route = createFileRoute("/")({
  component: () => <Text>Home</Text>,
});
```

```tsx
// src/routes/posts.tsx
import { Outlet, createFileRoute } from "@solid-gpui/router";

export const Route = createFileRoute("/posts")({ component: Outlet });
```

A loader receives typed route parameters, validated search, and router context.
Read its result reactively with `useMatch`:

```tsx
// src/routes/posts.$postId.tsx
import { Text } from "@solid-gpui/core";
import { createFileRoute, useMatch } from "@solid-gpui/router";

export const Route = createFileRoute("/posts/$postId")({
  loader: ({ params }) => ({ title: `Post ${params.postId}` }),
  component: PostPage,
});

function PostPage() {
  const match = useMatch({ from: "/posts/$postId" });
  return <Text>{match().loaderData?.title}</Text>;
}
```

Directories and dotted names can be mixed. TanStack's generator also handles
index routes, `$param` segments, `$` splats, `_` pathless layouts, route groups,
and non-nested routes. See the upstream naming guide for their full semantics.
Keep sidebars and other persistent state in layout routes; navigating between
children should replace only the content below their `Outlet`.

## Create and mount the router

Import the generated tree and register your router's type for navigation and
match inference:

```ts
// src/router.ts
import { createRouter } from "@solid-gpui/router";
import { routeTree } from "./routeTree.gen";

export function createWindowRouter(initialPath = "/") {
  return createRouter({ routeTree, initialEntries: [initialPath] });
}

declare module "@solid-gpui/router" {
  interface Register {
    router: ReturnType<typeof createWindowRouter>;
  }
}
```

Mount one router per surface. For an external Bun application:

```tsx
// src/app.tsx
import { mountApplication } from "@solid-gpui/core";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { RouterProvider } from "@solid-gpui/router";
import { createWindowRouter } from "./router";

mountApplication<string>({
  hotKey: import.meta.hot ? import.meta.url : undefined,
  transport: () => new StdioTransport(),
  setup(previousPath = "/") {
    const router = createWindowRouter(previousPath);
    return {
      render: () => <RouterProvider router={router} />,
      captureState: () => router.state.location.href,
    };
  },
});
```

For Embedded Bun or QuickJS, use your application's transport and mount lifecycle;
the route tree and `RouterProvider` stay the same. Generate routes before bundling.
Routers can share the finalized tree, but must have separate histories and router
instances. Do not mutate the tree after creating the first router.

To share application services across windows, create them once and pass the same
references through each router's `context`. Use `createRootRouteWithContext` to
type that context. Solid context owners do not cross surface roots. See
[Routing and shared application state](../packages/solid-gpui-router/README.md#routing-and-shared-application-state).

## Navigate and read state

`Link` renders a native `Pressable`. `useNavigate` returns a navigation function;
`BackButton` and `useCanGoBack` use the current surface's history.

```tsx
<Link to="/posts/$postId" params={{ postId: "42" }}>
  Open post
</Link>
```

`useLocation`, `useRouterState`, and `useMatch` return Solid accessors. Call the
accessor inside JSX or a reactive computation. `useMatch({ from: routeId })`
exposes `params`, `search`, `loaderData`, `loaderDeps`, and `context`. The adapter
currently uses these hooks instead of TanStack Solid's route-bound helpers such
as `Route.useParams()` and `Route.useLoaderData()`.

Loaders, `beforeLoad`, validated search, redirects, and not-found handling use
TanStack Router Core. Import `redirect` and `notFound` from `@solid-gpui/router`.
Routes can provide native `pendingComponent`, `errorComponent`, and
`notFoundComponent`; built-in native views cover the default states.

## Lazy routes and platform boundaries

To defer a route component, keep data options in its normal route file and put
UI options in a matching `.lazy.tsx` file:

```tsx
// src/routes/posts.$postId.lazy.tsx
import { Text } from "@solid-gpui/core";
import { createLazyFileRoute, useMatch } from "@solid-gpui/router";

export const Route = createLazyFileRoute("/posts/$postId")({
  component: () => {
    const match = useMatch({ from: "/posts/$postId" });
    return <Text>{match().loaderData?.title}</Text>;
  },
});
```

Remove `component` from the corresponding non-lazy file. Vite may split the
result according to the application's bundling configuration. QuickJS delivery
must include all route code in its self-contained bundle. Automatic TanStack
component extraction (`autoCodeSplitting`) is not exposed by this adapter.

The native router uses memory history. It disables browser document navigation,
DOM scroll restoration, view transitions, and hover preloading. Open external
URLs through the native root's `openUrl` API. The website synchronizes its router
with browser hash URLs in its web entry; desktop applications do not need that
bridge. TanStack's DOM components, SSR/hydration, and browser devtools are not
part of the native adapter.

Code-defined routes remain available through `createRootRoute`, `createRoute`,
and `addChildren`. The [website source](../examples/website/src/routes/__root.tsx)
is a working file-based example shared by browser and desktop builds.
