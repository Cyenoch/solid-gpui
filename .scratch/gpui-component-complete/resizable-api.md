# Resizable 原生 API 与 typed child 接入核对

建议把 ComponentChild 的通用实现下移 gpui-base；ResizablePanelGroup 持有该 wrapper 并在注入原生 state/axis/index 后收尾。ResizableState 必须按稳定 panel key 对齐尺寸，拖动应标识真实分隔线两侧的 panel，而非持久保存数组下标。Panel 的使用位置在候选树 publication 前验证，避免孤立原生 Panel 的 panic。

本次只读研究仅覆盖 ComponentChild 分层、ResizablePanelGroup/Panel/State，以及框架的父入口校验。未修改实现，未运行仓库测试或原生 UI。本地 HEAD 为 `1bf74901244d8af95d7d547de5a0fb5ce362b2fc`，判断包含当前未提交实现；以下指纹用于定位读取快照，行号可能随其他代理编辑变化。

| 文件 | SHA-256 |
| --- | --- |
| component/src/component_child.rs | `21bde172c2d7a22b1f12695700f1e256589f2b494c12d3c8226bf9b0f6c04b20` |
| base/src/resizable/mod.rs | `d81acd463d174e1ace7597cba339abbb466f633213c9d4c6bc6034c417bb626a` |
| base/src/resizable/panel.rs | `ad8788438270c1edb15064113bfaa8465dc4a26fe40acf174129ab6907fe6b2d` |
| solid-gpui/src/native/component.rs | `c7d62217e3c6ecf8636f61b1185a0e42bbb4558a7a0673d7d948d66664ba2db1` |
| solid-gpui/src/renderer.rs | `ee4172b7075c52ca82ec0e80c45514c983fe31fc1dcfd0740a352a8a5df2af07` |

## ComponentChild 下移的具体分层

当前 ComponentChild<T> 的核心只用 gpui 与 std；阻止整体移动的是 component 自己的 Sizable/Size、SidebarItem，以及延迟 Sidebar render 的 helper。[当前实现](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/component_child.rs:1)

| 内容 | 所在层与具体调整 |
| --- | --- |
| `ComponentChild<T>`、Clone、From<T>、with_boundary、map_native、render_with、Deref/DerefMut、IntoElement、Styled | 移到新的 `base/src/component_child.rs`，在 base/lib.rs 声明模块并导出。字段继续私有，boundary 保留现有 `Option<Rc<dyn Fn(AnyElement)->AnyElement>>`，不需要新依赖。T 本身不加 IntoElement bound，继续支持描述器。[base 依赖](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/Cargo.toml:28) |
| `Collapsible for ComponentChild<T>` | 同时移动到 base。trait 实际是 `gpui_base::component_traits::Collapsible`；component 只是转导出。base 根的 `Collapsible` 名字指另一个原生控件，不要误用根导出的同名类型。[trait 定义](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/component_traits.rs:28)、[component 转导出](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/component_traits.rs:1) |
| `Sizable for ComponentChild<T>` | 留在 component。Size/Sizable 定义在 component，且本层拥有该 trait，可以为 base 的类型实现它。[定义](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/sizing.rs:178) |
| `SidebarItem for ComponentChild<T>`、DeferredSidebarItem | 留在 component。SidebarItem 是本层 trait，impl 合法。helper 通过公开 map_native 延迟 render，不需要接触 base 的私有字段；继承当前已修好的子 scope 时机。[当前 helper](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/component_child.rs:81) |
| component 对外导出 | `pub use gpui_base::ComponentChild`，只保留同一个类型的 façade 转导出，不创建第二套 wrapper。旧 component_child.rs 可成为本层 trait impl 的模块。[现有导出](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/lib.rs:158) |
| root NativeChild 桥接 | 保留 `cfg(feature="gpui-component")`；From 的目标可明确写 `gpui_base::ComponentChild<T>`。gpui_component 的同名转导出是同一类型，不能对两个路径重复实现 From。默认 crate::native 继续只依赖 gpui，原有 Table/Sidebar root 代理无需改变职责。[桥接](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:143) |

