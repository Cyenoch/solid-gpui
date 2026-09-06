---
name: solid-gpui
description: solid-gpui 应用开发指南。用于 SolidJS 原生页面、路由、主题、输入、虚拟列表、Vite/Bun 热重载和生成的 native API 集成；性能设计或卡顿诊断同时使用 gpui-performance。
---

# solid-gpui 开发

## 1. 找到真实入口与状态边界

读取项目的 CONTEXT.md、包 README 和对应示例。确认 Solid universal JSX transform、
运行时入口、Surface、host profile 及扩展能力。
涉及 Vite、Bun 或 hot reload 时读取 [热重载指南](../../../docs/hot-reload.md)，核对解析条件、epoch 和资源清理。
阅读 [开发约定](references/application.md)，按当前任务选择相关章节。

**完成标准**：说明哪些状态由 Solid 拥有、哪些由原生拥有；找到现有应用入口和所需 API
的真实导出，不用 React、DOM 或 Web CSS 的行为推断 native 行为。

## 2. 让变化保持局部

将 signals 的读取保留在 JSX/响应式计算内，用 memo 保存昂贵派生结果；相关状态更新
按现有 runtime 的 batch 合并。生命周期绑定 onCleanup/Surface，而不是永久全局任务。
路由 shell、导航与 pane 在页面切换时保留，页面内容放在 Outlet。

涉及布局、高频事件、列表、异步重活时同时读取 [GPUI 性能 skill](../gpui-performance/SKILL.md)。

**完成标准**：一个局部交互的变化范围可解释；没有每次输入重建整页、没有无消费者的
高频监听，也没有为了少提交而改变事件或命令顺序。

## 3. 按原生契约实现

使用实际类型定义选择 style/事件/命令；窗口宽度来自窗口尺寸 store，颜色来自响应式
主题。使用生成的扩展组件或 native client；协议变化走 schema → generator → checker。

列表、输入、原生函数三条分支的具体不变量见
[关键验收场景](references/verification.md)。

**完成标准**：布局在窄/宽窗口可用；明暗主题正文、控件与占位文字可读；异常路径和
资源释放明确；没有历史兼容壳或绕过生成器的手写协议字段。

## 4. 验证并交付

先跑改动涉及的关键测试与类型检查。UI 问题检查真实原生窗口；性能问题按
[性能分析指南](../../../docs/performance-analysis.md) 保留对照证据。
核对本仓库 task 名称后运行，避免把示例命令当作永不变化的 API。

**完成标准**：展示触发→结果，交代检查与限制；交互、滚动位置和数据正确性没有为性能
让步。新经验进入相关参考的一个位置，并说明成立条件。
