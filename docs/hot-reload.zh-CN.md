# 使用 Vite 与 Bun 开发原生应用

外部 Bun 用于快速迭代；内嵌 Bun 是 Bun 应用的预期生产打包运行时；QuickJS 是以 Rust 为主应用的界面运行时。定位与当前交付支持的区别见[运行时策略](runtime-strategy.md)。

Vite 8 管理模块图、文件监听、HMR 和生产打包。插件使用基于 Oxc 的官方 `@solidjs/compiler` 2.0.0-rc.6 编译 universal JSX，使用 `oxc-transform` 0.148.0 转换 TypeScript。应用运行时仍为 Solid 1.9.15。Bun 执行 Vite RunnableDevEnvironment/ModuleRunner 和应用 JavaScript。GPUI 宿主保留原生窗口，通过现有 stdio 连接接收新 epoch 的 Snapshot。原生模块环境不需要 HTML、DOM 或 WebView。

## 仓库开发

`examples/website` 拥有共享网站与原生应用，桌面入口和 Vite 配置在原生窗口中运行 Components 与 Showcase。

- `bun run website:native:dev`：构建所需包，以 Vite/Bun 启动宿主。
- `bun run --cwd examples/website build:native`：生成 `examples/website/dist-native/main.native.js`。
- `target/debug/website-host bun --conditions=browser examples/website/dist-native/main.native.js`：构建宿主后运行 bundle。

保存应用代码会在现有窗口重新挂载。Rust、协议 schema、生成的原生 API 或 Vite 配置变化需重启开发命令；Rust 变化也需重建宿主。

## 应用集成

安装 `@solid-gpui/core` 和 Vite 8，创建配置：

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/core/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx" })],
});
```

让现有原生宿主启动 `node_modules/.bin/solid-gpui-dev src/app.tsx`。入口使用带 browser condition 的 Bun，从当前目录加载 Vite 配置；第二参数可指定配置文件。自定义 Bun 启动器可调用 `@solid-gpui/core/vite/dev` 的 `startDev(entry, configFile?)`，返回值提供 Vite 关闭方法。插件发布供 Bun 执行的 TypeScript 源码。

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";

mountApplication<number>({
  transport: () => new StdioTransport(),
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

插件为配置入口注入 HMR 接受逻辑。添加 `/// <reference types="vite/client" />` 声明 `import.meta.hot`。库源码开发别名见 `examples/website/vite.native.config.ts`，普通应用通常使用包导出。

## 状态与失败边界

重载会重新挂载应用，不自动保留各组件信号。`captureState` 返回可 structured clone 的数据，下一代 setup 接收它。网站保留路由、语言和窗口尺寸，组件状态、原生输入与滚动缓存重建。不要跨代保留 Solid owner、Root/router 实例、函数或原生资源。

候选 setup、render 和首帧准备成功后，释放旧 owner 与 root，在相同 surface ID 发布新 epoch。语法错误和同步 setup/render 错误保留旧页面，修复并保存后重试。首次启动失败因没有旧页面而关闭 Vite 并退出。

`onMount` 在提交后运行，其错误和异步副作用不在回滚边界内；I/O 失败也不保证回滚。应用顶层副作用发生在候选准备外。资源应在 setup 或组件中创建，通过 `onCleanup` 释放。迟到异步结果应在重建 timer 前检查是否已释放。

必需的 transport factory 只创建一次应用连接，跨重载保留，最终释放时关闭，自定义 transport 同样如此。替换 generation 使同一 hotKey 的旧句柄失效。手动释放后重开已退役 surface 不属于 HMR 生命周期。

## 生产 bundle

`solid-gpui-build` 与 Vite、Bun preload 共用 Solid/Oxc 转换。编译器在构建阶段的 Bun 中执行，不进入 QuickJS。显式选择目标：

```sh
node_modules/.bin/solid-gpui-build --runtime bun src/app.tsx dist/app.js
```

Bun 入口使用 StdioTransport，由现有宿主和 `bun --conditions=browser dist/app.js` 运行。内嵌 Bun 使用 EmbeddedTransport，要求 embedded-bun feature，并应提供导入共享应用组合的独立入口。

以 Rust 为主、使用 QuickJS 的应用采用内嵌传输和共享组件的独立入口：

```tsx
import { mountApplication } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { App } from "./app";

mountApplication({
  transport: () => new EmbeddedTransport(),
  setup: () => ({ render: () => <App /> }),
});
```

构建自包含 ESM，再启动启用 QuickJS 的宿主：

```sh
node_modules/.bin/solid-gpui-build --runtime quickjs src/quickjs.tsx dist/app.js
cargo run -p solid-gpui --features quickjs --bin solid-gpui-host -- --runtime quickjs dist/app.js
```

通用宿主命令运行核心宿主组件。使用自定义 Native Module 或可选 gpui-component 集成的应用，应启动启用对应模块与 feature 的自身宿主。

