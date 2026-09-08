# 运行时策略

三种运行模式具有不同产品定位。这些定位描述预期开发与交付方式，不代表所有发布流水线已经完成。

编写方式与运行时选择独立：JavaScript 可直接运行，JSX/TSX 只使用 Vite 编译。Bun 保留自己的 API；native 模块在两种编写方式中使用同一契约，见 [Vite 集成](vite.md)。

## 定位

| 运行时 | 主要定位 | 应用职责 |
| --- | --- | --- |
| 外部 Bun | 快速迭代，包括 Vite 热重载 | Bun 可以实现领域逻辑与服务，也可以调用 Rust Native Module |
| 内嵌 Bun | 使用 Bun 服务的应用最终生产打包 | 保留 Bun 应用模型，在原生宿主内运行 Bun/JSC |
| 内嵌 QuickJS | 主要能力由 Rust 实现的应用界面运行时 | JSX/TSX 负责界面组合、交互和响应式展示状态，Rust 负责领域能力与系统服务 |

界面状态仍是真正的应用代码：信号、事件处理、路由、局部验证和展示计算应按需放在 JSX/TSX。QuickJS 的定位不是静态标记，而是让文件、网络、长期领域任务等主要能力由 Rust 实现，通过 Native Module 暴露，避免在 QuickJS 中重建 Bun/Node 服务。

三种模式共用 Solid 通用渲染器和原生契约。GPUI 始终负责窗口、输入、布局与绘制。应用职责分配和进程结构是两个独立决定，参见 [ADR-0017](adr/0017-runtime-engines.md)。

## 预期流程与当前支持

Bun 应用使用外部 Bun 与 Vite 开发，再通过内嵌 Bun 打包。必须验证真实内嵌运行时；外部 Bun 可执行文件的测试通过，不代表另一个固定版本的内嵌 Bun 及其服务 API 已验证。

以 Rust 为主的应用通过 QuickJS 交付界面。外部 Bun 与 Vite 可以快速迭代，但共享界面仍需在真实 QuickJS VM 中执行，以发现不支持的依赖和平台假设。使用 `bun run quickjs:dev` 或 `vite`（配置 `runtime: "quickjs"`） 启用 QuickJS 应用重载。

| 范围 | 当前实现 | 剩余工作 |
| --- | --- | --- |
| 外部 Bun 开发 | 默认宿主模式；Vite 依赖重载、显式状态传递与失败恢复已有集成覆盖 | 运行时内部演进时保持快速开发路径 |
| 内嵌 Bun 生产 | 可选 `embedded-bun` feature；仅 macOS，有界传输、生命周期测试及原生 Gallery 交互记录 | 完整应用打包、签名、依赖验证流水线，以及各目标平台验证 |
| QuickJS 生产 | 可选 `quickjs` feature；自包含 ESM 可嵌入可执行文件；当前 Gallery 打包使用此路径 | 完成各桌面平台发布验证；候选归档不等于原生桌面正确性证明 |
| QuickJS 热重载 | 开发重建替换 VM，保留原生宿主；生产模式每个运行时只计算一个 bundle | 使用显式状态及 setup 持有的 Surface 标识；原生或配置变化需重启 |

当前[分发指南](distribution.md)描述 QuickJS Gallery 包，不证明内嵌 Bun 打包已经完成。[内嵌 Gallery 验收记录](../.scratch/native-authoring-research/gallery-qualification.md)只证明选定交互与退出行为，不证明发布打包或性能。

## 通信基线

| 模式 | 当前传输 | 所有权边界 |
| --- | --- | --- |
| 外部 Bun | `StdioTransport` 通过子进程 stdin/stdout 传输带长度前缀的二进制协议 | 具有独立所有权的字节跨进程传递 |
| 内嵌 Bun | `EmbeddedTransport` 使用进程内有界字节队列 | 字节跨运行时线程与 GPUI 线程传递，不是系统 stdio 管道 |
| QuickJS | `EmbeddedTransport` 使用进程内有界字节队列 | 字节跨 QuickJS worker 与 GPUI 线程传递 |

任何模式都不在 JavaScript 和 GPUI 线程之间直接共享存活的 JS 对象、Solid owner、闭包或 GPUI 句柄。引擎内部 Rust 绑定与跨线程共享 VM 值是不同概念。生成的原生客户端已经提供跨边界类型化操作。

权威传输约束见[协议指南](protocol.md)和 [ADR-0017](adr/0017-runtime-engines.md)。研究建议不能在尚未实现时替换这些契约。

## 性能解释

内嵌 Bun 移除独立 Bun 进程，但仍需初始化并执行 Bun/JSC。进程内传输仍有排队、同步、编解码成本，不意味着对象零拷贝或更高原生 FPS。QuickJS 不携带 Bun 服务，但解释器仍须执行界面 JavaScript。原生布局与绘制依旧是共同的 GPUI 工作。

内嵌 Bun 首次构建会编译固定版本、带补丁的原生 Bun 库。复用构建目录可减少重复工作，但不会跳过版本与补丁检查。这一成本与应用启动时间不同，参见 [ADR-0002](adr/0002-embedded-bun-runtime.md)。

目前这些定位尚未建立受控的三运行时性能对比。应使用相同界面、负载、release 配置和原生宿主，按[性能流程](performance-analysis.md)分别记录启动到内容、总进程内存、JS 工作、传输成本和原生输入到呈现延迟。

## 设计研究

研究记录区分实现事实与提案，并引用一手来源：

- [通信与所有权](../.scratch/runtime-strategy/transport-research.md)。
- [QuickJS 热重载和 Vite 集成](../.scratch/runtime-strategy/quickjs-hot-reload.md)。

推荐保留外部 Bun 的带帧 stdio，并让两种内嵌引擎共用显式原生帧桥接。背压必须传到渲染调度器；仅更换适配器或再加一层队列无法实现有界生产。不建议直接共享存活的 JS/GPUI 对象。大块缓冲区所有权转移是独立优化，需要测量与显式内存记账。

QuickJS 重载使用外部 Bun/Vite 工具，在全新的 QuickJS Runtime 中替换整个 bundle。稳定的 Rust supervisor 管理 epoch、状态交接、候选验证、激活和退役，保留窗口与 Rust 服务。Vite 无需在 QuickJS 内执行。Vite 插件提供同一套 QuickJS 构建策略：单个自包含模块、平台初始化和不支持导入的检查。只有测量证明重建或计算成本值得额外复杂度时，才考虑自定义 ModuleRunner。

当前桥接和整包 QuickJS 重载已经实现；ModuleRunner 与大块缓冲区所有权转移仍是由测量决定的备选方案。参见[重载流程](hot-reload.md)。