移动后如果把 Collapsible impl 留在 component，trait 与 self 类型都变成外部类型，会违反 orphan rule；Sizable 与 SidebarItem 则因 trait 仍本地而允许。规则依据见 [Rust Reference](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence)。本报告没有另行编译移动后的实现。

ResizablePanelGroup 的 `children: Vec<ResizablePanel>` 改成 `Vec<ComponentChild<ResizablePanel>>`；child/children 接受 `Into<ComponentChild<ResizablePanel>>`。render 中仍设置真实 panel 的 panel_ix、axis、state、handle_appearance，再转换为元素。使用 map_native 或现有 DerefMut 均可保留 boundary；不要先把 panel 擦为 AnyElement，也不添加额外布局 Div。[存储/入口](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/panel.rs:31)、[原生注入点](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/panel.rs:163)

## 当前状态错误的触发条件

| 条件 | 当前代码结果 |
| --- | --- |
| `[A,B,C]` 重排为 `[C,A,B]`，数量不变 | sync_panels_count 不移动任何项；sizes、size preference、bounds 仍按旧下标，C 获得 A 的状态。[count 同步](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/mod.rs:171) |
| 删除中间 B | 仅 group count 同步会截尾 C 的状态，剩余 C 接到 B 的尺寸。dock 已因该问题维护额外 child order，并调用按索引插删；它明确不能处理 reorder。[dock 注释](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/dock/dock_area.rs:103)、[旧同步 helper](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/dock/dock_area.rs:1691) |
| 正在拖动的 panel 被删除、group 截尾或 clear | resizing_panel_ix 可能失效。remove_panel 只在旧 ix 大于删除位置时减一，等于时不清；clear 和 count 截尾也不清。[删除/clear](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/mod.rs:220) |
| 同一拖动期间树更新 | MouseMove 在 paint 时捕获 current_ix，回调又按该下标 expect panel；会错改其他面板或在越界时 panic。MouseUp 同样依赖 paint 时的旧值。[事件安装](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/panel.rs:441) |
| 零 panel 时错误拖动状态进入 resize worker | `old_sizes.len() - 1` 先减一，空数组可能下溢；正确取消状态之外仍应在 worker 入口明确要求至少两项和有效 handle。[worker](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/mod.rs:278) |

## 最小原生状态方案

### 1. 明确 panel identity

给 ResizablePanel 增加必需的 `key: ElementId`，建议构造接口变成 `resizable_panel(key)`。solid-gpui adapter 使用 `cx.id()`，它是稳定 host node id；原生 Rust 调用者传语义 key，不能以 enumerate 下标或每次 render 新生成的值冒充身份。[host id](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:85)

当前 `From<T: Into<AnyElement>> for ResizablePanel` 和 `From<ResizablePanelGroup> for ResizablePanel` 自动创建无名 Panel，不适合必需 key。建议删除这两条隐式转换，调用者明确 `resizable_panel(key).child(content_or_group)`。不保留无 key 的 positional 分支，也不要求额外用户 key props：JS 用 host id 即可。[隐式转换](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/panel.rs:120)

constructor 迁移涉及的现有路径集中在 resizable 自身测试/示例、base/text/text_view.rs、base/dock/dock_area.rs、component/setting/settings.rs、component/tests/base_compat.rs 和 base 的 resizable showcase。只更新这些原生调用者的 key，不扩展组件适配范围。

### 2. 按 key 对齐完整状态

将 key 放进 ResizablePanelState，保留现有公开 `sizes()` 的有序视图。用 `sync_panels(axis, specs)` 替代 count-only 同步；specs 由当前真实 Panel 的 key/initial_size/size_range/visible 构成。一次 reconcile 移动旧 `(panel_state, measured_size)` 对，按新 key 顺序重建：已有 key 保留实际尺寸与 preference，新 key 使用自身初始策略，删除 key 丢弃自己的状态。不要只重排 sizes 而遗漏 size、bounds、range 等同一 panel 的字段。

