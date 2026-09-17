# 选择运行时

可以直接编写 JavaScript，或用 Vite 编译 JSX/TSX。运行时选择与编写方式独立：Bun 保留自身 API，QuickJS 要求自包含 JS，并通过 Rust 提供服务。见 [Vite 集成](vite.zh-CN.md)。

Solid GPUI 提供三种原生运行模式，共用 Solid 组件 API、原生控件与 Rust 渲染引擎。选择取决于应用能力由谁实现，以及开发、交付方式。

## 运行时比较

| 项目 | 外部 Bun | 内嵌 Bun | QuickJS |
| --- | --- | --- | --- |
| 主要用途 | 快速开发 | Bun 应用的生产打包 | Rust 能力与 JSX/TSX 界面 |
| JavaScript 运行位置 | 子进程 | 原生应用中的专用线程 | 原生应用中的专用线程 |
| 文件与网络 | Bun 服务或 Rust 命令 | Bun 服务或 Rust 命令 | Rust 命令 |
| 原生界面 | GPUI | GPUI | GPUI |
| 传输 | 二进制 stdio | 原生帧桥接 | 原生帧桥接 |
| 用户端运行时 | Bun 可执行文件 | 承载 Bun/JSC 的单一自包含可执行文件 | 可执行文件中内嵌 QuickJS |

打包命令、前置条件与各平台状态只有一个权威来源：[分发应用](distribution.zh-CN.md#内嵌-bun-静态应用)及其[平台状态](distribution.zh-CN.md#平台状态与当前证据)。Windows 打包仍为实验性，Linux 仅到原生准备；本地构建通过不等于发布验收。

## 使用外部 Bun 开发

外部 Bun 提供较短的开发反馈周期。在本仓库中执行：

```sh
bun run website:native:dev
```

你自己的应用请按[入门](getting-started.zh-CN.md)操作：安装包、运行 `bun run generate`，再用 `bun --bun vite` 开发。Vite 监听应用依赖，在现有原生窗口内替换应用。当前路由、搜索条件等状态通过 `captureState` 保留，组件局部信号与原生编辑缓存重新挂载。参见[热重载](hot-reload.zh-CN.md)。

Bun 可以实现业务逻辑与服务，也可以调用 Rust Native Module。打包后的应用保持相同组合方式。

## 使用内嵌 Bun 打包

依赖 Bun 服务的应用以内嵌 Bun 为交付路径。它生成一个应用可执行文件，其中包含 GPUI 宿主、Bun/JSC 运行时，以及序列化后的应用及其声明的资源和 Worker 入口。产品契约不含附属文件：最终用户无需安装 Bun 或 Node，也不需要可执行文件旁边的 JavaScript 目录树或 `node_modules`。

```sh
solid-gpui embedded package \
  --entry <Vite 产物 JS> \
  --bun <固定版本 Bun 可执行文件> \
  --output <应用> \
  --manifest <应用 Cargo.toml> --package <应用 crate>
```

`solid-gpui embedded package` 由 `@solid-gpui/vite` 提供。库形式是
`@solid-gpui/vite/embedded` 的
`packageEmbeddedApplication({ sdkRoot, entry, output, bun, application, assets, workers, ... })`，
它显式接收 `sdkRoot`——即拥有固定 Bun/Rust 后端的 SDK checkout——因此消费方既不导入仓库私有
文件，也不复制工具链。本仓库以 `bun run embedded:package` 运行同一驱动。
`application.manifest`/`application.package` 指定应用自有的 Cargo manifest 与 crate，
`application.features` 追加其 feature，`application.main` 替换生成的 Rust 入口。结果报告包含
可执行文件及其摘要、Rust triple 与图目标、类型化的 `entry` 身份（`role: "application"`）以及每个
worker 身份，打包脚本无需自行推导。

打包器接收 Vite 产物入口，用与固定修订号匹配的 Bun 可执行文件序列化，并与针对预编译 WebKit/JSC 归档构建的原生 Bun 图链接。Vite 仍是唯一的应用编译器，Bun 与 Node 内置模块仍作为运行期 import 保留。固定版本的序列化器只是构建期工具：它不会随包分发，目标机器也不需要它。首次构建需要编译 Bun 原生构建图，工作量远高于界面重建；预编译 WebKit/JSC 归档避免了重建 JavaScriptCore，受支持的构建缓存与提取出的原生清单可避免重复工作。

通过 `--workers` 与 `--assets` 声明应用 Worker 和资源。运行时打开的任意目标文件系统路径不会自动打包；应嵌入这些应用资源，或明确准备所需外部文件。

宿主通过 `EmbeddedBunAdapter::start_packaged(entry)` 启动此类应用，`entry` 是打包器生成的虚拟图键；它与开发用的 `start(path)`（从磁盘加载文件）有意分离。找不到自身图的打包会话会直接失败，而不会去执行其他文件。应用使用 `EmbeddedTransport`，且绝不与 GPUI 线程共享可变 JS 对象。应用可用 `@solid-gpui/core/embedded` 的 `completeEmbedded(code)` 报告自身退出码（以 `supportsEmbeddedCompletion()` 探测支持），宿主经 `EmbeddedBunAdapter::result()` 读取；即使 VM 退出状态只有一个字节，完整退出码也能保留。

内嵌打包仍是实验性的。[平台状态表](distribution.zh-CN.md#平台状态与当前证据)是唯一的验证证据：目前没有任何目标被声明为受支持或已验证，不支持的 triple 会提前失败并列出支持矩阵。`--check-bundle` 只是启动冒烟检查：它启动两个会话并要求各自产出初始 Snapshot，不覆盖输入、资源、Worker 或图形，通过它不等于平台验证。架构、部署限制与剩余工作见[运行时策略](runtime-strategy.zh-CN.md)。

## 使用 QuickJS，让 Rust 实现应用能力

文件、网络、领域操作和长时间任务由 Rust 实现时，选择 QuickJS。JSX/TSX 仍负责信号、事件处理、路由及展示状态。通过[生成的 Native Module](rust-bridge.zh-CN.md) 暴露 Rust 操作。

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
bun --bun vite build # solidGpui({ entry: "src/app.tsx", runtime: "quickjs" })
```

使用真实 QuickJS 引擎开发时，在应用的工作区根清单配置[解释器开发 profile](hot-reload.zh-CN.md#应用构建配置)，构建启用 `quickjs` 的原生宿主，然后运行：

```sh
bun --bun vite
```

外部开发工具监听并打包代码，宿主准备新的 QuickJS VM，验证替代应用后再切换。Rust 服务和原生窗口保持存活。显式捕获的状态以有界 JSON 数据跨代传递。Rust、协议及构建配置变化需要重启开发命令。参见[重载生命周期](hot-reload.zh-CN.md)。

## 性能与所有权

内嵌执行消除了子进程管道，但队列与二进制编码仍有成本。两种内嵌引擎都与 Rust 交换具有独立所有权的帧；原生命令提供类型化结果，不把 GPUI 句柄暴露给 JavaScript。原生消费者落后时，背压暂停事件驱动的生产，所有待处理队列保持有界。从可执行文件镜像加载应用不会跳过解析与执行，也不代表运行时免费。

切换运行时不会自动提升滚动或帧率。GPUI 仍负责原生布局和绘制，部分滚动完全不进入 JS。按照[性能指南](performance-analysis.zh-CN.md)，分别测量启动、总内存、JS 计算和输入到呈现延迟。
