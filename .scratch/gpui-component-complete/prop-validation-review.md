# 数值与集合属性的 publication 前验证核对

本次限定读取 `primitives.rs`、`groups.rs`、`content.rs`、`extra_elements.rs`、`table_elements.rs` 的 JS 属性及其实际 vendor 消费路径。确认 5 类问题：Rating/Pagination 将小整数输入放大为大量原生元素，Form/Field 能放大为上万条 grid tracks，部分像素参数接受负数，以及 DescriptionItem 的 span 可以越过真实列范围。没有把“类型是整数但未加校验”本身当成缺陷，也未扩展到 NativeView 状态组件。

仅修改本报告。执行了一组独立临时 Taffy 0.13.0 布局探针，未修改仓库测试、未运行原生窗口。工作区 HEAD 为 `1bf74901244d8af95d7d547de5a0fb5ce362b2fc` 加当前未提交实现；其他代理会继续编辑，以下是读取快照指纹。

| Adapter 文件 | 行数 | SHA-256 |
| --- | ---: | --- |
| primitives.rs | 429 | `29eefbc4ba1eabf1fc3ec48575b7ae52743b2df76cb523ad314299b1a40c2de6` |
| groups.rs | 301 | `6ecd180e870d56172593a2ac10d37c2045e33881a0b0ed4f38e4db3f0a11fbc1` |
| content.rs | 326 | `a34e1a0d8c5b8e90317f24e2c493060f4e4ebb43194357a3117027713768a9ae` |
| extra_elements.rs | 123 | `08c923fec2355df77c1a9313af95aa95125418a4d787347c52c8a009f95c224a` |
| table_elements.rs | 174 | `b4f1d4eb05c8447c80ab1e6607715238db31f06aae2d71255a3546f11f174758` |

## 需要前置拒绝的实际输入

### 1. P1 — Rating.max 直接决定每次 render 的元素与闭包数量

入口是 `max: u32`，无上限地传入 `.max(max as usize)`；Rating 的 render 执行 `for ix in 1..=max`，每项构造 Div、Icon，并可能绑定 mousemove/click listener。`max=4_294_967_295` 是可通过 native JSON/u32 解码的很短输入，但会尝试创建约 43 亿项；disabled=true 也仍然循环构造元素。[adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/primitives.rs:297)、[无上限 builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/rating.rs:67)、[实际循环](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/rating.rs:149)

最小修正：为 max 使用 publication 阶段可解码的有界值类型，拒绝大于固定 `MAX_RATING_STARS` 的值。可先定 100；**100 是建议的产品上限，源码只证明必须有工作量上限**。max=0 当前只是空控件，不需要为防 panic 而拒绝；value>max 已由 value/max builder clamp，不能重复报成索引越界。

### 2. P1 — Pagination.visiblePages 与 totalPages 有两条独立的放大路径

`currentPage/totalPages/visiblePages` 都是裸 u32。非 compact 模式先生成 page_numbers：当 `total<=visible` 时分配 `1..=total` 的完整 Vec，其他分支也按 visible 生成连续 page range。更隐蔽的是，visiblePages=5 并不能使 totalPages 安全：每个 Ellipsis 在打开时遍历其整个隐藏 range，为每页创建 PopupMenuItem。菜单 `.max_h(240).scrollable(true)` 只限制视口，不限制构造数量。[adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/primitives.rs:233)、[页项算法](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/base/src/pagination.rs:163)、[非 compact 入口](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/pagination.rs:170)、[菜单完整 range](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/pagination.rs:218)

具体触发：`totalPages=visiblePages=4_294_967_295, compact=false` 在首次 render 放大；`totalPages=4_294_967_295, visiblePages=5, compact=false` 初始页条很小，点击省略号后放大。是大量工作/OOM 风险，不是数学上的无限循环。

最小修正：在 decoded props 校验阶段，对**非 compact** 的有效页项数量与每个 ellipsis 的 range 长度分别设预算；用现有算法的 O(1) 边界公式先计算长度，不能先调用 `.items()` 构造完再数。简化而充分的初始 contract 可为 `visiblePages<=100 && totalPages<=1000`；这些数字是建议上限，可按实际产品容量调整。若希望允许更大 total，则必须改变菜单的原生渲染方式，本报告不展开该实现。compact=true 只构造前后两个按钮，没有此项放大，不应把它误列成相同故障。currentPage=0、totalPages=0、currentPage>total 与 visiblePages<5 已有原生归一化，不是新的 panic 点。

### 3. P2 — Form.columns 与 Field 的 grid 属性可生成上万 tracks

