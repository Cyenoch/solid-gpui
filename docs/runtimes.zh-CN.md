# 选择运行时

Solid GPUI 的三种原生运行模式共享 Solid 组件 API、原生控件与 Rust 渲染引擎。根据应用服务由谁实现、如何交付来选择。JavaScript 可直接运行；所有运行时的 JSX/TSX 都使用 [Vite](vite.zh-CN.md)。

## 运行时比较

| | 外部 Bun | 内嵌 Bun | QuickJS |
| --- | --- | --- | --- |
| 主要用途 | 快速开发 | Bun 应用的单一可执行文件交付 | Rust 服务与 Solid 界面 |
| JavaScript 运行位置 | 子进程 | 应用内的专用线程 | 应用内的专用线程 |
| 文件与网络 | Bun 服务或 Rust 命令 | Bun 服务或 Rust 命令 | Rust 命令 |
| 传输 | 二进制 stdio | 原生帧桥接 | 原生帧桥接 |
| 用户端运行时 | Bun 可执行文件 | 链接进可执行文件的 Bun/JSC | 链接进可执行文件的 QuickJS |

内嵌 Bun 打包仍为实验性。选择交付方式前，请查阅权威的[平台状态](distribution.zh-CN.md#平台状态与当前证据)。

## 使用外部 Bun 开发

无论领域逻辑在 Bun 还是 Rust，外部 Bun 都适合快速迭代。自己的应用请按[入门](getting-started.zh-CN.md)操作；本仓库运行：

```sh
bun run website:native:dev
```

Vite 在现有原生窗口中替换应用。用 [`captureState`](capture-state.zh-CN.md) 保留显式界面状态；激活和原生重建行为见[热重载](hot-reload.zh-CN.md)。

## 使用内嵌 Bun 打包

交付的应用需要 Bun 服务时，选择内嵌 Bun。打包器将 GPUI、Bun/JSC 与应用声明的模块图链接为单一可执行文件，目标机器无需另装 Bun 或携带 JavaScript 目录。

使用公开 CLI `solid-gpui embedded package` 或 `@solid-gpui/vite/embedded` 的 `packageEmbeddedApplication`。SDK checkout、固定序列化器、Cargo 输入、资源、Worker 和验收要求统一见[分发指南](distribution.zh-CN.md#内嵌-bun-静态应用)。[运行时策略](runtime-strategy.zh-CN.md)说明原生镜像和会话生命周期。

## 使用 QuickJS，让 Rust 实现应用能力

文件、网络、领域操作和长时间任务由 Rust 实现时，选择 QuickJS。Solid 仍负责信号、事件处理、路由与展示状态；Rust 服务通过 [Native Module](rust-bridge.zh-CN.md) 暴露。

QuickJS 提供定时器、Promise 和原生路由所需的平台原语，不提供 Bun 或 Node 模块、DOM 或网络 `fetch`。预先打包 JavaScript 依赖，在 Vite 插件中选择 `runtime: "quickjs"`。配置见 [Vite 集成](vite.zh-CN.md)，真实 VM 开发见 [QuickJS 应用重载](hot-reload.zh-CN.md#quickjs-应用重载)。即使日常使用 Bun 迭代，也应在 QuickJS 中验证共享界面。

## 性能与所有权

每种运行时都与 Rust 交换具有独立所有权的帧；GPUI 负责布局、绘制与原生输入。内嵌执行移除了子进程管道，但保留队列、编码和 JS 执行成本。仅切换运行时不会改善滚动或帧率。按照[性能流程](performance-analysis.zh-CN.md)分别测量启动、内存、JS 工作和输入到呈现延迟。
