# 应用开发约定

## 入口和响应式边界

`@solid-gpui/core/runtime` 提供本项目的 Solid universal runtime。读取该包 README 中
当前 JSX transform 配置；jsxImportSource 只提供类型，不能替代 universal 编译。
运行时要用 browser 条件解析 Solid 的客户端响应式实现。入口选择现有示例的
createRoot/transport 与 router 组合，不在已有 root 之外建立第二个事件接收循环。

Solid component 通常只执行初始化一次。把 signal 读取留在 JSX 属性、memo 或 effect
追踪范围内，避免在初始化时 destructure 响应式 props 后期待它自己更新。昂贵筛选使用
createMemo；相关 signal 更新通过 runtime 的 batch 合并。把 timer、监听和资源清理
绑定 Solid owner/Surface。不要把 React useEffect/useMemo 的生命周期套到这里。

窗口与绘制、输入光标/选择/IME、滚动瞬态由 native 管理；Solid 拥有应用业务状态。
同一状态不要在 JS 和 Rust 各自独立推进。参考 root CONTEXT.md 与 ADR-0012。

## 页面结构与滚动

参照 Gallery App：稳定 shell 容纳 header、navigation、content pane、footer，Outlet
只替换内容。导航 pane 的身份与滚动句柄不随 route 选择重建。内容 pane 与导航独立；
页面切换是否重置内容滚动是 UX 决策，不能连带重置导航。

Style 是本项目协议类型，不是浏览器 CSS 全集。核对 `renderer/types.ts` 与原生
`paint/style.rs`。尤其 `gap`、`alignItems`、`justifyContent` 会隐式开启 flex column。
普通纵向内容流可用 block + margin，需空间分配时才用 flex。滚动视口必须有受约束尺寸；
分配剩余宽高的 pane 配合 minWidth/minHeight 与 shrink，长内容在 pane 内溢出。

窗口尺寸使用 window size store/hook，在响应式分支内计算 breakpoint；不能只读启动
宽度。单行按钮行高+padding+border 决定高度；说明文字、多行输入与动态卡片保留可增长
高度。测试宽→窄→宽过程与最终内容可达性。

## 列表

大数据使用 `VirtualList`，itemKey 返回稳定唯一字符串/数字，重排和过滤后仍指向同一
业务项。仅对已提交范围 renderItem；estimatedItemSize 是估计，不能当成所有内容必定
等高的承诺。overscan 用于减少边界空白，也增加创建、协议和布局成本，应实测选择。

滚动窗口前移一行时，重叠项应保留Solid owner与Host Node，仅新进入项创建、离开项清理。
`key`字段本身不保证复用，需验证renderItem次数与dispose；当前值和绝对index不变才复用，
同key替换新值或index变化必须更新内容。范围以绘制后的真实viewport为准，overscan只加一次；
原生overdraw测量范围不是可见范围，滚轮旧offset与paint双路上报会造成范围抖动。

首屏需要一个可见且有高度的 native boundary。Gallery 的320px容器需要 flex column，
VirtualList public style 应落在 boundary，内部 List 填满它。空数据恢复或末尾过滤后，
提交范围满足 `0 <= start <= end <= count`，且非空数据有实际行。应用调用公开列表 API，
不要设置 `__rangeStart` 等实现字段。

## 输入、事件与耗时操作

TextInput 使用真实组件和其 change/selection/command API；不要用可点击 Text 假造可
编辑输入，也不要把受控 value 的回传当作原生光标/IME 状态重置。

onPointerMove、拖拽和滚动监听仅在消费它们时注册。处理器里避免全量排序/重建长树与
逐事件 console 输出；先测频率和每次工作量。事件批次有顺序语义，参考 ADR-0010。

计算放入适当 runtime 的后台工作或异步原生服务，再把有界结果提交给 UI。生成的
native client 只提供调用通道，不保证实现自动离开 UI 线程；核对实际注册 handler。
展示 pending/失败状态，重复请求用明确取消或过期结果策略，不静默吞掉旧请求。

## 主题、扩展和生成代码

颜色从响应式主题取 semantic token；文字、背景、border、placeholder、disabled、
selected、hover 作为组合检查。嵌套 Text 不一定继承父控件预期颜色，要检查实际绘制。

使用包导出的生成组件与函数 client。扩展能力由 HostProfile/registry 决定；某个 host
没有适配器时不要加假成功 fallback。新增 wire 数据通过 protocol.bop 和生成器；
native 函数通过 Rust 定义及 native-codegen。生成产物须与源和 checker 一起验证。

## FPS 组件

窗口级性能信息由原生 `gpui-performance::PerformanceMonitor` 提供，可用于任何 GPUI
host；参见 [组件 README](../../../../crates/gpui-performance/README.md)。它不是 JSX
组件，也不是 JS timer 推算的 FPS。solid-gpui 的 provider host 已集成；自定义 host 在
初始化时创建一个实体并放进窗口 overlay。应用层不要另做一套每帧协议回传。

原生 Extension 初始 Snapshot 与后续 Patch 必须共用能力规则。分析按钮的 disabled 更新
曾因 Patch 漏掉 Extension listener 而退出；只测初始挂载或只测 native 函数本身不能
覆盖这条路径。新增控件要验证有 listener 时属性变化与 listener 替换/移除。
