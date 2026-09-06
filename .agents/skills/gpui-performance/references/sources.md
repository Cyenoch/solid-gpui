# 源码依据与适用范围

核对日期：2026-09-05。这里记录规则的来源，不把参考仓库版本当作运行版本。

- 实际运行依赖：Cargo.lock 中的 `gpui-pre 0.3.3`。核对 registry 源码的
  `src/window.rs`、`src/profiler.rs`、`src/view.rs`、`src/executor.rs`。
- Zed 对照提交：`5a9b9558db01a6b906cec2fb70a797affdc58cdd`，本地 `references/zed`。
- 本仓库实测证据：[scroll-performance.md](../../../../docs/scroll-performance.md)。

## 一手源码：查什么、据此决定什么

链接均固定提交；升级后重新核对相应函数，而不是保留过时 API 示例。

| 入口 | 核对所得 |
| --- | --- |
| [window.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/window.rs)：`on_next_frame`、`request_animation_frame`、`draw` | 请求下一帧会产生帧需求；测量组件不能靠自激重绘证明 APP 原来流畅。 |
| [profiler.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/profiler.rs)：`record_draw_timing`、`record_present`、`FrameDurationSnapshot` | draw duration 和 dirty-to-present 是不同边界；present 不是显示器 scanout。累计 histogram 需要相减得到区间。 |
| [view.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/view.rs)：`ViewElementCacheKey`、`prepaint` | 缓存取决于 bounds、mask、text style 和 dirty/refresh 状态；组件抽取本身不提供缓存。 |
| [uniform_list.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/elements/uniform_list.rs)：`uniform_list`、`measure_item` | 等高约束成立时才能通过代表行测量推导其他行布局。 |
| [list.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/elements/list.rs)：`ListState`、`ListMeasuringBehavior` | 可见范围测量和预先测量全部项有不同成本；持久状态维护滚动和重测。 |
| [context.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/app/context.rs)：`notify`、`spawn`、`spawn_in` | notify 是实体变化通知；spawn 传入弱句柄，Task 必须有生命周期。 |
| [taffy.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/taffy.rs)：`compute_layout` | GPUI 布局进入 Taffy 测量。嵌套 flex 是否昂贵仍需具体页面 profile，不能从源码入口直接推出耗时占比。 |

## 已有 GPUI skill：采纳与修正

这些 skill 是开发建议的来源，不是运行时行为的权威；行为以对应源码与测试为准。

- Zed [gpui-bench](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/.agents/skills/gpui-bench/SKILL.md)：采纳生产路径、feature 隔离、串行测量、响应性与完成量双指标。其 bench API、release-fast profile 和平台能力要在目标项目重新确认；本仓库 TestAppContext 审计不冒充该生产 benchmark。
- Zed [gpui-test](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/.agents/skills/gpui-test/SKILL.md)：采纳固定 seed、GPUI timer、pending-task 诊断；确定性调度验证不等于真实显示表现。
- AprilNEA [gpui](https://github.com/AprilNEA/gpui-skills/blob/3961d8cb27c345c04b592e79794eff0d278865ed/gpui/SKILL.md) 与 [gpui-component](https://github.com/AprilNEA/gpui-skills/blob/3961d8cb27c345c04b592e79794eff0d278865ed/gpui-component/SKILL.md)：核对了 notify、Task、background-spawn、flexbox 规则及组件/主题索引。保留批量通知、任务所有权和主题一致性；“全部布局用 h/v_flex”收窄为需要弹性分配时使用。其自身 notify 规则也说明无可见变化应跳过通知，不能只照抄标题中的 always。
- cnwzhu [gpui-async](https://github.com/cnwzhu/gpui-skills/blob/393a3da0a6e0d379d0e8aaeb7d856951641b47cf/gpui-async/SKILL.md) 与 [gpui-elements](https://github.com/cnwzhu/gpui-skills/blob/393a3da0a6e0d379d0e8aaeb7d856951641b47cf/gpui-elements/SKILL.md)：保留前后台分工、存储 Task、稳定组件结构。其 tokio timer 示例需要对应 runtime，不能用于本项目 GPUI 确定性测试；拆分组件提升组织性，不等于减少底层元素或布局量。

## 本项目实测形成的规则

Kanban 持续卡顿由单行按钮 intrinsic 尺寸放大嵌套测量；显式控制尺寸改善且用户验收。
Overview 的无弹性分配需求的外层 flex 堆叠改为块布局改善且用户验收。这两条是针对
具体工作负载验证的设计经验，不是“固定所有高度”或“flex 一律慢”的通则。
VirtualList 首屏空白与空/过滤后范围错误由原生 bounds 和真实 wire range 测试锁定。

## Skill 编写依据

按用户指定的 [writing-great-skills](https://github.com/mevatron/mattpocock-skills/blob/139f1166fdbf378db5463fd3aa0d58a68b8e730b/plugins/mattpocock-skills/skills/writing-great-skills/SKILL.md)
公开 Codex 适配版和本仓库 writing-for-agents 编排：入口描述用于触发，步骤给出可检查
完成条件，专项材料按需加载。没有复制上游长篇规则或绑定它们的工具命令。