这个过程可用一次局部 HashMap 做 O(n) 索引；保持顺序的 render 快路不需要每帧分配全量 map。无结构改变时更新必要 constraints，并保留现有 initial_size 的“仅初次创建”语义，不能每帧用 props 覆盖拖动后的尺寸。新节点初始化应有明确状态（例如 measured 标记），不要用“尺寸恰好等于 PANEL_MIN_SIZE”识别新节点；当前 update_panel_size 正使用该数值哨兵。[现有初始化](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/mod.rs:200)

“保留尺寸”指尺寸跟随身份：相同容器的纯 reorder 不交换或重新初始化宽度；添加/删除后可继续按原生容器规则重新分配剩余空间，不能承诺所有 panel 永远保持相同像素。原来的比例 resize、flex 初始策略、range clamp 和 settling frame 仍保留，不另写一套布局分配算法。[容器分配](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/mod.rs:350)、[settling frame](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/panel.rs:175)

同组 key 必须唯一。JS 的 node id 已提供唯一性；原生 API 将重复 key 视为调用契约错误，在 reconcile 开始前检查，不能先局部修改 state 再发现重复。若公开 insert_panel 保留，它也应接受 key 并遵守同一唯一性，不制造匿名状态槽。

### 3. 拖动跟随真实分隔线

将持久的 resizing_panel_ix 换成 `ResizeDrag { before_key, after_key, axis }`（可再有 session/generation）。它表示相邻两面板之间的 handle；临时索引只在执行一次原生 resize 算法时解析。

- unrelated panel 插删或移动后，两 key 仍相邻且轴/可见性有效：继续同一拖动，并解析新的 index。
- 任意一侧删除、隐藏，邻接关系改变，或 axis 改变：取消这次拖动。不要让当前 ix 指向新的邻居并继续改它的尺寸。
- group MouseMove/MouseUp 回调捕获 state，不捕获 paint 时的 current_ix；处理事件时读取 live drag。MouseMove 在有效邻接与已测量 bounds 下调用现有 resize_panel_at_handle；MouseUp 只对仍有效的会话完成一次并发送一次 on_resize。取消不伪造完成事件。
- on_drag 捕获左右 key，并核对它们仍属于当前 group 的该 handle。panel prepaint 更新也携带 key（必要时结构 generation），按当前身份解析，拒绝已退役的更新；不能让旧闭包凭 index 写到新项。
- clear/reset/删除当前项统一使失效 drag 退役；worker 在 `<2 panels` 或没有后继时明确退出，消除空数组减一。

Panel 的 GPUI element id 与 handle id 也宜由 key/邻接 key 推导；不要继续用原生 panel_ix 作为长期 UI 身份。key 只承载身份，真实 axis、range、bounds 与原生样式优先级依然由 Panel/State 管理。

### 4. dock 是本接口必须处理的一处调用者

不能只改 group 而放任 dock 继续管理另一套 count/index 同步。dock 已有真实 `PaneNode::id()`；render_node 构造 Panel 时传该 id。其 `sync_split_panels` 及紧接的 sync_panels_count 可合并为调用同一个按 key 同步入口；保留现有“树的 sizes 有变化时才 adopt_sizes”的授权规则与 scale_sizes_to，避免每帧覆盖用户拖动或 viewport resize 的结果。[调用点](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/dock/dock_area.rs:1029)、[panel 构造](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/dock/dock_area.rs:1451)

这是原生 state API 的必要调用者迁移，不要求修改 dock 的 pane tree、持久化格式或拖放业务。原有 CachedSplit.sizes 仍用于判断树是否重新授权尺寸；child order 可以交回 ResizableState，不保留两个重复重排算法。

## publication 前拒绝孤立 Panel

原生 ResizablePanel::render 在 `visible=true` 时 expect state 与 panel index。无父 Group 的 Panel 会在绘制阶段 panic；`visible=false` 虽先返回空 Div，也不能使非法结构变合法，因为切回 visible 即触发同一路径。[原生 precondition](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/panel.rs:301)

当前 validation 只有父→child 的 concrete type 校验；Panel 作为普通 View 的 child 没有反向限制。[当前校验](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:503) 建议增加小而明确的 contract：