Form.columns 是 u16，直接进入 `grid_cols`；Field.colSpan 是 u16，colStart/colEnd 是 i16，直接写入 grid placement。它们不受 host style 属性验证保护，因为这些是 native DTO 字段而非 HostStyle。[Form adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/groups.rs:261)、[Field adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/groups.rs:276)、[Form grid](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/form/form.rs:105)、[Field placement](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/form/field.rs:271)

锁定依赖的实际链路是 GPUI 0.3.3 → Taffy 0.13.0；GPUI 将这些值直接转换成 repeat/GridPlacement。Taffy 已限制 grid 每个方向最多 10,000 tracks，所以**不能报告 u16 最大值必然整数溢出或无限循环**。但一个字段的数值输入仍可生成上万 tracks：`columns=65535`、`colSpan=65535`、`colStart=-32768/32767` 均走此路径。[GPUI setters](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/styled.rs:752)、[转换](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/taffy.rs:452)、[Taffy 限制](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/taffy-0.13.0/src/compute/grid/types/coordinates.rs:7)

隔离探针只用 600×100 的 grid 和一个空 Field，按同样的 GPUI 属性映射喂给 Taffy：columns=65535 产生零宽字段，用时约 5.3 ms；colSpan=65535 约 7.0 ms；极端正负 colStart 约 4.4–4.7 ms，同样出现零宽字段。该数字是 debug Taffy CPU 探针，**不是原生呈现或性能验收**；它证实了小属性对布局工作的明显放大。

最小修正：Form 的显式 columns 与 Field 的 span/line 采用统一、小而明确的 grid 预算，不能只限父 columns，否则 child placement 仍能生成大量 implicit tracks。可先用 64 列、span<=64、`abs(colStart/colEnd)<=65`；位置的绝对值应转 i32 后计算，避免对 i16::MIN 直接 abs。这里 64 是建议预算。columns/span 的 0 值以及 line=0 在当前 Taffy 中安全归一化；若将它们规范为非零，那属于明确的领域 contract 收紧，不是修复已证实 panic。不要把 CSS 支持的负 line index 或反向 start/end 一概误判为错误。

### 4. P2 — 负的像素参数直接进入布局或原生窗口几何

以下字段都是未经语义验证的 f32；native JSON 的有限数/安全整数检查会放行 `-1`。

| 属性 | 真实消费与触发 | 最小规则 |
| --- | --- | --- |
| Form.labelWidth | `.label_width(px(value))` 存入 FieldProps，horizontal Field 的 wrap_label 直接 `.w(width).flex_shrink_0()`；负宽度进入原生布局。vertical 时该 width 不使用。[adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/groups.rs:265)、[builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/form/form.rs:47)、[消费](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/form/field.rs:257) | labelWidth>=0；0 可表达无固定标签宽度，不必强制正数。 |
| DescriptionList.labelWidth | horizontal 时直接给 label Div 设置 w；负数没有 clamp。[adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/content.rs:90)、[builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:170)、[消费](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:367) | labelWidth>=0。 |
| WindowBorder.shadowSize | client decorations 模式直接 `window.set_client_inset(shadow_size)`，并用于四边 padding/inner frame 坐标；负值改变平台 inset，不仅是无效外观。[adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/content.rs:315)、[builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/window_border.rs:55)、[平台调用](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/window_border.rs:147) | shadowSize>=0。 |
| WindowBorder.resizeHitSize | `band=hit_size+hit_size` 作为 hit zone width/height；负数直接产生负 hit-area 尺寸。[几何](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/window_border.rs:306) | resizeHitSize>=0；0 可关闭有效命中带。 |

可共用非负像素 DTO 类型并在 Deserialize 阶段拒绝。有限数已经受到 native JSON guard 保护，不应另报 NaN/Infinity 可从当前 JS 正常传入。[现有 guard](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/json_guard.rs:57) 未执行 Linux client-decoration 窗口验证；这里的负参数传递和负 hit-zone 计算是直接源码证据。窗口小于合法 shadow padding 时的几何处理属于 native viewport 规则，不能靠一个与 viewport 无关的属性上限完全解决，未将其扩展为本次 publication 缺陷。

### 5. P2 — DescriptionItem.span 超过真实 columns 会产生空首行和非法列占比

DescriptionList.columns 本身已经 clamp 到 1..10，但 DescriptionItem.span 原样保留。group_item_rows 首次放入超范围 span 时，先建立空首行，再因 `current_span+span>columns` 推入新行；结尾只删除尾部空行，因此空首行保留下来。绘制又直接使用 `span/columns` 作为 flex_basis。例：columns=1、首项 span=2 产生空首行和 200% basis；span=65535 会传入 65,535 倍 basis。[adapter](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/content.rs:102)、[未校验 span](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:121)、[分行](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:235)、[basis](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:330)

