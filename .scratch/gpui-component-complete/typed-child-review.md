# NativeItems<T> 的节点边界核对

结论：当前 typed-child 路径保留了真实原生类型及声明事件，但**绕过子 Host Node 的完整 namespace 和 onLayout**。样式分两类：`styled_element` 的直接样式仍被应用；普通 `element` 依赖外层包装的样式会丢失。通用 pointer/key 事件不是 typed-child 独有回归：现有 NativeComponentProps 并未承诺这些通用事件，普通 Extension 同样提前返回，不经过 View/Pressable 的通用监听器。

本次只读核对基于工作区 HEAD `1bf74901244d8af95d7d547de5a0fb5ce362b2fc` 加正在开发的未提交改动；当前实际依赖已改为 `vendor/gpui-component` path，GPUI 为 `gpui-pre 0.3.3`。未运行仓库测试或原生窗口，没有性能/呈现验收结论。其他代理可能继续编辑，下列行号与读取指纹用于定位本次判断。

| 文件 | SHA-256（读取时） |
| --- | --- |
| native/component.rs | `9bb1c7471f255d19c3dfe200c45a56f11189304104f0e9cf68360eb9856bfd12` |
| renderer/extensions.rs | `d770d637ff1942b816731bff192c7f0342c50c90f526fc795cca0f899c51a866` |
| renderer.rs | `fd14dc33b42b34c95694a27889d4f08bec0665d7f11b5c850ce797dc1ee2ce87` |
| renderer/paint/mod.rs | `95fa7c4a2b202e1f47989c067be333e9f3dd5a6861e130bed8fbdcb20c73677c` |

## 两条路径在哪里分叉

普通 child：`ChildIterator::next → render_node_for_extension → render_node → extensions::render → instance.render → ElementInstance::build → style → scope_native_element`（native_style=true），或 `wrapper Div + style + measure`（false）。证据：[迭代与 typed 分支](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/extensions.rs:360)、[普通 Extension 尾部](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/extensions.rs:567)。

typed child：`ElementContext::typed_children → next_native → instance.build_native → ElementInstance::build → Box<Any> → downcast<T> → Vec<T> → 父控件消费 T`。最后没有携带 Host Node 渲染边界；仅保留 T 自身的字段。[NativeItems](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:78)、[build_native/build](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:426)。

| 核对项 | 当前结果 | 证据与影响 |
| --- | --- | --- |
| 节点 namespace | typed child 绕过 | `scope_native_element` 的透明 Element 以 node id 为 `Element::id`，在实际 request_layout/prepaint/paint 期间形成作用域。仅 T 的某个内部 id 或构建时传 cx.id 不等价；Tab、TablePart 等并不全都将 host id 放在外层，父容器也会修改内部 id/index。[boundary](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/paint/mod.rs:64) |
| onLayout | typed child 不上报自身 bounds | 测量在 MeasuredElement::prepaint，next_native 未创建它。后代普通 child 仍可能上报，不能代替 typed child 的 frame。JS 所有 native components 明确提供 onLayout。[测量](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/paint/mod.rs:110)、[JS contract](/Users/jgbingzi/workspace/sp/solid-gpui/packages/solid-gpui/src/native.ts:43)、[layout dispatch](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer.rs:968) |
| Styled T 的 style | 保留 | styled_element 选择 apply_style_to_extension；build 对 T 直接应用 context.style；next_native 从 style_for_node 取得动画 frame style 优先值。[构造策略](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:265)、[build](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:434)、[frame style](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/animation.rs:466) |
| 非 Styled T 的 style | 丢失 | element 使用 identity apply_style，普通渲染在最后套有样式 Div；typed 分支没有这一步。现有 children_tests 的 Item 就是此类，但没有设置 style。[策略](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:265)、[wrapper](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/extensions.rs:615)、[测试 fixture](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/children_tests.rs:42) |
| Native Event<T> | 没有因 downcast 丢失 | T 内声明事件闭包继续持有 Event sink；实例统一 reconcile/update，不是 typed 时重新 mount。父控件覆盖同一个 callback 字段是另一层原生语义。[事件](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:12)、[实例同步](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer.rs:679) |
| 通用 pointer/key/hover/scroll | 当前不属 NativeComponentProps 通用能力 | JS host props 仅 style/children/onLayout/ref，另外是模块声明事件；普通 Extension 在通用绑定前提前返回。不能声称本修正“恢复了通用事件”。如果要扩展支持，需要单独 contract/订阅设计。[JS](/Users/jgbingzi/workspace/sp/solid-gpui/packages/solid-gpui/src/native.ts:149)、[Extension 早退](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/paint/mod.rs:189)、[View handlers](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/renderer/paint/mod.rs:266) |

