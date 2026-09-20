# 应用性能分析

本流程供开发者和代理使用。先确认用户动作执行了预期工作，再测量性能。通用规则见 [GPUI 性能技能](../.agents/skills/gpui-performance/SKILL.md)，Solid 应用约定见 [solid-gpui 技能](../.agents/skills/solid-gpui/SKILL.md)，历史事件与数据见[滚动性能](scroll-performance.zh-CN.md)。

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

| 指标                     | 测量内容                        | 不能证明什么                             |
| ------------------------ | ------------------------------- | ---------------------------------------- |
| 活跃 FPS                 | 绘制间隔数除以活跃时长          | CPU 成本倒数、显示刷新率或扫描输出 FPS   |
| CPU draw p50/p95/p99/max | 原生构建、布局和绘制的 CPU 成本 | 全部排队和输入延迟，也不能独立证明流畅度 |
| 输入到呈现               | GPUI 输入到平台呈现边界         | 仅滚轮延迟或屏幕实际发光时间             |
| 失效到呈现               | 失效请求到呈现边界              | 闲置、启动、缩放区间之间的直接比较       |
| 吞吐量与完成时间         | 完成工作量及总耗时              | 长轮询是否饿死界面前台                   |
| TestAppContext 时间      | 确定性平台上的 CPU 归因与几何   | 真实 GPU 提交、vsync 或触控板分派        |

120 Hz 每帧约 8.33 ms，60 Hz 约 16.67 ms，应按真实平台与负载选择目标。短时高 FPS 可能掩盖长尾输入延迟，闲置低 FPS 通常只是内容没有变化。多个区间 p95 的范围不是整体 p95，应合并底层分布后计算总体分位数。

### 图片内存测量

图片内存应分层报告：编码源字节、解码后的 CPU 像素存储，以及 compositor/GPU atlas 存储。进程 RSS 或平台“内存”计数可能同时包含这些层、分配器余量、映射文件、应用其他状态与驱动记账，不能单独证明任一层的用量。记录图片来源与尺寸、布局逻辑尺寸、显示缩放、格式、动画状态、窗口活跃状态，以及样本处于冷启动、加载中、稳定、离屏还是最后一个 owner 释放后。

同时测量峰值与保留内存。JPEG 可在缩放前使用原生 1/8、1/4 或 1/2 解码；PNG 和其他光栅格式则先按完整源尺寸解码，再生成目标变体，因此全分辨率瞬时存储可能使加载峰值远高于稳定状态。SVG 按目标尺寸栅格化。GIF/WebP 动画保留当前帧、预取帧与 compositor 状态，而不是所有解码帧，但活跃窗口与变体仍会占用内存。不要用编码字节数代替解码或 GPU 大小。

单项输入、源像素、输出像素、抓取和解码限制是准入与并发预算，不是整个应用的内存上限。多个活跃来源、目标桶、窗口、动画及无关应用状态会累加。系统没有非活跃解码图片缓存，但应等到最后一个渲染 owner 以及为安全重绘保留的旧帧都释放后再测量回收。因此，活跃图片内存和稀疏退役 Surface 跟踪都不存在绝对常量内存边界。

## 3. 比较开启与关闭监视器

宿主使用固定 GPUI Kit 中的 [gpui-fps](../vendor/gpui-kit/crates/fps/README.md)。
每个窗口持有自己的 monitor Entity，关闭时释放。通过 `ComponentHost::with_performance_monitor(true)`
显式开启；所有构建默认关闭，website 原生宿主显式开启。

主读数默认是根据原生 presentation 事件计数的 **FPS**。右键切换到 **MAX FPS**，即采样 CPU draw
成本的倒数，并限制到显示器刷新率。这是完整重绘能力估计，不是实际帧节奏。
**FRAME/P95** 表示 CPU draw 时间，**DROP** 表示超出时间预算的 draw 比例（不是合成器丢帧率），
**INV** 表示原生失效。CPU/GPU/内存来自各平台进程采样，均不能替代物理输入延迟。

监视器跳过最初八个 draw 样本，并从绘制成本统计排除标记为仅 HUD 读数更新的 draw。
可见 HUD 每 500 ms 刷新，包括内容闲置时，因此它自身可能产生 presentation 事件。
HUD 不再被渲染后会停止读数时钟；失焦但仍在渲染的窗口可能继续刷新。这替换了旧的纯被动 HUD；可见 HUD 的闲置 FPS 不能作为基线。

