# 原生滚动性能

## 布局契约

GPUI 原生样式映射只支持有限属性，不是 CSS。flexGrow 让子元素增长，不会让父元素变成 flex 容器。垂直父级（包括路由包装层）应使用 `flexDirection: "column"`。有界滚动工作区需要 minWidth: 0、minHeight: 0，并沿父链直到窗口都使用合适的 flex 布局。

消费剩余空间的面板应在父主轴上以零尺寸开始，再增长：

```tsx
<View style={{ height: 0, flexGrow: 1, flexDirection: "row", minHeight: 0 }}>
  <View style={{ width: 190, flexShrink: 0, minHeight: 0, overflow: "scroll" }}>{navigation}</View>
  <View style={{ width: 0, flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "scroll" }}>{content}</View>
</View>
```

height: 0 是初始 flex 尺寸，不是最终高度。工作区增长到页头/页脚布局后剩余的空间。自动初始尺寸会先测量全部内容，再缩回窗口。不要将此规则用于按内容定尺寸的卡片、标签或固有宽度列，它们的自然尺寸是契约的一部分。

## 复现与回归

原有路由 Gallery 回归随旧应用移除。下方数据是历史证据，不代表当前网站已通过验收。运行保留的原生滚动回归与网站导航检查：

```sh
cargo test -p solid-gpui --lib renderer::scroll_tests -- --nocapture
bun run task website-navigation-check
```

当前网站测量应启动 `bun run task website-native-profile` 并遵循[性能分析](performance-analysis.md)。根据测量机器选择时间预算；确定性原生测试不测量 GPU 呈现。

仍需在原生应用中持续滚动导航/内容，滚动后点击、缩放、切路由和主题。基准通过不等于可见应用验收。

## 2026-09-05 诊断

466 个真实 Gallery 节点的旧双绘制循环 p95 约 26 ms。分离第二次绘制后，在 slotmap 优化已启用时导航 p95 为 12.04 ms。工作区垂直 basis 归零后导航/内容为 6.21/6.39 ms，shell 和内容宽度初始值归零后为 5.66/5.87 ms。初始视口独立滚动断言全部通过。用户最初确认原生滚动流畅，随后报告宽屏/缩放后的 Overview 和 Drag & Drop 仍卡顿，后者不缩放也卡。初次验收没有覆盖这些情况，早期窗口缩放流畅是独立证据。

扩展回归覆盖两种主题、新建 560/800/1280/1680 宽窗口、真实 Drag & Drop 路由，以及同一窗口 800→1280→800→1280 后导航。每阶段应用同一 root 的真实 Solid Patch，检查已滚动侧栏位置不变，可捕获新 Snapshot 无法发现的保留状态错误。

水平剩余空间列也需零初始宽度：ResponsiveRow 仅对正增长、非堆叠列应用，grow: 0 列保留固有宽度。动作旁的卡片和章节标题同理。图标/标签按钮直接拥有内容与颜色，去掉不必要嵌套行。

宽屏 Overview 内容基线 p95 13.34 ms，Drag & Drop 16.78 ms；修改后一轮为 7.19/7.75 ms，小窗口浅色 Drag & Drop 约 3 ms。其他轮次仍有 8–11 ms 尾部，并出现明确 8.333 ms 超预算。这说明显著改善，不是稳定 120 Hz 验收。不要反复重跑直到通过以隐藏波动。持续原生触控板/GPU 呈现仍需新验收；CUA 合成滚动未使应用内容移动，不能作为正面证据。原生主题切换和 Benchmark 单步更新已视觉验证。

导航还有独立生命周期缺陷：小型兄弟路由测试将布局挂载 11 次而非一次。RouterProvider、NativeMatch、Outlet 现通过 memoized 路由身份/渲染状态控制挂载，match 数据经 context 保持响应式。必须同时测试挂载生命周期与真实滚动位置，不用恢复 offset 掩盖重新挂载。

受控实验与约束：

