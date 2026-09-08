# 路由

`@solid-gpui/router` 将 TanStack Router Core 接入 Solid GPUI。新应用推荐使用文件路由：文件定义路由层级，构建工具生成路由树及 TypeScript 类型关系。

本页介绍 Solid GPUI 的接入方式和原生平台差异。通用概念与应用模式请参考 [TanStack Router Solid 文档](https://tanstack.com/router/latest/docs/framework/solid)，尤其是[文件路由](https://tanstack.com/router/latest/docs/framework/solid/routing/file-based-routing)、[文件命名约定](https://tanstack.com/router/latest/docs/framework/solid/routing/file-naming-conventions)、[数据加载](https://tanstack.com/router/latest/docs/framework/solid/guide/data-loading)、[搜索参数](https://tanstack.com/router/latest/docs/framework/solid/guide/search-params)和[路由上下文](https://tanstack.com/router/latest/docs/framework/solid/guide/router-context)。移植示例时，使用 `@solid-gpui/router` 导入和原生组件。

## 配置生成器

在现有 Solid GPUI 应用中安装：

```sh
bun add @solid-gpui/router
```

在 universal JSX 插件之前配置路由插件：

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

默认读取 Vite 根目录下的 `src/routes`，生成 `src/routeTree.gen.ts`。插件会在模块解析前和生产构建时运行，并跟随文件新增、修改、重命名和删除更新路由树。浏览器 GPUI 宿主也可使用同一个插件，不要在同一目录同时启用 TanStack 的网页框架路由插件。

文件事件按顺序处理；即使路由删除与生成文件的无变化通知同时到达，也会更新路由树，不会遗漏删除事件。

可配置 `routesDirectory`、`generatedRouteTree`、`routeFilePrefix`、`routeFileIgnorePrefix`、`routeFileIgnorePattern`、`indexToken`、`routeToken`、`quoteStyle` 和 `semicolons`。命名 token 使用字符串；通常保留默认约定即可。

独立类型检查或 面向 Bun/QuickJS 的 Vite 构建前可以直接生成：

```ts
// generate-routes.ts
import { generateRoutes } from "@solid-gpui/router/generator";

await generateRoutes({ root: import.meta.dirname });
```

```sh
bun generate-routes.ts
bunx tsc --noEmit
```

提交 `routeTree.gen.ts`，并将它排除在格式化和 lint 范围之外；CI 类型检查前重新生成。保留路由文件中的具名 `Route` 导出。生成器会在文件移动后更新 `createFileRoute(...)` 中的 ID，并为空文件生成适用于原生环境的骨架。应用运行时代码仅导入主入口，构建工具使用 `/vite` 和 `/generator`。

## 定义文件路由

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

根路由使用 `createRootRoute`，并通过 `Outlet` 渲染子路由：

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

普通页面通过 `createFileRoute` 定义：

```tsx
// src/routes/index.tsx
import { Text } from "@solid-gpui/core";
import { createFileRoute } from "@solid-gpui/router";

export const Route = createFileRoute("/")({
  component: () => <Text>Home</Text>,
});
```

`posts.tsx` 定义 `/posts` 布局并渲染 `Outlet`。`posts.$postId.tsx` 定义带动态参数的子页面：

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

目录与点号命名可以混合使用，也支持 index、`$param`、`$` splat、`_` 无路径布局、路由分组与非嵌套路由。完整规则以 TanStack 命名文档为准。将侧栏和持久状态放入布局路由，让子页面切换只替换 `Outlet` 以下的内容。

## 创建并挂载 router

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

每个 surface 创建独立 router，通过 `RouterProvider` 挂载。例如外部 Bun 应用：

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

Embedded Bun 和 QuickJS 使用应用对应的 transport 与挂载生命周期；路由树和 Provider 相同。先生成，再打包。多个窗口可共享已完成的路由树，但 router 与 history 必须独立；创建第一个 router 后不再修改路由树。

应用级服务在运行时创建一次，将相同引用传入各窗口的 router `context`，并用 `createRootRouteWithContext` 约束类型。Solid context 不会跨 surface owner 共享。参见[路由与应用共享状态](../packages/solid-gpui-router/README.md#routing-and-shared-application-state)。

## 导航与状态

`Link` 渲染原生 `Pressable`。`useNavigate` 返回导航函数；`BackButton` 和 `useCanGoBack` 使用当前 surface 的历史。

```tsx
<Link to="/posts/$postId" params={{ postId: "42" }}>
  Open post
</Link>
```

`useLocation`、`useRouterState` 和 `useMatch` 返回 Solid accessor，应在 JSX 或响应式计算中调用。通过 `useMatch({ from: routeId })` 读取 `params`、`search`、`loaderData`、`loaderDeps` 和 `context`。当前适配层使用这些 hook，不提供 TanStack Solid 的 `Route.useParams()` 或 `Route.useLoaderData()` 等路由实例方法。

loader、`beforeLoad`、搜索参数验证、重定向和 not-found 遵循 TanStack Router Core。从 `@solid-gpui/router` 导入 `redirect` 与 `notFound`。路由可提供原生 `pendingComponent`、`errorComponent` 和 `notFoundComponent`，未提供时使用内置原生状态视图。

## 延迟加载与平台边界

将数据选项保留在普通路由文件，UI 选项移入匹配的 `.lazy.tsx` 文件，并从普通文件中移除 `component`：

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

Vite 是否拆分代码取决于应用构建配置。QuickJS 交付必须将路由代码包含在自包含 bundle 中。适配层没有开放 TanStack 的自动组件抽取选项 `autoCodeSplitting`。

原生 router 使用 memory history，禁用浏览器文档跳转、DOM 滚动恢复、view transitions 与 hover 预加载。外部 URL 通过 native root 的 `openUrl` 打开。网站在浏览器入口同步 hash URL，桌面应用无需该桥接。TanStack 的 DOM 组件、SSR/hydration 和浏览器 devtools 不属于原生适配层。

代码路由仍可通过 `createRootRoute`、`createRoute` 和 `addChildren` 定义。[网站路由源码](../examples/website/src/routes/__root.tsx)是浏览器与桌面共用的文件路由实例。