对比应只改变 `with_performance_monitor`，重新构建并保留两个二进制身份和测量数据，
固定窗口尺寸、输入和数据集。环境变量不覆盖应用策略。
`frame-profile` 与 `bun run task website-native-profile` 继续单独记录 CPU draw、
输入到 presentation、失效到 presentation 区间指标。物理输入到显示验收仍需真实平台输入。

同一个诊断二进制可串行对比 360 次更新、500 行数据的负载。只有此诊断入口接受
`SOLID_GPUI_PROFILE_HUD`，不会改变应用宿主策略。先完成构建，再依次执行：

```sh
cargo build -p solid-gpui --bin solid-gpui-profile --features frame-profile
SOLID_GPUI_PROFILE_HUD=0 target/debug/solid-gpui-profile bun --conditions=browser scripts/native-commit-profile.ts --native
SOLID_GPUI_PROFILE_HUD=1 target/debug/solid-gpui-profile bun --conditions=browser scripts/native-commit-profile.ts --native
```

每次约 12 秒，检查所有增量更新并包含窄/宽窗口缩放，最终内容视口被改变时会拒绝该次采样。它覆盖原生提交与 presentation，
不代表物理触控板延迟。测量期间不要并行编译或运行其他基准。

## 4. 复现矩阵与正确性

每次只改变一个因素。滚动应覆盖首次进入、持续滚动、闲置后滚动、窄→宽→窄→宽缩放、断点两侧、嵌套面板及导航。记录准确路由和目标面板。长列表还应覆盖首尾行、空数据恢复、接近末尾时过滤为少量结果，以及重排。

确认内容确实移动，固定窗口元素不动，独立面板不联动，末尾内容可到达，按钮和文本仍在。只有滚轮日志而没有内容位移不算复现滚动负载。合成滚轮曾无法移动本应用；此时应在独立诊断窗口收集真实触控板输入并记录采集区间，不能继续将无效合成输入视为有效测试。

网站导航回归检查：

```sh
bun run task website-navigation-check
```

该检查通过原生 Vite 流水线挂载真实网站，验证组件路由变化保留侧栏并发送增量更新。它是生命周期正确性检查，不是几何或 CPU 基准。Showcase 滚动和缩放验收仍使用原生测量流程。

### 窗口移动与弹层锚点

分别统计边界回调和根渲染次数。仅位置变化的回调仍同步窗口状态并通知边界观察器，
但不强制刷新整棵树。视口尺寸、DPI 缩放、显示器标识、窗口视觉状态和影响内容的
指针变化仍是渲染输入。客户区外的指针位移本身不应重建视图树；内容悬停、光标变化
和活动拖拽必须保持正确，包括非活动窗口。内容若依赖屏幕位置，可在边界观察器中
显式通知更新。

已打开的弹层应利用当前绘制锚点跟随父窗口移动，不为重定位而通知父根渲染。
调整尺寸、待提交内容和缺失锚点应使用布局后的几何。在原生宿主验证嵌套弹层、
可编辑内容、锚点裁剪／移除和父窗口关闭。

还要检查重复同尺寸回调、真实缩放、显示器／DPI 变化、同尺寸最大化／装饰变化及
最小化恢复。强制恢复帧独立于边界相等判断。分别报告确定性测试平台计数、原生程序
移动、真实标题栏拖拽及视觉检查。强制刷新参考模式不是未经修改的历史二进制；
渲染计数不等于 FPS 或 CPU 降幅，macOS 结果也不代表 Windows 或 Linux 已验收。

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

### 批量工作与保留几何