- 单独优化 slotmap 对旧循环只改善约 6%，仍保留已优化 gpui-pre 和 taffy 开发依赖。
- 将每个 ResponsiveRow 子包装改为 flex column 使滚动 p95 恶化到约 21.8 ms，已撤销。
- 采样主要指向 Taffy block/flex 布局。休眠 transport 线程出现在 sample 中不代表耗 CPU。
- MeasuredElement 委托 request-layout，不是额外 Taffy 盒。
- 未节流或丢弃滚轮，未添加 scene/geometry 缓存。GPUI view 缓存要求确定尺寸，key 包含 origin 和 content mask，移动卡片未必适合缓存。
- 临时 DEBUG-scroll-live 输出已删除，滚轮日志不证明帧延迟。
- provider 固定视口像素、仅优化局部 renderer、Kanban 跨轴对齐均未证明额外收益，实验已撤销。

## 主题回归

统计、看板、预览和日志使用普通主题 surface，codeBg/codeText 仅用于代码。固定色块前景根据自身亮度选择，不依赖应用主题。共享按钮拥有图标和标签样式，防止嵌套 Text 静默退回黑色。占位文本测试检查两种模式下合成对比度，而非固定透明度；旧 20% 前景在浅色输入上只有 1.48:1。

回归时从真实路由测试和新原生采集开始。固定二进制、视口、包产物、profile 和计时范围，每次只改一项。生产修改与测试校准分开，破坏几何或交互的收益不能接受。

## VirtualList 尺寸与范围恢复

Gallery 列表必须填满 320 像素 flex-column 视口。原生 VirtualList 事件边界拥有公开样式，内部 GPUI List 填满边界。只给内部 List flexGrow 会让无样式边界塌缩，仅修改父 flex 也未修复空白。路由原生回归检查真实首行边界、内部滚轮移动和外层静止。

还要检查空→非空数据，以及接近末尾时过滤。验证已提交行和范围，不只 itemCount。旧问题包括空初始范围填充后仍为空，以及过滤产生 90..1 反向范围。当前当旧范围为空或超出新数据时，从初始窗口重新建立提交范围。