## 最小但根本的修正

保留“具体 T + 节点来源与收尾操作”，在**原生父控件完成读取、修改 T 之后，最终 IntoElement 时**安装 boundary。不要先把 T 擦成 AnyElement，也不要先包一层布局 Div 再送入原生父控件。

建议 root 的 `crate::native` 自有 `NativeChild<T>`，vendor 放一个不依赖 solid-gpui 的泛型 `ComponentChild<T>`。两者均持有真实 T 和可选可克隆的不可变 `Rc<dyn Fn(AnyElement) -> AnyElement>` finalizer；消费一个实际渲染 occurrence 时调用一次。不能用只能消耗一次的共享 closure：Sidebar 会克隆来源并重复渲染可见项，详见下文。提供 raw T 的转换、只读 `as_ref`、保留 finalizer 的 `map_native<T -> U>`；最终 IntoElement 转换 T 并调用 finalizer。此类型是通用组成边界，不需要每个叶控件增加 renderer 字段。

solid-gpui 的 `NativeItems<T>` 携带相应带来源的 child，而非裸 Vec<T>；来源包括稳定 node identity、布局订阅/事件路由生命周期和样式放置策略。`next_native` 创建来源；`build_native` 只构造真实 T。普通渲染与 typed 最终渲染共享一个 finish 边界，确保执行一次。避免提供默认丢弃来源的 `into_inner` 快捷路径。

| 父组件 | 保留真实类型的最小调整 |
| --- | --- |
| ButtonGroup | children 改存 `ComponentChild<Button>`；仍经 as_ref 读取 selected，map_native 设置 toggled/corners/edges/size/variant/compact/outline/回调；最后才 IntoElement。原生组仍决定单选/多选、连接边框和 pressed 语义。[当前算法](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/button/button_group.rs:153) |
| TabBar | children 改存 `ComponentChild<Tab>`；仍在真实 Tab 上设置 variant/size/selected/原生 group callback/prepaint bounds hook，再最终渲染。保留 indicator/overflow menu 依赖的 Tab 信息；最终化必须晚于这些配置。[消费点](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/tab/tab_bar.rs:426) |
| 原生 Table | 已有 ChildElement 接口，不必降级成 Div。对带 boundary 的 TablePart wrapper 代理 Styled、Sizable、with_ix，IntoElement 时安装 scope。AnyChildElement 继续先 with_ix/with_size，再转换。所有 TableHeader/Body/Row/Cell 等每层都保留自己的来源。[现有 TablePart](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/table_elements.rs:6)、[上游 ChildElement](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/element_ext.rs:14) |

其他强类型容器使用同一泛型组合边界。若父控件消费的是描述器并延迟生成元素，应将 finalizer 一同保留到描述器最终渲染处；不能为了取其中字段而剥掉来源。

透明 boundary 复用 MeasuredElement 的“返回同一个子 LayoutId、prepaint/paint 直接委托”设计：不会改变 flex/grid 的直接布局项，也不会增加测量层级。Styled 类型的样式仍直接应用真实 T；父组件覆盖 size/variant 的现有优先级保持。非 Styled 类型保留普通路径的外盒样式策略，但需要额外布局盒才能表达的样式，不应谎称透明：若具体父组件要求直接布局项，则 contract 应要求其 typed child 支持 native style，或为该父接口提供明确可用的样式适配；不可静默忽略。

