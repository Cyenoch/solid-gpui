# JS 事件循环、GPUI 与 Tokio 核对

日期：2026-09-06。实现和验收均基于当前工作区。Embedded Bun 使用固定源码 `34cbb9a40b4bd1bd767d134a7065e66c2432a676`、`nightly-2026-07-20` 和本次重新构建的 macOS 动态库；没有使用旧库替代本次补丁。

## 业务命令执行器

原来的模块 Future 仅由 GPUI `background_spawn` 轮询，没有 Tokio 上下文。现在 `native/executor.rs` 的 `NativeExecutor` 由 `NativeModules` 注册集合拥有，按需启动持续工作的 Tokio multi-thread runtime，启用 timer 和 I/O。

- async 工厂及整个 Future 都在 Tokio 内执行；sync 命令在 blocking pool 执行。GPUI 前台不执行 `block_on`，JS 对象和 GPUI Entity 不进入 Tokio。
- 调用链为 JS Promise → 协议字节 → GPUI 校验 → Tokio → GPUI 完成回调 → 协议字节 → JS 事件循环 → Promise resolve/reject。后续 JS 由 Bun 的微任务机制执行。
- 每 Surface 最多 32 个请求，整个注册集合最多 128 个实际任务；许可由实际任务持有，取消已运行的 blocking 工作不会提前释放容量。
- Root 拥有请求 Task；关窗、epoch 重置和 Root Drop 清理任务。Tokio JoinHandle 使用 abort-on-drop；旧 epoch 的结果在接触新请求 ID 之前被拒绝。
- 最后 owner 释放时使用 `shutdown_background`，避免 GPUI 前台等待线程。已运行的 Rust blocking 函数采用 Rust/Tokio 的正常执行语义，不能被安全强杀；自行 detach 的子任务也不自动继承父调用的取消。

