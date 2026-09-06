# APP 性能分析指南

面向后续 AI 与开发者。先证明用户的触发动作产生了预期工作，再讨论性能。
GPUI 通用开发规则见 [gpui-performance skill](../.agents/skills/gpui-performance/SKILL.md)，
Solid 应用约定见 [solid-gpui skill](../.agents/skills/solid-gpui/SKILL.md)。
历史故障与测量记录保留在 [scroll-performance.md](scroll-performance.md)。

## 1. 建立一次测量的身份

在本地创建本次记录目录，记录以下字段；机器负载、版本或窗口变化后不要混用样本：

```text
revision / dirty diff identity:
executable path + hash:
Cargo profile / features / actual GPUI source:
runtime (process or embedded) / host profile / route / dataset:
viewport logical size / scale / theme / target display rate:
monitor on/off / diagnostic logging on/off:
trigger / duration / warmup / cache state:
background load:
correctness signal:
artifact paths:
```

可用 `git rev-parse HEAD`、`git diff --stat`、Cargo.lock、`cargo tree -e features`、
`shasum -a 256 <binary>` 确认身份。dirty workspace 的 HEAD 不是完整源身份，应保存本次
改动或其 hash；保留用户已有改动。先构建，再计时，避免边编译边采集。

## 2. 选择指标，不混淆边界

| 指标 | 表示什么 | 不能证明什么 |
| --- | --- | --- |
| 活跃 FPS | 活跃段内 draw 间隔数/实际时间 | 不是CPU时长倒数、显示器刷新率或scanout FPS |
| CPU draw p50/p95/p99/max | 原生构建、布局、绘制的CPU成本 | 不包含所有排队/输入延迟，不能单独证明流畅 |
| input-to-present | GPUI输入到平台呈现边界 | 不是纯wheel指标，也不等于光子到达屏幕时间 |
| dirty-to-present | 失效请求到呈现边界 | 空闲、启动或跨resize区间不能直接横比 |
| throughput/completion | 完成量和总耗时 | 不能排除UI前台被长poll饿死 |
| TestAppContext计时 | 确定性平台下CPU定位与几何反馈 | 不代表真实GPU提交、vsync或触控板分发 |

120Hz一帧约8.33ms，60Hz约16.67ms；目标由实际平台和场景决定。
短暂高FPS也可能掩盖长尾输入延迟。低空闲FPS通常只是没有内容变化。
区间p95的范围不是整体p95；合并原始分布后才能报告整体分位数。

## 3. 实时监视器与无监视器对照

通用组件与接入方法见 [gpui-performance](../crates/gpui-performance/README.md)。
监视器被动跟随绘制采样，空闲冻结读数与曲线；统计窗口、500ms活动分段阈值及其
无法区分长阻塞/空闲的限制，以组件README为准。严重停顿依靠外部profile与输入延迟。

正常Gallery启动默认显示。需要对照时，检查本仓库实际task名称后运行：

```sh
# 原生区间日志 + 实时浮层
bun run task gallery-profile

# 同样工作负载，去掉浮层构建与绘制开销
SOLID_GPUI_PERF_MONITOR=0 bun run task gallery-profile
```

日志开关由host的frame-profile feature控制；FPS浮层自身不启动timer或通知循环。
开启/关闭对照时使用相同二进制、窗口、输入和数据量，保留两次数据。

## 4. 复现矩阵与正确性

一次改变一个因素。滚动类至少覆盖：首次进入、持续滚动、静止后开始滚动、
窄→宽→窄→宽、breakpoint两侧、内外嵌套pane、导航切换。记录准确route与目标pane。
长列表增加首屏、末尾、空数据恢复、末尾过滤到少量数据、重排。

确认内容位移、固定chrome不动、独立pane不串动、最终内容可达、按钮/文字未被删除。
滚轮日志到达但内容没移动不算复现。本环境曾出现自动化wheel无位移：此时让用户在
独立诊断窗口真实触控板输入，记录采集时间，不反复把合成wheel当有效测试。

本仓库已有的几何和CPU定位入口：

```sh
bun run task gallery-scroll-audit
```

它从PAGES派生route，含窄窗口和retained-window resize；用于发现问题与检查正确性。
这是测试平台审计，不是生产benchmark资格证明。正确性断言保留，计时阈值仅在
受控环境显式启用。不要为了稳定计时删掉交互、说明文字或有意义的验证。

## 5. 归因：沿实际路径取证

```text
native input → native state / optional JS event → Solid reactive update
→ commit bytes → native foreground application → invalidation
→ request_layout → layout/measure → prepaint → paint → present
```

不是所有滚动都经过JS；先看实际路径。分开测协议提交成本与纯原生滚动成本。
源码定位入口及固定版本见 [source map](../.agents/skills/gpui-performance/references/sources.md)。

macOS可先用sample获得15秒主线程调用栈。先确认PID属于本次APP，避免选到Bun子进程
或用户的另一个窗口；以下占位符必须替换成已确认的进程和新产物路径：

```sh
sample <native-app-pid> 15 1 -file <new-local-output.sample.txt>
```

需要更精确归因时使用本机可用的Instruments/Time Profiler或xctrace。采集限制在15–30秒，
外层最多五分钟；绑定相同二进制、dSYM与架构。只读聚合和目标主线程，不把完整trace
倒进上下文。符号折叠/去重导致归因不可信时，另建符号分析binary重新采样，并声明该
编译选项不能混入正常计时对照。追踪文件保留本地。

按profile区分布局、塑形、构建scene、前台poll、锁/IO、GPU，提出可证伪预测：
“若这组intrinsic测量是主因，暂时隔离该区域应减少布局成本”。最小化只用于定位，
最终修复恢复全部业务功能。一项实验无改善就撤回，不层层叠加假设。

## 6. 生产benchmark资格

当要报告生产吞吐/调度/渲染性能时，读取Zed的gpui-bench，并核对当前链接版本提供的
bench-support和BenchAppContext接口。某个上游profile或宏参数不存在时，不编造替代API。

- 从生产输入边界驱动真正的构造器、队列、执行器和渲染路径。
- 完整feature图不能有test-support；测试与benchmark可拆package避免feature统一污染。
- 先有界smoke再测量；每次最多五分钟，串行执行。
- 准备、测量、正确性验证分开；固定数据量与冷热状态，保留完成数与顺序。
- 前台重活同时观察独立UI信号与完成时间；队列负载超过容量并有持续生产者。
- 无vsync的headless预算超限是代理指标，不称为实际display dropped frames。

## 7. 验收与经验更新

使用同一工作负载回放基线和候选，报告原始数值、退化、机器噪声和未覆盖项。
缩小的场景修好后必须回到原始页面、原生输入和resize过程。只保留关键正确性测试；
用户确认、原生profile、测试平台结果单独标注。

结论至少回答：触发条件是什么？耗时在哪？改动为什么减少工作？哪些功能不变量已
验证？原生是否复现/是否通过？哪个产物能复查？有什么尚未确认？

将可推广规则放进相应skill参考，具体窗口/数字放入事件记录。规则附适用条件和反例；
以替换过时结论维护文档，避免新旧“待确认/已确认”长期并列误导下次分析。