finalizer 只安装本 contract 已支持的 node scope/onLayout/样式处理，不普遍注入所有高频事件。生命周期令牌应防止卸载/epoch 更新后的 deferred layout callback 写到后来的节点。每帧按被消费 child 数 O(n) 构建，不引入全树扫描、全局持久来源表或每帧双重构建；无 onLayout 订阅时不调度 layout 回调。

## 关键验证场景

1. **一个机制测试覆盖 scope + layout + styles**：两个同类型 typed children，内部刻意用相同局部 keyed id，分别带不同 node id、onLayout、直接 style；点击只改变对应状态，宽度 patch 后仅对应 frame 变化且普通/typed 的样式结果相同。另一个 non-Styled child 明确验证外盒样式策略。
2. **ButtonGroup/TabBar 真实父控件测试**：检查组选择事件、连接边框/variant、Tab indicator bounds 和每个 child onLayout；改变 child 顺序保持 host identity；无 group callback 时 child onPress 可达，有 group callback 时遵守上游覆盖语义。TabBar 上游明确声明 group on_click 会忽略 child on_click，不能把“父子都触发”当修复。[上游声明](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/tab/tab_bar.rs:165)
3. **原生 Table 嵌套测试**：Table→Header/Body→Row→Head/Cell，各层 onLayout 有独立有效 bounds；父 size/with_ix 传播，col_span/原生 table a11y 结构和宽窄 resize 不变。透明 scope 不能增加真实 layout wrapper。
4. **生命周期/原子性**：保留现有 wrong concrete type 拒绝与无 remount 测试；加入卸载/epoch 替换后晚到测量和事件不投递，普通 child 与 typed child 均只构建/最终化一次。

现有 children_tests 只证明更新可渲染、slot 内容有效、错误 child 类型原子拒绝、owner 退役；listener_id 全为 0，没有 style、交互、namespace 或 layout 检查，因此不能排除本次缺口。[现有测试](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/children_tests.rs:56)

以上是源码判断和建议测试，未执行这些测试。原生窗口还需检查真实焦点/键盘、Tab 动画与 Table 窄宽 resize；确定性测试仅作为结构与交互语义证据。

## 默认 feature、转换合法性和代理位置

`crate::native` 默认可用，而 `gpui-component` 与 `gpui-base` 都是可选依赖；`NativeChild<T>` 的定义、Clone（仅 T: Clone）、map_native、基础 IntoElement/Styled 实现只依赖 gpui 与 root renderer。不要从默认 native 路径导入 gpui-base。仅在 `#[cfg(feature = "gpui-component")]` 下提供 vendor 转换和相关 Sizable、ChildElement、Collapsible、SidebarItem 代理。[feature 边界](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/Cargo.toml:29)

`impl<T> From<NativeChild<T>> for gpui_component::ComponentChild<T>` **合法**：输入 NativeChild<T> 是本 crate 的本地类型，前面的外部 ComponentChild<T> 覆盖了 T；不存在在首个本地类型之前的 uncovered T。这也能与 vendor 的 `impl<T> From<T> for ComponentChild<T>` 共存。已用真实两个临时 crate 经 `rustc 1.98.1 (48a229cea 2026-09-01)` 编译并运行转换断言，输出 PASS；临时文件已自动清理。这只验证 coherence，不是项目编译或 UI 验收。[Rust Reference 的 orphan/coherence 规则](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence)

桥接应转移全部字段，不要形成 NativeChild<ComponentChild<T>> 双层收尾。可提供 vendor `ComponentChild::from_parts(value, finalizer)`，root 的拆分操作限 crate 内使用；公开消费 API 保留 boundary。各代理的调用顺序：Styled 返回内部 T 的 style；Sizable/ChildElement/Collapsible 用 map_native 改真实 T；普通 IntoElement 在最后安装 scope；SidebarItem 需要延迟 render，见下文。vendor 自身的 ComponentChild 同样实现其父控件所需 trait，root 的可选代理则覆盖不转换到 vendor wrapper 的 Table/泛型 Sidebar 路径。

## 当前 adapters 确实涉及的父 collection