1. ComponentDefinition 增加 `requires_typed_parent: bool`，仅 ResizablePanel 标记 true。对应 builder 可命名 `with_typed_parent_required()`；若由宏声明，可用 `#[component(typed_parent = true)]`，扩展现有 options parser 并传到 Definition。这个标记不禁止 Panel 自己拥有普通内容。
2. ExtensionAdapter 暴露 accepted child_type 与 requires_typed_parent，复用现有 element_type。Panel 必须有 concrete element_type；其实际父入口必须宣告消费该类型。不要按组件字符串名特判，也不把所有叶组件一并改成 typed-only。
3. 在 SolidRoot::validate_extension_tree 的同一候选 store 上验证反向关系。合法入口只有：直接 parent 是 Extension 且 default_child_group=None；或 immediate parent 是该 Extension **指定 default_child_group 对应的生成 View**。后一种只跨这一层已声明的 slot container，不穿透任意 View/ScrollView 或任意祖先。选中的 adapter.child_type 必须等于 Panel.element_type。[child summary 的同一 content 规则](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/extensions.rs:25)
4. 根节点、普通 View 下、命名 slot 下，以及不消费 Panel 的父组件下均返回有 node id/parent id 的 InvalidChildren；不 mount 原生 Panel、不捕获 panic、不自动补 Group、不把 Panel 退化成 Div。嵌套布局明确写 Group→Panel→Group→Panel；父 Group 不能凭隐式 From 自动把直接 Group child 当 Panel。
5. Snapshot 与 Patch 已在 `candidate.apply_*` 后、`self.store=candidate` 和 native reconcile 前调用 validate_extension_tree，直接在此加入校验即可保持原子拒绝。合法树中途被 Move 到普通容器的 Patch 也走同一入口，旧 tree/revision/instances 保持不变。[Snapshot 门](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer.rs:529)、[Patch 门](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer.rs:609)

父限制属于模块 contract，应在 ModuleDefinition::description 中序列化稳定的 requiresTypedParent 或推导的 parentComponents 名称，并参与 digest；不要把 Rust TypeId 序列化。若该元信息仅用于 catalog/文档，typescript generator 像 childComponents 一样从运行时 descriptor 中剔除；若决定输出给 JS，则同时扩展 NativeComponentDescriptor。通过项目 generator 更新产物，不手改生成文件。[description/digest](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/module.rs:178)、[descriptor 剔除](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/module.rs:253)

## 最少必要验证

- **分层编译**：base 独立检查能使用带 boundary PanelGroup；component 检查 SidebarItem/Sizable；root 默认 features 和 gpui-component features 都能编译，确保没有 base→component 依赖与重复 alias impl。
- **一次 state 关键测试**：给 A/B/C 不同实测尺寸，纯 reorder 后尺寸随 key；删除中间项与插入新 key 后旧项身份、initial_size 与 container 分配语义正确；拖动邻接 pair 随无关前项移动继续、删除任一侧/clear 后取消且后续 mousemove 不 panic。校验真实 state，不仅比较临时 key 数组。
- **一次原生拖动/布局测试**：原有两 panel 拖动尺寸与单次 on_resize 保持；增加拖动时重排/删除、轴切换，以及 Panel own onLayout/style，确认透明 boundary 未改变 flex 直接项与 handle 命中。
- **publication 原子性测试**：孤立 visible/hidden Panel、普通 View 中 Panel、命名 slot 中 Panel 均拒绝；有效 Group 下 Panel 通过；Move 使有效 Panel 非法时不发布 revision、不 mount/remount。一个 slot-aware typed fixture 验证仅 default content lane 允许，避免祖先搜索误放行。

现有 resizable 测试覆盖固定/弹性初始尺寸、外部 state settling、programmatic resize、原生拖动与 cross-axis size；没有稳定 key reorder 或删除活跃拖动的证明。[现有测试](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/resizable/mod.rs:407) 本报告只给出需保留和补充的验证范围，不宣称这些检查已通过。
