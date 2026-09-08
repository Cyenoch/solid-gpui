# 选择运行时

Solid GPUI 提供三种原生运行模式，共用 Solid 组件 API、原生控件与 Rust 渲染引擎。选择取决于应用能力由谁实现，以及开发、交付方式。

## 运行时比较

| 项目 | 外部 Bun | 内嵌 Bun | QuickJS |
| --- | --- | --- | --- |
| 主要用途 | 快速开发 | Bun 应用的生产打包 | Rust 能力与 JSX/TSX 界面 |
| JavaScript 运行位置 | 子进程 | 原生应用中的专用线程 | 原生应用中的专用线程 |
| 文件与网络 | Bun 服务或 Rust 命令 | Bun 服务或 Rust 命令 | Rust 命令 |
| 原生界面 | GPUI | GPUI | GPUI |
| 传输 | 二进制 stdio | 原生帧桥接 | 原生帧桥接 |
| 用户端运行时 | Bun 可执行文件 | 内嵌 Bun/JSC 库 | 可执行文件中内嵌 QuickJS |
| 当前平台验证 | 需检查目标桌面 | macOS 内嵌 | macOS、Linux、Windows 候选包；桌面验证另行进行 |

## 使用外部 Bun 开发

外部 Bun 提供较短的开发反馈周期。在本仓库中执行：

```sh
bun run website:native:dev
```

Vite 监听应用依赖，在现有原生窗口内替换应用。当前路由、搜索条件等状态通过 `captureState` 保留，组件局部信号与原生编辑缓存重新挂载。参见[热重载](hot-reload.md)。

Bun 可以实现业务逻辑与服务，也可以调用 Rust Native Module。生产版本可复用相同应用组合，改用内嵌 Bun 入口。

## 使用内嵌 Bun 打包

依赖 Bun 服务的应用以内嵌 Bun 为生产交付方向。它使用 `EmbeddedTransport`，宿主必须启用 `embedded-bun` Cargo feature。运行时在原生进程内执行，但不会与 GPUI 线程共享可变 JS 对象。

首次构建需要编译固定版本、带补丁的 Bun/JSC 库，工作量可能远高于界面重建，后续构建应复用受支持的缓存。当前内嵌目标为 macOS。现有 website 最终归档流水线使用 QuickJS，不能据此认定内嵌 Bun 已通过发布验证。参见[分发应用](distribution.md)和[运行时策略](runtime-strategy.md)。

## 使用 QuickJS，让 Rust 实现应用能力

文件、网络、领域操作和长时间任务由 Rust 实现时，选择 QuickJS。JSX/TSX 仍负责信号、事件处理、路由及展示状态。通过[生成的 Native Module](rust-bridge.md) 暴露 Rust 操作。

QuickJS 提供定时器、Promise 和原生路由所需的平台原语，但不提供 Bun、Node 模块、DOM 或网络 `fetch`。JavaScript 依赖应预先打包，不在运行时加载外部包。

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";

mountApplication({
  transport: () => new EmbeddedTransport(),
  setup: () => ({ render: () => <Text>Hello from Rust and Solid</Text> }),
});
```

构建生产入口：

```sh
node_modules/.bin/solid-gpui-build --runtime quickjs src/app.tsx dist/app.js
```

使用真实 QuickJS 引擎开发时，在应用的工作区根清单配置[解释器开发 profile](hot-reload.md#application-build-configuration)，构建启用 `quickjs` 的原生宿主，然后运行：

```sh
node_modules/.bin/solid-gpui-quickjs-dev src/app.tsx target/debug/my-app
```

外部开发工具监听并打包代码，宿主准备新的 QuickJS VM，验证替代应用后再切换。Rust 服务和原生窗口保持存活。显式捕获的状态以有界 JSON 数据跨代传递。Rust、协议及构建配置变化需要重启开发命令。参见[重载生命周期](hot-reload.md)。

## 性能与所有权

内嵌执行消除了子进程管道，但队列与二进制编码仍有成本。两种内嵌引擎都与 Rust 交换具有独立所有权的帧；原生命令提供类型化结果，不把 GPUI 句柄暴露给 JavaScript。原生消费者落后时，背压暂停事件驱动的生产，所有待处理队列保持有界。

切换运行时不会自动提升滚动或帧率。GPUI 仍负责原生布局和绘制，部分滚动完全不进入 JS。按照[性能指南](performance-analysis.md)，分别测量启动、总内存、JS 计算和输入到呈现延迟。
