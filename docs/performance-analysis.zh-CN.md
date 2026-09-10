# 应用性能分析

本流程供开发者和代理使用。先确认用户动作执行了预期工作，再测量性能。通用规则见 [GPUI 性能技能](../.agents/skills/gpui-performance/SKILL.md)，Solid 应用约定见 [solid-gpui 技能](../.agents/skills/solid-gpui/SKILL.md)，历史事件与数据见[滚动性能](scroll-performance.md)。

## 1. 标识测量环境

为本次运行创建本地产物目录，记录以下字段。机器负载、版本或窗口几何发生变化后，不要合并样本。

```text
revision / dirty diff identity:
executable path + hash:
Cargo profile / features / actual GPUI source:
runtime (Bun process, embedded Bun, or QuickJS) / host profile / route / dataset:
viewport logical size / scale / theme / target display rate:
monitor on/off / diagnostic logging on/off:
trigger / duration / warmup / cache state:
background load:
correctness signal:
artifact paths:
```

使用 `git rev-parse HEAD`、`git diff --stat`、`Cargo.lock`、`cargo tree -e features` 和 `shasum -a 256 <binary>` 标识构建。仅 HEAD 无法标识有未提交修改的工作区，应保留相关 diff 或哈希，同时保护用户现有修改。构建完成后再收集数据。

## 2. 明确指标边界

| 指标 | 测量内容 | 不能证明什么 |
| --- | --- | --- |
| 活跃 FPS | 绘制间隔数除以活跃时长 | CPU 成本倒数、显示刷新率或扫描输出 FPS |
| CPU draw p50/p95/p99/max | 原生构建、布局和绘制的 CPU 成本 | 全部排队和输入延迟，也不能独立证明流畅度 |
| 输入到呈现 | GPUI 输入到平台呈现边界 | 仅滚轮延迟或屏幕实际发光时间 |
| 失效到呈现 | 失效请求到呈现边界 | 闲置、启动、缩放区间之间的直接比较 |
| 吞吐量与完成时间 | 完成工作量及总耗时 | 长轮询是否饿死界面前台 |
| TestAppContext 时间 | 确定性平台上的 CPU 归因与几何 | 真实 GPU 提交、vsync 或触控板分派 |

120 Hz 每帧约 8.33 ms，60 Hz 约 16.67 ms，应按真实平台与负载选择目标。短时高 FPS 可能掩盖长尾输入延迟，闲置低 FPS 通常只是内容没有变化。多个区间 p95 的范围不是整体 p95，应合并底层分布后计算总体分位数。

## 3. 比较开启与关闭监视器

可复用组件与集成见 [gpui-performance](../crates/gpui-performance/README.md)。它被动采样绘制，在闲置时冻结读数和图表。其 README 定义了采样窗口、500 ms 活动阈值，以及无法区分长期停顿和闲置的限制。严重停顿需结合外部分析与输入延迟调查。

应用宿主在所有构建中默认隐藏监视器。使用 `ComponentHost::with_performance_monitor(true)` 显式开启。网站在 `examples/website/native/src/main.rs` 中开启；将参数改为 `false` 并重新构建，即可进行无覆盖层对比。环境变量不会覆盖应用策略。使用 `bun run task website-native-profile` 输出区间日志。

宿主 `frame-profile` feature 启用区间日志。FPS 覆盖层不会启动定时器或通知循环。两次运行应使用相同二进制、窗口、输入和数据，保留两组结果。

## 4. 复现矩阵与正确性

每次只改变一个因素。滚动应覆盖首次进入、持续滚动、闲置后滚动、窄→宽→窄→宽缩放、断点两侧、嵌套面板及导航。记录准确路由和目标面板。长列表还应覆盖首尾行、空数据恢复、接近末尾时过滤为少量结果，以及重排。

确认内容确实移动，固定窗口元素不动，独立面板不联动，末尾内容可到达，按钮和文本仍在。只有滚轮日志而没有内容位移不算复现滚动负载。合成滚轮曾无法移动本应用；此时应在独立诊断窗口收集真实触控板输入并记录采集区间，不能继续将无效合成输入视为有效测试。

网站导航回归检查：

```sh
bun run task website-navigation-check
```

该检查通过原生 Vite 流水线挂载真实网站，验证组件路由变化保留侧栏并发送增量更新。它是生命周期正确性检查，不是几何或 CPU 基准。Showcase 滚动和缩放验收仍使用原生测量流程。

## 5. 沿真实路径归因

```text
native input → native state / optional JS event → Solid reactive update
→ commit bytes → native foreground application → invalidation
→ request_layout → layout/measure → prepaint → paint → present
```

滚动不一定经过 JavaScript。检查真实路径，分别测量协议提交和纯原生滚动。源码位置与固定版本见[源码索引](../.agents/skills/gpui-performance/references/sources.md)。

macOS 可先用 `sample` 采集 15 秒主线程。确认 PID 属于目标原生应用，而非 Bun 子进程或用户其他窗口。将两个占位符替换为已验证 PID 和新的本地产物路径：

```sh
sample <native-app-pid> 15 1 -file <new-local-output.sample.txt>
```

更精确归因使用本地 Instruments/Time Profiler 或 `xctrace`。采集保持 15–30 秒，外层超时不超过五分钟。匹配二进制、dSYM 和架构，只检查聚合与目标主线程，不将整个 trace 塞入代理上下文。符号折叠或去重影响归因时，构建独立分析二进制重新采集，并说明其编译选项使其不能参与普通时间对比。trace 保留在本地。

