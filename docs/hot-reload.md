# Vite + Bun 原生开发

Vite 8 负责 universal JSX 转换、模块图、文件监听、HMR 和生产 bundling。
Bun 执行 Vite 的 RunnableDevEnvironment/ModuleRunner 和应用 JavaScript。
GPUI host 保留原生窗口，通过同一 stdio 连接接收新 epoch 的 snapshot。
这是原生应用的模块执行环境；不需要 HTML、DOM 或 WebView。

## 仓库开发

`examples/gallery` 是独立的 Bun Gallery 项目，拥有全部通用页面、状态和应用挂载。
`examples/gallery-vite` 只提供 Vite 配置和入口，通过 workspace 包导入前者。
Vite 依赖属于 Vite Gallery 项目，workspace 根目录只负责统一任务和安装。
普通版本运行 `bun run gallery`；也可进入任一项目运行 `bun run dev`。

- `bun run gallery:vite`：构建所需包并启动 host + Vite/Bun Gallery。
- `bun run --cwd examples/gallery-vite build`：Vite 生成 `examples/gallery-vite/dist/main.js`。
- `target/debug/gallery-host bun --conditions=browser examples/gallery-vite/dist/main.js`：运行生产 bundle（host 需先构建）。

应用代码保存后原地重挂载。Rust、协议 schema、生成的 native API 或 Vite 配置变化需要重启开发命令；Rust 变化需要重新构建 host。

## 应用接入

安装 `@solid-gpui/core` 和 Vite 8，创建配置：

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/core/vite";
export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx" })],
});
```

让已有 native host 启动 `node_modules/.bin/solid-gpui-dev src/app.tsx`。
该入口使用 Bun（包含 `browser` condition），加载当前目录 Vite 配置；第二个参数可指定配置文件。
也可在自己的 Bun 启动器调用 `startDev(entry, configFile?)`，其返回值用于关闭 Vite。
插件包直接发布 TypeScript 源码，面向 Bun 运行环境。

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { createSignal } from "solid-js";

mountApplication<number>({
  hotKey: import.meta.hot ? import.meta.url : undefined,
  setup(previous = 0) {
    const [count] = createSignal(previous);
    return {
      render: () => <Text>{count()}</Text>,
      captureState: () => count(),
    };
  },
});
```

插件为配置中的入口注入 HMR accept。使用 `/// <reference types="vite/client" />` 声明 `import.meta.hot`。
库源码的开发映射参考`examples/gallery-vite/vite.config.ts`；用户应用通常直接使用包导出。

## 状态与失败边界

这是应用级重挂载，不是自动保留每个组件 signal 的 Fast Refresh。
`captureState` 返回可 structured-clone 的数据，下一代 `setup` 接收它。
Gallery 保存路由、主题、搜索和窗口尺寸；组件局部状态、原生输入和滚动缓存重建。
不要跨代保存 Solid owner、Root、router 实例、函数或 native 资源。

新一代 setup/render 和首帧准备成功后，旧 owner/root 释放，发布同 surfaceId 的新 epoch。
语法错误或同步 setup/render 错误不会卸载旧页面；修复后下一次保存恢复。
第一次启动失败会关闭 Vite 并退出，因为没有上一版可展示。
`onMount` 在提交后执行，其错误和异步副作用不属于回滚边界；I/O 故障也不保证回滚。
应用顶层副作用在候选准备之外，资源应放在 `setup`/组件里，用 `onCleanup` 释放。
晚到的异步结果应检查 disposed 状态，避免卸载后重新创建 timer。

默认 stdio transport 跨代复用，最终 dispose 时关闭；自定义 transport 由调用方拥有。
同一 hotKey 的旧 handle 在替换后失效。手动 dispose 后重新打开已退休的 native surface 不属于 HMR。

## 防止再次踩坑

- Solid 的 universal renderer 需要客户端响应式实现。Vite SSR 的 `conditions` 和 `externalConditions` 都配置 `browser`，Bun 也以该 condition 启动。
- 使用 Vite ModuleRunner 的 HMR；`server.hmr: false` 也会关闭服务端 HMR。不要叠加 Bun `--hot` 或浏览器 HMR client。
- stdout 仅发送带长度前缀的协议字节。Vite 和应用诊断使用 stderr（`console.error`）。
- 原生构建目录 `target` 不参与 watch；禁用没有 HTML 入口意义的自动依赖发现，避免扫描嵌入式 Bun vendor 测试数据。
- 自动验收实际保存 TSX 依赖、语法失败、渲染失败及恢复，解码整个 stdout，并检查 epoch/显式状态和 owner 清理。只测试 transform 不能证明 HMR 成立。

关键测试：`scripts/hot-reload.test.ts`、`packages/solid-gpui/tests/application.test.ts`。
参考：[Vite framework Environment API](https://vite.dev/guide/api-environment-frameworks)、[Vite runtime API](https://vite.dev/guide/api-environment-runtimes)。
