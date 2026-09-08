# 入门

## 安装

```sh
bun install --frozen-lockfile
bun run task --help
bun run check
```

## 创建组件

Solid GPUI 的 runtime 模块提供 Solid 客户端响应式原语，以及通用渲染器的 `createComponent` 函数。

```ts
import { Pressable, Text, View } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";

export function Counter() {
  const [count, setCount] = createSignal(0);
  return createComponent(View, {
    style: { padding: 24, gap: 12 },
    get children() {
      return [
        createComponent(Text, { children: () => `Count: ${count()}` }),
        createComponent(Pressable, {
          accessibilityRole: "button",
          accessibilityLabel: "Increment",
          onPress: () => setCount((value) => value + 1),
          children: createComponent(Text, { children: "Increment" }),
        }),
      ];
    },
  });
}
```

getter 和函数形式的子内容会建立响应式读取。当 `count` 改变时，Solid 渲染 effect 更新原始文本 Host Node，根节点发送一个增量 Patch。

## 挂载根节点

```ts
import { createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { Counter } from "./counter";

const root = createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
root.render(() => createComponent(Counter, {}));
```

向 `root.render` 传入创建函数，使组件在根节点的 Solid owner 下创建，并将后续响应式工作绑定到正确的原生 surface。

## JSX

JSX 是可选的。`@solid-gpui/core/vite`、仓库 Bun preload 和 `solid-gpui-build` 共用基于 Oxc 的官方 Solid universal 转换，并使用 `@solid-gpui/core/runtime`。TypeScript 配置中将 `jsx` 设为 `preserve`，将 `jsxImportSource` 设为 `@solid-gpui/core`，后者选择宿主元素类型。编译器固定为 `@solidjs/compiler` 2.0.0-rc.6，应用运行时仍为 Solid 1.9.15。

以 Rust 为主、使用 QuickJS 的应用应从 `@solid-gpui/core/embedded` 导入 `EmbeddedTransport`，构建自包含的 ESM 入口。参见[运行时选择](runtimes.md)和[生产构建](hot-reload.md)。

## 在桌面运行网站

在工作区根目录执行 `bun run website:native`，通过 Bun 构建并启动网站；执行 `bun run website:native:dev` 使用原生热重载。在 `examples/website` 内，`bun run dev` 启动 Web 版本，`bun run dev:native` 启动原生版本。两者共用应用与 `@solid-gpui/router` 路由树，参见[热重载](hot-reload.md)。

## 无显示环境测试

```ts
import { MemoryTransport, View, createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

const transport = new MemoryTransport();
const root = createRoot(transport);
root.render(() => createComponent(View, {}));
console.log(transport.submitted.length); // Snapshot frame
root.unmount();
```

`MemoryTransport` 用于验证带帧渲染输出和根命令契约。原生布局、绘制、对话框和平台窗口行为需要有显示环境的宿主验证。