依据：[Tokio GUI/同步代码桥接](https://tokio.rs/tokio/topics/bridging)、[JoinHandle 的 detach 与 abort 语义](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html)。

## Embedded Bun 的完整运行时边界

旧的 `setInterval(0)` 轮询和阻塞 `sync_channel.send` 已移除。嵌入实现直接使用固定 Bun 源码中的 `VmHandle`、`JsPoster`、`ManagedTask`、事件循环 keepalive 和 JSC termination trap。

| 所有者 | 责任与释放时机 |
| --- | --- |
| 进程级 Bun owner 线程 | 首次初始化 JSC 主 RunLoop，在线程停泊期间保留身份；串行运行 VM 会话 |
| 一次 VM 会话 | 单一准入；所有 Surface 共享；完成 VM 清理后释放准入 |
| Rust adapter State | 拥有两个有界字节队列和 C 控制句柄；保持到同步 run 和回调全部结束 |
| Bun JS 线程 | 唯一拥有 JS Strong roots、输入分发和 JS 回调调用权 |
| weak VM gate / poster | 跨线程唤醒与终止；VM 关闭后拒收并回收任务载荷 |

宿主输入合并唤醒，一个 JS turn 最多消费 64 帧或 1 MiB，然后进入 Bun 的 after-yield 队列，让 timer/I/O 获得执行机会。单个合法大帧完整交付后立即让出。启动期间收到的输入保留到首个 data listener 安装；没有定时器补偿丢失唤醒。

输出采用真实 write/drain 语义：返回 false 的当前帧已经接收，宿主腾出空间后再投递 drain。输入和输出分别持有 keepalive，EOF 后最后一次背压输出仍能完成。输入队列最多 4096 帧和一个最大帧的字节预算；输出高水位为 32 帧或 16 MiB，忽略背压继续写入会在硬上限报错。UI 发送不等待消费者腾出队列容量，不丢帧也不改序。

`close_input` 在已接收输入之后发送 stdin end/close，并保留最终输出和 beforeExit 异步工作。无条件关停使用 JSC termination trap 中断不让出的 JS，再由 owner 在线程外层执行 VM teardown。Adapter Drop 只发终止请求；应用退出在 GPUI 后台 executor 等待完整 teardown。

嵌入式 `process.exit(code)` 运行其退出监听器、记录状态并终止当前 VM，不调用宿主进程退出。清理时先撤销回调与 Strong roots，再关闭 VM 的 Worker、网络/定时器等资源及线程局部指针，保留进程级 HTTP 服务基础设施和默认 runtime options。后续会话分配新的 ScriptExecutionContext ID，旧会话消息无法误投给新 VM。解析器在会话边界失效目录、负缓存和配置数据，同时复用稳定目录/Entry 存储。

## 实际发现并修复的根因

1. 跨线程 `EventLoop::wakeup` 形成并发可变 loop 借用：改为在 weak gate 保护下调用底层线程安全 wake。
2. 空队列检查和清除 scheduled 间存在弱内存序丢唤醒：改用 acquire/release 原子交换；Loom 保存旧协议反例和新协议探索结果。
3. 每次新建线程/复用主 context ID 不符合 JSC 主线程身份：改为持久 owner 和每会话唯一 context ID。
4. 主线程退出 cleanup 会停掉进程级 HTTP 能力且未完整释放嵌入 VM：建立独立 embedded teardown，保留进程设施，清理会话资源。
5. Resolver 的进程全局目录/负缓存遮蔽新文件：会话边界失效且回收旧配置，实际验证同路径改写和随后新建入口。
6. Bun.serve 读取空的全局 CLI Context：启动前使用 canonical `write_context_no_parse` 初始化进程拥有的默认选项和 log，不解释宿主 argv。
7. 默认 Node 条件选择 Solid SSR 导出、不产生渲染 effect：嵌入入口使用 browser condition，与现有 process/Vite 入口一致。

## 验收

`cargo test -p solid-gpui --features embedded-bun --test host_embedded_counter -- --nocapture` 使用本次构建的真实 Bun/JSC，单进程内依次覆盖：

- 初始输出、同路径源码改写、新建入口、真实 Solid 计数器输入/提交往返。
- 初始化完成前 512 个输入；第二活跃会话被拒绝；异常监听器恢复后队列继续处理。
- 128 个连续输出，跨越 write(false)/drain，严格检查顺序和最后一个已接收帧。
- 队列排空后 EOF/end/close，beforeExit 继续调度 timer 并自然退出。
- 连续三次 Worker 消息、Bun.serve、localhost fetch 和 timer；带仍存活的 Worker、未完成 fetch 和活跃服务关停。
- 入口忙循环、输入回调忙循环和无限微任务被终止；process.exit(7) 退出监听器只执行一次，宿主继续运行。
- VM 发布前立即关停、异步入口 reject、失败退出之后再次完成 Solid 计数器往返。

Rust workspace `--lib --tests` 254 项通过；doctests 2 项通过、1 项保持仓库既有忽略；开启 embedded feature 的 workspace 严格 Clippy 和 package CI 均通过。正式入口 `bun run task embedded-check` 已实际执行并通过真实 VM 矩阵及 feature Clippy。Package CI 包含生成器一致性、类型、运行时、打包消费者和格式检查。原始记录：[workspace](workspace-tests.log)、[doctests](doctests.log)、[Clippy](clippy.log)、[package CI](package-ci.log)、[embedded-check](embedded-check.log)。Tokio 另有真实 timer/TCP、容量、panic、取消和关停检查。

Loom 只证明所建模的原子唤醒协议，没有被当作整个 Bun/JSC 的内存安全证明。当前 native 库构建和 VM 验收平台为 macOS；这些结果不是其他平台或发布产物的认证。实际 Gallery 原生窗口已完成 Rust 组件更新、函数成功/业务错误/再次成功及正常关窗验收，见 [原生验收记录](gallery-qualification.md)。

构建入口也已修复：嵌套 Bun 构建显式清除外层 Rust/Clippy wrappers，始终使用固定 nightly；CI 的缓存键包含 embedded Rust overlays，`embedded-check` 是实际注册并执行的任务。