最小修正：普通 Item 要求 `1<=span<=effective_parent_columns`。只限 span<=10 不充分：parent columns=1、span=10 仍触发。separator=true 的分支没有使用传入 span，不应把忽略的值当成真实 range 故障。standalone DescriptionItem 当前由 DescriptionPart 包成默认 3 列的原生 DescriptionList；如继续支持这个入口，其有效上限是 3。[standalone 入口](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/content.rs:14)、[默认列数](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:144)

这项需要 candidate composition 检查父 columns 与对应 child 的 span/separator，不能只给单个 u16 套独立 newtype。可以复用候选树验证期间已解析的 native DTO，不应先构建原生元素再检验。当前 validate_composition 的 summary 只有 kind/type 等信息；若要支持该关系，应添加只读访问这些 child props 的明确通道，而非从 renderer 再遍历所有组件。span=0 不会让 for 循环停住：它只是不推进列占用、给出零 basis，因此属于无效占位 contract，不能称为无限循环。

## 已核对但不列为新缺陷的优先项

| 属性/路径 | 不报的依据 |
| --- | --- |
| Stepper.selectedIndex | 只经 checked_step 用于 `<`/`<=` 比较；step、aria position 与显示编号来自真实 items.enumerate()，不会按 selectedIndex 分配元素或数组索引。巨大值会显示全通过，不是本次要求的 panic/工作放大/负尺寸问题。[父消费](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/stepper/stepper.rs:125)、[trigger](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/stepper/trigger.rs:103) |
| TabBar/RadioGroup.selectedIndex | Tab indicator 明确检查 selected_ix<num_tabs；RadioGroup 只与枚举 ix 比较。没有未经检查的索引。[Tab guard](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/tab/tab_bar.rs:197)、[Radio](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/radio.rs:384) |
| AvatarGroup.limit；Badge.count/max | Avatar 只 take(limit) 已有 items，不按 limit 创建条目；Badge 只比较并格式化数字。[Avatar](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/avatar/avatar_group.rs:104)、[Badge](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/badge.rs:131) |
| ShimmerText.durationMs=0 | ShimmerStyle.duration 已 clamp 到 1 ms；不发生除零。[实际 builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/shimmer.rs:68) |
| DescriptionList.columns=0/65535 | builder clamp 到 1..10，不是除零或大列数分配。[实际 builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/description_list.rs:192) |
| TableHead/TableCell.colSpan | `.max(1)` 已处理零；最大 u16 是非常宽的有限 min-width（100px×65535），但没有按 span 构造同数量的 cells/tracks。未找到本次要求的额外 panic 或工作量放大路径，不能仅因没有上限就列入强制校验。[Head](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/table/table.rs:450)、[render](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/table/table.rs:498)、[Cell](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-component/crates/component/src/table/table.rs:541) |
| ControlSize、Percentage、extra_elements | ControlSize 仅四个 enum 值；Percentage 已验证 0..100。extra_elements 没有未检查的 JS 数字/数值集合属性；其列表 item id 来源是 cx.id，不是用户数值 props。[types](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/mod.rs:27)、[Percentage](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/mod.rs:99)、[extra id](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/extra_elements.rs:63) |

NativeItems/NativeSlot 集合的循环数量由实际 host tree children 决定；本次未找到这五个文件将一个集合元素递归扩展成与其数字字段同数量元素的其他路径，没有提出通用 children-count 上限。

## 最小验证落点与已执行证据

单属性工作量/像素规则在 Rust Deserialize 类型执行，沿现有 ComponentDefinition::validate 的 decode 阶段进入 publication gate。Pagination 的 compact/数量关系以及 Description 的父子关系必须在组合校验执行；不要通过 render-time clamp/panic catch 隐藏非法输入。Snapshot/Patch 的 candidate 验证已经位于 store 替换前，保留该原子路径。[decode gate](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:550)

临时 Taffy 探针按实际 GPUI 的 repeat/GridPlacement 映射，分别运行 `columns=0`、`span=0`、`line=0`、最大 columns/span、最小/最大 i16 line、反向 line range 八种输入；全部正常返回，没有 panic 或 3 秒超时。前三个零值产生正常 600px 宽字段，反向 3→1 的 line range 正常解析为 400px；这排除了错误的“零值/负 line 一定崩溃”结论。临时源文件与构建目录已自动清理；未把这些探针加入仓库测试。

实现后只需保留关键验证：超限 Rating、两种非 compact Pagination 放大输入、Form/Field 极端 grid、四种负像素参数和 Description 父子 span 关系均在 publication 前拒绝，旧 revision/instances 不变；另外保留正常边界值与 compact 大 total 的通过场景。无需为每个原生 builder 的既有 clamp 镜像新增测试。