本范围来自当前 `src/components` 的全部 `NativeItems<...>` 使用：**19 个父入口，12 个 vendor 具体类型 collection，另有 5 个 Table trait 接口和 2 个泛型 Sidebar 接口**。下表只列这些在用接口，不包含菜单 builder、data-table delegates 等无此 typed 路径的 API。行号会随并行格式化变动，以类型和字段名为准。

| 在用父控件 / adapter | 当前 vendor 存储 | 必须保留的消费步骤与调整 |
| --- | --- | --- |
| Accordion / groups::accordion | `children: Vec<AccordionItem>` | 改为 ComponentChild<AccordionItem>；读取 open，再 map_native 设置 index/last/size/disabled/回调。当前 adapter `.item(move |_| item)` 要支持 closure 返回带 boundary child，或直接提供明确的 child 接口。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/accordion.rs:25) |
| AvatarGroup / groups::avatar_group | `avatars: Vec<Avatar>` | 改存 wrapper；limit/take/reverse 保持，父 size 与重叠 margin 应在真实 Avatar 上完成。被 limit 隐藏的条目不应伪造 onLayout。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/avatar/avatar_group.rs:13) |
| Breadcrumb / groups::breadcrumb | `items: Vec<BreadcrumbItem>` | 改存 wrapper；map_native 设置 id/is_last，separator 仍是父级生成项。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/breadcrumb.rs:15) |
| ButtonGroup / groups::button_group | `children: Vec<Button>` | 改存 wrapper；selection、disabled、边框与组 callback 保持原有优先级，见前表。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/button/button_group.rs:21) |
| ToggleGroup / groups::toggle_group | `items: Vec<Toggle>` | 改存 wrapper；as_ref 读取 checked/disabled，map_native 设置 segmented corners/edges、size/variant、点击捕获。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/button/toggle.rs:228) |
| RadioGroup / groups::radio_group | `radios: Vec<Radio>` | 改存 wrapper；map_native 设置 id/position_in_set/size_of_set/disabled/checked/回调，保留 a11y；adapter 的 with_size 走 Sizable 代理。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/radio.rs:273) |
| TabBar / groups::tab_bar | `children: SmallVec<[Tab; 2]>` | 改存 wrapper，仍读取 label/icon/disabled 建 overflow menu 元信息，indicator prepaint hook 在最后收尾前设置。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/tab/tab_bar.rs:47) |
| Stepper / groups::stepper | `items: Vec<StepperItem>` | 改存 wrapper，map_native 设置 step/size/checked_step/layout/text_center/disabled/is_last/回调。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/stepper/stepper.rs:15) |
| Form / groups::form | `fields: Vec<Field>` | 改存 wrapper；map_native 调用 field.props(ix, props)，再作为直接 grid item 收尾。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/form/form.rs:16) |
| DescriptionList / content::description_list | `items: Vec<DescriptionItem>` | 改存 descriptor wrapper；group_item_rows 也保留 wrapper，并通过 as_ref 读取 span；最终 match 生成原生 item Div 时 map_native<T→Div> 再收尾，不能在分组时丢掉来源。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:214) |
| SidebarMenu / sidebar::sidebar_menu | `items: Vec<SidebarMenuItem>` | 改存 wrapper；adapter 从 SidebarEntry 转换必须 `map_native(|entry| entry.0)` 后桥接，collapsed 代理保持来源，显式 SidebarItem::render 延迟执行。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/sidebar/menu.rs:22) |
| SidebarMenuItem / sidebar::sidebar_menu_item | `children: Vec<Self>` | 改存 ComponentChild<Self>，Clone 保留不可变来源；子菜单开启后每个 item 独立 scope，不能仅修最外层 SidebarMenu。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/sidebar/menu.rs:105) |
| Sidebar、SidebarGroup / 两个同名 adapter | `Vec<E>`，`E: SidebarItem` | 无需改 vendor collection API；令 E 为 NativeChild<SidebarPart>，root 的 SidebarKind::Group 内对应泛型也更新，代理 Clone/Collapsible/SidebarItem。[来源](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/sidebar/group.rs:9) |
| Table、TableHeader、TableBody、TableFooter、TableRow / 五个 table_elements adapter | `AnyChildElement` | 无需改 vendor collection；NativeChild<TablePart> 代理 IntoElement/Styled/Sizable/ChildElement，使现有 with_ix→with_size→into_any 顺序自然收尾。[接口](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/element_ext.rs:14) |