`ScrollShadow` 可以装饰一个方向匹配的直接子项（核心或原生 VirtualList），不再创建第二个滚动视口。原生提交发布子项的视口能力，命令和绘制共用同一个保留句柄。原生列表移出包装器时会释放装饰并恢复自身滚动条。公开组合与尺寸契约见[组件指南](gpui-components.zh-CN.md#覆盖范围)。

链接的 GPUI List 在首次布局和宽度变化时必须保留未测量行的高度估计，否则滚动条只反映已测量的部分；全量测量则会破坏虚拟化。宽度变化仍会使实际测量失效，保留值只是估计，可见行会按新宽度重新测量。没有新增 `measure_all` 路径。

原生组合回归用十万条逻辑数据和十条已提交行检查有界可见范围、滚轮、滚动条、命令、尺寸变化及完整估算范围。600×600、缩放系数 2 的 Bun 原生验证使用 32/40 像素变高行，走通末尾、反复改宽度、过滤及清空恢复：初始存活十行，观测峰值十三行。原生滚轮令首行从第 1 行推进到第 13 行，独立横向列表保持不动。这是工作量和原生正确性证据，不是 CPU 帧时间、输入延迟或显示 FPS 测量。

最终严格验证等待原生布局与范围反馈，不捕获断言失败。过滤到三行后 offset 收敛到零且存活三个 owner；清空释放全部 owner，恢复数据后重新挂载首个窗口。这些命令、改宽度与数据切换期间累计创建 38 个 owner，同时存活最多 13 个。原生横向列表在 `x=360` 时报告 `3..10` 范围，并通过自己的命令到达第 100 项。

## 捕获跨三列断点后的持续卡顿

用户报告的是 Kanban 缩放到三列后持续卡顿，不是一次冷滚轮。冷像素序列每次都移动也不能关闭报告。移除列包装使窄屏变慢；移除卡片列表增长或按钮工具栏滚动包装未证明收益，均已撤销。

测量真实输入与呈现：

```sh
bun run task website-native-profile 2> /tmp/solid-gpui-frame-profile.log
```

可选 frame-profile 启用原生 profiler，按区间报告样本数、draw p50/p95、失效到呈现 p95、输入到呈现 p95，以及视口和活动窗口状态。不注入输入、不调度重绘、不逐滚轮记录。直方图按观测差分，跨缩放区间丢弃。输入样本为零不提供延迟证据。呈现指 GPUI 提交边界，不是物理屏幕扫描。

在稳定尺寸下比较单列与三列的数秒真实滚动，保留准确二进制/profile 和区间，不与 TestPlatform CPU 时间合并，不以几何通过宣称原生修复。当时 CUA 点击可用，但滚轮不移动内容，剩余复现需要真实触控板。

用户随后使用真实触控板：793×733 的稳定活跃区间 draw p95 4.432 ms、输入到呈现 6.861 ms（68 draws/67 inputs）；1223×733 区间为 11.805–11.837 ms 和 57.967–100.663 ms；1147×733 为 8.172/34.472 ms。这些是原生观测，不是 TestPlatform。排除跨闲置和零输入区间，profiler 统计全部原生输入，不限滚轮。第二次滚动 CPU 采样确认 Taffy flex/block 为主要热点；仅 header 和 provider-flex 修改未证明收益，已撤销。

## 已确认的 Kanban 修复与全局检查

用户反复缩放进出三列后确认滚动流畅。关键改动在共享 Button：单行标签显式行高，控件高度由行高、padding、border 决定。旧固有高度导致外围嵌套 flex 反复测量。诊断中移除动作按钮降低宽屏成本，恢复全部动作并使用显式控件尺寸保留收益。可比 CPU 循环（含重复缩放）p95 从约 7.8–8.4 降至 4.8–5.0 ms。之后 1303×835 原生区间 draw p95 约 6–7 ms、输入到呈现约 7 ms；尺寸不同，不能算匹配窗口百分比。用户原生确认是验收信号。

该契约只适用于单行控件，不用于内容卡片、任意子内容或多行输入。不要以固定内容高度隐藏布局工作。原生回归验证真实变体标签位于控件内，共享 Button 将修复传播到其他页面，也检查了已有固定高度基准单元、色块与虚拟行。

旧 Gallery 退役时，该调查使用的专用全路由几何 harness 一并移除。共享网站现有聚焦导航回归：

```sh
bun run task website-navigation-check
```

它验证路由切换保留侧栏和增量更新，不测原生滚动或呈现。当前 Showcase 按[性能分析](performance-analysis.md)在宽窄窗口用真实输入验证 Collections 列表。CPU 测量和原生验收分开，闲置区间或并发构建不算滚动证据。

## Overview：流式章节不应累积 flex 测量

Kanban 验收后，用户仍在 Overview 库目录复现持续卡顿。这是独立验收案例，全路由几何通过不能否定原生反馈。

受控缩减定位目录为主要成本。移除 playground 只有轻微改善；目录行固定高度、零 flex basis 或将行扁平化为 Link 都无明确收益，已撤销，固定高度还会裁切换行说明。

保留改动使用 block flow 布置页面章节、库分类和目录项，以显式 margin 保持间距。水平图标/文字/箭头仍为 flex，说明高度保持固有值。这些外层垂直栈不分配剩余空间，无需 flex 重复测量后代固有尺寸。

相邻 1280×600 CPU 运行中，原版内容 p95 8.478 ms，修改后 4.462 ms（8 次预热后 48 个计时滚轮帧）。保留窗口 800→1280→800→1280→1680 缩放后 p95 3.917–4.128 ms。产物为 `/tmp/solid-gpui-overview-control-final.log`、`/tmp/solid-gpui-overview-block-final.log`、`/tmp/solid-gpui-overview-block-resize.log`。这是 CPU 证据，不保证原生呈现。用户后来确认重载诊断窗口的滚动和缩放流畅，完成 Overview 验收。1264×759 活跃区间 draw p95 4.477–7.741 ms，输入到呈现 4.772–11.223 ms。排除无输入闲置区间；这些是区间分位数，不是合并分位数或匹配视口前后对比。原始证据 `/tmp/solid-gpui-profile-app.log`。

设计规则：区分流式文档和 flex 空间分配。独立垂直章节用 block flow 加 margin，确需对齐或弹性分配时用 flex。不要全局替换 flex、添加无失效机制的几何缓存，或为达标限制多行文字。比较相邻受控运行，机器其他负载影响绝对值。

最终包类型检查、带 frame-profile 的宿主 Clippy、格式、原生 Button 标签边界、保留缩放和 560px Overview 独立滚动均通过。紧凑窗口成对 CPU p95 从 11.435 降至 8.355 ms，仍接近该负载下 8.33 ms 预算，不能宣称普遍 120 Hz。

## 原生 FPS 监视器

当前 HUD 已替换为固定 GPUI Kit 的 `gpui-fps::FpsMonitor` Entity，默认显示实际 presentation 节奏，可切换为重绘能力估计。500 ms 读数更新与历史被动监视器不同，指标定义和受控比较见[分析指南](performance-analysis.md)。

## VirtualList 内部滚动：owner 保留与范围反馈（2026-09-05）

用户明确卡顿发生在列表内部，复现了两个与时间无关的工作量回归：

- 十行提交窗口从 [0,10) 移至 [1,11) 会再调用十次 renderItem。使用 Solid mapArray 保留重叠 owner 后，仅创建进入行、释放离开行、保留其余九行；同 key 新值仍更新，卸载释放全部 owner。
- 一次滚轮产生冲突范围 (0,7) 和 (5,10)：GPUI 回调给旧 offset，渲染给不含 overscan 的新可见行。现在只有一个布局后来源：实际视口相交加一次 overscan，在 100px/20px 行回归中得到 (3,12)。原生 overdraw 重测不能扩大请求，同一测试使四十行测量失效并确认无额外事件。

普通 Gallery CPU 循环使用已提交行内真实合成位移，不覆盖 Bun 往返或原生 vsync。初始 p50/p95 3.207/3.506 ms，后续候选 4.485/11.166 和 5.619/9.815 ms。后来发现多个 UnityShaderCompiler 各占约 90–95% CPU，因此无法证明可比 CPU 改善，也未停止无关进程。确定性收益是减少行创建并只发一次稳定范围通知。

产物：`/tmp/list-identity-red.log`、`/tmp/list-identity-green.log`、`/tmp/list-range-red.log`、`/tmp/list-core-tests.log`、`/tmp/list-host-tests.log`、`/tmp/list-scroll-baseline.log`、`/tmp/list-scroll-candidate-repeat.log`。自动滚轮当时仍不移动内容。重建诊断应用后，用户按要求进行内部滚动和缩放并确认流畅。这是体验验收，与受污染 CPU 对比独立。稳定规则记录在 solid-gpui 技能应用参考。

## 嵌套 VirtualList 滚轮边界（2026-09-05）

GPUI List 不主动停止事件传播。solid-gpui 外边界在 List 处理滚轮后比较逻辑 offset，只在实际移动时停止传播；顶部或底部不移动的事件交给祖先。每次事件都更新比较值，包括两帧之间多个事件。无条件停止会困住边界滚动，完全不拦截会同时移动列表和祖先。一次事件若将列表移到边界，由列表消费，下一次不移动事件才交祖先。

关键原生回归 `virtual_list_at_top_passes_wheel_to_outer_content` 验证列表在顶部时外层移动，列表仍可移动时外层静止。

链接的 gpui-pre 0.3.3 List 通过 hitbox.should_handle_scroll(window) 路由事件，没有直接锁定滚动手势目标的 API。页面滚动将列表移到指针下时，后续事件仍滚动列表。按用户要求保留原生行为，不添加 timer 手势猜测或修改 GPUI 依赖。确定性测试通过不证明真实触控板验收。

## 嵌套滚动所有权（2026-09-08）

滚轮事件应由其方向上仍可移动的最内层视口消费。先将新偏移限制在合法范围，再判断是否发生移动：移动后停止传播，到达边界或内容没有溢出时允许祖先处理。首次到达边界的事件仍由内层消费，后续事件才向外传递。没有纵向溢出的横向列表应将纵向滚轮交给页面。

该策略位于 GPUI 的共享 Div 交互与变高 List，覆盖普通溢出容器、UniformList、组件 VirtualList 及复用列表的控件。InputBase 与 ScrollableMask 也按实际移动消费事件。不要为每个示例添加滚轮拦截，它会掩盖共享层缺失的事件所有权，也可能让边界处无法继续滚动。滚动条拖动单独拥有鼠标拖拽事件。

`components::scroll_views::tests` 的原生回归同时检查内外层偏移，覆盖像素与行滚轮、边界、横向列表及未溢出内容。只断言列表移动，无法发现页面同时移动的问题。