QuickJS 无环境 Bun/Node 服务，不在运行时加载外部包。打包 JavaScript 依赖，将文件和网络等服务放在 Rust Native Module。定时器与原生命令 Promise 支持界面工作，但它不是浏览器或完整 Bun 环境。构建初始化原生路由需要的 URL/search、事件及取消原语。Response 故意不带 body，仅承载重定向元数据，非 null body 会被拒绝，不提供 Fetch body 方法或网络 fetch。

`AbortSignal.any` 合并本地取消来源（包括 timeout），保留第一个取消原因。嵌套组合在来源 listener 执行前完成，重入取消和停止事件传播不会改变结果。待处理组合由来源保留，取消时从全部来源解绑。清理时 abort 操作拥有的 controller，避免在应用级 controller 上积累待处理组合。Vite 模块 HMR 在外部 Bun 执行；QuickJS 开发使用下述整应用重载，生产只执行一个已构建入口。

## QuickJS 应用重载

在真实 QuickJS 引擎中迭代 Rust 主导的界面：

```sh
bun run quickjs:dev # Counter demo in the actual QuickJS engine
```

其他应用应编译启用 quickjs 的宿主，再执行 `solid-gpui-quickjs-dev src/quickjs.tsx target/debug/my-app`。入口使用 EmbeddedTransport 和 mountApplication。Vite 在 VM 外监听文件，现有 QuickJS 打包器执行与生产一致的平台和不支持导入检查。独立于 UI 协议的 loopback 开发连接传输有界 bundle。QuickJS 内不安装浏览器客户端、WebSocket、fetch 或 Vite ModuleRunner。

构建成功后创建新 VM，保留 Rust 宿主、窗口和服务。旧 VM 在微任务检查点暂停并导出显式 captureState。宿主验证每个候选 Surface 及应用配置，再在前台切换路由。旧 epoch 输入与回复不会调用新一代。原生缓存和组件局部信号重新挂载，替换后重新发送当前窗口观测值。

QuickJS 捕获状态必须是无环 JSON：普通对象、稠密数组、字符串、布尔值、有限数值和 null。访问器、隐藏属性、symbol、函数、非普通对象、稀疏数组、负零及非有限数被拒绝。预算为 1 MiB、100,000 个访问值、64 层嵌套。未提供捕获值与显式 null 不同。Bun 同 VM HMR 仍使用 structured clone 契约。

候选准备接受初始 Snapshot 与应用配置。原生副作用放在激活后的 onMount，暂存期间不能任意调用原生命令。有效 loading view 可作为首帧，路由加载可以在激活后完成。准备截止时间三秒，失败或过期候选被丢弃，旧代恢复。重建请求以最新为准，只保留一个待处理 bundle 和一个候选。

候选必须准确描述当前已打开的 Surface 集合。setup 中创建的辅助根应使用稳定显式 Surface ID、共用连接，默认 epoch 来自宿主 generation。在应用 owner 注册清理，在 captureState 中包含辅助界面状态。暂存时开关窗口或改变窗口集合会拒绝候选，不进行部分替换。零窗口 keep-alive 重载保留连接与激活确认序列。

激活后异步错误和原生副作用不回滚。开发 VM 失败时最后原生树仍可见，后续编辑可用最后捕获状态与当前激活元数据恢复。捕获状态只是检查点，不保证保留之后的变化。Rust、配置及原生契约变化仍需重启。

## 外部 Bun 运行约束

- 通用渲染器需要客户端响应式实现；Vite SSR 的 conditions 和 externalConditions 均配置 browser，Bun 也使用该 condition。
- 使用 Vite ModuleRunner HMR；`server.hmr: false` 同时禁用服务端 HMR。不要叠加 Bun --hot 或浏览器 HMR 客户端。
- stdout 只放带长度前缀的协议字节，Vite 与应用诊断用 console.error 写 stderr。
- 不监听原生 target 目录。禁用自动依赖发现，因为没有 HTML 入口，自动扫描可能进入内嵌 Bun vendor 测试数据。
- 集成验证必须保存 TSX 依赖、引入语法和渲染失败、恢复、解码完整 stdout，并检查 epoch、显式状态及 owner 清理。仅转换测试不能证明 HMR 正常。

关键测试：`scripts/hot-reload.test.ts` 和 `packages/solid-gpui/tests/application.test.ts`。参考 [Vite Environment API](https://vite.dev/guide/api-environment-frameworks) 与 [Vite runtime API](https://vite.dev/guide/api-environment-runtimes)。

## 文件路由

在 `solidGpui()` 之前添加来自 `@solid-gpui/router/vite` 的 `solidGpuiRouter()`。插件在应用加载前生成路由树，并跟随文件新增、修改、重命名和删除更新。接入方式及平台差异见 [Router](router.zh-CN.md)。
