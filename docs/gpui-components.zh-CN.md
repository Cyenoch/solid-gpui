# 在 Solid 中使用 gpui-component

SDK 从 `@solid-gpui/core/components` 暴露 **144 个生成的 JSX 组件/描述符和 10 个原生函数（8 个计算、2 个外观）**。链接的实现为 gpui-component 0.6.0，提交 `928c3eb776a3d733d9b771f7dea27a6a79242ced`；声明式状态接入点记录在 `vendor/gpui-component/SOLID-GPUI.md`。

导入组件即选择其真实 GPUI 实现。Solid 拥有应用数据和子内容组合，原生 Entity 拥有焦点、文本编辑、滚动、菜单交互、停靠和在途工作。原生回调读取已提交数据并排队事件，不同步执行 JS。

```tsx
import { createSignal } from "solid-js";
import { Button, Input } from "@solid-gpui/core/components";

const [name, setName] = createSignal("");
<Input value={name()} onChange={(change) => setName(change.value)} />;
<Button label="Clear" onPress={() => setName("")} />;
```

生成文件 `packages/solid-gpui/src/components.ts` 是 API 参考，不要手工修改。`bun run task native-codegen` 从真实 Rust 宿主生成 SDK 与 website 绑定，`bun run task native-codegen-check` 验证一致性。自定义组件、属性、事件和命令使用同一生成器。

## 覆盖范围

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
| 外观       | useNative().getTheme/setTheme，支持 light、dark、system，应用于原生 Component 和 Base 主题                                                                                                                                                   |
| 计算       | useNative().scaleLinear/scalePoint/scaleBand/scaleOrdinal、pieArcs、arcCentroid、stackSeries、sankeyLayout                                                                                                                                   |

部分上游类型属于其他控件，不是独立屏幕元素：

- 宿主每个窗口安装一次 Root、NotificationList、文本选择层、模态层及主题。Dialog、Sheet 和 Notification 使用该 Root。
- Scrollbar 与 ScrollableMask 由 Scrollable 和列表/表格/消息滚动 API 集成，并使用对应原生滚动句柄。FocusTrapContainer 是 FocusTrap 的实现，DropdownMenuPopover 是 DropdownMenu 的实现。

**ScrollShadow** 提供带动态边缘淡出的滚动区域，内部滚动视图、滚动条和淡出共用原生滚动句柄。
`axis="horizontal"` 控制左右边缘，`axis="vertical"`（默认）控制上下边缘。
纵向区域需要限定高度，横向区域可由内容决定高度，无需额外嵌套 `Scrollable`。

`color` 应匹配周围背景色，默认使用主题背景；`fadeSize` 控制最大淡出范围，默认 24 像素，0 表示关闭淡出。
接近边界的最后 `fadeSize` 像素内，淡出范围和不透明度随剩余距离逐渐减小。
起点前缘完全清晰，终点后缘完全清晰；内容完全容纳时既无淡出，也无滚动条。
原生绘制直接读取滚动几何信息，滚轮、拖动、`scrollTo` 和尺寸变化均无需 JavaScript 滚动订阅即可更新。
覆盖层不拦截输入，绘制在滚动条下方，同时支持 `onScroll` 和 `getScrollPosition`。

`scrollbarVisibility` 默认为 `"always"`，也支持 `"hover"` 和 `"scrolling"`。
若覆盖式滚动条会挡住内容，请沿滚动条所在边缘预留内边距。
网站的 Markdown 表格和组件 API 表格在 Web 与桌面端共用横向 `ScrollShadow`。

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
