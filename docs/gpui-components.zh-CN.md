# 在 Solid 中使用 GPUI Kit

SDK 从 `@solid-gpui/core/components` 暴露生成的原生组件、描述符与命令。
实现来自 [GPUI Kit](https://github.com/longbridge/gpui-kit)，固定提交
`05433bd8e9e75af2f3aa508141b78ba21bfa6261`（0.6.1 及后续变更）。本地状态与生命周期接入点记录在
[`vendor/gpui-kit/SOLID-GPUI.md`](../vendor/gpui-kit/SOLID-GPUI.md)。

Solid 拥有应用数据、路由与子内容组合。原生 Entity 拥有焦点、编辑、滚动、菜单、停靠、动画和在途工作。
原生回调排队事件，不同步执行 Solid JS。

| Kit 层 | Solid GPUI 的用途 |
| --- | --- |
| `gpui-component` | 带样式的原生控件、编辑器、Carousel、文本、图表及窗口弹层；crate 名称仍为 `gpui-component`。 |
| `gpui-base` | 原生交互与状态、无样式控件、过渡、弹簧、关键帧、交错延迟与 presence。 |
| `gpui-kit` | 独立原生集成测试包使用的 facade 和无窗口系统交互辅助工具。 |
| `gpui-fps` | 显式开启、按窗口持有的性能 HUD；参见[指标定义](performance-analysis.md)。 |
| `gpui-kit-assets` | 仅提供 Kit 控件内部需要的默认图标；应用图标归 Iconify。 |

运行依赖位于 `vendor/gpui-kit`，`references/gpui-kit` 是匹配的固定上游检出。
Solid 应用继续由 Bun 或 QuickJS 运行；不链接 Shell，它不参与应用状态或路由。

```tsx
import { createSignal } from "solid-js";
import { Button, Input } from "@solid-gpui/core/components";

const [name, setName] = createSignal("");
<Input value={name()} onChange={(change) => setName(change.value)} />;
<Button label="Clear" onPress={() => setName("")} />;
```

生成文件 `packages/solid-gpui/src/components.ts` 是 API 参考，不要手工修改。`bun run task native-codegen` 从真实 Rust 宿主生成 SDK 与 website 绑定，`bun run task native-codegen-check` 验证一致性。自定义组件、属性、事件和命令使用同一生成器。

配置 Vite 的 `native` 后，`@solid-gpui/core/components` 与 Motion 会使用所选
宿主生成的组件契约。Rust 变化会自动重建宿主并替换开发会话；见[开发会话管理](hot-reload.zh-CN.md#开发会话管理)。

## 覆盖范围

### Carousel

`Carousel` 保留一个原生视口和选择状态。使用带稳定身份的 `CarouselItem` 子项；
非受控选择在 Solid 重排后跟随同一项目。通过 `selectedIndex` 控制选择，
应用外部值时用 `ackEditSeq` 确认 `onChange({ index, editSeq })`。
ref 提供 `select(index)`、`next()`、`previous()` 和 `getSelectedIndex()`。
`previous` / `next` slot 可以替换控制按钮。垂直方向必须设置正数 `viewportHeight`，
水平方向应有界定的宽度。最多接受 1024 个项目。

```tsx
import { Carousel, CarouselItem, Label } from "@solid-gpui/core/components";
<Carousel viewportHeight={160} pagination looping>
  <CarouselItem accessibilityLabel="Overview"><Label text="Overview" /></CarouselItem>
  <CarouselItem accessibilityLabel="Details"><Label text="Details" /></CarouselItem>
</Carousel>;
```

### 编辑器选择与语言规则

`Editor` 默认启用原生 `autoClose` 和 `smartIndent`，支持多光标、有方向的选择、原生插入和撤销。
`Input`、`NumberInput`、`Textarea` 和 `Editor` 提供 `getSelections()` 与
`setSelections({ ranges, editSeq })`。每个范围使用 UTF-8 字节偏移 `anchorByte` / `headByte`；
首个范围为活动选择，多范围仅适用于 Editor。UTF-8 序列或 CRLF 内部偏移、过期编辑序号、
IME 组合期间的选择变更都会在修改状态前拒绝。接受 1–1024 个范围，重叠按原生编辑规则合并。

`useNative().configureEditorLanguage()` 可安装括号对、带 `notIn` 上下文的自动闭合对和缩进规则。
配置按语言在应用内共享，应先配置再打开编辑器。模式使用 Rust 正则表达式，源最多 4096 字节，
编译程序最多 1 MiB。编辑规则不会添加语法高亮 grammar。

### Markdown 元数据与图标来源

`TextView format="markdown" frontmatter` 启用顶部 YAML 元数据渲染。
支持的简单标量显示为描述列表，复合或不支持的 YAML 显示为原生代码块。
此能力必须显式开启且仅适用于 Markdown；它不是通用 YAML 解析器。

组件图标 slot 通过 `ComponentIcon` 接受已注册的 Iconify 名称或 `{ svg: "<svg …>…</svg>" }`。
生成的独立 `Icon` 使用 `source`，例如 `<Icon source="lucide:check" />`。
SVG 校验后由原生保留，源文本限制为 64 KiB。Button、菜单、侧栏、设置、树、命令、表格和停靠图标均可使用。
核心 `Icon name="…"` API 及离线 Iconify 目录仍可使用，参见 [Iconify](iconify.md)。

### 原生动画与自定义控件

`Motion` 接受 `{ x, y, opacity }` 目标和由 `type` 区分的 `transition`、`spring`、`keyframes`。
省略的偏移为零，不透明度为一。逐帧采样和重绘请求留在 GPUI，完成后发送 `onComplete({ playbackId })`。
更新 `playbackId` 重播关键帧；过渡从当前目标变化，弹簧保留速度。
关键帧接受 2–128 个停靠点、播放方向、重复次数（`null` 为无限重复）。
过渡与关键帧支持 `stagger={{ index, count, intervalMs, origin }}`，
`origin` 可为 `first`、`last`、`center`。时长及绝对延迟不超过 60 秒，并遵循宿主减少动画设置。

通过 Solid `Presence` helper 将子 owner 保留至退出完成：

```tsx
import { Presence } from "@solid-gpui/core/motion";
import { Label } from "@solid-gpui/core/components";
<Presence show={open()} durationMs={180} reveal>
  {() => <Label text="Details" />}
</Presence>;
```

反转退出会保留原子内容，过期完成事件不能销毁新的子 owner；最终退出和父 owner 清理都会释放资源。
`NativePresence` 是自行管理子生命周期时使用的底层生成视图。应用路由仍由 `@solid-gpui/router` 拥有。

`BaseButton`、`BaseCheckbox`、`BaseSwitch` 和 `BaseToggle` 提供原生键盘、指针、焦点和无障碍行为，
应用定义子内容与样式。需提供 `accessibilityLabel`、受控状态和回调；
BaseCheckbox 支持 `unchecked`、`checked`、`indeterminate`。

| 原生家族   | JS 入口                                                                                                                                                                                                                                      |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 基础控件   | Alert、Avatar/AvatarGroup、Badge、Button/ButtonGroup、Toggle/ToggleGroup、Checkbox、Clipboard、Icon、Kbd、Label、Link、Pagination、Progress/ProgressCircle、Radio/RadioGroup、Rating、Separator、ShimmerText、Skeleton、Spinner、Switch、Tag |
| 编辑与选择 | Input、Textarea、Editor、NumberInput、OtpInput、ColorPicker、Slider、Calendar、DatePicker、Select、Combobox、Caret                                                                                                                           |
| 数据与滚动 | List/ListItem/ListSeparatorItem、SearchableListItemElement、DataTable、Tree、VirtualList、MessageScroller、Command、TextView/Text、Scrollable、ScrollShadow、FocusTrap                                                                       |
| 组合       | Accordion/AccordionItem、Breadcrumb/BreadcrumbItem、Collapsible、DescriptionList/DescriptionItem/DescriptionText、Form/Field、GroupBox、ResizablePanelGroup/ResizablePanel、Stepper/StepperItem、Tab/TabBar                                  |
| 消息与附件 | 生成目录中的所有 Attachment、Bubble、Marker 和 Message 元素                                                                                                                                                                                  |
| 导航与设置 | Sidebar 及其 Header/Footer/ToggleButton/Group/Menu/MenuItem；Settings、SettingPage/SettingGroup/SettingItem/SettingField/SettingCustomItem；StatusBar、TitleBar、WindowBorder                                                                |
| 覆盖层     | Dialog/AlertDialog 与 DialogContent/Description/Footer/Close/Action/Header/Title、Sheet、Popover、HoverCard、Tooltip、PopupMenu、ContextMenu、DropdownMenu、DropdownButton、AppMenuBar、NativeMenu、Notification                             |
| 停靠       | DockArea，布局描述符创建真实原生 TabGroup 与 TilesState 容器                                                                                                                                                                                 |
| 图表       | LineChart、AreaChart、BarChart、CandlestickChart、PieChart、RadarChart、SankeyChart                                                                                                                                                          |
| 底层绘图   | Plot 的 axis/grid/labels/line/area/bar/radialLine/arc 原语；PlotTooltip、PlotCrossLine、PlotDot                                                                                                                                              |
| 外观       | useNative().getTheme/setTheme、setApplicationTheme、getMotionPreference/setMotionPreference；应用主题令牌与动效偏好                                                                                                                                                   |
| 计算       | useNative().scaleLinear/scalePoint/scaleBand/scaleOrdinal、pieArcs、arcCentroid、stackSeries、sankeyLayout                                                                                                                                   |

部分上游类型属于其他控件，不是独立屏幕元素：

- 宿主每个窗口安装一次 Root、NotificationList、文本选择层、模态层及主题。Dialog、Sheet 和 Notification 使用该 Root。
- Scrollbar 与 ScrollableMask 由 Scrollable 和列表/表格/消息滚动 API 集成，并使用对应原生滚动句柄。FocusTrapContainer 是 FocusTrap 的实现，DropdownMenuPopover 是 DropdownMenu 的实现。

**ScrollShadow** 提供带动态边缘淡出的滚动区域。默认情况下它拥有滚动视口、滚动条和原生句柄。它可以改为借用一个**直接子项**的视口：该子项必须是来自 `@solid-gpui/core` 的核心 `VirtualList`，或来自 `@solid-gpui/core/components` 的生成原生 `VirtualList`，且方向必须与 `ScrollShadow` 的轴匹配。此时列表继续拥有自己的原生滚动，`ScrollShadow` 借用该视口来控制淡出、`scrollbarVisibility`、`scrollTo`、`getScrollPosition` 和 `onScroll`；不会创建外层滚动区域或重复滚动条。父视口需要限定尺寸（横向虚拟列表也需要限定高度），直接子项使用 `style={{ widthPercent: 100, heightPercent: 100 }}` 填满视口。

这种委托范围刻意保持狭窄：只识别一个方向匹配的直接子项。包装器、多个子项、嵌套列表和方向不匹配的子项不会被自动发现，并继续使用独立 `ScrollShadow` 行为。核心 `VirtualList` 的 data/renderItem 形式会虚拟化 Solid owner 和宿主节点；生成的原生 `VirtualList` 子项只虚拟化原生行的绘制，仍会创建所有提供的 Solid 子项。不要把大数据集展开成 JSX 子项；这类工作负载应使用核心 data/renderItem 形式。

`axis="horizontal"` 控制左右边缘，`axis="vertical"`（默认）控制上下边缘。独立的纵向区域需要限定高度；独立的横向区域可以由内容决定高度。无需额外嵌套 `Scrollable`。

`color` 应匹配周围背景色，默认使用主题背景；`fadeSize` 控制最大淡出范围，默认 24 像素，0 表示关闭淡出。接近边界的最后 `fadeSize` 像素内，淡出范围和不透明度随剩余距离逐渐减小。起点前缘完全清晰，终点后缘完全清晰；内容完全容纳时既无淡出，也无滚动条。原生绘制直接读取滚动几何信息，滚轮、拖动、`scrollTo` 和尺寸变化均无需 JavaScript 滚动订阅即可更新。覆盖层不拦截输入，绘制在滚动条下方，同时支持 `onScroll` 和 `getScrollPosition`。

`scrollbarVisibility` 默认为 `"always"`，也支持 `"hover"` 和 `"scrolling"`。若覆盖式滚动条会挡住内容，请沿滚动条所在边缘预留内边距。网站的 Markdown 表格和组件 API 表格在 Web 与桌面端共用横向 `ScrollShadow`。

固定上游完整需求清单位于 `.scratch/gpui-component-complete/upstream-inventory.md`，将构造描述符和内部/条件类型与 138 个普通公共渲染接口分开列出。

## Popover 呈现范围

`Popover` 在当前 GPUI 窗口内渲染，不能越过窗口边界。它支持原生 GPUI 焦点和关闭行为，但不会创建 AppKit `NSPopover` 或独立的系统弹层窗口。

需要超出所属窗口时，从 `@solid-gpui/core` 导入 `SystemPopover`。它通过内容工厂创建独立的所属 Surface，并共享 Solid 上下文。API、多显示器行为、平台支持及原生验收限制见[系统弹层](system-popover.md)。

## 子内容与原生状态

原生组合父组件在发布提交前检查子类型，例如 AvatarGroup 消费 Avatar，ButtonGroup 消费 Button，ResizablePanelGroup 消费 ResizablePanel，Settings 消费 SettingPage → SettingGroup → SettingItem → SettingField。错误子类型或属性拒绝候选提交并保留旧树。

命名 JSX 插槽使用独立 `slots` 对象，例如 `<Popover slots={{ trigger: <Button label="Open" /> }}>...</Popover>`。标量 title 与 slots.title 不同。插槽是独立已提交子树，在原生父布局中保留宿主边界。delegate 控件使用稳定数据 key，原生 API 需要任意内容时使用显式 slot index。排序、过滤和加载时保持 key 稳定。

ContextMenu 的触发器也放在 slots.trigger，默认子内容是自定义菜单内容。宿主加载组件库 icons/ 和 SDK Iconify 资源。未显式设置包装样式的原生子组件保留父布局约束，其身份与事件作用域不额外添加布局盒。

受控编辑事件包含编辑序号，生成绑定负责确认，应用提供值并响应语义事件。父重渲染不会重置焦点、选区、组合输入和撤销；显式替换命令则有意替换编辑内容。

列表、表格和树只渲染可见范围。搜索/加载事件描述当前原生请求，完成命令拒绝被替代请求。大数据应保持数据形式，不应每次鼠标移动都重建完整行树。

## 外观

`await useNative().setTheme("dark")` 更新应用级 Component 主题及其 Base 投射，也支持 light 和 system。getTheme 返回选择模式及实际 dark 标记。宿主默认跟随系统外观。Solid 样式 token 应同步相同选择，主题作为原生 App 全局状态作用于全部窗口。

这些短界面操作使用 `CommandDefinition::foreground`；计算和 I/O 继续通过 Tokio。前台命令有类型验证，需要已挂载窗口，不能阻塞窗口。

## 设置

```tsx
import { Input, SettingField, SettingGroup, SettingItem, SettingPage, Settings } from "@solid-gpui/core/components";

<Settings>
  <SettingPage name="general" title="General">
    <SettingGroup name="identity" title="Identity">
      <SettingItem title="Display name" keywords={["profile"]}>
        <SettingField dirty={name() !== ""} onReset={() => setName("")}>
          <Input value={name()} onChange={(change) => setName(change.value)} />
        </SettingField>
      </SettingItem>
    </SettingGroup>
  </SettingPage>
</Settings>;
```

页面/组名称标识原生选择和状态。重排或搜索过滤不会将编辑状态转移给其他项目。Settings 提供选择/搜索事件及 select、setQuery、getState 命令。

SettingField 使用传入的原生控件。需要禁用交互时将 disabled 传给该控件，自定义字段外观不能禁用任意后代。整个原生子树停止渲染时，窗口局部临时状态可能过期；保留的 Input/Editor Entity 属于已提交宿主节点，遵循自身生命周期。

## 停靠与持久化

```tsx
import { DockArea, Input, Label, type DockAreaRef } from "@solid-gpui/core/components";

let dock: DockAreaRef | undefined;
<DockArea
  ref={(value) => (dock = value)}
  panes={[
    { name: "editor", title: "Editor", contentSlot: 0 },
    { name: "preview", title: "Preview", contentSlot: 1 },
  ]}
  initialLayout={{ center: { kind: "tabs", panes: ["editor", "preview"] } }}
>
  <Input value={name()} onChange={(change) => setName(change.value)} />
  <Label text={name()} />
</DockArea>;
```

pane name 标识保留的原生 Panel Entity，slot index 选择 DockArea 子内容。支持 titleSlot、titleSuffixSlot、工具栏按钮、原生菜单和 JSON data。重排配置或更新内容/装饰保留 Entity 与当前布局。关闭 pane 将其移出布局，但可保留配置供 addPane 重开；从 panes 移除则释放 Entity。initialLayout 中所有引用必须指向已配置 pane，删除配置时同步更新声明。

initialLayout 只在挂载时应用，之后用户原生交互拥有几何。显式替换使用 replaceLayout，持久化使用 dump/load，类型化快照包含版本、pane 数据、分割尺寸、活动标签、dock 状态、tile 边界与堆叠顺序。加载要求所有引用已在当前 owner 配置，未知 pane 会在改变布局前失败。

ref 命令还支持增删移动、标签选择、缩放、侧 dock 切换/缩放/折叠策略、tile 几何、置顶、撤销和重做。移动定位 pane name 或区域，不使用持久原生 NodeId。跨容器移动不发 Removed。布局事件携带 revision，按需取快照，不要每个拖动步骤都序列化完整布局。

## 菜单与覆盖层

PopupMenu、ContextMenu、DropdownMenu 支持带 key 的菜单树及自定义子项。打开期间更新会保留焦点，并按 key 协调选择项。AppMenuBar 反映现有应用菜单 API，所有权仍在应用级。

NativeMenu 使用系统弹窗实现，无系统后端的平台使用上游绘制实现。contextMenu、press、manual 触发和 show({x,y}) 使用窗口坐标。系统菜单显示打开时快照，后续属性变化在下次打开应用。选择动作作用域绑定挂载视图，卸载即撤销。

Dialog、Sheet 和 Notification 使用原生 owner/session token。旧对话框请求完成不能关闭新对话框，旧 owner 卸载不能移除其他 owner 当前覆盖层。原生覆盖控件保留父动作/焦点行为。通知按当前 key/session 更新，使用原生堆栈与计时器。

## 绘图数据与工作量限制

图表在属性提交时编译数据，原生执行命中测试与绘制。Line/Area/Bar/Radar 提供原生交互 tooltip；固定版本 Candlestick/Pie/Sankey API 没有交互 tooltip setter。Radar 标签可用已提交子插槽，柱填充支持原生渐变。

Plot 坐标为逻辑像素，径向与圆弧角度为弧度。原生 line/area/radial 按上游连接语义省略 null 点，不把 null 转为零。Stack 对缺失值明确遵循原生算法的零值。Pie 省略零/null 切片并返回原始输入索引。Sankey 布局值可能被缩放（如开平方），业务汇总应使用原始 link 值。

桥接拒绝非有限数、无效范围/OHLC、循环或缺节点的 Sankey 链接、错误 series 宽度和无界工作。当前上限：16,384 个 plot item、32 个 chart/stack series、256 个 Plot 原语、512 个 Sankey 节点/4,096 个链接/32 次迭代，以及 128 个 dock pane/256 个容器/16 层布局。Ordinal 批量查找也限制原生线性搜索工作。Pagination 省略页选择器即使总页数极大也保持常量工作量。

## 验证

`bun run ci` 运行生成契约、TypeScript 检查/测试及 Rust 检查。审计与发布构建独立运行，见[持续集成](ci.md)。适配器旁的原生集成测试覆盖状态标识、过期请求、组合边界、菜单路由、停靠持久化和数值/工作量限制。真实 website 交互与呈现检查独立于确定性 TestAppContext 渲染，原生测量要求见[性能分析](performance-analysis.md)。