区分布局、文本塑形、场景构建、前台轮询、锁/I/O 和 GPU 工作。提出可证伪预测，例如隔离固有尺寸测量热点应降低布局成本。缩减页面只用于定位原因，最终修复应恢复全部应用行为。无改善的实验应撤销，避免积累未经验证的假设。

## 6. 验证生产基准

报告生产吞吐量、调度或渲染性能前，阅读 Zed 的 `gpui-bench`，检查链接版本提供的 `bench-support` 和 `BenchAppContext` API。不要为不存在的上游 profile 或宏参数编造替代品。

- 从生产输入边界驱动真实构造器、队列、执行器和渲染。
- 完整 feature 图中排除 `test-support`；必要时拆分测试与基准包，避免 Cargo feature 合并。
- 先执行有界冒烟检查，再串行测量；每次运行不超过五分钟。
- 分开准备、测量和正确性检查；固定数据规模及冷暖状态，保留完成数量与顺序。
- 前台繁重工作同时观察独立界面信号和完成时间；用持续生产者将队列推过容量。
- 无 vsync 的无头预算超限仅是代理指标，不是真实屏幕丢帧。

## 7. 验收与记录

对基线和候选重放同一负载，报告原始值、回归、机器噪声和未测试情况。修复简化案例后，回到原始页面、原生输入及缩放序列。只保留关键正确性测试，明确区分用户确认、原生采样和测试平台结果。

结论应说明触发条件、昂贵工作、改动为何减少工作、验证过的行为约束、原生复现和验收状态、可审阅产物及剩余不确定性。

可复用规则放入相关技能参考，具体窗口尺寸和测量放入事件记录。写明适用条件及反例，替换过时结论，避免未解决与后续已确认结论缺少上下文地并列。

## 原生提交归因

专用诊断宿主关闭 FPS 浮层，使用应用实际的 CommitPump 与 SolidRoot：

```sh
cargo build --locked -p solid-gpui --bin solid-gpui-profile --features frame-profile
cargo tree --locked -p solid-gpui --features frame-profile -e normal,build,features
target/debug/solid-gpui-profile bun --conditions=browser scripts/native-commit-profile.ts --native
```

先完成编译并确认依赖特性不含 test-support。原生场景约每秒更新 30 次，总计 360 次，窗口宽度依次为 800→560→1280→800，完成后退出。增加 `--hold` 可在自动阶段结束后检查按钮、输入、实际滚动位移及窄/宽窗口布局；每次测试不超过五分钟。程序调整窗口尺寸不代表真实触控板或操作系统连续缩放验收。

`solid_commit_stages` 输出前台线程各阶段的累计毫秒数，counts 顺序与字段相同。queue 包含通道背压与前台交接等待，不含 runtime reader 收到帧之前的时间；decode 包含线协议防护检查；tree 包含树修改与依赖收集，dependencies 是其子阶段；validate 是 Extension 合约验证。commit 包含树事务、合约校验、原生状态同步与通知，不含 decode 和随后依赖窗口的 Extension 实例更新；commit 的计数也包含命令入队，命令实际执行不在此范围；比较 Snapshot/Patch 成本时要控制负载。extensions 单独记录这些实例更新。render 只包含 SolidRoot 的准备工作与 GPUI 元素构建，不包含后续布局、绘制、GPU 或呈现。嵌套阶段不能与外层相加，区间累计值不是逐提交跟踪或端到端延迟。

同一宿主输出 `[solid-gpui-frame-profile]` 的 CPU draw 与失效/输入到呈现直方图区间。结合有界原生 CPU 采样，区分元素构建、Taffy 布局、文字整形、prepaint 和场景构建；闲置或非活动窗口区间应分开报告。日志器不会主动请求重绘，未启用 frame-profile 或测试插桩时不会读取性能时钟。

JavaScript 场景分别测量同步信号传播与信号到编码帧交付的耗时。后者包括提交构建、编码与微任务调度，在调用 transport.submit 之前截止；它不是 IPC 延迟或纯 Solid runtime 耗时。每次测量必须产生一个内容正确的文本更新操作。纯内存规模测试：

```sh
bun --conditions=browser scripts/native-commit-profile.ts
```

确定性原生 CPU 归因实验需要显式运行，并与编译和其他测量分开：

```sh
cargo test --locked -p solid-gpui --lib snapshot_apply_and_first_draw_scaling_guard -- --ignored --nocapture
```

Snapshot 和 Patch 分阶段测量均经过实际宿主入口。每个样本释放自己的测试窗口，避免之前的场景累积并干扰后续负载。提交计时在 GPUI 更新回调内部截止，避免把 TestAppContext 随后自动处理 effect 与绘制的时间算入提交。单独 draw 计时包含强制 CPU 绘制和 executor draining，不是生产帧间隔。常规回归检查事务正确性与依赖工作量，不使用依赖机器的毫秒阈值。

### 原生更新与缓存范围

Patch 记录修改节点，并推导原始/最终祖先与类型化子节点的归属依赖。验证成功后发布 revision 和统一变更集合，供事件路由、原生实例、焦点观察器和缓存失效消费。大树中修改一个文本不应克隆整树或验证无关 Extension 合约；结构修改仍可能访问被移位的兄弟节点、变化的子列表及依赖组合。

提交变小不保证布局变便宜。缓存 GPUI Entity 需要原生状态 owner 显式失效。在共享根的面板实验中，只增加 `.cached(style)` 会同时跳过静态面板和变化的计数器：存储已到 Count: 12，渲染内容仍为 Count: 0，因此该实验被撤回。后续区域设计必须连接提交及纯原生输入、滚动、动画和异步内容的失效路径，并验证实际渲染内容后再接受性能收益。尺寸、裁剪和继承文字样式同样属于缓存依赖。