普通 child/children/items 的 raw Rust 调用可通过 `Into<ComponentChild<T>>` 直接接受真实 T；不应保留一套能绕过 boundary 的旧 typed adapter API。

## T→U、Sidebar 重复渲染与描述器的必要细节

`map_native<U>(self, f: impl FnOnce(T) -> U) -> NativeChild<U>` 可以安全保留 provenance，条件是它只移动 value 并原样保留来源和 style policy，不 render、不 finalize、不重新生成 host id。原始 child concrete-type 校验发生在 host 构造 T 之前/期间，后续 parent 内部换成 U 不应改注册 contract。映射必须保持一个 Host Node 对应一个最终渲染根；将 T 展开成多个独立元素或只拿字段后丢弃其渲染主体，不能自动宣称保留节点语义。ComponentChild 同样提供 T→U 映射，并只在 IntoElement 实现处要求 T: IntoElement；类型本身和 map 不加该限制，才能携带尚无 IntoElement 的 DescriptionItem。

SidebarEntry→SidebarMenuItem 是明确的一对一映射，现有 entry.0 上的原生 style 仍在原值里；TablePart 可以完全保留 enum，通过 trait 代理消费，不需要拆 enum。若确需映射 TablePart 某个 variant，仍移动完整 wrapper 来源并保持同一个根，不要使用裸 match 结果代替 NativeChild。

**Sidebar 的 Clone 是真实运行要求。** `SidebarItem: Collapsible + Clone`；Sidebar 的 list callback 用 `self.content.get(ix).cloned()`，同一项会随 viewport/重绘再次生成元素。因此不可克隆的 Box<dyn FnOnce> 不够，Rc<RefCell<Option<FnOnce>>> 也会导致第一次之后失去 boundary。建议 Rc<dyn Fn> 包含不可变 host identity/epoch/弱生命周期引用，每次新渲染 occurrence 生成新的 MeasuredElement；epoch 失效仍由 renderer 的路由检查处理。[trait](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/sidebar/mod.rs:211)、[可见项克隆](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/sidebar/mod.rs:449)

**SidebarItem 不能先 render 再 scope。** 其 render 直接调用 window.use_keyed_state；若代理先 `T::render` 得到 AnyElement 再套 boundary，状态创建已经发生在父 namespace。代理应返回 `finish(DeferredSidebarItem { value, id }.into_any_element())`，其中私有泛型 DeferredSidebarItem 的 RenderOnce 才调用 `T::render(value, id, window, cx)`。这样原生 render 在子 boundary 的 request_layout 内执行，同时保留原生 id 参数、collapsed 行为和实际具体 T。不要在接口调用时捕获 window/cx 的短借用。[状态创建](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/sidebar/menu.rs:265)

DescriptionPart→DescriptionItem 的 provenance 也能原样移动，但它不自动解决 style 放置：DescriptionItem 是由父控件生成直接 flex item 的描述器。若把 non-Styled 的外盒 Div 直接搬进该路径，可能把 span/flex_basis 留在内层而改变布局。这里必须把 Host style 应用到父控件已有的 item 根 Div（需要明确的 styled-finalize/deferred-style hook），然后只安装透明 scope；或在 contract 明确限制不支持的样式。不能仅通过保存 finalizer 宣称描述器 style 全部正确。此约束同样属于 generic render boundary 的样式策略，不要求每个叶控件携带 HostStyle。

关键验证场景再补一项真正必要的 Sidebar 情况：开关子菜单、滚出再滚回、重排两个含相同局部 keyed id 的条目后，open state 与 onLayout 仍各自归属相应 host node；每个重新渲染 occurrence 有 boundary，且卸载 epoch 后不投递旧回调。这验证 Clone 和延迟 render 两个实际消费入口。
