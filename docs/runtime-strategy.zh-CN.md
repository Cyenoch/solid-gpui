# 运行时策略

三种运行模式具有不同产品定位。这些定位描述预期开发与交付方式；打包命令、前置条件、各平台证据与签名只有[分发指南](distribution.zh-CN.md#内嵌-bun-静态应用)这一个权威来源。本指南说明各模式如何构建与归属，以及架构带来的实际限制。

编写方式与运行时选择独立：JavaScript 可直接运行，JSX/TSX 只使用 Vite 编译。Bun 保留自己的 API，运行时不选择第二套编译器。见 [Vite 集成](vite.zh-CN.md)。

## 定位

| 运行时 | 主要定位 | 应用职责 |
| --- | --- | --- |
| 外部 Bun | 快速迭代，包括 Vite 热重载 | Bun 可以实现领域逻辑与服务，也可以调用 Rust Native Module |
| 内嵌 Bun | 依赖 Bun 服务的应用以单一自包含可执行文件交付 | 保留 Bun 应用模型，在原生宿主内运行 Bun/JSC，并从镜像中直接执行应用 |
| 内嵌 QuickJS | 主要能力由 Rust 实现的应用界面运行时 | JSX/TSX 负责界面组合、交互和响应式展示状态，Rust 负责领域能力与系统服务 |

界面状态仍是真正的应用代码：信号、事件处理、路由、局部验证和展示计算应按需放在 JSX/TSX。QuickJS 的定位不是静态标记，而是让文件、网络、长期领域任务等主要能力由 Rust 实现，通过 Native Module 暴露，避免在 QuickJS 中重建 Bun/Node 服务。

三种模式共用 Solid 通用渲染器和原生契约。GPUI 始终负责窗口、输入、布局与绘制。应用职责分配和进程结构是两个独立决定，参见 [ADR-0017](adr/0017-runtime-engines.md)。

## 内嵌 Bun 静态产物

`bun scripts/bun-static-package.ts`（即 `bun run embedded:package`）为每个目标生成一个应用可执行文件：GPUI 宿主、Bun/JSC 运行时，以及序列化后的应用及其声明的资源和 Worker 入口，全部位于同一个原生镜像中。它不需要另行提供 Bun 或 Node 可执行文件、JavaScript 目录树、`node_modules` 或 Bun/JSC 动态库。固定版本的序列化器是**构建期**输入，不是运行时依赖；操作系统库及其他目标/配置特有的原生依赖仍需验证。Windows 打包仍为实验性，Linux 仅到原生准备：各目标验证范围见[平台状态](distribution.zh-CN.md#平台状态与当前证据)，命令、前置条件与发布步骤见[分发指南](distribution.zh-CN.md#内嵌-bun-静态应用)。

### 单一镜像，单一依赖图

打包器先准备固定版本的 Bun 源码（修订号取自 `crates/solid-gpui-bun-sys/bun-build.json`，并应用 `bun_embed.patch` 与内嵌覆盖），以 `embed-native` 模式针对**预编译** WebKit/JSC 归档构建 Bun 原生构建图，然后生成一个临时 Cargo 工作区，其 `src/main.rs` 引入生成的应用程序图与宿主入口。该工作区依赖启用 `embedded-bun` 的 GPUI 宿主 crate，以及以 `rlib` 形式提供的 Bun Rust 入口层（`bun_bin` 开启 `solid-gpui-embed`），因此 Bun 与应用在 Bun 固定工具链下编译为同一依赖图，共用同一 panic 策略，并继承 Bun 的唯一全局分配器；应用不得再添加第二个 `#[global_allocator]`。独立的 Bun Rust `staticlib` 会在链接前被拒绝，schema 或目标不匹配的原生清单同样会被拒绝。原生构建产出 `embed-native.json`（对象文件、归档、链接参数、Rust flags、环境与 cargo profile），打包器直接消费该清单，而不是重建编译器响应文件；CLI 输入、启动代码以及 GPU 与窗口所有权仍属于应用，而不是 Bun CLI main。

### 应用交付：内嵌模块图

Solid 应用仍由 Vite 编译。随后打包器用固定版本的 Bun 序列化器在 browser 条件下序列化 Vite 产物入口（以及 `--workers` 入口和 `--assets`），提取独立模块图，按固定格式校验后丢弃中间产物。校验会直接拒绝预编译 JSC 字节码与原生插件，并要求图的入口标识恰好等于由应用入口推导出的虚拟键（macOS 为 `/$bunfs/root/<name>`，Windows 为 `B:/~BUN/root/<name>`）。

校验通过的负载被嵌入可执行文件自身的图段：Mach-O 上放入 `__BUN,__bun`（以 16 KiB 对齐强定义 `BUN_COMPILED` 符号），PE 上放入 `.bun`（段偏移 0 处是 8 字节小端长度前缀）。运行时直接从镜像映射该负载。构建期与运行期都不会解压到磁盘，也不需要可变副本，因为图只包含源码。

需要明确说明的后果：

- 图是进程级单例：同一进程内的每个会话执行的都是同一个内嵌图。
- 图的虚拟路径可通过 Bun 的图感知文件 API 访问。由 GPUI 原生代码直接消费的资源不会被自动重映射，必须有意识地通过宿主资源 API 传入字节。
- 这不是沙箱。非图内路径仍然指向真实文件系统。
- 序列化器无法发现的动态 import 不会静默回退到开发者检出目录；应显式声明为入口，或接受可见的模块缺失失败。

### 启动 API 与会话生命周期

应用通过 `EmbeddedBunAdapter::start_packaged(entry)` 启动打包会话，`entry` 是打包器生成的图键。它与 `start(path)` 有意分离：后者会规范化文件系统入口，保持开发路径不变。启动失败的会话不会静默回退到其他文件：若可执行文件没有可用图，或没有该入口，会话会失败，并通过 commit/status 路径报告负的 `packaged_graph_status`（`UNAVAILABLE`、`MALFORMED`、`NOT_VIRTUAL`、`MISSING`、`BYTECODE`、`NATIVE_LIBRARY`），此时不会执行任何代码。打包标识沿用未变的五函数 C ABI（`bun_embedded_create`/`run`/`wake`/`terminate`/`destroy`），通过既有入口指针加长度参数传递，并用一个标签字节区分图键与文件系统路径；没有新增第二个 ABI 函数、图安装调用或第二套传输。

生命周期就是[通信基线](#通信基线)中已记录的内嵌生命周期：每个进程一个常驻 Bun owner 线程、同时只有一个活动会话、每个会话一个全新 VM，`shutdown` 会等待完全拆除后才允许下一个会话。图单例在 VM 初始化之前被采用，并在拆除后继续存在；会话状态则不会保留。

### 预编译 WebKit/JSC 策略

打包器始终以 `--webkit=prebuilt` 配置 Bun 原生构建，因此 WTF、JavaScriptCore 与 bmalloc 都来自与 Bun 固定的 WebKit 修订号匹配的发布归档，并区分目标操作系统、架构、libc ABI 以及 debug/release/ASAN 变体。这里不重新编译 JavaScriptCore 源码，该配方是平台矩阵，而不是一个到处通用的归档。各变体不可互换：ASAN 会改变 `WTF::Vector` 布局，混用带与不带 ASAN 的输入可能破坏内存。缺少所需变体时必须让构建失败，而不是静默编译或用无关引擎替代。

### 部署限制

- **不提取捆绑原生库。** 模块图校验会拒绝 JSC 字节码与 module-info、N-API addon，以及任何 `.node`、`.dll`、`.so`、`.dylib` 条目，因为它们只能靠运行时写盘来服务。内嵌构建还会禁用主 VM 与 Worker 共用的原生提取函数，并拒绝 FFI 模块图路径；这不是外部文件系统或 FFI 访问沙盒。
- **没有通用 fork/cluster JavaScript 解释器。** 内嵌 `fork()` 会拒绝默认解释器，以及与 `process.execPath` 完全相等的显式 `execPath`；`cluster.fork()` 复用此限制。宿主不是 Bun CLI，普通外部进程启动不变。镜像内并发应使用声明过的 Worker 入口，并相对于 `import.meta.dirname` 而不是进程工作目录解析路径。
- **资源闭包由应用负责。** 通过 `--workers` 与 `--assets` 声明应用 Worker 和资源。运行时打开的任意目标文件系统路径不会自动嵌入；应把这些应用资源加入模块图，或明确准备所需外部文件。
- **打包器只产出裸可执行文件**：不附带应用 bundle、图标、元数据、entitlement、许可说明或归档，生成 macOS `.app`、归档或校验和文件仍属于发布步骤。
- **Linux 打包仅为准备阶段**，公开的 Windows 支持在通过[验收门槛](distribution.zh-CN.md#windows-验收门槛)前保持实验性。

打包限制的完整权威清单见[分发指南](distribution.zh-CN.md#运行时与打包限制)。

## 构建期输入与目标机器要求

| 输入或要求 | 构建机 | 目标机器 |
| --- | --- | --- |
| 固定版本 Bun 源码 | 需要：在该修订号上浅克隆并应用 `bun_embed.patch` 与内嵌覆盖 | 从不需要 |
| 固定版本 Bun 可执行文件 | 作为序列化器使用；若报告的修订号与固定值不符则拒绝继续 | 从不需要，也不会被打包 |
| 工具链 | Bun 固定版本对应的 rustup 工具链、Bun 图所需的 C/C++ 与 LLVM、Ninja 1.13.0，以及目标 sysroot/SDK | 不需要 |
| 预编译 WebKit/JSC | 作为该目标变体的构建输入下载；不从源码编译 | 不作为附属文件分发，其代码已在可执行文件内 |
| 应用 JavaScript | Vite 产物被序列化进图 | 可执行文件旁边不需要 |
| 运行期文件 | — | 不需要任何 Bun 或 Node 文件；可执行文件本身就是应用 |
| 操作系统库 | — | 原生镜像的正常系统依赖（窗口系统、GPU 驱动、系统提供的 ICU/DirectX 模块） |

`--source`、`--macos-sdk`、`--deployment-target`、`--winsysroot`、`--base-executable`、`--main`、`--assets` 与 `--workers` 用于显式提供固定检出目录、交叉目标 sysroot、应用自有 Rust 宿主入口或目标平台的 Bun 基础可执行文件，而不是靠推断。跨目标序列化在没有 `--base-executable` 时会拒绝执行，因为固定版本并不保证可以静默下载另一个基础镜像。

## 预期流程

Bun 应用使用外部 Bun 与 Vite 开发，再通过内嵌 Bun 打包。必须用真实内嵌运行时和真实打包产物验证；外部 Bun 可执行文件的测试通过，不代表另一个固定版本的内嵌 Bun 已验证，某一配置下的构建也不能验证实际交付的配置。

以 Rust 为主的应用通过 QuickJS 交付界面。外部 Bun 与 Vite 提供快速界面迭代，但共享界面仍需在真实 QuickJS VM 中执行，以发现不支持的依赖和平台假设。QuickJS 重载（`bun run quickjs:dev`，或 `vite` 配置 `runtime: "quickjs"`）替换 VM 而保留原生宿主；生产模式每个运行时只计算一个 bundle。

## 通信基线

| 模式 | 当前传输 | 所有权边界 |
| --- | --- | --- |
| 外部 Bun | `StdioTransport` 通过子进程 stdin/stdout 传输带长度前缀的二进制协议 | 具有独立所有权的字节跨进程传递 |
| 内嵌 Bun | `EmbeddedTransport` 使用进程内有界字节队列 | 字节跨运行时线程与 GPUI 线程传递，不是系统 stdio 管道 |
| QuickJS | `EmbeddedTransport` 使用进程内有界字节队列 | 字节跨 QuickJS worker 与 GPUI 线程传递 |

把应用嵌入镜像不改变这一边界：JavaScript 仍与 Rust 交换具有独立所有权的帧，打包会话使用与开发会话相同的适配器、队列与背压规则。背压必须传到渲染调度器；仅更换适配器或再加一层队列无法实现有界的内嵌生产。任何模式都不在 JavaScript 和 GPUI 线程之间直接共享存活的 JS 对象、Solid owner、闭包或 GPUI 句柄；引擎内部 Rust 绑定与跨线程共享 VM 值是不同概念。

权威传输约束见[协议指南](protocol.md)和 [ADR-0017](adr/0017-runtime-engines.md)。研究建议不能在尚未实现时替换这些契约。

## 性能解释

内嵌 Bun 移除独立 Bun 进程，但仍需初始化并执行 Bun/JSC，进程内传输仍有排队、同步、编解码成本，不意味着对象零拷贝或更高原生 FPS。把运行时链接进应用本身也不会带来 JIT 预热优势或启动捷径：图仍会在每个会话中被解析与执行。QuickJS 不携带 Bun 服务，但解释器仍须执行界面 JavaScript，原生布局与绘制依旧是共同的 GPUI 工作。

内嵌 Bun 首次构建会从固定源码编译 Bun 原生构建图，但复用预编译 WebKit/JSC 归档而非重建 JavaScriptCore；受支持的构建缓存与提取出的原生清单可避免重复该工作。构建成本不等于应用启动时间，也不能替代固定版本校验：版本、补丁与目标变体检查仍会执行。参见 [ADR-0002](adr/0002-embedded-bun-runtime.md)。

目前这些定位尚未建立受控的三运行时性能对比。应使用相同界面、负载、release 配置和原生宿主，按[性能流程](performance-analysis.md)分别记录启动到内容、总进程内存、JS 工作、传输成本和原生输入到呈现延迟。

## 设计研究

研究记录区分实现事实与提案，并引用一手来源：

- [通信与所有权](../.scratch/runtime-strategy/transport-research.md)。
- [QuickJS 热重载和 Vite 集成](../.scratch/runtime-strategy/quickjs-hot-reload.md)。
- [可移植单一可执行文件的内嵌 Bun](../.scratch/runtime-strategy/windows-embedded-bun.md)：静态链接、图嵌入与 Windows 可移植性分析。除非本指南说明某阶段已实现，其阶段表属于提案。

内嵌帧桥接、整包 QuickJS 重载、由 Vite 持有的 QuickJS 构建策略以及静态内嵌 Bun 打包器均已实现。大块缓冲区所有权转移与自定义 ModuleRunner 仍是需要测量才能决定的备选方案，也不建议直接共享存活的 JS/GPUI 对象。参见[重载流程](hot-reload.zh-CN.md#quickjs-应用重载)。