- JavaScript 立即更新兄弟链，每个变化父节点在提交时统一生成索引；LIS 规划器按顺序 wire 索引发出最少同父移动，左右旋转都只需一个 Move。链和已发布属性一起回滚。
- 原生 Patch 在操作结束后统一派生脏 Text 祖先并同步兄弟索引；单个非法操作仍按顺序拒绝，不能被后续删除掩盖。验证完成后才发布 revision。
- VirtualList 为响应式数据变化保留独立身份快照，原生按 revision 链执行带高度估值的区间替换，保留区间外测量结果并调整逻辑滚动锚点。同数量重排也会失效；仅视口变化复用 edit；Snapshot 重建基线，不重放旧 delta。JS 快照和 diff 仍为 O(数据长度)，估值变化和完整 Snapshot 仍可能进行全量工作。
- 延迟动画保留采样起始样式，用一个 executor 截止时间唤醒，不在等待期间请求动画帧。活动动画、重定向、删除、减少动态效果与 epoch 切换仍保留完成/取消语义。
- 只有显式 `onLayout` 订阅才发出布局事件；每个 root 按观察顺序合并待发数据，在调度前去重。输入、选区、滚动的内部几何不依赖公开订阅，旧 revision、监听绑定及失效路由不能发送延迟事件。
- 图表和 NativePlot 使用有界、按 primitive 分配的路径槽。键包含形状用途和完整几何输入；当前颜色、渐变、透明度、缩放与裁剪仍在绘制时应用。只复用细分几何，不缓存整张场景，平移路径仍有成本。
  替换图元时释放新类型不再使用的槽位，即使图元总数不变。双路径形状变为单路径形状会释放多余槽位，无路径图元不保留任一槽位；未变化且仍使用的槽位保持热缓存。

### 2026 年 9 月 CPU 对照

串行本地探针使用 Bun 1.4.2、macOS arm64 Apple M5 Pro、gpui-pre 0.3.5，以及开启包级优化的 solid-gpui 开发构建。准备、断言、编译不在计时区间内，以下均为中位数，不是预算承诺。

| 负载                             |                      原审计 |                  优化后 |
| -------------------------------- | --------------------------: | ----------------------: |
| 8,000 行反转，JS signal 到 frame |                   308.43 ms |                 5.52 ms |
| 反转兄弟索引工作                 |           63,988,001 次访问 |            8,000 次写入 |
| 8,000 行左旋转 wire 输出         | 7,999 Moves / 248,019 bytes |       1 Move / 81 bytes |
| 同左旋转，原生树应用             |                   474.73 ms |                 4.21 ms |
| 单段落 4,000 个富文本 run 全改   |                   126.31 ms |                 1.93 ms |
| 同段落只改一个 run               |                    0.033 ms |                0.034 ms |
| 百万项原生列表追加               |         37.64 ms 全量 reset | 0.0115 ms 带估值 splice |

探针检查了顺序、身份、revision、文本、数量和逻辑滚动锚点。列表计时不含 JS 快照/diff 或布局；索引访问和写入是不同计数，说明移除的放大而非同一操作的耗时。原生反转仍有向量移动和当前位置查找，不宣称整体线性。

独立几何缓存 seam 中，8,192 点折线冷/暖为 13.83/3.95 ms，Area fill 为 224.04/0.54 ms，Sankey ribbon 为 31.75/3.79 微秒。计时外验证构建次数、顶点和边界；不包含投影、键哈希及原生绘制，不是完整图表帧时间。

工作站未隔离其他负载。这些结果不代表 release、Windows、低端设备、GPU、FPS 或物理输入延迟验收。

显示器恢复后，使用真实 macOS 宿主（非 TestAppContext）、2× 缩放、关闭性能监视器进行交互。截图确认：兄弟旋转、24 个富文本 run 更新、延迟透明度动画完成、滚动到第 5,000 项（offset 140,000）、前插保留阅读位置、末项 9,999 可达、过滤为三行后 offset 为零，以及 616→972→616 逻辑像素宽度反复缩放后内容保留。

验收发现并修复清空→恢复缺陷：保留的 Solid VirtualList 重新 Create 时沿用 data revision 3，而原生要求从零开始。现在属性汇总后重置新原生实例的 revision/edit，同时保留当前身份快照。wire 回归先失败后通过；真实窗口随后成功恢复 10,000 行。SDK 测试现为 118 项通过。

用户随后于 2026 年 9 月 20 日确认已完成人工验收，本轮优化的原生验收任务关闭。这是用户确认的验收结果，与上文自动化检查和 CPU 测量分别记录。

此前自动化限制作为历史证据保留：桌面 Space/全屏 VM 切换使同数量反转和图表数据双向交互未能完成，合成滚轮也未建立可靠位移证据。用户确认关闭验收状态，不将这些自动化项目追记为通过，也不补造逐项轨迹。本次确认没有附带新的 GPU 呈现、FPS、输入延迟、release 构建、Windows 或低端设备测量。
